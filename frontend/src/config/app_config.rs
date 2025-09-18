#[derive(Debug, Clone)]
pub struct AppConfig {
    pub api_base_url: String,
    pub app_name: String,
    pub version: String,
    pub environment: Environment,
    pub features: FeatureFlags,
}

#[derive(Debug, Clone)]
pub enum Environment {
    Development,
    Staging,
    Production,
}

#[derive(Debug, Clone)]
pub struct FeatureFlags {
    pub offline_support: bool,
    pub background_sync: bool,
    pub push_notifications: bool,
    pub advanced_reports: bool,
}

impl AppConfig {
    pub fn load() -> Self {
        Self {
            api_base_url: "/api".to_string(),
            app_name: "Sistem Akuntansi Indonesia".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            environment: if cfg!(debug_assertions) { 
                Environment::Development 
            } else { 
                Environment::Production 
            },
            features: FeatureFlags {
                offline_support: true,
                background_sync: true,
                push_notifications: false,
                advanced_reports: true,
            },
        }
    }
    
    pub fn is_development(&self) -> bool {
        matches!(self.environment, Environment::Development)
    }
    
    pub fn is_production(&self) -> bool {
        matches!(self.environment, Environment::Production)
    }
}