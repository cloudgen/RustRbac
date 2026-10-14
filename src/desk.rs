//! Shared command desk for the terminal and the web listener.
//! requirement-domain-rust-rbac — one service for both surfaces.

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::access::Access;
use crate::audit::Audit;
use crate::catalog::Catalog;
use crate::error::AppError;
use crate::local_login::{LocalLogin, SavedBody, SignInNote};
use crate::password::PasswordHash;
use crate::report::Report;
use crate::schema::Schema;
use crate::session_file::SessionFile;
use crate::settings::Settings;
use crate::store::{field, Store};
use crate::web::WebApp;

#[derive(Clone, Debug, Default)]
pub struct VerbOptions {
    pub user_name: String,
    pub password: String,
    pub nick_name: String,
    pub dept_id: String,
    pub role_id: String,
    pub user_id: String,
    pub status: String,
    pub menu_ids: String,
    pub data_scope: String,
    pub dept_ids: String,
    pub menu_name: String,
    pub parent_id: String,
    pub menu_type: String,
    pub perms: String,
    pub path: String,
    pub post_code: String,
    pub post_name: String,
    pub dept_name: String,
    pub config_key: String,
    pub config_name: String,
    pub config_value: String,
    pub dict_type: String,
    pub dict_label: String,
    pub dict_value: String,
    pub notice_title: String,
    pub notice_type: String,
    pub content: String,
    pub token: String,
    pub engine: String,
    pub force: bool,
}

#[derive(Debug)]
pub enum ResumeError {
    NeedAccount,
    Rejected(AppError),
    Other(AppError),
}

impl ResumeError {
    pub fn into_app(self) -> AppError {
        match self {
            Self::NeedAccount => AppError::new(
                "user-name is required.",
                "rust-rbac login --user-name NAME --password SECRET",
            ),
            Self::Rejected(error) | Self::Other(error) => error,
        }
    }
}

pub struct Desk {
    pub settings: Arc<Mutex<Settings>>,
    pub sessions: SessionFile,
    saved: LocalLogin,
    note: Mutex<SignInNote>,
    pub store: Arc<Store>,
    pub access: Arc<Access>,
    pub catalog: Arc<Catalog>,
    pub report: Report,
    pub schema: Arc<Schema>,
    web: Mutex<Option<WebApp>>,
}

impl Desk {
    pub fn new(environ: Option<HashMap<String, String>>) -> Self {
        let (map, home) = match environ {
            Some(map) => {
                let home = map.get("HOME").cloned().unwrap_or_default();
                (map, home)
            }
            None => process_settings(),
        };
        let settings = Arc::new(Mutex::new(Settings::from_map(map, home)));
        let (sessions, saved) = {
            let guard = settings.lock().expect("settings");
            (SessionFile::new(&guard), LocalLogin::new(guard.app_home()))
        };
        let store = Arc::new(Store::new(Arc::clone(&settings)));
        let audit = Arc::new(Audit::new(Arc::clone(&store)));
        let access = Arc::new(Access::new(Arc::clone(&store), Arc::clone(&audit)));
        let catalog = Arc::new(Catalog::new(
            Arc::clone(&store),
            Arc::clone(&access),
            Arc::clone(&audit),
        ));
        let schema = Arc::new(Schema::new(Arc::clone(&store), PasswordHash));
        Self {
            settings,
            sessions,
            saved,
            note: Mutex::new(SignInNote::None),
            store,
            access,
            catalog,
            report: Report,
            schema,
            web: Mutex::new(None),
        }
    }

    pub fn ensure_database(&self) -> Result<String, AppError> {
        self.schema.ensure()
    }

    pub fn db_init(&self) -> Result<String, AppError> {
        let state = self.ensure_database()?;
        let label = self.settings.lock().expect("settings").storage_label()?;
        Ok(format!("Database {label}: {state}."))
    }

    pub fn login(&self, user_name: &str, password: &str) -> Result<String, AppError> {
        let user = self
            .access
            .login(user_name, password, "127.0.0.1", "cli", "rust-rbac")?;
        self.sessions.write(field(&user, "token"))?;
        self.saved.save(user_name, password)?;
        self.set_note(SignInNote::Saved);
        Ok(format!("Signed in as {}.", field(&user, "user_name")))
    }

    pub fn resume_sign_in(&self) -> Result<String, ResumeError> {
        let token = self.sessions.read();
        if !token.is_empty() {
            match self.access.actor_from_token(&token) {
                Ok(Some(user)) => {
                    return Ok(format!("Signed in as {}.", field(&user, "user_name")));
                }
                Ok(None) => {}
                Err(error) => return Err(ResumeError::Other(error)),
            }
        }
        match self.saved.load() {
            SavedBody::Absent => Err(ResumeError::NeedAccount),
            SavedBody::Reject => {
                self.saved.clear().map_err(ResumeError::Other)?;
                self.set_note(SignInNote::Rejected);
                Err(ResumeError::Rejected(AppError::new(
                    "The saved sign-in was rejected.",
                    "rust-rbac login --user-name NAME --password SECRET",
                )))
            }
            SavedBody::Ready {
                user_name,
                password,
            } => match self.login(&user_name, &password) {
                Ok(text) => Ok(text),
                Err(error) => {
                    self.saved.clear().map_err(ResumeError::Other)?;
                    self.set_note(SignInNote::Rejected);
                    Err(ResumeError::Rejected(error))
                }
            },
        }
    }

    pub fn logout(&self) -> Result<String, AppError> {
        if self.saved.clear()? {
            self.set_note(SignInNote::Cleared);
        }
        let token = self.sessions.read();
        if token.is_empty() {
            return Err(not_signed_in());
        }
        self.access.logout(&token)?;
        self.sessions.clear();
        Ok("Signed out.".to_string())
    }

    pub fn take_note(&self) -> SignInNote {
        let mut note = self.note.lock().expect("note");
        std::mem::replace(&mut *note, SignInNote::None)
    }

    fn set_note(&self, note: SignInNote) {
        *self.note.lock().expect("note") = note;
    }

    pub fn whoami(&self) -> Result<String, AppError> {
        let actor = self.require_actor()?;
        self.access.describe(&actor)
    }

    pub fn serve_background(&self) -> Result<String, AppError> {
        self.ensure_database()?;
        let guard = self.web_mut();
        let url = guard.as_ref().expect("web").start_background()?;
        Ok(format!(
            "Web interface: {url}\nIt stays up while this process is open."
        ))
    }

    pub fn stop_web(&self) {
        if let Some(web) = self.web.lock().expect("web").as_ref() {
            web.stop();
        }
    }

    pub fn apply_engine(&self, token: &str) -> Result<String, AppError> {
        self.settings.lock().expect("settings").apply_choice(token)
    }

    pub fn engine_choice_lines(&self) -> Result<Vec<String>, AppError> {
        self.settings.lock().expect("settings").choice_lines()
    }

    pub fn execute(&self, verb: &str, options: &VerbOptions) -> Result<String, AppError> {
        if verb == "db-engine" {
            return self.apply_engine(&options.engine);
        }
        if verb == "db-init" {
            return self.db_init();
        }
        self.ensure_database()?;
        if verb == "login" {
            if options.user_name.trim().is_empty() && options.password.trim().is_empty() {
                return self.resume_sign_in().map_err(ResumeError::into_app);
            }
            return self.login(
                need(options, "user_name", "rust-rbac login --user-name NAME")?,
                need(options, "password", "rust-rbac login --user-name NAME")?,
            );
        }
        if verb == "logout" {
            return self.logout();
        }
        if verb == "whoami" {
            return self.whoami();
        }
        if verb == "serve" {
            return self.serve_background();
        }
        let actor = self.require_actor()?;
        match verb {
            "user-list" => Ok(self.report.users(&self.access.user_list(&actor)?)),
            "user-add" => {
                let row = self.access.user_add(
                    &actor,
                    need(options, "user_name", "rust-rbac user-add --user-name NAME --nick-name NAME --dept-id N --role-id N")?,
                    need(options, "nick_name", "set --nick-name")?,
                    whole(need(options, "dept_id", "set --dept-id")?, "dept-id")?,
                    whole(need(options, "role_id", "set --role-id")?, "role-id")?,
                    optional(options, "password"),
                )?;
                Ok(format!("Added {}.", field(&row, "user_name")))
            }
            "user-edit" => {
                let nick = optional(options, "nick_name");
                let dept = optional(options, "dept_id");
                let status = optional(options, "status");
                if nick.is_empty() && dept.is_empty() && status.is_empty() {
                    return Err(AppError::new(
                        "user-edit needs a name, a department, or a status.",
                        "rust-rbac user-edit --user-id N --nick-name NAME",
                    ));
                }
                let row = self.access.user_edit(
                    &actor,
                    whole(need(options, "user_id", "set --user-id")?, "user-id")?,
                    if nick.is_empty() { None } else { Some(nick) },
                    if dept.is_empty() {
                        None
                    } else {
                        Some(whole(dept, "dept-id")?)
                    },
                    if status.is_empty() {
                        None
                    } else {
                        Some(status)
                    },
                )?;
                Ok(format!("Updated {}.", field(&row, "user_name")))
            }
            "user-remove" => {
                let name = self.access.user_remove(
                    &actor,
                    whole(need(options, "user_id", "set --user-id")?, "user-id")?,
                )?;
                Ok(format!("Removed {name}."))
            }
            "user-reset-password" => {
                let name = self.access.user_reset_password(
                    &actor,
                    whole(need(options, "user_id", "set --user-id")?, "user-id")?,
                    need(options, "password", "set --password")?,
                )?;
                Ok(format!("Password updated for {name}."))
            }
            "role-list" => Ok(self.report.roles(&self.access.role_list(&actor)?)),
            "role-grant" => {
                let scope = optional(options, "data_scope");
                let row = self.access.role_grant(
                    &actor,
                    whole(need(options, "role_id", "set --role-id")?, "role-id")?,
                    &ints(need(options, "menu_ids", "set --menu-ids")?, "menu-id")?,
                    if scope.is_empty() { None } else { Some(scope) },
                    &ints(optional(options, "dept_ids"), "dept-id")?,
                )?;
                Ok(format!(
                    "Granted role {} ({}), data_scope {}.",
                    field(&row, "role_id"),
                    field(&row, "role_key"),
                    field(&row, "data_scope")
                ))
            }
            "menu-list" => Ok(self.report.menus(&self.access.menu_list(&actor)?)),
            "menu-add" => {
                let menu_id = self.access.menu_add(
                    &actor,
                    need(options, "menu_name", "set --menu-name")?,
                    whole(need(options, "parent_id", "set --parent-id")?, "parent-id")?,
                    need(options, "menu_type", "set --menu-type M, C, or F")?,
                    optional(options, "perms"),
                    optional(options, "path"),
                )?;
                Ok(format!("Added menu {menu_id}."))
            }
            "dept-list" => Ok(self.report.depts(&self.access.dept_list(&actor)?)),
            "dept-add" => {
                let dept_id = self.access.dept_add(
                    &actor,
                    need(options, "dept_name", "set --dept-name")?,
                    whole(need(options, "parent_id", "set --parent-id")?, "parent-id")?,
                )?;
                Ok(format!("Added department {dept_id}."))
            }
            "post-list" => Ok(self.report.posts(&self.access.post_list(&actor)?)),
            "post-add" => {
                let post_id = self.access.post_add(
                    &actor,
                    need(options, "post_code", "set --post-code")?,
                    need(options, "post_name", "set --post-name")?,
                )?;
                Ok(format!("Added post {post_id}."))
            }
            "config-list" => Ok(self.report.configs(&self.catalog.config_list(&actor)?)),
            "config-set" => {
                let key = self.catalog.config_set(
                    &actor,
                    need(options, "config_key", "set --config-key")?,
                    need(options, "config_value", "set --config-value")?,
                )?;
                Ok(format!("Set {key}."))
            }
            "config-add" => {
                let key = self.catalog.config_add(
                    &actor,
                    need(options, "config_key", "set --config-key")?,
                    need(options, "config_name", "set --config-name")?,
                    need(options, "config_value", "set --config-value")?,
                )?;
                Ok(format!("Added parameter {key}."))
            }
            "dict-list" => Ok(self.report.dicts(&self.catalog.dict_list(&actor)?)),
            "dict-add" => {
                let label = self.catalog.dict_add(
                    &actor,
                    need(options, "dict_type", "set --dict-type")?,
                    need(options, "dict_label", "set --dict-label")?,
                    need(options, "dict_value", "set --dict-value")?,
                )?;
                Ok(format!("Added dictionary value {label}."))
            }
            "notice-list" => Ok(self.report.notices(&self.catalog.notice_list(&actor)?)),
            "notice-add" => {
                let kind = optional(options, "notice_type");
                let title = self.catalog.notice_add(
                    &actor,
                    need(options, "notice_title", "set --notice-title")?,
                    if kind.is_empty() { "1" } else { kind },
                    optional(options, "content"),
                )?;
                Ok(format!("Added notice {title}."))
            }
            "oper-log" => Ok(self.report.oper_log(&self.catalog.oper_log(&actor)?)),
            "login-log" => Ok(self.report.login_log(&self.catalog.login_log(&actor)?)),
            "online-list" => Ok(self.report.online(&self.catalog.online_list(&actor)?)),
            "online-logout" => {
                let count = self
                    .catalog
                    .online_logout(&actor, need(options, "token", "rust-rbac online-list")?)?;
                Ok(format!("Closed {count} session."))
            }
            _ => Err(AppError::new(
                format!("Unknown verb '{verb}'."),
                "rust-rbac help",
            )),
        }
    }

    fn require_actor(&self) -> Result<HashMap<String, String>, AppError> {
        let token = self.sessions.read();
        if token.is_empty() {
            return Err(not_signed_in());
        }
        self.access
            .actor_from_token(&token)?
            .ok_or_else(not_signed_in)
    }

    fn web_mut(&self) -> std::sync::MutexGuard<'_, Option<WebApp>> {
        let mut guard = self.web.lock().expect("web");
        if guard.is_none() {
            *guard = Some(WebApp::new(
                Arc::clone(&self.settings),
                Arc::clone(&self.access),
                Arc::clone(&self.catalog),
                Arc::clone(&self.schema),
            ));
        }
        guard
    }
}

impl Drop for Desk {
    fn drop(&mut self) {
        self.stop_web();
    }
}

fn not_signed_in() -> AppError {
    AppError::new("Not signed in.", "rust-rbac login --user-name NAME")
}

fn option_value<'a>(options: &'a VerbOptions, name: &str) -> &'a str {
    match name {
        "user_name" => &options.user_name,
        "password" => &options.password,
        "nick_name" => &options.nick_name,
        "dept_id" => &options.dept_id,
        "role_id" => &options.role_id,
        "user_id" => &options.user_id,
        "status" => &options.status,
        "menu_ids" => &options.menu_ids,
        "data_scope" => &options.data_scope,
        "dept_ids" => &options.dept_ids,
        "menu_name" => &options.menu_name,
        "parent_id" => &options.parent_id,
        "menu_type" => &options.menu_type,
        "perms" => &options.perms,
        "path" => &options.path,
        "post_code" => &options.post_code,
        "post_name" => &options.post_name,
        "dept_name" => &options.dept_name,
        "config_key" => &options.config_key,
        "config_name" => &options.config_name,
        "config_value" => &options.config_value,
        "dict_type" => &options.dict_type,
        "dict_label" => &options.dict_label,
        "dict_value" => &options.dict_value,
        "notice_title" => &options.notice_title,
        "notice_type" => &options.notice_type,
        "content" => &options.content,
        "token" => &options.token,
        "engine" => &options.engine,
        _ => "",
    }
}

fn need<'a>(options: &'a VerbOptions, name: &str, next: &str) -> Result<&'a str, AppError> {
    let value = option_value(options, name).trim();
    if value.is_empty() {
        return Err(AppError::new(
            format!("{} is required.", name.replace('_', "-")),
            next,
        ));
    }
    Ok(value)
}

fn optional<'a>(options: &'a VerbOptions, name: &str) -> &'a str {
    option_value(options, name).trim()
}

fn whole(value: &str, label: &str) -> Result<i64, AppError> {
    value.trim().parse().map_err(|_| {
        AppError::new(
            format!("{label} must be a whole number."),
            "pass a whole number",
        )
    })
}

fn ints(text: &str, label: &str) -> Result<Vec<i64>, AppError> {
    let mut found = Vec::new();
    for part in text.split(',') {
        let part = part.trim();
        if !part.is_empty() {
            found.push(whole(part, label)?);
        }
    }
    Ok(found)
}

fn process_settings() -> (HashMap<String, String>, String) {
    let mut map = HashMap::new();
    for (key, value) in std::env::vars() {
        if key == "HOME" || key.starts_with("RUSTRBAC_") {
            map.insert(key, value);
        }
    }
    let home = map.get("HOME").cloned().unwrap_or_default();
    (map, home)
}
