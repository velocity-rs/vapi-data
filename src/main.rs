mod config;
mod db;
mod errors;
mod log;
mod response;
mod router;
mod schema;

use log::{error, info};
use std::collections::HashMap;

mod handlers;

#[tokio::main]
async fn main() {
    if let Some(log_levels) = config::get::<HashMap<String, String>>("log_levels") {
        log::setup(log_levels)
    }
    info!("Logger setup completed");

    match router::ServiceRouter::new().await {
        Ok(router) => {
            router.start().await;
        }
        Err(e) => {
            error!("Error starting Service Router {}", e);
        }
    }

    info!("Starting data service");
}
