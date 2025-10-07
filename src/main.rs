mod errors;
mod response;
mod router;
mod schema;
use std::sync::Arc;

use log::{error, info};
use tokio::sync::RwLock;

use router::AppState;

mod handlers;

pub type SharedState = Arc<RwLock<AppState>>;

#[tokio::main]
async fn main() {
    if let Some(log_levels) = config::get("log_levels") {
        log::setup(log_levels)
    }
    info!("Logger setup completed");

    let server_addr: Option<String> = config::get::<String>("server.addr");
    let server_port: Option<String> = config::get::<String>("server.port");

    match router::ServiceRouter::new(server_addr, server_port).await {
        Ok(router) => {
            router.start().await;
        }
        Err(e) => {
            error!("Error starting Service Router {}", e);
        }
    }

    info!("Starting data service");
}
