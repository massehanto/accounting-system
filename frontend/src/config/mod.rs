// frontend/src/config/mod.rs
pub mod app_config;

pub use app_config::*;

use std::sync::OnceLock;

static CONFIG: OnceLock<AppConfig> = OnceLock::new();

pub fn get_config() -> &'static AppConfig {
    CONFIG.get_or_init(|| AppConfig::load())
}