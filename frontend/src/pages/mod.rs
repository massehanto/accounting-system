// frontend/src/pages/mod.rs
pub mod dashboard;
pub mod login;
pub mod chart_of_accounts;
pub mod journal_entries;
pub mod company_management;
pub mod accounts_payable;
pub mod accounts_receivable;
pub mod inventory_management;
pub mod tax_management;
pub mod reports;

pub use dashboard::*;
pub use login::*;
pub use chart_of_accounts::*;
pub use journal_entries::*;
pub use company_management::*;
pub use accounts_payable::*;
pub use accounts_receivable::*;
pub use inventory_management::*;
pub use tax_management::*;
pub use reports::*;