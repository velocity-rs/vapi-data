use std::fmt::Display;

use axum::{
    Json, Router, debug_handler,
    response::IntoResponse,
    routing::{delete, patch, post, put},
};

use log::{debug, error, info};
use serde_json::json;
use thiserror::Error;
use tokio::net::TcpListener;

const ADDR: &str = "0.0.0.0";
const PORT: &str = "12000";

#[derive(Debug)]
pub struct ServiceRouter {
    listener: TcpListener,
}

impl Display for ServiceRouter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.listener.local_addr() {
            Ok(local_addr) => {
                write!(
                    f,
                    "Router -> addr: {}, port: {}",
                    local_addr.ip(),
                    local_addr.port()
                )
            }
            Err(e) => {
                error!("Error getting local addr {}", e);
                Err(std::fmt::Error)
            }
        }
    }
}

#[derive(Debug, Error)]
pub enum RouterError {
    #[error("Could not bind address")]
    AddrBindingFailed(String),
    #[error("Could not retreive local address")]
    LocalAddressFailure(String),
}

impl ServiceRouter {
    pub async fn new(addr: Option<String>, port: Option<String>) -> Result<Self, RouterError> {
        debug!("Checking addr and port and setting to default values if none provided");
        let server_addr = if let Some(a) = addr {
            a
        } else {
            String::from(ADDR)
        };
        let server_port = if let Some(p) = port {
            p
        } else {
            String::from(PORT)
        };

        debug!("Checking if configuration is valid and can be used");
        let server_config = [server_addr, server_port].join(":");

        match TcpListener::bind(server_config).await {
            Ok(tcp_listener) => match tcp_listener.local_addr() {
                Ok(addr) => {
                    debug!("TCP address {} successfully bound", addr);
                    let service_router = ServiceRouter {
                        listener: tcp_listener,
                    };

                    return Ok(service_router);
                }
                Err(e) => {
                    error!("IO error occured retreiving local bind address {}", e);
                    Err(RouterError::LocalAddressFailure(e.to_string()))
                }
            },
            Err(e) => {
                error!("Error binding address {}", e);
                return Err(RouterError::AddrBindingFailed(e.to_string()));
            }
        }
    }

    /// Return a core router for Create, Update, Patch, Delete
    fn routes() -> Router {
        Router::new()
            .route("/{org}/{app}/{namespace}/{object}/{version}", post(todo))
            .route("/{org}/{app}/{namespace}/{object}/{version}", put(todo))
            .route("/{org}/{app}/{namespace}/{object}/{version}", patch(todo))
            .route("/{org}/{app}/{namespace}/{object}/{version}", delete(todo))
    }

    fn find_routes() -> Router {
        Router::new()
            .route("/{org}/{app}/{namespace}/{object}/{version}", post(todo))
            .route("/{org}/{app}/{object}/{version}", post(todo))
    }

    pub async fn start(self) {
        let app = Router::new()
            .nest("/api", Self::routes())
            .nest("/api/findall", Self::find_routes())
            .nest("/api/findone", Self::find_routes());

        info!("Starting server: Velocity API Data");

        match axum::serve(self.listener, app).await {
            Ok(_) => info!("Server started"),
            Err(e) => error!("Error starting server {}", e),
        }
    }
}

#[debug_handler]
pub async fn todo() -> impl IntoResponse {
    Json(json!({"hello":"world"}))
}
