use crate::models::{Claims, RefreshTokenClaims, TokenPair, AuthUser};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use std::env;
use thiserror::Error;
use uuid::Uuid;

#[derive(Error, Debug)]
pub enum JwtError {
    #[error("Failed to encode JWT: {0}")]
    Encoding(#[from] jsonwebtoken::errors::Error),
    #[error("Invalid token")]
    InvalidToken,
    #[error("Token expired")]
    TokenExpired,
    #[error("Missing JWT secret")]
    MissingSecret,
}

pub struct JwtManager {
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    algorithm: Algorithm,
}

impl JwtManager {
    pub fn new() -> Result<Self, JwtError> {
        let secret = env::var("JWT_SECRET")
            .map_err(|_| JwtError::MissingSecret)?;
        
        let encoding_key = EncodingKey::from_secret(secret.as_bytes());
        let decoding_key = DecodingKey::from_secret(secret.as_bytes());
        
        Ok(Self {
            encoding_key,
            decoding_key,
            algorithm: Algorithm::HS256,
        })
    }

    pub fn generate_token_pair(&self, user: &AuthUser) -> Result<TokenPair, JwtError> {
        let now = Utc::now();
        let access_token_expiry = now + Duration::hours(1); // 1 hour for access token
        let refresh_token_expiry = now + Duration::days(30); // 30 days for refresh token
        
        let jti = Uuid::new_v4().to_string();
        
        // Access token claims
        let access_claims = Claims {
            sub: user.id.to_string(),
            email: user.email.clone(),
            company_id: user.company_id.to_string(),
            full_name: user.full_name.clone(),
            exp: access_token_expiry.timestamp(),
            iat: now.timestamp(),
            jti: jti.clone(),
        };

        // Refresh token claims
        let refresh_claims = RefreshTokenClaims {
            jti: jti.clone(),
            user_id: user.id.to_string(),
            exp: refresh_token_expiry.timestamp(),
            iat: now.timestamp(),
        };

        let header = Header::new(self.algorithm);
        
        let access_token = encode(&header, &access_claims, &self.encoding_key)?;
        let refresh_token = encode(&header, &refresh_claims, &self.encoding_key)?;

        Ok(TokenPair {
            access_token,
            refresh_token,
            expires_in: 3600, // 1 hour in seconds
            token_type: "Bearer".to_string(),
        })
    }

    pub fn verify_access_token(&self, token: &str) -> Result<Claims, JwtError> {
        let validation = Validation::new(self.algorithm);
        
        match decode::<Claims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                let now = Utc::now().timestamp();
                if token_data.claims.exp < now {
                    return Err(JwtError::TokenExpired);
                }
                Ok(token_data.claims)
            }
            Err(_) => Err(JwtError::InvalidToken),
        }
    }

    pub fn verify_refresh_token(&self, token: &str) -> Result<RefreshTokenClaims, JwtError> {
        let validation = Validation::new(self.algorithm);
        
        match decode::<RefreshTokenClaims>(token, &self.decoding_key, &validation) {
            Ok(token_data) => {
                let now = Utc::now().timestamp();
                if token_data.claims.exp < now {
                    return Err(JwtError::TokenExpired);
                }
                Ok(token_data.claims)
            }
            Err(_) => Err(JwtError::InvalidToken),
        }
    }

    pub fn extract_token_from_header(auth_header: &str) -> Option<&str> {
        if auth_header.starts_with("Bearer ") {
            Some(&auth_header[7..])
        } else {
            None
        }
    }
}

impl Default for JwtManager {
    fn default() -> Self {
        Self::new().expect("Failed to create JWT manager")
    }
}