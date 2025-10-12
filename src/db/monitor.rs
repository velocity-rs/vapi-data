use log::{error, info, trace};
use mongodb::Client;
use mongodb::bson::doc;
use tokio::sync::mpsc::Sender;
use tokio::time::{Duration, sleep};

use crate::db::RepositoryError;

pub async fn monitor(
    error_sender: Sender<Result<(), RepositoryError>>,
    client: Client,
    interval: Duration,
) {
    info!("Starting MongoDB monitor with interval: {:?}", interval);

    //let monitor_client = client.clone();
    loop {
        sleep(interval).await;
        let result = match client.database("admin").run_command(doc! {"ping": 1}).await {
            Ok(_) => Ok(()),
            Err(e) => Err(RepositoryError::PingFailed(e.to_string())),
        };
        if let Err(e) = error_sender.send(result).await {
            error!("Error sending monitor result: {}", e);
        }
        sleep(interval).await;
    }
}
