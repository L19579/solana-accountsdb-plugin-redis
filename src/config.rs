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
    pub account_data_notifications_enabled: Option<bool>,
    pub transaction_data_notifications_enabled: Option<bool>,
    pub ignore_system_accounts: Option<bool>,
    pub ignore_vote_transactions: Option<bool>,
    pub accounts: Option<Vec<String>>,
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
