use crate::config::GatewayConfig;
use std::collections::HashMap;

pub struct ServiceRegistry {
    services: HashMap<String, ServiceEndpoint>,
}

#[derive(Debug, Clone)]
pub struct ServiceEndpoint {
    pub name: String,
    pub base_url: String,
    pub health_path: String,
    pub is_healthy: bool,
}

impl ServiceRegistry {
    pub fn new(config: &GatewayConfig) -> Self {
        let mut services = HashMap::new();
        
        services.insert("auth".to_string(), ServiceEndpoint {
            name: "auth-service".to_string(),
            base_url: config.auth_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("company".to_string(), ServiceEndpoint {
            name: "company-management-service".to_string(),
            base_url: config.company_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("accounts".to_string(), ServiceEndpoint {
            name: "chart-of-accounts-service".to_string(),
            base_url: config.chart_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("ledger".to_string(), ServiceEndpoint {
            name: "general-ledger-service".to_string(),
            base_url: config.ledger_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("tax".to_string(), ServiceEndpoint {
            name: "indonesian-tax-service".to_string(),
            base_url: config.tax_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("payables".to_string(), ServiceEndpoint {
            name: "accounts-payable-service".to_string(),
            base_url: config.payables_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("receivables".to_string(), ServiceEndpoint {
            name: "accounts-receivable-service".to_string(),
            base_url: config.receivables_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("inventory".to_string(), ServiceEndpoint {
            name: "inventory-management-service".to_string(),
            base_url: config.inventory_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });
        
        services.insert("reporting".to_string(), ServiceEndpoint {
            name: "reporting-service".to_string(),
            base_url: config.reporting_service_url.clone(),
            health_path: "/health".to_string(),
            is_healthy: true,
        });

        Self { services }
    }

    pub fn get_service(&self, service_name: &str) -> Option<&ServiceEndpoint> {
        self.services.get(service_name)
    }

    pub fn get_service_url(&self, service_name: &str, path: &str) -> Option<String> {
        self.get_service(service_name)
            .map(|endpoint| format!("{}{}", endpoint.base_url, path))
    }

    pub async fn proxy_request(
        &self,
        service_name: &str,
        path: &str,
        method: axum::http::Method,
        headers: axum::http::HeaderMap,
        body: axum::body::Bytes,
    ) -> Result<reqwest::Response, reqwest::Error> {
        let url = self.get_service_url(service_name, path)
            .ok_or_else(|| reqwest::Error::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Service {} not found", service_name)
            )))?;

        let client = reqwest::Client::new();
        let mut request = match method {
            axum::http::Method::GET => client.get(&url),
            axum::http::Method::POST => client.post(&url),
            axum::http::Method::PUT => client.put(&url),
            axum::http::Method::DELETE => client.delete(&url),
            axum::http::Method::PATCH => client.patch(&url),
            _ => client.get(&url),
        };

        // Forward headers
        for (key, value) in headers.iter() {
            if let (Ok(key_str), Ok(value_str)) = (key.as_str(), value.to_str()) {
                // Skip some headers that shouldn't be forwarded
                if !["host", "content-length", "transfer-encoding"].contains(&key_str.to_lowercase().as_str()) {
                    request = request.header(key_str, value_str);
                }
            }
        }

        // Add body if present
        if !body.is_empty() {
            request = request.body(body.to_vec());
        }

        request.send().await
    }
}