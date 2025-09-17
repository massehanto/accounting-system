use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ServiceConfig {
    pub name: String,
    pub version: String,
    pub bind_address: String,
    pub database_url: String,
    pub log_level: String,
}

impl ServiceConfig {
    pub fn from_env(service_name: &str) -> anyhow::Result<Self> {
        let service_key = service_name.to_uppercase().replace("-", "_");
        
        Ok(Self {
            name: service_name.to_string(),
            version: env::var("SERVICE_VERSION").unwrap_or_else(|_| "1.0.0".to_string()),
            bind_address: env::var(format!("{}_SERVICE_BIND", service_key))
                .unwrap_or_else(|_| "0.0.0.0:3000".to_string()),
            database_url: env::var(format!("{}_DATABASE_URL", service_key))?,
            log_level: env::var("RUST_LOG")
                .unwrap_or_else(|_| "info".to_string()),
        })
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct DatabaseConfig {
    pub max_connections: u32,
    pub min_connections: u32,
    pub connection_timeout: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            max_connections: env::var("DB_MAX_CONNECTIONS")
                .unwrap_or_else(|_| "20".to_string())
                .parse()
                .unwrap_or(20),
            min_connections: env::var("DB_MIN_CONNECTIONS")
                .unwrap_or_else(|_| "5".to_string())
                .parse()
                .unwrap_or(5),
            connection_timeout: 30,
        }
    }
}