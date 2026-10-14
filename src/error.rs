//! Operator-facing failures.
//! requirement-rust-error-handling — the message and the next step stay together.

use std::fmt;

#[derive(Debug, Clone)]
pub struct AppError {
    pub message: String,
    pub next: String,
}

impl AppError {
    pub fn new(message: impl Into<String>, next: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            next: next.into(),
        }
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{}", self.message)
    }
}

impl std::error::Error for AppError {}
