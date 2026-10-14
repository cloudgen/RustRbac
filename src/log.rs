//! Status file. It is not sys_oper_log and it does not store a password.
//! requirement-rust-logging — StatusLog::open is constructed from main.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

pub struct StatusLog {
    path: String,
}

impl StatusLog {
    pub fn open(logdir: &str, app_name: &str) -> Self {
        let directory = if logdir.is_empty() {
            let home = std::env::var("RUSTRBAC_HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .or_else(|| std::env::var("HOME").ok())
                .unwrap_or_else(|| "/tmp".to_string());
            if std::env::var("RUSTRBAC_HOME")
                .ok()
                .filter(|value| !value.is_empty())
                .is_some()
                || logdir.is_empty() && std::env::var("HOME").is_ok()
            {
                let base = std::env::var("RUSTRBAC_HOME").unwrap_or_else(|_| {
                    Path::new(&home)
                        .join(".local")
                        .join("RustRbac")
                        .to_string_lossy()
                        .into_owned()
                });
                Path::new(&base).join("log").to_string_lossy().into_owned()
            } else {
                String::new()
            }
        } else {
            logdir.to_string()
        };
        if !directory.is_empty() {
            let _ = fs::create_dir_all(&directory);
        }
        let path = if directory.is_empty() {
            String::new()
        } else {
            Path::new(&directory)
                .join(format!("{app_name}-status.log"))
                .to_string_lossy()
                .into_owned()
        };
        let log = Self { path };
        log.log_message("StatusLog open", "INFO", "StatusLog");
        log
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn log_message(&self, message: &str, level: &str, component: &str) {
        if self.path.is_empty() {
            return;
        }
        let line = format!("{level} {component} {message}\n");
        if let Ok(mut file) = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
        {
            let _ = file.write_all(line.as_bytes());
        }
    }
}
