//! Local web pages for the same RBAC actions as the terminal.
//! requirement-web-interface — loopback pages, cookie session, permission checks.
//! requirement-html-theme — vue3 or adminlte. The form names stay. There is no script.
//! requirement-html-body-font — both skins set the document body to 13px.
//! requirement-rbac-menu — the menus page edits, removes, and saves sort.

use std::collections::{HashMap, HashSet};
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::{self, JoinHandle};

use crate::access::Access;
use crate::catalog::Catalog;
use crate::error::AppError;
use crate::schema::Schema;
use crate::settings::Settings;
use crate::store::field;

const COOKIE: &str = "rust_rbac_session";
const ELEMENT_PLUS_CSS: &str = include_str!("html/element-plus.css");
const RUSTRBAC_CSS: &str = include_str!("html/rustrbac.css");
const ADMINLTE_CSS: &str = include_str!("html/adminlte.css");
const CSP: &str = "default-src 'none'; style-src 'self' 'unsafe-inline'; form-action 'self'";

pub struct HttpReply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: String,
}

struct Listener {
    stop: Arc<AtomicBool>,
    join: Option<JoinHandle<()>>,
    addr: String,
}

pub struct WebApp {
    settings: Arc<Mutex<Settings>>,
    access: Arc<Access>,
    catalog: Arc<Catalog>,
    schema: Arc<Schema>,
    listener: Mutex<Option<Listener>>,
}

impl WebApp {
    pub fn new(
        settings: Arc<Mutex<Settings>>,
        access: Arc<Access>,
        catalog: Arc<Catalog>,
        schema: Arc<Schema>,
    ) -> Self {
        Self {
            settings,
            access,
            catalog,
            schema,
            listener: Mutex::new(None),
        }
    }

    pub fn url(&self) -> Result<String, AppError> {
        if let Some(listener) = self.listener.lock().expect("listener").as_ref() {
            return Ok(format!("http://{}/", listener.addr));
        }
        let (host, port) = self.settings.lock().expect("settings").bind()?;
        Ok(format!("http://{host}:{port}/"))
    }

    pub fn start_background(&self) -> Result<String, AppError> {
        {
            let guard = self.listener.lock().expect("listener");
            if let Some(listener) = guard.as_ref() {
                if !listener.stop.load(Ordering::Relaxed) {
                    return Ok(format!("http://{}/", listener.addr));
                }
            }
        }
        let (host, port) = self.settings.lock().expect("settings").bind()?;
        let bound = TcpListener::bind((host.as_str(), port)).map_err(|_| {
            AppError::new(
                format!("Refusing to listen on {host}."),
                "keep RUSTRBAC_BIND=127.0.0.1",
            )
        })?;
        let actual = bound.local_addr().map_err(|_| {
            AppError::new(
                "The web listener did not start.",
                "keep RUSTRBAC_BIND=127.0.0.1",
            )
        })?;
        let addr = format!("{}:{}", actual.ip(), actual.port());
        let stop = Arc::new(AtomicBool::new(false));
        let flag = Arc::clone(&stop);
        let app = self.share();
        let join = thread::spawn(move || serve_loop(bound, app, flag));
        *self.listener.lock().expect("listener") = Some(Listener {
            stop,
            join: Some(join),
            addr: addr.clone(),
        });
        Ok(format!("http://{addr}/"))
    }

    pub fn stop(&self) {
        let mut guard = self.listener.lock().expect("listener");
        if let Some(listener) = guard.as_mut() {
            listener.stop.store(true, Ordering::Relaxed);
            let _ = TcpStream::connect(&listener.addr);
            if let Some(join) = listener.join.take() {
                let _ = join.join();
            }
        }
        *guard = None;
    }

    fn share(&self) -> Arc<WebApp> {
        Arc::new(WebApp {
            settings: Arc::clone(&self.settings),
            access: Arc::clone(&self.access),
            catalog: Arc::clone(&self.catalog),
            schema: Arc::clone(&self.schema),
            listener: Mutex::new(None),
        })
    }

    pub fn handle(
        &self,
        method: &str,
        target: &str,
        headers: &[(String, String)],
        body: &str,
        client_ip: &str,
    ) -> HttpReply {
        let path = target.split('?').next().unwrap_or("/");
        let query = target.split_once('?').map(|(_, rest)| rest).unwrap_or("");
        let method = method.to_ascii_uppercase();
        if method == "GET" && path == "/element-plus.css" {
            return self.css(ELEMENT_PLUS_CSS);
        }
        if method == "GET" && path == "/rustrbac.css" {
            return self.css(RUSTRBAC_CSS);
        }
        if method == "GET" && path == "/adminlte.css" {
            return self.css(ADMINLTE_CSS);
        }
        if method == "GET" && path == "/login" {
            return self.login_reply(200, "");
        }
        if method == "POST" && path == "/login" {
            return self.post_login(body, client_ip);
        }
        let user = self.user_from(headers);
        let Some(user) = user else {
            return redirect("/login", None, false);
        };
        if method == "POST" && !csrf_ok(&user, body) {
            return self.show(
                400,
                Some(&user),
                "Rejected",
                "<p>The form token did not match this session.</p>",
                path,
            );
        }
        match (method.as_str(), path) {
            ("GET", "/") => match self.home_page(&user) {
                Ok(body) => self.html(200, &body, &[]),
                Err(error) => self.fail(&user, error),
            },
            ("GET", "/users") => {
                self.guarded(&user, "system:user:list", |app| app.users_page(&user))
            }
            ("GET", "/roles") => {
                self.guarded(&user, "system:role:list", |app| app.roles_page(&user))
            }
            ("GET", "/menus") => {
                self.guarded(&user, "system:menu:list", |app| app.menus_page(&user, query))
            }
            ("GET", "/depts") => {
                self.guarded(&user, "system:dept:list", |app| app.depts_page(&user))
            }
            ("GET", "/posts") => {
                self.guarded(&user, "system:post:list", |app| app.posts_page(&user))
            }
            ("GET", "/configs") => {
                self.guarded(&user, "system:config:list", |app| app.configs_page(&user))
            }
            ("GET", "/dicts") => {
                self.guarded(&user, "system:dict:list", |app| app.dicts_page(&user))
            }
            ("GET", "/notices") => {
                self.guarded(&user, "system:notice:list", |app| app.notices_page(&user))
            }
            ("GET", "/database") => {
                self.guarded(&user, "system:config:edit", |app| app.database_page(&user))
            }
            ("POST", "/logout") => {
                let _ = self.access.logout(field(&user, "token"));
                redirect("/login", None, true)
            }
            ("POST", "/users/add") => self.after(&user, "/users", |app| {
                app.access.user_add(
                    &user,
                    form_first(body, "user_name"),
                    form_first(body, "nick_name"),
                    form_i64(body, "dept_id"),
                    form_i64(body, "role_id"),
                    form_first(body, "password"),
                )?;
                Ok(())
            }),
            ("POST", "/users/edit") => self.after(&user, "/users", |app| {
                let nick = form_first(body, "nick_name");
                let dept = form_first(body, "dept_id");
                let status = form_first(body, "status");
                app.access.user_edit(
                    &user,
                    form_i64(body, "user_id"),
                    if nick.is_empty() { None } else { Some(nick) },
                    if dept.is_empty() {
                        None
                    } else {
                        dept.parse().ok()
                    },
                    if status.is_empty() {
                        None
                    } else {
                        Some(status)
                    },
                )?;
                Ok(())
            }),
            ("POST", "/users/remove") => self.after(&user, "/users", |app| {
                app.access.user_remove(&user, form_i64(body, "user_id"))?;
                Ok(())
            }),
            ("POST", "/users/reset") => self.after(&user, "/users", |app| {
                app.access.user_reset_password(
                    &user,
                    form_i64(body, "user_id"),
                    form_first(body, "password"),
                )?;
                Ok(())
            }),
            ("POST", "/menus/add") => {
                let location = menu_location(body);
                self.after(&user, &location, |app| {
                    let menu_type = form_value(body, "menu_type");
                    let menu_type = if menu_type.trim().is_empty() {
                        "F".to_string()
                    } else {
                        menu_type
                    };
                    app.access.menu_add(
                        &user,
                        &form_value(body, "menu_name"),
                        posted_parent(&form_value(body, "parent_id"))?,
                        &menu_type,
                        &form_value(body, "perms"),
                        &form_value(body, "path"),
                    )?;
                    Ok(())
                })
            }
            ("POST", "/menus/edit") => {
                let location = menu_location(body);
                self.after(&user, &location, |app| {
                    app.access.menu_edit(
                        &user,
                        &form_value(body, "menu_id"),
                        &form_value(body, "menu_name"),
                        &form_value(body, "parent_id"),
                        &form_value(body, "menu_type"),
                        &form_value(body, "perms"),
                        &form_value(body, "path"),
                        &form_value(body, "visible"),
                    )?;
                    Ok(())
                })
            }
            ("POST", "/menus/remove") => {
                let location = menu_location(body);
                self.after(&user, &location, |app| {
                    app.access
                        .menu_remove(&user, &form_value(body, "menu_id"))?;
                    Ok(())
                })
            }
            ("POST", "/menus/sort") => {
                let location = menu_location(body);
                self.after(&user, &location, |app| {
                    app.access.menu_sort(
                        &user,
                        &form_values(body, "menu_id"),
                        &form_values(body, "order_num"),
                    )?;
                    Ok(())
                })
            }
            ("POST", "/roles/grant") => self.after(&user, "/roles", |app| {
                let menus = form_many_i64(body, "menu_id");
                let depts = form_many_i64(body, "dept_id");
                let scope = form_first(body, "data_scope");
                app.access.role_grant(
                    &user,
                    form_i64(body, "role_id"),
                    &menus,
                    if scope.is_empty() { None } else { Some(scope) },
                    &depts,
                )?;
                Ok(())
            }),
            ("POST", "/database") => self.post_database(&user, body),
            _ => self.show(
                404,
                Some(&user),
                "Not found",
                "<p>That page is not on this site.</p>",
                path,
            ),
        }
    }

    fn guarded(
        &self,
        user: &HashMap<String, String>,
        perm: &str,
        page: impl FnOnce(&Self) -> Result<String, AppError>,
    ) -> HttpReply {
        if let Err(error) = self.access.require(user, perm) {
            return self.fail(user, error);
        }
        match page(self) {
            Ok(body) => self.html(200, &body, &[]),
            Err(error) => self.fail(user, error),
        }
    }

    fn after(
        &self,
        user: &HashMap<String, String>,
        location: &str,
        work: impl FnOnce(&Self) -> Result<(), AppError>,
    ) -> HttpReply {
        match work(self) {
            Ok(()) => redirect(location, None, false),
            Err(error) => self.fail(user, error),
        }
    }

    fn fail(&self, user: &HashMap<String, String>, error: AppError) -> HttpReply {
        let code = if error.message.starts_with("Missing permission") {
            403
        } else {
            400
        };
        if error.message.starts_with("RUSTRBAC_THEME") {
            return self.html(400, &plain_error(&error), &[]);
        }
        let body = format!(
            "<p class=\"error\">{}</p><p>{}</p>",
            escape(&error.message),
            escape(&error.next)
        );
        self.show(code, Some(user), "Not completed", &body, "/")
    }

    fn show(
        &self,
        status: u16,
        user: Option<&HashMap<String, String>>,
        title: &str,
        body: &str,
        path: &str,
    ) -> HttpReply {
        match self.page(user, title, body, path) {
            Ok(html) => self.html(status, &html, &[]),
            Err(error) => self.html(400, &plain_error(&error), &[]),
        }
    }

    fn post_login(&self, body: &str, client_ip: &str) -> HttpReply {
        if let Err(error) = self.apply_engine(body, client_ip, false) {
            return self.login_reply(400, &error.message);
        }
        if let Err(error) = self.schema.ensure() {
            return self.login_reply(400, &error.message);
        }
        match self.access.login(
            form_first(body, "user_name"),
            form_first(body, "password"),
            client_ip,
            "web",
            "rust-rbac",
        ) {
            Ok(user) => redirect("/", Some(field(&user, "token")), false),
            Err(error) => self.login_reply(400, &error.message),
        }
    }

    fn post_database(&self, user: &HashMap<String, String>, body: &str) -> HttpReply {
        if let Err(error) = self.access.require(user, "system:config:edit") {
            return self.fail(user, error);
        }
        let before = self
            .settings
            .lock()
            .expect("settings")
            .engine()
            .unwrap_or_default();
        if let Err(error) = self.apply_engine(body, "127.0.0.1", true) {
            return self.fail(user, error);
        }
        let after = self
            .settings
            .lock()
            .expect("settings")
            .engine()
            .unwrap_or_default();
        if after != before {
            return redirect("/login", None, true);
        }
        redirect("/database", None, false)
    }

    fn apply_engine(
        &self,
        body: &str,
        client_ip: &str,
        allow_remote: bool,
    ) -> Result<(), AppError> {
        let posted = form_first(body, "engine").trim().to_ascii_lowercase();
        if posted.is_empty() {
            return Ok(());
        }
        let wanted = if posted == "1" || posted == "sqlite3" {
            "sqlite3"
        } else if posted == "2" || posted == "mysql" {
            "mysql"
        } else {
            return Err(AppError::new(
                "The database engine must be sqlite3 or mysql.",
                "choose sqlite3 or mysql",
            ));
        };
        let mut settings = self.settings.lock().expect("settings");
        if settings.engine()? == wanted {
            return Ok(());
        }
        let loopback = matches!(client_ip, "127.0.0.1" | "::1" | "localhost");
        if !allow_remote && !loopback {
            return Err(AppError::new(
                "The database engine can be changed from this computer.",
                "open the site on 127.0.0.1, or run rust-rbac db-engine",
            ));
        }
        settings.apply_choice(wanted)?;
        Ok(())
    }

    fn user_from(&self, headers: &[(String, String)]) -> Option<HashMap<String, String>> {
        let token = cookie_value(headers, COOKIE);
        if token.is_empty() {
            return None;
        }
        self.access.actor_from_token(&token).ok().flatten()
    }

    fn html(&self, status: u16, body: &str, extra: &[(String, String)]) -> HttpReply {
        let mut headers = vec![
            (
                "Content-Type".to_string(),
                "text/html; charset=utf-8".to_string(),
            ),
            ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
            ("Referrer-Policy".to_string(), "no-referrer".to_string()),
            ("Content-Security-Policy".to_string(), CSP.to_string()),
        ];
        headers.extend(extra.iter().cloned());
        HttpReply {
            status,
            headers,
            body: body.to_string(),
        }
    }

    fn css(&self, body: &str) -> HttpReply {
        HttpReply {
            status: 200,
            headers: vec![
                (
                    "Content-Type".to_string(),
                    "text/css; charset=utf-8".to_string(),
                ),
                ("X-Content-Type-Options".to_string(), "nosniff".to_string()),
                ("Referrer-Policy".to_string(), "no-referrer".to_string()),
                ("Content-Security-Policy".to_string(), CSP.to_string()),
            ],
            body: body.to_string(),
        }
    }

    fn login_reply(&self, status: u16, error: &str) -> HttpReply {
        match self.login_page(error) {
            Ok(body) => self.html(status, &body, &[]),
            Err(error) => self.html(400, &plain_error(&error), &[]),
        }
    }

    fn theme_name(&self) -> Result<String, AppError> {
        self.settings.lock().expect("settings").theme()
    }

    fn page(
        &self,
        user: Option<&HashMap<String, String>>,
        title: &str,
        body: &str,
        path: &str,
    ) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let inner = match user {
            Some(user) => self.shell(&theme, user, title, body, path),
            None => body.to_string(),
        };
        Ok(document(&theme, title, &inner))
    }

    fn shell(
        &self,
        theme: &str,
        user: &HashMap<String, String>,
        title: &str,
        body: &str,
        path: &str,
    ) -> String {
        let menu = self.menu(theme, user, path);
        let crumbs = breadcrumb(theme, title);
        let sign_out = format!(
            "<form class=\"{}\" method=\"post\" action=\"/logout\">{}{}</form>",
            if theme == "adminlte" {
                "form-horizontal"
            } else {
                "el-form"
            },
            hidden(user),
            submit_button(theme, "Sign out", false)
        );
        let main = format!("<h1>{}</h1>{body}", escape(title));
        if theme == "adminlte" {
            format!(
                "<div class=\"wrapper\"><aside class=\"main-sidebar sidebar-dark-primary\">{menu}</aside><div class=\"content-wrapper\"><header class=\"main-header\">{crumbs}{sign_out}</header><main class=\"content\">{main}</main></div></div>"
            )
        } else {
            format!(
                "<div class=\"app-wrapper\"><aside class=\"sidebar\">{menu}</aside><div class=\"main-container\"><header class=\"navbar\">{crumbs}{sign_out}</header><main class=\"app-main\">{main}</main></div></div>"
            )
        }
    }

    fn menu(&self, theme: &str, user: &HashMap<String, String>, path: &str) -> String {
        let pairs = [
            ("/", "Home", ""),
            ("/users", "Users", "system:user:list"),
            ("/roles", "Roles", "system:role:list"),
            ("/menus", "Menus", "system:menu:list"),
            ("/depts", "Departments", "system:dept:list"),
            ("/posts", "Posts", "system:post:list"),
            ("/configs", "Parameters", "system:config:list"),
            ("/dicts", "Dictionaries", "system:dict:list"),
            ("/notices", "Notices", "system:notice:list"),
            ("/database", "Database", "system:config:edit"),
        ];
        let mut items = String::new();
        for (href, label, perm) in pairs {
            if !perm.is_empty()
                && !self
                    .access
                    .allows(field(user, "user_id"), perm)
                    .unwrap_or(false)
            {
                continue;
            }
            items.push_str(&menu_item(theme, href, label, href == path));
        }
        if theme == "adminlte" {
            format!(
                "<ul class=\"nav nav-pills nav-sidebar flex-column\" role=\"menu\">{items}</ul>"
            )
        } else {
            format!(
                "<ul class=\"el-menu el-menu--vertical\" role=\"menubar\">{items}</ul>"
            )
        }
    }

    fn login_page(&self, error: &str) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let error_html = if error.is_empty() {
            String::new()
        } else {
            format!("<p class=\"error\">{}</p>", escape(error))
        };
        let fields = format!(
            "{}{}{}",
            self.engine_fields(&theme),
            field_row(
                &theme,
                "Account",
                &text_input(&theme, "user_name", "text", " autocomplete=\"username\"")
            ),
            field_row(
                &theme,
                "Password",
                &text_input(
                    &theme,
                    "password",
                    "password",
                    " autocomplete=\"current-password\""
                )
            )
        );
        let form = form_tag(
            &theme,
            "post",
            "/login",
            &format!("{fields}{}", submit_button(&theme, "Sign in", true)),
        );
        let body = format!(
            "<div class=\"login\"><h1>Sign in</h1>{error_html}{form}<p>Seeded accounts and initial passwords are in the product README. Change them after the first sign-in. This site listens on this computer only unless remote bind was explicitly allowed. The default database is the sqlite3 file. mysql needs RUSTRBAC_DB_PASSWORD.</p></div>"
        );
        Ok(document(&theme, "Sign in", &body))
    }

    fn engine_fields(&self, theme: &str) -> String {
        let mut settings = self.settings.lock().expect("settings");
        let current = settings.engine().unwrap_or_else(|_| "sqlite3".to_string());
        let path = settings.sqlite_path().unwrap_or_default();
        let mark = if settings.get("RUSTRBAC_SQLITE_FILE").is_empty() {
            " (default file)"
        } else {
            ""
        };
        let name = settings.get("RUSTRBAC_DB_NAME");
        drop(settings);
        format!(
            "<fieldset><legend>Database</legend>{}{}</fieldset>",
            radio_choice(theme, "engine", "sqlite3", &format!("sqlite3 {path}{mark}"), current == "sqlite3"),
            radio_choice(theme, "engine", "mysql", &format!("mysql {name}"), current == "mysql")
        )
    }

    fn home_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let who = self.access.describe(user).unwrap_or_default();
        let body = format!(
            "<p>Signed in as <strong>{}</strong>.</p><pre>{}</pre>",
            escape(field(user, "user_name")),
            escape(&who)
        );
        self.page(Some(user), "Home", &body, "/")
    }

    fn users_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let rows = self.access.user_list(user)?;
        let mut parts = vec![data_table(
            &theme,
            &[
                "user_id",
                "user_name",
                "nick_name",
                "dept_id",
                "dept_name",
                "status",
            ],
            &rows,
        )];
        if self
            .access
            .allows(field(user, "user_id"), "system:user:add")?
        {
            parts.push(add_user_form(&theme, user));
        }
        self.page(Some(user), "Users", &parts.join(""), "/users")
    }

    fn roles_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let rows = self.access.role_list(user)?;
        self.page(
            Some(user),
            "Roles",
            &data_table(
                &theme,
                &["role_id", "role_name", "role_key", "data_scope", "status"],
                &rows,
            ),
            "/roles",
        )
    }

    fn menus_page(&self, user: &HashMap<String, String>, query: &str) -> Result<String, AppError> {
        let picked = menu_query(query);
        let rows = self.access.menu_list(user)?;
        let can_add = self.access.allows(field(user, "user_id"), "system:menu:add")?;
        let can_edit = self
            .access
            .allows(field(user, "user_id"), "system:menu:edit")?;
        let can_remove = self
            .access
            .allows(field(user, "user_id"), "system:menu:remove")?;
        let view = menu_view(&rows, &picked);
        let known = menu_ids(&rows);
        let mut notice = String::new();
        let target = if picked.form == "edit" || picked.form == "remove" {
            rows.iter().find(|row| field(row, "menu_id") == picked.menu_id)
        } else {
            None
        };
        if picked.form == "edit" || picked.form == "remove" {
            if picked.menu_id.is_empty() {
                notice = "Choose a menu.".to_string();
            } else if target.is_none() {
                notice = "That menu is not in this list.".to_string();
            }
        }
        let show_add = picked.form == "add" && can_add;
        let show_edit = picked.form == "edit" && can_edit && target.is_some() && notice.is_empty();
        let show_remove =
            picked.form == "remove" && can_remove && target.is_some() && notice.is_empty();
        let parent_id = if known.contains(&picked.parent_id) {
            picked.parent_id.as_str()
        } else {
            "0"
        };
        let theme = self.theme_name()?;
        self.page(
            Some(user),
            "Menus",
            &paint_controls(
                &theme,
                &menu_html(
                    user,
                    &picked,
                    &rows,
                    &view,
                    can_add,
                    can_edit,
                    can_remove,
                    &notice,
                    show_add,
                    show_edit,
                    show_remove,
                    parent_id,
                    target,
                ),
            ),
            "/menus",
        )
    }

    fn depts_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let rows = self.access.dept_list(user)?;
        self.page(
            Some(user),
            "Departments",
            &data_table(
                &theme,
                &["dept_id", "parent_id", "dept_name", "ancestors", "status"],
                &rows,
            ),
            "/depts",
        )
    }

    fn posts_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let rows = self.access.post_list(user)?;
        self.page(
            Some(user),
            "Posts",
            &data_table(
                &theme,
                &["post_id", "post_code", "post_name", "post_sort", "status"],
                &rows,
            ),
            "/posts",
        )
    }

    fn configs_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let rows = self.catalog.config_list(user)?;
        self.page(
            Some(user),
            "Parameters",
            &data_table(
                &theme,
                &["config_key", "config_name", "config_value", "config_type"],
                &rows,
            ),
            "/configs",
        )
    }

    fn dicts_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let rows = self.catalog.dict_list(user)?;
        self.page(
            Some(user),
            "Dictionaries",
            &data_table(&theme, &["dict_type", "dict_label", "dict_value", "status"], &rows),
            "/dicts",
        )
    }

    fn notices_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let rows = self.catalog.notice_list(user)?;
        self.page(
            Some(user),
            "Notices",
            &data_table(
                &theme,
                &["notice_id", "notice_title", "notice_type", "status"],
                &rows,
            ),
            "/notices",
        )
    }

    fn database_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let theme = self.theme_name()?;
        let sentence = self.settings.lock().expect("settings").engine_sentence()?;
        let body = format!(
            "<p>{} The next sign-in uses the saved engine.</p>{}",
            escape(&sentence),
            form_tag(
                &theme,
                "post",
                "/database",
                &format!(
                    "{}{}{}",
                    self.engine_fields(&theme),
                    hidden(user),
                    submit_button(&theme, "Save", false)
                )
            )
        );
        self.page(Some(user), "Database", &body, "/database")
    }
}

fn redirect(location: &str, cookie: Option<&str>, clear: bool) -> HttpReply {
    let mut headers = vec![
        ("Location".to_string(), location.to_string()),
        ("Content-Length".to_string(), "0".to_string()),
    ];
    if let Some(cookie) = cookie {
        headers.push((
            "Set-Cookie".to_string(),
            format!("{COOKIE}={cookie}; HttpOnly; SameSite=Lax; Path=/"),
        ));
    }
    if clear {
        headers.push((
            "Set-Cookie".to_string(),
            format!("{COOKIE}=; HttpOnly; SameSite=Lax; Path=/; Max-Age=0"),
        ));
    }
    HttpReply {
        status: 303,
        headers,
        body: String::new(),
    }
}

fn hidden(user: &HashMap<String, String>) -> String {
    format!(
        "<input type=\"hidden\" name=\"csrf\" value=\"{}\">",
        escape(field(user, "csrf"))
    )
}

fn data_table(theme: &str, columns: &[&str], rows: &[HashMap<String, String>]) -> String {
    let adminlte = theme == "adminlte";
    let head_class = if adminlte {
        ""
    } else {
        " class=\"el-table__cell el-table-column\""
    };
    let cell_class = if adminlte {
        ""
    } else {
        " class=\"el-table__cell\""
    };
    let row_class = if adminlte { "" } else { " class=\"el-table__row\"" };
    let head: String = columns
        .iter()
        .map(|name| format!("<th{head_class}>{}</th>", escape(name)))
        .collect();
    let mut body = String::new();
    if rows.is_empty() {
        body.push_str(&format!(
            "<tr{row_class}><td{cell_class} colspan=\"{}\">(none)</td></tr>",
            columns.len()
        ));
    }
    for row in rows {
        body.push_str(&format!("<tr{row_class}>"));
        for name in columns {
            body.push_str(&format!(
                "<td{cell_class}>{}</td>",
                escape(field(row, name))
            ));
        }
        body.push_str("</tr>");
    }
    if adminlte {
        format!(
            "<div class=\"card\"><div class=\"card-body table-responsive p-0\"><table class=\"table table-bordered table-striped\"><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div></div>"
        )
    } else {
        format!(
            "<div class=\"el-table\"><div class=\"el-table__body-wrapper\"><table class=\"el-table__header el-table__body\"><thead><tr>{head}</tr></thead><tbody>{body}</tbody></table></div></div>"
        )
    }
}

fn document(theme: &str, title: &str, inner: &str) -> String {
    let links = if theme == "adminlte" {
        "<link rel=\"stylesheet\" href=\"/adminlte.css\">"
    } else {
        "<link rel=\"stylesheet\" href=\"/element-plus.css\"><link rel=\"stylesheet\" href=\"/rustrbac.css\">"
    };
    format!(
        "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><meta name=\"viewport\" content=\"width=device-width, initial-scale=1\"><title>{}</title>{links}</head><body>{inner}</body></html>",
        escape(title)
    )
}

fn plain_error(error: &AppError) -> String {
    format!(
        "<p class=\"error\">{}</p><p>{}</p>",
        escape(&error.message),
        escape(&error.next)
    )
}

fn breadcrumb(theme: &str, title: &str) -> String {
    if theme == "adminlte" {
        format!(
            "<ol class=\"breadcrumb\" aria-label=\"Breadcrumb\"><li class=\"breadcrumb-item\">Home</li><li class=\"breadcrumb-item\">{}</li></ol>",
            escape(title)
        )
    } else {
        format!(
            "<div class=\"el-breadcrumb\" aria-label=\"Breadcrumb\"><span class=\"el-breadcrumb__item\">Home</span><span class=\"el-breadcrumb__item\">{}</span></div>",
            escape(title)
        )
    }
}

fn menu_item(theme: &str, href: &str, label: &str, active: bool) -> String {
    let href = escape(href);
    let label = escape(label);
    if theme == "adminlte" {
        let link = if active { "nav-link active" } else { "nav-link" };
        let current = if active { " aria-current=\"page\"" } else { "" };
        format!(
            "<li class=\"nav-item\" role=\"menuitem\"{current}><a class=\"{link}\" href=\"{href}\">{label}</a></li>"
        )
    } else {
        let item = if active {
            "el-menu-item is-active"
        } else {
            "el-menu-item"
        };
        let current = if active { " aria-current=\"page\"" } else { "" };
        format!(
            "<li class=\"{item}\" role=\"menuitem\"{current}><a href=\"{href}\">{label}</a></li>"
        )
    }
}

fn form_tag(theme: &str, method: &str, action: &str, inner: &str) -> String {
    let class = if theme == "adminlte" {
        "form-horizontal"
    } else {
        "el-form"
    };
    format!(
        "<form class=\"{class}\" method=\"{method}\" action=\"{action}\">{inner}</form>"
    )
}

fn field_row(theme: &str, label: &str, control: &str) -> String {
    if theme == "adminlte" {
        format!("<div class=\"form-group\"><label>{label}</label><div>{control}</div></div>")
    } else {
        format!(
            "<div class=\"el-form-item\"><label class=\"el-form-item__label\">{label}</label><div class=\"el-form-item__content\">{control}</div></div>"
        )
    }
}

fn text_input(theme: &str, name: &str, kind: &str, extra: &str) -> String {
    if theme == "adminlte" {
        format!("<input class=\"form-control\" type=\"{kind}\" name=\"{name}\"{extra}>")
    } else {
        format!(
            "<div class=\"el-input\"><div class=\"el-input__wrapper\"><input class=\"el-input__inner\" type=\"{kind}\" name=\"{name}\"{extra}></div></div>"
        )
    }
}

fn submit_button(theme: &str, label: &str, large: bool) -> String {
    if theme == "adminlte" {
        let size = if large { " btn-lg" } else { "" };
        format!("<button class=\"btn btn-primary{size}\" type=\"submit\"><span>{label}</span></button>")
    } else {
        let size = if large { " el-button--large" } else { "" };
        format!(
            "<button class=\"el-button el-button--primary{size}\" type=\"submit\"><span>{label}</span></button>"
        )
    }
}

fn radio_choice(theme: &str, name: &str, value: &str, label: &str, checked: bool) -> String {
    let mark = if checked { " checked" } else { "" };
    let label = escape(label);
    if theme == "adminlte" {
        format!(
            "<div class=\"form-check\"><label class=\"form-check-label\"><input class=\"form-check-input\" type=\"radio\" name=\"{name}\" value=\"{value}\"{mark}> {label}</label></div>"
        )
    } else {
        format!(
            "<label class=\"el-radio\"><span class=\"el-radio__input\"><input class=\"el-radio__original\" type=\"radio\" name=\"{name}\" value=\"{value}\"{mark}><span class=\"el-radio__inner\"></span></span><span class=\"el-radio__label\">{label}</span></label>"
        )
    }
}

fn add_user_form(theme: &str, user: &HashMap<String, String>) -> String {
    let fields = format!(
        "{}{}{}{}{}{}",
        hidden(user),
        field_row(theme, "Account", &text_input(theme, "user_name", "text", "")),
        field_row(theme, "Name", &text_input(theme, "nick_name", "text", "")),
        field_row(theme, "Department id", &text_input(theme, "dept_id", "text", "")),
        field_row(theme, "Role id", &text_input(theme, "role_id", "text", "")),
        field_row(
            theme,
            "Password",
            &text_input(theme, "password", "password", "")
        )
    );
    let form = form_tag(
        theme,
        "post",
        "/users/add",
        &format!("{fields}{}", submit_button(theme, "Add", false)),
    );
    if theme == "adminlte" {
        format!(
            "<div class=\"card card-primary card-outline\"><div class=\"card-body\"><h2>Add a user</h2>{form}</div></div>"
        )
    } else {
        format!("<div class=\"el-dialog\"><div class=\"el-dialog__body\"><h2>Add a user</h2>{form}</div></div>")
    }
}

fn paint_controls(theme: &str, html: &str) -> String {
    let marked = html.replace("<input type=\"hidden\"", "<input data-keep=\"hidden\"");
    let painted = if theme == "adminlte" {
        marked
            .replace(
                "<input type=\"radio\"",
                "<input class=\"form-check-input\" type=\"radio\"",
            )
            .replace("<input ", "<input class=\"form-control\" ")
            .replace("<select ", "<select class=\"form-control\" ")
            .replace("<button ", "<button class=\"btn btn-primary\" ")
            .replace("<form ", "<form class=\"form-horizontal\" ")
            .replace(
                "<table>",
                "<div class=\"card card-primary card-outline\"><div class=\"card-body table-responsive p-0\"><table class=\"table table-bordered table-striped\">",
            )
            .replace("</table>", "</table></div></div>")
    } else {
        marked
            .replace(
                "<input type=\"radio\"",
                "<input class=\"el-radio__original\" type=\"radio\"",
            )
            .replace("<input ", "<input class=\"el-input__inner\" ")
            .replace("<select ", "<select class=\"el-input__inner\" ")
            .replace(
                "<button ",
                "<button class=\"el-button el-button--primary\" ",
            )
            .replace("<form ", "<form class=\"el-form\" ")
            .replace(
                "<table>",
                "<div class=\"el-table\"><table class=\"el-table__body\">",
            )
            .replace("</table>", "</table></div>")
    };
    painted.replace("<input data-keep=\"hidden\"", "<input type=\"hidden\"")
}

fn escape(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn cookie_value(headers: &[(String, String)], name: &str) -> String {
    for (key, value) in headers {
        if key.eq_ignore_ascii_case("cookie") {
            for part in value.split(';') {
                if let Some((key, token)) = part.trim().split_once('=') {
                    if key == name {
                        return token.to_string();
                    }
                }
            }
        }
    }
    String::new()
}

fn form_pairs(body: &str) -> Vec<(String, String)> {
    let mut pairs = Vec::new();
    for part in body.split('&') {
        if part.is_empty() {
            continue;
        }
        let (key, value) = part.split_once('=').unwrap_or((part, ""));
        pairs.push((decode(key), decode(value)));
    }
    pairs
}

fn form_first<'a>(body: &'a str, name: &str) -> &'a str {
    for part in body.split('&') {
        if let Some((key, value)) = part.split_once('=') {
            if decode_cmp(key, name) {
                return value;
            }
        }
    }
    ""
}

fn decode_cmp(raw: &str, name: &str) -> bool {
    decode(raw) == name
}

fn form_i64(body: &str, name: &str) -> i64 {
    form_first(body, name).parse().unwrap_or(0)
}

fn form_values(body: &str, name: &str) -> Vec<String> {
    form_pairs(body)
        .into_iter()
        .filter(|(key, _)| key == name)
        .map(|(_, value)| value)
        .collect()
}

fn form_value(body: &str, name: &str) -> String {
    form_values(body, name).into_iter().next().unwrap_or_default()
}

fn form_many_i64(body: &str, name: &str) -> Vec<i64> {
    form_pairs(body)
        .into_iter()
        .filter(|(key, _)| key == name)
        .filter_map(|(_, value)| value.parse().ok())
        .collect()
}

fn csrf_ok(user: &HashMap<String, String>, body: &str) -> bool {
    let posted = form_first(body, "csrf");
    !posted.is_empty() && posted == field(user, "csrf")
}

fn decode(value: &str) -> String {
    let mut out = String::new();
    let bytes = value.as_bytes();
    let mut index = 0;
    while index < bytes.len() {
        match bytes[index] {
            b'+' => {
                out.push(' ');
                index += 1;
            }
            b'%' if index + 2 < bytes.len() => {
                let hex = &value[index + 1..index + 3];
                if let Ok(byte) = u8::from_str_radix(hex, 16) {
                    out.push(byte as char);
                    index += 3;
                } else {
                    out.push('%');
                    index += 1;
                }
            }
            byte => {
                out.push(byte as char);
                index += 1;
            }
        }
    }
    out
}

fn serve_loop(listener: TcpListener, app: Arc<WebApp>, stop: Arc<AtomicBool>) {
    let _ = listener.set_nonblocking(false);
    while !stop.load(Ordering::Relaxed) {
        let Ok((mut stream, peer)) = listener.accept() else {
            break;
        };
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let mut buffer = [0u8; 8192];
        let Ok(count) = stream.read(&mut buffer) else {
            continue;
        };
        let text = String::from_utf8_lossy(&buffer[..count]).to_string();
        let ip = peer.ip().to_string();
        let reply = parse_and_handle(&app, &text, &ip);
        let _ = write_reply(&mut stream, &reply);
    }
}

fn parse_and_handle(app: &WebApp, raw: &str, ip: &str) -> HttpReply {
    let mut lines = raw.split("\r\n");
    let request = lines.next().unwrap_or("");
    let mut parts = request.split_whitespace();
    let method = parts.next().unwrap_or("GET");
    let target = parts.next().unwrap_or("/");
    let mut headers = Vec::new();
    for line in lines.by_ref() {
        if line.is_empty() {
            break;
        }
        if let Some((key, value)) = line.split_once(':') {
            headers.push((key.trim().to_string(), value.trim().to_string()));
        }
    }
    let body = raw
        .split_once("\r\n\r\n")
        .map(|(_, body)| body)
        .unwrap_or("");
    app.handle(method, target, &headers, body, ip)
}

struct MenuQuery {
    menu_name: String,
    state: String,
    closed: Vec<i64>,
    form: String,
    parent_id: String,
    menu_id: String,
}

struct MenuNode {
    menu_id: i64,
    parent_id: i64,
    menu_name: String,
    menu_type: String,
    perms: String,
    order_num: i64,
    path: String,
    visible: String,
    full: Vec<usize>,
    kept: Vec<usize>,
}

struct ShownMenu {
    menu_id: i64,
    menu_name: String,
    menu_type: String,
    perms: String,
    order_num: i64,
    path: String,
    visible: String,
}

struct MenuView {
    rows: Vec<ShownMenu>,
    closed_text: String,
}

fn posted_parent(raw: &str) -> Result<i64, AppError> {
    let text = raw.trim();
    if text.is_empty() {
        return Ok(0);
    }
    digit_id(text).ok_or_else(|| {
        AppError::new(
            format!("Unknown parent menu {text}."),
            "rust-rbac menu-list",
        )
    })
}

fn digit_id(raw: &str) -> Option<i64> {
    let text = raw.trim();
    if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    text.parse().ok()
}

fn menu_location(body: &str) -> String {
    let name = form_value(body, "keep_name");
    let state = form_value(body, "keep_state");
    let closed = form_value(body, "keep_closed");
    menu_href(&[
        ("menu_name", name.as_str()),
        ("state", state.as_str()),
        ("closed", closed.as_str()),
    ])
}

fn menu_href(pairs: &[(&str, &str)]) -> String {
    let mut params = Vec::new();
    for (key, value) in pairs {
        let text = value.trim();
        if text.is_empty() || (*key == "parent_id" && text == "0") {
            continue;
        }
        params.push(format!("{key}={}", url_encode(text)));
    }
    if params.is_empty() {
        "/menus".to_string()
    } else {
        format!("/menus?{}", params.join("&"))
    }
}

fn url_encode(value: &str) -> String {
    let mut out = String::new();
    for byte in value.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' => out.push(byte as char),
            b' ' => out.push('+'),
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

fn query_first(query: &str, name: &str) -> String {
    form_value(query, name)
}

fn menu_query(query: &str) -> MenuQuery {
    let menu_name = query_first(query, "menu_name").trim().to_string();
    let mut state = query_first(query, "state").trim().to_string();
    if state != "0" && state != "1" {
        state.clear();
    }
    let mut closed = Vec::new();
    let mut seen = HashSet::new();
    for part in query_first(query, "closed").split(',') {
        let Some(ident) = digit_id(part) else {
            continue;
        };
        if seen.insert(ident) {
            closed.push(ident);
        }
    }
    let mut form = query_first(query, "form").trim().to_string();
    if form != "add" && form != "edit" && form != "remove" {
        form.clear();
    }
    let parent = query_first(query, "parent_id");
    let parent_id = if digit_id(&parent).is_some() {
        parent.trim().to_string()
    } else {
        "0".to_string()
    };
    let raw_id = query_first(query, "menu_id");
    let menu_id = if digit_id(&raw_id).is_some() {
        raw_id.trim().to_string()
    } else {
        String::new()
    };
    MenuQuery {
        menu_name,
        state,
        closed,
        form,
        parent_id,
        menu_id,
    }
}

fn menu_ids(rows: &[HashMap<String, String>]) -> HashSet<String> {
    rows.iter()
        .map(|row| field(row, "menu_id").to_string())
        .collect()
}

fn menu_view(rows: &[HashMap<String, String>], picked: &MenuQuery) -> MenuView {
    let mut nodes = Vec::new();
    let mut index_of = HashMap::new();
    for row in rows {
        let Some(menu_id) = digit_id(field(row, "menu_id")) else {
            continue;
        };
        let parent_id = digit_id(field(row, "parent_id")).unwrap_or(0);
        let order_num = digit_id(field(row, "order_num")).unwrap_or(0);
        index_of.insert(menu_id, nodes.len());
        nodes.push(MenuNode {
            menu_id,
            parent_id,
            menu_name: field(row, "menu_name").to_string(),
            menu_type: field(row, "menu_type").to_string(),
            perms: field(row, "perms").to_string(),
            order_num,
            path: field(row, "path").to_string(),
            visible: {
                let text = field(row, "visible");
                if text.is_empty() { "0" } else { text }.to_string()
            },
            full: Vec::new(),
            kept: Vec::new(),
        });
    }
    let mut children: HashMap<i64, Vec<usize>> = HashMap::new();
    for (index, node) in nodes.iter().enumerate() {
        let parent = if node.parent_id == node.menu_id || !index_of.contains_key(&node.parent_id) {
            0
        } else {
            node.parent_id
        };
        children.entry(parent).or_default().push(index);
    }
    for kids in children.values_mut() {
        kids.sort_by_key(|index| (nodes[*index].order_num, nodes[*index].menu_id));
    }
    let roots = children.get(&0).cloned().unwrap_or_default();
    for index in 0..nodes.len() {
        let menu_id = nodes[index].menu_id;
        if let Some(kids) = children.get(&menu_id) {
            nodes[index].full = kids.clone();
        }
    }
    let name_q = picked.menu_name.to_lowercase();
    let state = picked.state.clone();
    let mut seen = HashSet::new();
    let mut kept_roots = Vec::new();
    for index in roots {
        if prepare_menu(&mut nodes, index, &name_q, &state, &mut seen) {
            kept_roots.push(index);
        }
    }
    let mut shown = Vec::new();
    let mut branch_ids = Vec::new();
    let closed: HashSet<i64> = picked.closed.iter().copied().collect();
    for index in kept_roots {
        emit_menu(
            &nodes,
            index,
            &closed,
            &mut shown,
            &mut branch_ids,
        );
    }
    let active: Vec<String> = branch_ids
        .iter()
        .filter(|ident| closed.contains(ident))
        .map(|ident| ident.to_string())
        .collect();
    MenuView {
        rows: shown,
        closed_text: active.join(","),
    }
}

fn menu_hit(node: &MenuNode, name_q: &str, state: &str) -> bool {
    let name_ok = name_q.is_empty() || node.menu_name.to_lowercase().contains(name_q);
    let visible_ok = state.is_empty() || node.visible == state;
    name_ok && visible_ok
}

fn prepare_menu(
    nodes: &mut [MenuNode],
    index: usize,
    name_q: &str,
    state: &str,
    seen: &mut HashSet<i64>,
) -> bool {
    let menu_id = nodes[index].menu_id;
    if !seen.insert(menu_id) {
        nodes[index].kept.clear();
        return false;
    }
    let full = nodes[index].full.clone();
    if name_q.is_empty() && state.is_empty() {
        nodes[index].kept = full.clone();
        for child in full {
            prepare_menu(nodes, child, name_q, state, seen);
        }
        return true;
    }
    if menu_hit(&nodes[index], name_q, state) && !name_q.is_empty() && state.is_empty() {
        attach_menu(nodes, index, seen);
        return true;
    }
    let mut kept = Vec::new();
    for child in full {
        if prepare_menu(nodes, child, name_q, state, seen) {
            kept.push(child);
        }
    }
    let hit = menu_hit(&nodes[index], name_q, state);
    nodes[index].kept = kept;
    hit || !nodes[index].kept.is_empty()
}

fn attach_menu(nodes: &mut [MenuNode], index: usize, seen: &mut HashSet<i64>) {
    let full = nodes[index].full.clone();
    let mut kept = Vec::new();
    for child in full {
        let menu_id = nodes[child].menu_id;
        if !seen.insert(menu_id) {
            continue;
        }
        kept.push(child);
        attach_menu(nodes, child, seen);
    }
    nodes[index].kept = kept;
}

fn emit_menu(
    nodes: &[MenuNode],
    index: usize,
    closed: &HashSet<i64>,
    shown: &mut Vec<ShownMenu>,
    branch_ids: &mut Vec<i64>,
) {
    let node = &nodes[index];
    let is_branch = !node.kept.is_empty();
    if is_branch {
        branch_ids.push(node.menu_id);
    }
    let shut = is_branch && closed.contains(&node.menu_id);
    shown.push(ShownMenu {
        menu_id: node.menu_id,
        menu_name: node.menu_name.clone(),
        menu_type: node.menu_type.clone(),
        perms: node.perms.clone(),
        order_num: node.order_num,
        path: node.path.clone(),
        visible: node.visible.clone(),
    });
    if is_branch && !shut {
        for child in node.kept.clone() {
            emit_menu(nodes, child, closed, shown, branch_ids);
        }
    }
}

fn menu_html(
    user: &HashMap<String, String>,
    picked: &MenuQuery,
    all_rows: &[HashMap<String, String>],
    view: &MenuView,
    can_add: bool,
    can_edit: bool,
    can_remove: bool,
    notice: &str,
    show_add: bool,
    show_edit: bool,
    show_remove: bool,
    parent_id: &str,
    target: Option<&HashMap<String, String>>,
) -> String {
    let mut body = String::new();
    if !notice.is_empty() {
        body.push_str(&format!("<p class=\"error\">{}</p>", escape(notice)));
    }
    body.push_str("<form method=\"get\" action=\"/menus\">");
    body.push_str(&format!(
        "<label>Menu name <input type=\"text\" name=\"menu_name\" value=\"{}\"></label>",
        escape(&picked.menu_name)
    ));
    body.push_str("<label>Status <select name=\"state\">");
    for (value, label) in [("" , "All"), ("0", "Show"), ("1", "Hide")] {
        let selected = if picked.state == value { " selected" } else { "" };
        body.push_str(&format!(
            "<option value=\"{value}\"{selected}>{label}</option>"
        ));
    }
    body.push_str("</select></label>");
    if !view.closed_text.is_empty() {
        body.push_str(&format!(
            "<input type=\"hidden\" name=\"closed\" value=\"{}\">",
            escape(&view.closed_text)
        ));
    }
    body.push_str("<button type=\"submit\">Search</button> <a href=\"/menus\">Reset</a></form>");
    let keep = [
        ("menu_name", picked.menu_name.as_str()),
        ("state", picked.state.as_str()),
        ("closed", view.closed_text.as_str()),
    ];
    body.push_str("<form method=\"get\" action=\"/menus\">");
    for (key, value) in keep {
        if !value.is_empty() {
            body.push_str(&format!(
                "<input type=\"hidden\" name=\"{key}\" value=\"{}\">",
                escape(value)
            ));
        }
    }
    body.push_str("<p>");
    if can_add {
        let href = menu_href(&[
            ("menu_name", picked.menu_name.as_str()),
            ("state", picked.state.as_str()),
            ("closed", view.closed_text.as_str()),
            ("form", "add"),
        ]);
        body.push_str(&format!("<a href=\"{}\">Add</a> ", escape(&href)));
    }
    if can_edit {
        body.push_str("<button type=\"submit\" name=\"form\" value=\"edit\">Edit</button> ");
        body.push_str("<button type=\"submit\" form=\"menu-sort\">Save sort</button> ");
    }
    body.push_str("</p><table><tr>");
    if can_edit {
        body.push_str("<th>Choose</th>");
    }
    body.push_str("<th>Menu name</th><th>Sort</th><th>Path</th><th>Type</th><th>Visible</th><th>Permission</th>");
    if can_add || can_edit || can_remove {
        body.push_str("<th>Action</th>");
    }
    body.push_str("</tr>");
    for row in &view.rows {
        body.push_str("<tr>");
        let ident = row.menu_id.to_string();
        if can_edit {
            body.push_str(&format!(
                "<td><input type=\"radio\" name=\"menu_id\" value=\"{ident}\"></td>"
            ));
        }
        body.push_str(&format!("<td>{}</td>", escape(&row.menu_name)));
        if can_edit {
            body.push_str(&format!(
                "<td><input name=\"order_num\" form=\"menu-sort\" value=\"{}\"><input type=\"hidden\" name=\"menu_id\" form=\"menu-sort\" value=\"{ident}\"></td>",
                row.order_num
            ));
        } else {
            body.push_str(&format!(
                "<td><input readonly value=\"{}\"></td>",
                row.order_num
            ));
        }
        body.push_str(&format!("<td>{}</td>", escape(&row.path)));
        body.push_str(&format!("<td>{}</td>", escape(menu_type_label(&row.menu_type))));
        body.push_str(&format!(
            "<td>{}</td>",
            if row.visible == "1" { "Hide" } else { "Show" }
        ));
        body.push_str(&format!("<td>{}</td>", escape(&row.perms)));
        if can_add || can_edit || can_remove {
            body.push_str("<td>");
            if can_edit {
                let href = menu_href(&[
                    ("menu_name", picked.menu_name.as_str()),
                    ("state", picked.state.as_str()),
                    ("closed", view.closed_text.as_str()),
                    ("form", "edit"),
                    ("menu_id", ident.as_str()),
                ]);
                body.push_str(&format!("<a href=\"{}\">Edit</a> ", escape(&href)));
            }
            if can_add {
                let href = menu_href(&[
                    ("menu_name", picked.menu_name.as_str()),
                    ("state", picked.state.as_str()),
                    ("closed", view.closed_text.as_str()),
                    ("form", "add"),
                    ("parent_id", ident.as_str()),
                ]);
                body.push_str(&format!("<a href=\"{}\">Add</a> ", escape(&href)));
            }
            if can_remove {
                let href = menu_href(&[
                    ("menu_name", picked.menu_name.as_str()),
                    ("state", picked.state.as_str()),
                    ("closed", view.closed_text.as_str()),
                    ("form", "remove"),
                    ("menu_id", ident.as_str()),
                ]);
                body.push_str(&format!("<a href=\"{}\">Remove</a>", escape(&href)));
            }
            body.push_str("</td>");
        }
        body.push_str("</tr>");
    }
    body.push_str("</table></form>");
    if can_edit {
        body.push_str("<form id=\"menu-sort\" method=\"post\" action=\"/menus/sort\">");
        body.push_str(&hidden(user));
        body.push_str(&keep_fields(picked, &view.closed_text));
        body.push_str("</form>");
    }
    if show_add {
        body.push_str("<h2>Add a menu</h2><form method=\"post\" action=\"/menus/add\">");
        body.push_str(&hidden(user));
        body.push_str(&keep_fields(picked, &view.closed_text));
        body.push_str("<p>Name <input name=\"menu_name\"></p><p>Parent <select name=\"parent_id\">");
        body.push_str(&parent_options(all_rows, parent_id));
        body.push_str("</select></p><p>Type ");
        body.push_str(type_radios(""));
        body.push_str("</p><p>Permission <input name=\"perms\"></p><p>Path <input name=\"path\"></p><p><button type=\"submit\">Add</button></p></form>");
    }
    if show_edit {
        if let Some(row) = target {
            body.push_str("<h2>Edit a menu</h2><form method=\"post\" action=\"/menus/edit\">");
            body.push_str(&hidden(user));
            body.push_str(&format!(
                "<input type=\"hidden\" name=\"menu_id\" value=\"{}\">",
                escape(field(row, "menu_id"))
            ));
            body.push_str(&keep_fields(picked, &view.closed_text));
            body.push_str(&format!(
                "<p>Name <input name=\"menu_name\" value=\"{}\"></p><p>Parent <select name=\"parent_id\">",
                escape(field(row, "menu_name"))
            ));
            body.push_str(&parent_options(all_rows, field(row, "parent_id")));
            body.push_str("</select></p><p>Type ");
            body.push_str(type_radios(field(row, "menu_type")));
            body.push_str("</p><p>Visible ");
            body.push_str(visible_radios(field(row, "visible")));
            body.push_str(&format!(
                "</p><p>Permission <input name=\"perms\" value=\"{}\"></p><p>Path <input name=\"path\" value=\"{}\"></p><p><button type=\"submit\">Edit</button></p></form>",
                escape(field(row, "perms")),
                escape(field(row, "path"))
            ));
        }
    }
    if show_remove {
        if let Some(row) = target {
            body.push_str("<h2>Remove a menu</h2><form method=\"post\" action=\"/menus/remove\">");
            body.push_str(&hidden(user));
            body.push_str(&format!(
                "<input type=\"hidden\" name=\"menu_id\" value=\"{}\">",
                escape(field(row, "menu_id"))
            ));
            body.push_str(&keep_fields(picked, &view.closed_text));
            body.push_str(&format!(
                "<p>{}</p><p><button type=\"submit\">Remove</button></p></form>",
                escape(field(row, "menu_name"))
            ));
        }
    }
    body
}

fn keep_fields(picked: &MenuQuery, closed: &str) -> String {
    format!(
        "<input type=\"hidden\" name=\"keep_name\" value=\"{}\"><input type=\"hidden\" name=\"keep_state\" value=\"{}\"><input type=\"hidden\" name=\"keep_closed\" value=\"{}\">",
        escape(&picked.menu_name),
        escape(&picked.state),
        escape(closed)
    )
}

fn parent_options(rows: &[HashMap<String, String>], selected: &str) -> String {
    let mut body = String::new();
    let mark = if selected.is_empty() || selected == "0" {
        " selected"
    } else {
        ""
    };
    body.push_str(&format!("<option value=\"0\"{mark}>Root</option>"));
    for row in rows {
        let ident = field(row, "menu_id");
        let mark = if ident == selected { " selected" } else { "" };
        body.push_str(&format!(
            "<option value=\"{}\"{mark}>{}</option>",
            escape(ident),
            escape(field(row, "menu_name"))
        ));
    }
    body
}

fn type_radios(selected: &str) -> &'static str {
    match selected {
        "M" => "<label><input type=\"radio\" name=\"menu_type\" value=\"M\" checked> Directory</label> <label><input type=\"radio\" name=\"menu_type\" value=\"C\"> Page</label> <label><input type=\"radio\" name=\"menu_type\" value=\"F\"> Button</label>",
        "C" => "<label><input type=\"radio\" name=\"menu_type\" value=\"M\"> Directory</label> <label><input type=\"radio\" name=\"menu_type\" value=\"C\" checked> Page</label> <label><input type=\"radio\" name=\"menu_type\" value=\"F\"> Button</label>",
        "F" => "<label><input type=\"radio\" name=\"menu_type\" value=\"M\"> Directory</label> <label><input type=\"radio\" name=\"menu_type\" value=\"C\"> Page</label> <label><input type=\"radio\" name=\"menu_type\" value=\"F\" checked> Button</label>",
        _ => "<label><input type=\"radio\" name=\"menu_type\" value=\"M\"> Directory</label> <label><input type=\"radio\" name=\"menu_type\" value=\"C\"> Page</label> <label><input type=\"radio\" name=\"menu_type\" value=\"F\"> Button</label>",
    }
}

fn visible_radios(selected: &str) -> &'static str {
    if selected.trim() == "1" {
        "<label><input type=\"radio\" name=\"visible\" value=\"0\"> Show</label> <label><input type=\"radio\" name=\"visible\" value=\"1\" checked> Hide</label>"
    } else {
        "<label><input type=\"radio\" name=\"visible\" value=\"0\" checked> Show</label> <label><input type=\"radio\" name=\"visible\" value=\"1\"> Hide</label>"
    }
}

fn menu_type_label(menu_type: &str) -> &'static str {
    match menu_type {
        "M" => "Directory",
        "C" => "Page",
        "F" => "Button",
        _ => "Page",
    }
}

fn write_reply(stream: &mut TcpStream, reply: &HttpReply) -> std::io::Result<()> {
    let reason = match reply.status {
        200 => "OK",
        302 | 303 => "See Other",
        400 => "Bad Request",
        403 => "Forbidden",
        404 => "Not Found",
        _ => "OK",
    };
    let mut head = format!("HTTP/1.1 {} {reason}\r\n", reply.status);
    for (key, value) in &reply.headers {
        head.push_str(&format!("{key}: {value}\r\n"));
    }
    if !reply
        .headers
        .iter()
        .any(|(key, _)| key.eq_ignore_ascii_case("content-length"))
    {
        head.push_str(&format!("Content-Length: {}\r\n", reply.body.len()));
    }
    head.push_str("Connection: close\r\n\r\n");
    stream.write_all(head.as_bytes())?;
    stream.write_all(reply.body.as_bytes())?;
    Ok(())
}
