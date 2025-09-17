use std::env;

#[derive(Debug, Clone)]
pub struct GatewayConfig {
    pub auth_service_url: String,
    pub company_service_url: String,
    pub chart_service_url: String,
    pub ledger_service_url: String,
    pub tax_service_url: String,
    pub payables_service_url: String,
    pub receivables_service_url: String,
    pub inventory_service_url: String,
    pub reporting_service_url: String,
    pub jwt_secret: String,
    pub request_timeout_seconds: u64,
}

impl GatewayConfig {
    pub fn from_env() -> anyhow::Result<Self> {
        Ok(Self {
            auth_service_url: env::var("AUTH_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3001".to_string()),
            company_service_url: env::var("COMPANY_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3002".to_string()),
            chart_service_url: env::var("CHART_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3003".to_string()),
            ledger_service_url: env::var("LEDGER_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3004".to_string()),
            tax_service_url: env::var("TAX_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3005".to_string()),
            payables_service_url: env::var("PAYABLES_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3006".to_string()),
            receivables_service_url: env::var("RECEIVABLES_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3007".to_string()),
            inventory_service_url: env::var("INVENTORY_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3008".to_string()),
            reporting_service_url: env::var("REPORTING_SERVICE_URL")
                .unwrap_or_else(|_| "http://localhost:3009".to_string()),
            jwt_secret: env::var("JWT_SECRET")?,
            request_timeout_seconds: env::var("REQUEST_TIMEOUT_SECONDS")
                .unwrap_or_else(|_| "30".to_string())
                .parse()?,
        })
    }
}