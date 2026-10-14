//! RustRbac library. The binary is `rust-rbac`.
//! requirement-rust-modules — one module per concern.
//! requirement-domain-rust-rbac — online RBAC and centralized configuration.

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_NAME: &str = "RustRbac";
pub const CONSOLE_NAME: &str = "rust-rbac";

pub mod about;
pub mod access;
pub mod audit;
pub mod catalog;
pub mod cli;
pub mod desk;
pub mod error;
pub mod local_login;
pub mod log;
pub mod menu;
pub mod password;
pub mod report;
pub mod schema;
pub mod schema_sql;
pub mod session_file;
pub mod settings;
pub mod store;
pub mod texts;
pub mod web;
