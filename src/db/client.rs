use log::{error, info};
use mongodb::{
    Client,
    bson::doc,
    options::{ReadPreference, RunCommandOptions, SelectionCriteria},
};
use std::time::Duration;
use tokio::sync::mpsc::channel;

use super::monitor;
use crate::{config, db::RepositoryError};

pub async fn init() -> Result<Client, RepositoryError> {
    let mongo_url = match config::get::<String>("MONGO_URL") {
        Some(s) => s,
        None => {
            return Err(RepositoryError::ConfigError(String::from(
                "Mongo URL Not Set",
            )));
        }
    };

    let client = match Client::with_uri_str(mongo_url).await {
        Ok(c) => c,
        Err(e) => {
            error!("Error creating MongoDB client: {}", e);
            return Err(RepositoryError::ClientError(e.into()));
        }
    };

    // create a sender channel to communicate with the monitor thread
    // let (error_sender, mut error_receiver) = tokio::sync::mpsc::channel(100);
    let (error_sender, mut error_receiver) = channel(100);
    let monitor_interval = config::get::<u64>("MONITOR_INTERVAL").unwrap_or(10);

    let c = client.clone();
    tokio::spawn(async move {
        info!("Starting MongoDB monitor task");
        monitor::monitor(error_sender, c, Duration::from_secs(monitor_interval)).await;
    });
    // spawn a task to listen for errors from the monitor
    tokio::spawn(async move {
        loop {
            match error_receiver.recv().await {
                Some(Ok(_)) => {
                    log::debug!("MongoDB cluster is healthy");
                }
                Some(Err(e)) => {
                    error!("MongoDB cluster health check failed: {}", e);
                }
                None => {
                    error!("Error receiver channel closed");
                }
            }
        }
    });

    Ok(client)
}
