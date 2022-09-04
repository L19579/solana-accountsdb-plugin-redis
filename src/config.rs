use {
    anyhow::Result,
    serde_json,
    serde::Deserialize,
    std::{
        io::prelude::*,
        fs::OpenOptions,
    },
};

#[derive(Deserialize, Clone, Debug)]
pub struct RedisDbCredentials{
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: String,
    pub database: String,
}

#[derive(Deserialize, Clone, Debug)]
pub struct GeyserConfig{
    pub redis_db_credentials: RedisDbCredentials,
    pub account_data_notifications_enabled: bool,
    pub transaction_data_notifications_enabled: bool,
    pub accounts: Vec<String>,
    pub preload_targeted_accounts: bool,
}

impl GeyserConfig {
    pub fn load(config_path: &str) -> Result<Self>{
        let mut file = OpenOptions::new()
            .read(true)
            .open(config_path)?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;

        Ok(serde_json::from_str::<GeyserConfig>(&contents)?)
    }
}
