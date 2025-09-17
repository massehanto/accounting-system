// src/components/mod.rs
pub mod sidebar;
pub mod header;
pub mod forms;
pub mod tables;
pub mod layout;
pub mod common;

// Component re-exports - THESE WERE MISSING
pub use sidebar::*;
pub use header::*;
pub use forms::*;
pub use tables::*;
pub use layout::*;
pub use common::*;