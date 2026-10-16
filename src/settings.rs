//! Process settings.
//! requirement-environment — RUSTRBAC_* names, the mode-0600 file, loopback bind.
//! requirement-database — sqlite3 file rust_rbac.sqlite3, mysql database rust_rbac.
//! requirement-html-theme — vue3 or adminlte. This file does not open the database.

use std::collections::HashMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use crate::error::AppError;

pub const APP_DIR: &str = "RustRbac";
pub const FILE_NAME: &str = "database";
pub const SQLITE_NAME: &str = "rust_rbac.sqlite3";
pub const PRODUCT_DB: &str = "rust_rbac";
pub const TEST_DB: &str = "rust_rbac_test";

const ENGINES: &[&str] = &["sqlite3", "mysql"];
const THEMES: &[&str] = &["vue3", "adminlte"];
const LOOPBACK: &[&str] = &["127.0.0.1", "localhost", "::1"];
/// Names this product must not open. They belong to other products.
const DENIED_DATABASES: &[&str] = &["pyrbac", "pyrbac_test", "ruoyi", "ruoyi_db", "ry"];

#[derive(Clone)]
pub struct Settings {
    pub environ: HashMap<String, String>,
    pub home: String,
    file_cache: Option<HashMap<String, String>>,
}

impl Settings {
    pub fn from_process() -> Self {
        let mut environ = HashMap::new();
        for (key, value) in std::env::vars() {
            if key == "HOME" || key.starts_with("RUSTRBAC_") {
                environ.insert(key, value);
            }
        }
        let home = environ.get("HOME").cloned().unwrap_or_default();
        Self {
            environ,
            home,
            file_cache: None,
        }
    }

    pub fn from_map(environ: HashMap<String, String>, home: impl Into<String>) -> Self {
        Self {
            environ,
            home: home.into(),
            file_cache: None,
        }
    }

    pub fn app_home(&self) -> String {
        if let Some(override_home) = self.environ.get("RUSTRBAC_HOME") {
            if !override_home.is_empty() {
                return override_home.clone();
            }
        }
        if self.home.is_empty() {
            return String::new();
        }
        Path::new(&self.home)
            .join(".local")
            .join(APP_DIR)
            .to_string_lossy()
            .into_owned()
    }

    pub fn database_file(&self) -> String {
        let folder = self.app_home();
        if folder.is_empty() {
            return String::new();
        }
        Path::new(&folder)
            .join(FILE_NAME)
            .to_string_lossy()
            .into_owned()
    }

    fn file_map(&mut self) -> HashMap<String, String> {
        if let Some(cached) = &self.file_cache {
            return cached.clone();
        }
        let mut found = HashMap::new();
        let path = self.database_file();
        if !path.is_empty() && Path::new(&path).is_file() {
            let mode = fs::metadata(&path)
                .map(|meta| meta.permissions().mode() & 0o777)
                .unwrap_or(0);
            if mode == 0o600 {
                if let Ok(text) = fs::read_to_string(&path) {
                    for line in text.lines() {
                        if line.is_empty() || line.starts_with('#') || !line.contains('=') {
                            continue;
                        }
                        let (key, value) = line.split_once('=').unwrap();
                        found.insert(key.trim().to_string(), value.to_string());
                    }
                }
            }
        }
        self.file_cache = Some(found.clone());
        found
    }

    pub fn get(&mut self, name: &str) -> String {
        if let Some(current) = self.environ.get(name) {
            if !current.is_empty() {
                return current.clone();
            }
        }
        let filed = self.file_map();
        if let Some(value) = filed.get(name) {
            if !value.is_empty() {
                return value.clone();
            }
        }
        match name {
            "RUSTRBAC_DB_HOST" => "127.0.0.1".to_string(),
            "RUSTRBAC_DB_PORT" => "3306".to_string(),
            "RUSTRBAC_DB_USER" => PRODUCT_DB.to_string(),
            "RUSTRBAC_DB_NAME" => PRODUCT_DB.to_string(),
            "RUSTRBAC_BIND" => "127.0.0.1".to_string(),
            "RUSTRBAC_PORT" => "8765".to_string(),
            _ => String::new(),
        }
    }

    pub fn engine(&mut self) -> Result<String, AppError> {
        let value = self.get("RUSTRBAC_DB_ENGINE").trim().to_ascii_lowercase();
        if value.is_empty() {
            return Ok("sqlite3".to_string());
        }
        if !ENGINES.contains(&value.as_str()) {
            return Err(AppError::new(
                "RUSTRBAC_DB_ENGINE must be sqlite3 or mysql.",
                "unset RUSTRBAC_DB_ENGINE, or set sqlite3 or mysql",
            ));
        }
        Ok(value)
    }

    pub fn sqlite_path(&mut self) -> Result<String, AppError> {
        let override_path = self.get("RUSTRBAC_SQLITE_FILE");
        if !override_path.is_empty() {
            return Ok(override_path);
        }
        let folder = self.app_home();
        if folder.is_empty() {
            return Err(AppError::new(
                "The home folder is not set.",
                "export HOME or RUSTRBAC_HOME",
            ));
        }
        Ok(Path::new(&folder)
            .join(SQLITE_NAME)
            .to_string_lossy()
            .into_owned())
    }

    pub fn storage_label(&mut self) -> Result<String, AppError> {
        if self.engine()? == "sqlite3" {
            self.sqlite_path()
        } else {
            Ok(self.get("RUSTRBAC_DB_NAME"))
        }
    }

    pub fn choice_lines(&mut self) -> Result<Vec<String>, AppError> {
        let path = self.sqlite_path()?;
        let mark = if self.get("RUSTRBAC_SQLITE_FILE").is_empty() {
            " (default file)"
        } else {
            ""
        };
        let mysql_name = self.get("RUSTRBAC_DB_NAME");
        let current = self.engine()?;
        Ok(vec![
            "Database engine. Enter keeps the current engine.".to_string(),
            format!("1 sqlite3  {path}{mark}"),
            format!("2 mysql    {mysql_name}"),
            format!("Current: {current}"),
        ])
    }

    pub fn engine_sentence(&mut self) -> Result<String, AppError> {
        if self.engine()? == "sqlite3" {
            let path = self.sqlite_path()?;
            Ok(format!("Database engine: sqlite3 {path}."))
        } else {
            let name = self.get("RUSTRBAC_DB_NAME");
            Ok(format!("Database engine: mysql {name}."))
        }
    }

    pub fn apply_choice(&mut self, token: &str) -> Result<String, AppError> {
        let text = token.trim().to_ascii_lowercase();
        if text.is_empty() {
            return self.engine_sentence();
        }
        let engine = if text == "1" || text == "sqlite3" {
            "sqlite3"
        } else if text == "2" || text == "mysql" {
            "mysql"
        } else {
            return Err(AppError::new(
                "The database engine must be sqlite3 or mysql.",
                "choose 1 for the sqlite3 file or 2 for mysql",
            ));
        };
        let env = self
            .environ
            .get("RUSTRBAC_DB_ENGINE")
            .cloned()
            .unwrap_or_default();
        let env = env.trim().to_ascii_lowercase();
        if !env.is_empty() {
            if !ENGINES.contains(&env.as_str()) {
                return Err(AppError::new(
                    "RUSTRBAC_DB_ENGINE must be sqlite3 or mysql.",
                    "unset RUSTRBAC_DB_ENGINE, or set sqlite3 or mysql",
                ));
            }
            if env != engine {
                return Err(AppError::new(
                    "RUSTRBAC_DB_ENGINE is set in the environment.",
                    "unset RUSTRBAC_DB_ENGINE to store the choice in the database file",
                ));
            }
            let sentence = self.engine_sentence()?;
            return Ok(format!("{sentence} The environment wins."));
        }
        self.save_engine(engine)?;
        self.engine_sentence()
    }

    pub fn save_engine(&mut self, engine: &str) -> Result<(), AppError> {
        let engine = engine.trim().to_ascii_lowercase();
        if !ENGINES.contains(&engine.as_str()) {
            return Err(AppError::new(
                "The database engine must be sqlite3 or mysql.",
                "choose 1 for the sqlite3 file or 2 for mysql",
            ));
        }
        self.write_setting("RUSTRBAC_DB_ENGINE", &engine, "choose the engine again")
    }

    pub fn theme(&mut self) -> Result<String, AppError> {
        self.file_cache = None;
        let value = self.get("RUSTRBAC_THEME").trim().to_ascii_lowercase();
        if value.is_empty() {
            return Ok("vue3".to_string());
        }
        if !THEMES.contains(&value.as_str()) {
            return Err(AppError::new(
                "RUSTRBAC_THEME must be vue3 or adminlte.",
                "unset RUSTRBAC_THEME, or set vue3 or adminlte",
            ));
        }
        Ok(value)
    }

    pub fn theme_lines(&self) -> Vec<String> {
        vec![
            "Theme. Enter uses vue3.".to_string(),
            "1 vue3".to_string(),
            "2 adminlte".to_string(),
        ]
    }

    pub fn theme_sentence(&mut self) -> Result<String, AppError> {
        Ok(format!("Theme: {}.", self.theme()?))
    }

    pub fn apply_theme(&mut self, token: &str) -> Result<String, AppError> {
        let text = token.trim().to_ascii_lowercase();
        let theme = if text.is_empty() || text == "1" || text == "vue3" {
            "vue3"
        } else if text == "2" || text == "adminlte" {
            "adminlte"
        } else {
            return Err(AppError::new(
                "The theme must be vue3 or adminlte.",
                "choose 1 for vue3 or 2 for adminlte",
            ));
        };
        let env = self
            .environ
            .get("RUSTRBAC_THEME")
            .cloned()
            .unwrap_or_default();
        let env = env.trim().to_ascii_lowercase();
        if !env.is_empty() {
            if !THEMES.contains(&env.as_str()) {
                return Err(AppError::new(
                    "RUSTRBAC_THEME must be vue3 or adminlte.",
                    "unset RUSTRBAC_THEME, or set vue3 or adminlte",
                ));
            }
            if env != theme {
                return Err(AppError::new(
                    "RUSTRBAC_THEME is set in the environment.",
                    "unset RUSTRBAC_THEME to store the choice in the database file",
                ));
            }
            let sentence = self.theme_sentence()?;
            return Ok(format!("{sentence} The environment wins."));
        }
        self.save_theme(theme)?;
        self.theme_sentence()
    }

    pub fn save_theme(&mut self, theme: &str) -> Result<(), AppError> {
        let theme = theme.trim().to_ascii_lowercase();
        if !THEMES.contains(&theme.as_str()) {
            return Err(AppError::new(
                "The theme must be vue3 or adminlte.",
                "choose 1 for vue3 or 2 for adminlte",
            ));
        }
        self.write_setting("RUSTRBAC_THEME", &theme, "choose the theme again")
    }

    fn write_setting(&mut self, key: &str, value: &str, again: &str) -> Result<(), AppError> {
        let folder = self.app_home();
        if folder.is_empty() {
            return Err(AppError::new(
                "The home folder is not set.",
                "export HOME or RUSTRBAC_HOME",
            ));
        }
        fs::create_dir_all(&folder).map_err(|_| {
            AppError::new(
                "The home folder could not be created.",
                format!("check RUSTRBAC_HOME, then {again}"),
            )
        })?;
        let _ = fs::set_permissions(&folder, mode_perm(0o700));
        let path = self.database_file();
        let mut lines = Vec::new();
        if Path::new(&path).is_file() {
            let mode = fs::metadata(&path)
                .map(|meta| meta.permissions().mode() & 0o777)
                .unwrap_or(0);
            if mode != 0o600 {
                return Err(AppError::new(
                    "The database file is not mode 0600.",
                    "chmod 0600 the database file, or remove it and choose again",
                ));
            }
            let text = fs::read_to_string(&path).map_err(|_| {
                AppError::new(
                    "The database file could not be read.",
                    format!("check the database file, then {again}"),
                )
            })?;
            lines = text.lines().map(|line| line.to_string()).collect();
        }
        let mut found = false;
        let mut rewritten = Vec::new();
        for line in lines {
            let line_key = line
                .split_once('=')
                .map(|(item, _)| item.trim())
                .unwrap_or("");
            if line_key == key {
                rewritten.push(format!("{key}={value}"));
                found = true;
            } else {
                rewritten.push(line);
            }
        }
        if !found {
            rewritten.push(format!("{key}={value}"));
        }
        let payload = rewritten.join("\n") + "\n";
        atomic_private(&path, payload.as_bytes())?;
        self.file_cache = None;
        Ok(())
    }

    pub fn password(&mut self) -> String {
        self.get("RUSTRBAC_DB_PASSWORD")
    }

    pub fn require_password(&mut self) -> Result<String, AppError> {
        let value = self.password();
        if value.is_empty() {
            Err(AppError::new(
                "The database password is not set.",
                "export RUSTRBAC_DB_PASSWORD and run rust-rbac db-init",
            ))
        } else {
            Ok(value)
        }
    }

    pub fn database(&mut self) -> Result<DbConfig, AppError> {
        let name = self.get("RUSTRBAC_DB_NAME");
        if name.is_empty()
            || !name
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
        {
            return Err(AppError::new(
                "RUSTRBAC_DB_NAME must be letters, digits, and underscores.",
                "export RUSTRBAC_DB_NAME=rust_rbac",
            ));
        }
        if DENIED_DATABASES.contains(&name.as_str()) {
            return Err(AppError::new(
                format!("This product does not open the database {name}."),
                "export RUSTRBAC_DB_NAME=rust_rbac or RUSTRBAC_DB_NAME=rust_rbac_test",
            ));
        }
        let port: i64 = self.get("RUSTRBAC_DB_PORT").parse().unwrap_or(0);
        if !(1..=65535).contains(&port) {
            return Err(AppError::new(
                "RUSTRBAC_DB_PORT must be a port number.",
                "export RUSTRBAC_DB_PORT=3306",
            ));
        }
        let password = self.require_password()?;
        Ok(DbConfig {
            host: self.get("RUSTRBAC_DB_HOST"),
            port: port as u16,
            user: self.get("RUSTRBAC_DB_USER"),
            password,
            database: name,
        })
    }

    pub fn bind(&mut self) -> Result<(String, u16), AppError> {
        let host = self.get("RUSTRBAC_BIND");
        let port_text = self.get("RUSTRBAC_PORT");
        let port: i64 = port_text.parse().unwrap_or(-1);
        if !(0..=65535).contains(&port) {
            return Err(AppError::new(
                "RUSTRBAC_PORT must be a port number. 0 asks the system for a free port.",
                "export RUSTRBAC_PORT=8765",
            ));
        }
        let allow = self
            .environ
            .get("RUSTRBAC_ALLOW_REMOTE")
            .map(String::as_str)
            == Some("yes");
        if !LOOPBACK.contains(&host.as_str()) && !allow {
            return Err(AppError::new(
                format!("Refusing to listen on {host}."),
                "keep RUSTRBAC_BIND=127.0.0.1, or export RUSTRBAC_ALLOW_REMOTE=yes",
            ));
        }
        Ok((host, port as u16))
    }
}

#[derive(Clone, Debug)]
pub struct DbConfig {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub password: String,
    pub database: String,
}

fn mode_perm(mode: u32) -> fs::Permissions {
    fs::Permissions::from_mode(mode)
}

fn atomic_private(path: &str, payload: &[u8]) -> Result<(), AppError> {
    let temporary: PathBuf = PathBuf::from(format!("{path}.tmp"));
    {
        let mut file = fs::OpenOptions::new()
            .write(true)
            .create(true)
            .truncate(true)
            .mode(0o600)
            .open(&temporary)
            .map_err(|_| {
                AppError::new(
                    "The database file could not be written.",
                    "check the home folder, then choose the engine again",
                )
            })?;
        file.write_all(payload).map_err(|_| {
            AppError::new(
                "The database file could not be written.",
                "check the home folder, then choose the engine again",
            )
        })?;
    }
    let _ = fs::set_permissions(&temporary, mode_perm(0o600));
    fs::rename(&temporary, path).map_err(|_| {
        AppError::new(
            "The database file could not be written.",
            "check the home folder, then choose the engine again",
        )
    })?;
    let _ = fs::set_permissions(path, mode_perm(0o600));
    Ok(())
}
