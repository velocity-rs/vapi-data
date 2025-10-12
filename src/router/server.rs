use std::fmt::Display;

use axum::{
    Json, Router,
    response::IntoResponse,
    routing::{delete, patch, post, put},
};

use axum_macros::debug_handler;
use log::{debug, error, info};
use mongodb::bson::de;
use serde_json::json;
use thiserror::Error;
use tokio::net::TcpListener;

use super::state::AppState;
use crate::handlers::create;
use crate::{config, db::Repository, router::state};
const ADDR: &str = "0.0.0.0";
const PORT: &str = "12000";

#[derive(Debug)]
pub struct ServiceRouter {
    listener: TcpListener,
    state: AppState,
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
    #[error("Path Config Error. {0} not configured")]
    PathConfigError(String),
}

impl ServiceRouter {
    pub async fn new() -> Result<Self, RouterError> {
        info!("Initializing Service Router");
        debug!("Checking addr and port and setting to default values if none provided");

        let server_addr: String =
            config::get::<String>("server.addr").unwrap_or_else(|| String::from(ADDR));

        let server_port: String =
            config::get::<String>("server.port").unwrap_or_else(|| String::from(PORT));

        debug!("Checking if configuration is valid and can be used");
        let server_config = [server_addr, server_port].join(":");

        let state = match state::AppState::new().await {
            Ok(state) => state,
            Err(e) => {
                error!("Error initializing state {}", e);
                std::process::exit(2);
            }
        };

        match TcpListener::bind(server_config).await {
            Ok(tcp_listener) => match tcp_listener.local_addr() {
                Ok(addr) => {
                    debug!("TCP address {} successfully bound", addr);
                    let service_router = ServiceRouter {
                        listener: tcp_listener,
                        state: state,
                    };
                    info!("Service Router initialized");
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

    pub async fn start(self) {
        let path = if let Ok(path) = Self::get_path() {
            info!("Using path: {}", path);
            path
        } else {
            error!("Error getting path from configuration");
            std::process::exit(3);
        };

        let op_path = format!("/api/{}", path);
        let find_path = format!("/api/find/{}", path);

        info!("Operation Path: {}", op_path);
        info!("Find Path: {}", find_path);

        let router = Router::new()
            .route(op_path.as_str(), post(create))
            .route(op_path.as_str(), put(todo))
            .route(op_path.as_str(), patch(todo))
            .route(op_path.as_str(), delete(todo))
            .route(find_path.as_str(), post(todo))
            .with_state(self.state);

        info!("Starting server: Velocity API Data");

        match axum::serve(self.listener, router).await {
            Ok(_) => info!("Server started"),
            Err(e) => error!("Error starting server {}", e),
        }
    }

    fn get_path() -> Result<String, RouterError> {
        let org = config::get::<String>("ORG")
            .ok_or_else(|| RouterError::PathConfigError("ORG".into()))?;
        let app = config::get::<String>("APP")
            .ok_or_else(|| RouterError::PathConfigError("APP".into()))?;
        let namespace = config::get::<String>("NAMESPACE")
            .ok_or_else(|| RouterError::PathConfigError("NAMESPACE".into()))?;
        let object = config::get::<String>("OBJECT")
            .ok_or_else(|| RouterError::PathConfigError("OBJECT".into()))?;
        let version = config::get::<String>("VERSION")
            .ok_or_else(|| RouterError::PathConfigError("VERSION".into()))?;
        Ok(format!(
            "{}/{}/{}/{}/{}",
            org, app, namespace, object, version
        ))
    }
}

#[debug_handler]
pub async fn todo() -> impl IntoResponse {
    Json(json!({"hello":"world"}))
}
