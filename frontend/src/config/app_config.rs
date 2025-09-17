// src/config/app_config.rs
pub struct AppConfig {
    pub api_base_url: String,
    pub app_name: String,
    pub version: String,
    pub environment: Environment,
}

#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

impl AppConfig {
    pub fn load() -> Self {
        Self {
            api_base_url: "/api".to_string(),
            app_name: "Sistem Akuntansi Indonesia".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: Environment::Development,
        }
    }
}