use std::sync::LazyLock;

use config::{Config, FileFormat};
use serde::Deserialize;

pub static CONFIG: LazyLock<AppConfig> = LazyLock::new(|| {
    let config = Config::builder()
        .add_source(config::File::new("Config.toml", FileFormat::Toml))
        .build()
        .expect("Failed to read configuration.");

    config
        .try_deserialize::<AppConfig>()
        .expect("Failed to deserialize configuration.")
});

#[derive(Deserialize, Clone)]
pub struct AppConfig {
    pub port: u16,
    pub database: DatabaseConfig,
}

#[derive(Deserialize, Clone)]
pub struct DatabaseConfig {
    pub username: String,
    pub password: String,
    pub host: String,
    pub port: u16,
    pub name: String,
}

impl DatabaseConfig {
    pub fn conn_string(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/{}",
            self.username, self.password, self.host, self.port, self.name
        )
    }

    pub fn conn_string_without_db(&self) -> String {
        format!(
            "postgres://{}:{}@{}:{}/",
            self.username, self.password, self.host, self.port
        )
    }
}
