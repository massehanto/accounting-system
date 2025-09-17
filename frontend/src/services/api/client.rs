// frontend/src/services/api/client.rs
use gloo_net::http::Request;
use crate::utils;

pub struct ApiClient {
    base_url: String,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self { base_url }
    }

    pub async fn get(&self, endpoint: &str) -> Result<gloo_net::http::Response, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = Request::get(&url);
        
        if let Some(token) = utils::get_token() {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }
        
        request.send().await
            .map_err(|e| format!("Request failed: {}", e))
    }

    pub async fn post<T: serde::Serialize>(&self, endpoint: &str, body: &T) -> Result<gloo_net::http::Response, String> {
        let url = format!("{}{}", self.base_url, endpoint);
        let mut request = Request::post(&url);
        
        if let Some(token) = utils::get_token() {
            request = request.header("Authorization", &format!("Bearer {}", token));
        }
        
        request.json(body)
            .map_err(|e| format!("Serialization failed: {}", e))?
            .send().await
            .map_err(|e| format!("Request failed: {}", e))
    }
}