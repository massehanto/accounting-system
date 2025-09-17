// frontend/src/error.rs
use leptos::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AppError {
    NetworkError(String),
    AuthenticationError(String),
    ValidationError(String),
    BusinessLogicError(String),
    UnexpectedError(String),
}

impl std::fmt::Display for AppError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AppError::NetworkError(msg) => write!(f, "Kesalahan jaringan: {}", msg),
            AppError::AuthenticationError(msg) => write!(f, "Kesalahan autentikasi: {}", msg),
            AppError::ValidationError(msg) => write!(f, "Kesalahan validasi: {}", msg),
            AppError::BusinessLogicError(msg) => write!(f, "Kesalahan bisnis: {}", msg),
            AppError::UnexpectedError(msg) => write!(f, "Kesalahan tidak terduga: {}", msg),
        }
    }
}

// Enhanced error conversion implementations
impl From<String> for AppError {
    fn from(msg: String) -> Self {
        AppError::UnexpectedError(msg)
    }
}

impl From<gloo_net::Error> for AppError {
    fn from(err: gloo_net::Error) -> Self {
        AppError::NetworkError(err.to_string())
    }
}

// Global error context
#[derive(Clone)]
pub struct ErrorContext {
    pub current_error: ReadSignal<Option<AppError>>,
    pub set_error: WriteSignal<Option<AppError>>,
    pub clear_error: Box<dyn Fn()>,
}

pub fn provide_error_context() -> ErrorContext {
    let (current_error, set_error) = create_signal(None::<AppError>);
    let clear_error = Box::new(move || set_error.set(None));
    
    ErrorContext {
        current_error,
        set_error,
        clear_error,
    }
}