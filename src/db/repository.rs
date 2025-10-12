use log::{error, info};
use mongodb::{Client, Collection, Database, bson::Document};
use std::{
    fmt::Display,
    sync::{Arc, Mutex, OnceLock},
};

use super::{client, errors::RepositoryError};
use crate::config;

#[derive(Debug)]
pub struct Repository {
    client: Client,
    pub database: Database,
    pub collection: Collection<Document>,
}

impl Display for Repository {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Repository -> db: {}, collection: {}",
            self.database.name(),
            self.collection.name()
        )
    }
}

impl Repository {
    pub async fn init() -> Result<Self, RepositoryError> {
        info!("Initializing MongoDB Repository");
        return match client::init().await {
            Ok(client) => {
                let db_name: String = config::get::<String>("NAMESPACE").unwrap_or_else(|| {
                    error!("DB Name configuration is not set");
                    std::process::exit(2);
                });

                let coll_name: String = config::get::<String>("APP").unwrap_or_else(|| {
                    error!("Collection Name configuration is not set");
                    std::process::exit(2);
                });
                let database = client.database(&db_name);
                let collection = database.collection(&coll_name);

                Ok(Repository {
                    client,
                    database,
                    collection,
                })
            }
            Err(e) => {
                error!("Failed to initialize MongoDB client: {}", e);
                Err(e)
            }
        };
    }

    pub fn get_database(&self) -> &Database {
        &self.database
    }
}
