use std::collections::HashMap;
use std::env;

#[derive(Clone)]
pub struct ServiceRegistry {
    services: HashMap<String, String>,
}

impl ServiceRegistry {
    pub fn new() -> Self {
        let mut services = HashMap::new();
        
        services.insert(
            "chart-of-accounts".to_string(),
            env::var("CHART_OF_ACCOUNTS_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3003".to_string())
        );
        
        services.insert(
            "general-ledger".to_string(),
            env::var("GENERAL_LEDGER_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3004".to_string())
        );
        
        services.insert(
            "accounts-payable".to_string(),
            env::var("ACCOUNTS_PAYABLE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3006".to_string())
        );
        
        services.insert(
            "accounts-receivable".to_string(),
            env::var("ACCOUNTS_RECEIVABLE_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3007".to_string())
        );
        
        services.insert(
            "inventory".to_string(),
            env::var("INVENTORY_MANAGEMENT_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3008".to_string())
        );
        
        services.insert(
            "tax".to_string(),
            env::var("INDONESIAN_TAX_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3005".to_string())
        );

        Self { services }
    }

    pub fn get_service_url(&self, service: &str) -> Option<&String> {
        self.services.get(service)
    }

    pub async fn call_service(
        &self,
        client: &reqwest::Client,
        service: &str,
        endpoint: &str,
        headers: &axum::http::HeaderMap,
    ) -> common::ServiceResult<serde_json::Value> {
        let service_url = self.get_service_url(service)
            .ok_or_else(|| common::ServiceError::ExternalService(
                format!("Service {} not found", service)
            ))?;
        
        let url = format!("{}{}", service_url, endpoint);
        let mut request = client.get(&url)
            .timeout(std::time::Duration::from_secs(30));
        
        // Forward authentication headers
        if let Some(user_id) = headers.get("X-User-ID") {
            request = request.header("X-User-ID", user_id);
        }
        if let Some(company_id) = headers.get("X-Company-ID") {
            request = request.header("X-Company-ID", company_id);
        }
        if let Some(auth) = headers.get("Authorization") {
            request = request.header("Authorization", auth);
        }
        
        let response = request.send().await
            .map_err(|e| common::ServiceError::ExternalService(
                format!("Failed to call {}: {}", service, e)
            ))?;
        
        if !response.status().is_success() {
            return Err(common::ServiceError::ExternalService(
                format!("Service {} returned status: {}", service, response.status())
            ));
        }
        
        response.json().await
            .map_err(|e| common::ServiceError::ExternalService(
                format!("Failed to parse response from {}: {}", service, e)
            ))
    }
}