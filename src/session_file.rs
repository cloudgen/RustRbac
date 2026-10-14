//! Terminal session file.
//! requirement-environment — mode 0600 under the application home.

use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;

use crate::error::AppError;
use crate::settings::Settings;

pub struct SessionFile {
    home: String,
}

impl SessionFile {
    pub fn new(settings: &Settings) -> Self {
        Self {
            home: settings.app_home(),
        }
    }

    pub fn path(&self) -> String {
        if self.home.is_empty() {
            return String::new();
        }
        Path::new(&self.home)
            .join("session")
            .to_string_lossy()
            .into_owned()
    }

    pub fn read(&self) -> String {
        let path = self.path();
        if path.is_empty() || !Path::new(&path).is_file() {
            return String::new();
        }
        let mode = fs::metadata(&path)
            .map(|meta| meta.permissions().mode() & 0o777)
            .unwrap_or(0);
        if mode != 0o600 {
            return String::new();
        }
        fs::read_to_string(&path)
            .unwrap_or_default()
            .trim()
            .to_string()
    }

    pub fn write(&self, token: &str) -> Result<(), AppError> {
        if self.home.is_empty() {
            return Err(AppError::new(
                "The home folder is not set.",
                "export HOME or RUSTRBAC_HOME",
            ));
        }
        fs::create_dir_all(&self.home).map_err(|_| {
            AppError::new(
                "The session folder could not be created.",
                "check RUSTRBAC_HOME",
            )
        })?;
        let _ = fs::set_permissions(&self.home, fs::Permissions::from_mode(0o700));
        let path = self.path();
        let temporary = format!("{path}.tmp");
        {
            let mut file = fs::OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&temporary)
                .map_err(|_| {
                    AppError::new(
                        "The session file could not be written.",
                        "check RUSTRBAC_HOME",
                    )
                })?;
            file.write_all(format!("{token}\n").as_bytes())
                .map_err(|_| {
                    AppError::new(
                        "The session file could not be written.",
                        "check RUSTRBAC_HOME",
                    )
                })?;
        }
        let _ = fs::set_permissions(&temporary, fs::Permissions::from_mode(0o600));
        fs::rename(&temporary, &path).map_err(|_| {
            AppError::new(
                "The session file could not be written.",
                "check RUSTRBAC_HOME",
            )
        })?;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        Ok(())
    }

    pub fn clear(&self) {
        let path = self.path();
        if !path.is_empty() {
            let _ = fs::remove_file(path);
        }
    }
}
