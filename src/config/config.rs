use config::{Case, Config, ConfigError};
use log::{debug, error, info, trace};
use std::{env, fmt::Debug, sync::OnceLock};

static CONFIG: OnceLock<Config> = OnceLock::new();

pub fn get<'a, T>(key: &str) -> Option<T>
where
    T: serde::Deserialize<'a> + Debug,
{
    if CONFIG.get().is_none() {
        info!("Loading configurations....");

        let file_src = config::File::with_name("config");
        let env_src = config::Environment::with_prefix("VAPI")
            .prefix_separator("_")
            .convert_case(Case::UpperSnake);

        let config = Config::builder()
            .add_source(file_src)
            .add_source(env_src)
            .build();

        match config {
            Ok(result) => {
                info!("Configurations loaded successfully");
                let _ = CONFIG.set(result);
            }
            Err(e) => panic!("Config error {}", e),
        }
    }

    let config = CONFIG.get().unwrap();
    //TODO: CHECK CASE SENSITIVITY
    match config.get::<T>(&key) {
        Ok(val) => Some(val),
        Err(e) => match e {
            ConfigError::NotFound(_) => {
                return None;
            }
            _ => {
                error!("Error getting config value for key {}: {}", key, e);
                return None;
            }
        },
    }
}
