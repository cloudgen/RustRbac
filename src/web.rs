//! Local web pages for the same RBAC actions as the terminal.
//! requirement-web-interface — loopback pages, cookie session, permission checks.

use std::collections::HashMap;
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
        let method = method.to_ascii_uppercase();
        if method == "GET" && path == "/login" {
            return self.html(200, &self.login_page(""), &[]);
        }
        if method == "POST" && path == "/login" {
            return self.post_login(body, client_ip);
        }
        let user = self.user_from(headers);
        let Some(user) = user else {
            return redirect("/login", None, false);
        };
        if method == "POST" && !csrf_ok(&user, body) {
            return self.html(
                400,
                &self.page(
                    Some(&user),
                    "Rejected",
                    "<p>The form token did not match this session.</p>",
                ),
                &[],
            );
        }
        match (method.as_str(), path) {
            ("GET", "/") => self.html(200, &self.home_page(&user), &[]),
            ("GET", "/users") => {
                self.guarded(&user, "system:user:list", |app| app.users_page(&user))
            }
            ("GET", "/roles") => {
                self.guarded(&user, "system:role:list", |app| app.roles_page(&user))
            }
            ("GET", "/menus") => {
                self.guarded(&user, "system:menu:list", |app| app.menus_page(&user))
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
            _ => self.html(
                404,
                &self.page(
                    Some(&user),
                    "Not found",
                    "<p>That page is not on this site.</p>",
                ),
                &[],
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
        let body = format!(
            "<p class=\"error\">{}</p><p>{}</p>",
            escape(&error.message),
            escape(&error.next)
        );
        self.html(code, &self.page(Some(user), "Not completed", &body), &[])
    }

    fn post_login(&self, body: &str, client_ip: &str) -> HttpReply {
        if let Err(error) = self.apply_engine(body, client_ip, false) {
            return self.html(400, &self.login_page(&error.message), &[]);
        }
        if let Err(error) = self.schema.ensure() {
            return self.html(400, &self.login_page(&error.message), &[]);
        }
        match self.access.login(
            form_first(body, "user_name"),
            form_first(body, "password"),
            client_ip,
            "web",
            "rust-rbac",
        ) {
            Ok(user) => redirect("/", Some(field(&user, "token")), false),
            Err(error) => self.html(400, &self.login_page(&error.message), &[]),
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
            (
                "Content-Security-Policy".to_string(),
                "default-src 'none'; style-src 'unsafe-inline'; form-action 'self'".to_string(),
            ),
        ];
        headers.extend(extra.iter().cloned());
        HttpReply {
            status,
            headers,
            body: body.to_string(),
        }
    }

    fn page(&self, user: Option<&HashMap<String, String>>, title: &str, body: &str) -> String {
        format!(
            "<!DOCTYPE html><html lang=\"en\"><head><meta charset=\"utf-8\"><title>{}</title><style>body{{font-family:sans-serif;margin:1.5rem;max-width:60rem}}nav a{{margin-right:.7rem}}table{{border-collapse:collapse;margin:1rem 0}}td,th{{border:1px solid #ccc;padding:.25rem .45rem;text-align:left}}.error{{color:#8a1f1f}}</style></head><body><h1>{}</h1>{}{}</body></html>",
            escape(title),
            escape(title),
            self.nav(user),
            body
        )
    }

    fn nav(&self, user: Option<&HashMap<String, String>>) -> String {
        let Some(user) = user else {
            return "<nav><a href=\"/login\">Sign in</a></nav>".to_string();
        };
        let pairs = [
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
        let mut parts = vec!["<a href=\"/\">Home</a>".to_string()];
        for (href, label, perm) in pairs {
            if self
                .access
                .allows(field(user, "user_id"), perm)
                .unwrap_or(false)
            {
                parts.push(format!("<a href=\"{href}\">{label}</a>"));
            }
        }
        parts.push(format!(
            "<form method=\"post\" action=\"/logout\" style=\"display:inline\">{}<button type=\"submit\">Sign out</button></form>",
            hidden(user)
        ));
        format!("<nav>{}</nav>", parts.join(" "))
    }

    fn login_page(&self, error: &str) -> String {
        let error_html = if error.is_empty() {
            String::new()
        } else {
            format!("<p class=\"error\">{}</p>", escape(error))
        };
        let body = format!(
            "{error_html}<form method=\"post\" action=\"/login\">{}<p><label>Account <input name=\"user_name\" autocomplete=\"username\"></label></p><p><label>Password <input name=\"password\" type=\"password\" autocomplete=\"current-password\"></label></p><p><button type=\"submit\">Sign in</button></p></form><p>Seeded accounts and initial passwords are in the product README. Change them after the first sign-in. This site listens on this computer only unless remote bind was explicitly allowed. The default database is the sqlite3 file. mysql needs RUSTRBAC_DB_PASSWORD.</p>",
            self.engine_fields()
        );
        self.page(None, "Sign in", &body)
    }

    fn engine_fields(&self) -> String {
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
        let radio = |value: &str, label: &str| {
            let checked = if current == value { " checked" } else { "" };
            format!(
                "<label><input type=\"radio\" name=\"engine\" value=\"{value}\"{checked}> {}</label>",
                escape(label)
            )
        };
        format!(
            "<fieldset><legend>Database</legend><p>{}</p><p>{}</p></fieldset>",
            radio("sqlite3", &format!("sqlite3 {path}{mark}")),
            radio("mysql", &format!("mysql {name}"))
        )
    }

    fn home_page(&self, user: &HashMap<String, String>) -> String {
        let who = self.access.describe(user).unwrap_or_default();
        let body = format!(
            "<p>Signed in as <strong>{}</strong>.</p><pre>{}</pre>",
            escape(field(user, "user_name")),
            escape(&who)
        );
        self.page(Some(user), "Home", &body)
    }

    fn users_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.access.user_list(user)?;
        let mut parts = vec![table(
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
            parts.push(format!(
                "<h2>Add a user</h2><form method=\"post\" action=\"/users/add\">{}<p>Account <input name=\"user_name\"></p><p>Name <input name=\"nick_name\"></p><p>Department id <input name=\"dept_id\"></p><p>Role id <input name=\"role_id\"></p><p>Password <input name=\"password\" type=\"password\"></p><p><button type=\"submit\">Add</button></p></form>",
                hidden(user)
            ));
        }
        Ok(self.page(Some(user), "Users", &parts.join("")))
    }

    fn roles_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.access.role_list(user)?;
        Ok(self.page(
            Some(user),
            "Roles",
            &table(
                &["role_id", "role_name", "role_key", "data_scope", "status"],
                &rows,
            ),
        ))
    }

    fn menus_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.access.menu_list(user)?;
        Ok(self.page(
            Some(user),
            "Menus",
            &table(
                &["menu_id", "parent_id", "menu_name", "menu_type", "perms"],
                &rows,
            ),
        ))
    }

    fn depts_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.access.dept_list(user)?;
        Ok(self.page(
            Some(user),
            "Departments",
            &table(
                &["dept_id", "parent_id", "dept_name", "ancestors", "status"],
                &rows,
            ),
        ))
    }

    fn posts_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.access.post_list(user)?;
        Ok(self.page(
            Some(user),
            "Posts",
            &table(
                &["post_id", "post_code", "post_name", "post_sort", "status"],
                &rows,
            ),
        ))
    }

    fn configs_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.catalog.config_list(user)?;
        Ok(self.page(
            Some(user),
            "Parameters",
            &table(
                &["config_key", "config_name", "config_value", "config_type"],
                &rows,
            ),
        ))
    }

    fn dicts_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.catalog.dict_list(user)?;
        Ok(self.page(
            Some(user),
            "Dictionaries",
            &table(&["dict_type", "dict_label", "dict_value", "status"], &rows),
        ))
    }

    fn notices_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let rows = self.catalog.notice_list(user)?;
        Ok(self.page(
            Some(user),
            "Notices",
            &table(
                &["notice_id", "notice_title", "notice_type", "status"],
                &rows,
            ),
        ))
    }

    fn database_page(&self, user: &HashMap<String, String>) -> Result<String, AppError> {
        let sentence = self.settings.lock().expect("settings").engine_sentence()?;
        let body = format!(
            "<p>{} The next sign-in uses the saved engine.</p><form method=\"post\" action=\"/database\">{}{}<p><button type=\"submit\">Save</button></p></form>",
            escape(&sentence),
            self.engine_fields(),
            hidden(user)
        );
        Ok(self.page(Some(user), "Database", &body))
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

fn table(columns: &[&str], rows: &[HashMap<String, String>]) -> String {
    let head: String = columns
        .iter()
        .map(|name| format!("<th>{}</th>", escape(name)))
        .collect();
    let mut body = String::new();
    if rows.is_empty() {
        body.push_str(&format!(
            "<tr><td colspan=\"{}\">(none)</td></tr>",
            columns.len()
        ));
    }
    for row in rows {
        body.push_str("<tr>");
        for name in columns {
            body.push_str(&format!("<td>{}</td>", escape(field(row, name))));
        }
        body.push_str("</tr>");
    }
    format!("<table><tr>{head}</tr>{body}</table>")
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
