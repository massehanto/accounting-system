// src/services/api/mod.rs
pub mod client;
pub mod auth_api;
pub mod accounting_api;
pub mod reports_api;
pub mod error_handling;

pub use client::*;
pub use auth_api::*;
pub use accounting_api::*;
pub use reports_api::*;
pub use error_handling::*;