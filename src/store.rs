//! Short database connections.
//! requirement-database — sqlite3 or mysql, parameters only, no foreign keys.
//! The product refuses pyrbac, pyrbac_test, ruoyi, ruoyi_db, and ry.

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::Path;
use std::sync::{Arc, Mutex};

use mysql::prelude::Queryable;
use rusqlite::params_from_iter;

use crate::error::AppError;
use crate::settings::Settings;

#[derive(Clone, Debug)]
pub enum Arg {
    Str(String),
    Int(i64),
}

impl From<&str> for Arg {
    fn from(value: &str) -> Self {
        Arg::Str(value.to_string())
    }
}

impl From<String> for Arg {
    fn from(value: String) -> Self {
        Arg::Str(value)
    }
}

impl From<i64> for Arg {
    fn from(value: i64) -> Self {
        Arg::Int(value)
    }
}

impl From<i32> for Arg {
    fn from(value: i32) -> Self {
        Arg::Int(i64::from(value))
    }
}

#[derive(Clone, Debug)]
pub struct SqlDone {
    pub rows: Vec<HashMap<String, String>>,
    pub lastrowid: i64,
    pub rowcount: i64,
}

pub struct Store {
    settings: Arc<Mutex<Settings>>,
}

impl Store {
    pub fn new(settings: Arc<Mutex<Settings>>) -> Self {
        Self { settings }
    }

    pub fn settings(&self) -> Arc<Mutex<Settings>> {
        Arc::clone(&self.settings)
    }

    pub fn query_all(
        &self,
        sql: &str,
        args: &[Arg],
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        Ok(self.run(sql, args, true)?.rows)
    }

    pub fn query_one(
        &self,
        sql: &str,
        args: &[Arg],
    ) -> Result<Option<HashMap<String, String>>, AppError> {
        let rows = self.query_all(sql, args)?;
        Ok(rows.into_iter().next())
    }

    pub fn execute(&self, sql: &str, args: &[Arg]) -> Result<SqlDone, AppError> {
        self.run(sql, args, false)
    }

    pub fn transaction<T>(
        &self,
        work: impl FnOnce(&mut Tx) -> Result<T, AppError>,
    ) -> Result<T, AppError> {
        let engine = self.engine()?;
        if engine == "sqlite3" {
            let connection = self.open_sqlite()?;
            connection
                .execute_batch("BEGIN")
                .map_err(|_| fail_statement())?;
            let mut tx = Tx::Sqlite(connection);
            match work(&mut tx) {
                Ok(value) => {
                    tx.finish(true)?;
                    Ok(value)
                }
                Err(error) => {
                    let _ = tx.finish(false);
                    Err(error)
                }
            }
        } else {
            let mut connection = self.open_mysql()?;
            connection
                .query_drop("START TRANSACTION")
                .map_err(|_| fail_statement())?;
            let mut tx = Tx::Mysql(connection);
            match work(&mut tx) {
                Ok(value) => {
                    tx.finish(true)?;
                    Ok(value)
                }
                Err(error) => {
                    let _ = tx.finish(false);
                    Err(error)
                }
            }
        }
    }

    fn engine(&self) -> Result<String, AppError> {
        let mut settings = self.settings.lock().expect("settings");
        settings.engine()
    }

    fn run(&self, sql: &str, args: &[Arg], fetch: bool) -> Result<SqlDone, AppError> {
        let engine = self.engine()?;
        if engine == "sqlite3" {
            let connection = self.open_sqlite()?;
            let mut tx = Tx::Sqlite(connection);
            tx.exec(sql, args, fetch)
        } else {
            let mut connection = self.open_mysql()?;
            connection
                .query_drop("START TRANSACTION")
                .map_err(|_| fail_statement())?;
            let mut tx = Tx::Mysql(connection);
            let done = tx.exec(sql, args, fetch)?;
            tx.finish(true)?;
            Ok(done)
        }
    }

    fn open_sqlite(&self) -> Result<rusqlite::Connection, AppError> {
        let path = {
            let mut settings = self.settings.lock().expect("settings");
            settings.sqlite_path()?
        };
        if let Some(folder) = Path::new(&path).parent() {
            if !folder.as_os_str().is_empty() {
                fs::create_dir_all(folder).map_err(|_| sqlite_open(&path))?;
                let _ = fs::set_permissions(folder, fs::Permissions::from_mode(0o700));
            }
        }
        let connection = rusqlite::Connection::open(&path).map_err(|_| sqlite_open(&path))?;
        connection
            .execute_batch("PRAGMA foreign_keys = OFF")
            .map_err(|_| sqlite_open(&path))?;
        let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
        Ok(connection)
    }

    fn open_mysql(&self) -> Result<mysql::Conn, AppError> {
        let config = {
            let mut settings = self.settings.lock().expect("settings");
            settings.database()?
        };
        let opts = mysql::OptsBuilder::new()
            .ip_or_hostname(Some(config.host.clone()))
            .tcp_port(config.port)
            .user(Some(config.user))
            .pass(Some(config.password))
            .db_name(Some(config.database.clone()));
        mysql::Conn::new(opts).map_err(|_| {
            AppError::new(
                format!("Cannot open the MySQL database {}.", config.database),
                "check RUSTRBAC_DB_HOST, RUSTRBAC_DB_USER, and RUSTRBAC_DB_PASSWORD, then rust-rbac db-init",
            )
        })
    }
}

pub enum Tx {
    Sqlite(rusqlite::Connection),
    Mysql(mysql::Conn),
}

impl Tx {
    pub fn run(&mut self, sql: &str, args: &[Arg]) -> Result<SqlDone, AppError> {
        self.exec(sql, args, true)
    }

    fn exec(&mut self, sql: &str, args: &[Arg], fetch: bool) -> Result<SqlDone, AppError> {
        match self {
            Tx::Sqlite(connection) => sqlite_exec(connection, sql, args, fetch),
            Tx::Mysql(connection) => mysql_exec(connection, sql, args, fetch),
        }
    }

    fn finish(&mut self, commit: bool) -> Result<(), AppError> {
        let sql = if commit { "COMMIT" } else { "ROLLBACK" };
        match self {
            Tx::Sqlite(connection) => {
                connection
                    .execute_batch(sql)
                    .map_err(|_| fail_statement())?;
            }
            Tx::Mysql(connection) => {
                connection.query_drop(sql).map_err(|_| fail_statement())?;
            }
        }
        Ok(())
    }
}

fn sqlite_exec(
    connection: &rusqlite::Connection,
    sql: &str,
    args: &[Arg],
    fetch: bool,
) -> Result<SqlDone, AppError> {
    let sql = adapt_sqlite(sql);
    let values: Vec<rusqlite::types::Value> = args
        .iter()
        .map(|arg| match arg {
            Arg::Str(value) => rusqlite::types::Value::Text(value.clone()),
            Arg::Int(value) => rusqlite::types::Value::Integer(*value),
        })
        .collect();
    if fetch {
        let mut statement = connection.prepare(&sql).map_err(|_| fail_statement())?;
        let mut rows = statement
            .query(params_from_iter(values.iter()))
            .map_err(|_| fail_statement())?;
        let mut found = Vec::new();
        loop {
            let next = rows.next().map_err(|_| fail_statement())?;
            let Some(row) = next else { break };
            found.push(sqlite_row(row)?);
        }
        Ok(SqlDone {
            rows: found,
            lastrowid: connection.last_insert_rowid(),
            rowcount: connection.changes() as i64,
        })
    } else {
        connection
            .execute(&sql, params_from_iter(values.iter()))
            .map_err(|_| fail_statement())?;
        Ok(SqlDone {
            rows: Vec::new(),
            lastrowid: connection.last_insert_rowid(),
            rowcount: connection.changes() as i64,
        })
    }
}

fn sqlite_row(row: &rusqlite::Row<'_>) -> Result<HashMap<String, String>, AppError> {
    let mut map = HashMap::new();
    let count = row.as_ref().column_count();
    for index in 0..count {
        let name = row
            .as_ref()
            .column_name(index)
            .map_err(|_| fail_statement())?
            .to_string();
        let value = match row.get_ref(index).map_err(|_| fail_statement())? {
            rusqlite::types::ValueRef::Null => String::new(),
            rusqlite::types::ValueRef::Integer(number) => number.to_string(),
            rusqlite::types::ValueRef::Real(number) => number.to_string(),
            rusqlite::types::ValueRef::Text(text) => String::from_utf8_lossy(text).into_owned(),
            rusqlite::types::ValueRef::Blob(bytes) => String::from_utf8_lossy(bytes).into_owned(),
        };
        map.insert(name, value);
    }
    Ok(map)
}

fn mysql_exec(
    connection: &mut mysql::Conn,
    sql: &str,
    args: &[Arg],
    fetch: bool,
) -> Result<SqlDone, AppError> {
    let params: Vec<mysql::Value> = args
        .iter()
        .map(|arg| match arg {
            Arg::Str(value) => mysql::Value::from(value.as_str()),
            Arg::Int(value) => mysql::Value::from(*value),
        })
        .collect();
    if fetch {
        let rows: Vec<mysql::Row> = connection
            .exec(sql, mysql::Params::Positional(params))
            .map_err(|_| fail_statement())?;
        let mapped = rows.into_iter().map(mysql_row).collect();
        Ok(SqlDone {
            rows: mapped,
            lastrowid: 0,
            rowcount: 0,
        })
    } else {
        connection
            .exec_drop(sql, mysql::Params::Positional(params))
            .map_err(|_| fail_statement())?;
        let last = connection.last_insert_id() as i64;
        let count = connection.affected_rows() as i64;
        Ok(SqlDone {
            rows: Vec::new(),
            lastrowid: last,
            rowcount: count,
        })
    }
}

fn mysql_row(row: mysql::Row) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for (index, column) in row.columns_ref().iter().enumerate() {
        let name = column.name_str().to_string();
        let value = row
            .get_opt::<mysql::Value, _>(index)
            .and_then(|item| item.ok())
            .unwrap_or(mysql::Value::NULL);
        map.insert(name, mysql_value(value));
    }
    map
}

fn mysql_value(value: mysql::Value) -> String {
    match value {
        mysql::Value::NULL => String::new(),
        mysql::Value::Bytes(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
        mysql::Value::Int(number) => number.to_string(),
        mysql::Value::UInt(number) => number.to_string(),
        mysql::Value::Float(number) => number.to_string(),
        mysql::Value::Double(number) => number.to_string(),
        mysql::Value::Date(year, month, day, hour, minute, second, _) => {
            format!("{year:04}-{month:02}-{day:02} {hour:02}:{minute:02}:{second:02}")
        }
        mysql::Value::Time(neg, days, hours, minutes, seconds, _) => {
            format!("{neg} {days} {hours}:{minutes}:{seconds}")
        }
    }
}

pub fn field<'a>(row: &'a HashMap<String, String>, name: &str) -> &'a str {
    if let Some(value) = row.get(name) {
        return value;
    }
    for (key, value) in row {
        if key.eq_ignore_ascii_case(name) {
            return value;
        }
    }
    ""
}

pub fn adapt_sqlite(sql: &str) -> String {
    let text = sql.replace("NOW()", "datetime('now','localtime')");
    replace_find_in_set(&text)
}

fn replace_find_in_set(sql: &str) -> String {
    let needle = "FIND_IN_SET(";
    let mut out = String::new();
    let mut index = 0;
    while let Some(pos) = sql[index..].find(needle) {
        let start = index + pos;
        out.push_str(&sql[index..start]);
        let after = start + needle.len();
        let rest = &sql[after..];
        let trimmed = rest.trim_start();
        if !trimmed.starts_with('?') {
            out.push_str(needle);
            index = after;
            continue;
        }
        let after_mark = after + (rest.len() - trimmed.len()) + 1;
        let rest2 = sql[after_mark..].trim_start();
        if !rest2.starts_with(',') {
            out.push_str(needle);
            index = after;
            continue;
        }
        let expr_start = after_mark + (sql[after_mark..].len() - rest2.len()) + 1;
        let skip = sql[expr_start..].len() - sql[expr_start..].trim_start().len();
        let expr_at = expr_start + skip;
        let Some(relative) = sql[expr_at..].find(')') else {
            out.push_str(needle);
            index = after;
            continue;
        };
        let expr = sql[expr_at..expr_at + relative].trim();
        out.push_str(&format!(
            "(instr(',' || {expr} || ',', ',' || ? || ',') > 0)"
        ));
        index = expr_at + relative + 1;
    }
    out.push_str(&sql[index..]);
    out
}

pub fn sqlite_statement(sql: &str) -> String {
    let mut text = sql.split_whitespace().collect::<Vec<_>>().join(" ");
    text = text.replace(" ENGINE=InnoDB DEFAULT CHARSET=utf8mb4", "");
    text = text.replace(
        "BIGINT NOT NULL AUTO_INCREMENT",
        "INTEGER PRIMARY KEY AUTOINCREMENT",
    );
    text = text.replace(
        "INT NOT NULL AUTO_INCREMENT",
        "INTEGER PRIMARY KEY AUTOINCREMENT",
    );
    text = replace_unique_key(&text);
    text = remove_secondary_keys(&text);
    text = remove_duplicate_primary(&text);
    while text.contains(", )") {
        text = text.replace(", )", ")");
    }
    text
}

fn replace_unique_key(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(pos) = rest.find("UNIQUE KEY ") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + "UNIQUE KEY ".len()..];
        let Some(paren) = after.find('(') else {
            out.push_str("UNIQUE KEY ");
            rest = after;
            continue;
        };
        let Some(end) = after[paren..].find(')') else {
            out.push_str("UNIQUE KEY ");
            rest = after;
            continue;
        };
        let cols = &after[paren..=paren + end];
        out.push_str("UNIQUE ");
        out.push_str(cols);
        rest = &after[paren + end + 1..];
    }
    out.push_str(rest);
    out
}

fn remove_secondary_keys(text: &str) -> String {
    let mut out = String::new();
    let mut rest = text;
    while let Some(pos) = rest.find(", KEY ") {
        out.push_str(&rest[..pos]);
        let after = &rest[pos + ", KEY ".len()..];
        let Some(paren) = after.find('(') else { break };
        let Some(end) = after[paren..].find(')') else {
            break;
        };
        rest = &after[paren + end + 1..];
    }
    out.push_str(rest);
    out
}

fn remove_duplicate_primary(text: &str) -> String {
    let marker = "INTEGER PRIMARY KEY AUTOINCREMENT";
    let mut text = text.to_string();
    let mut search = 0;
    while let Some(pos) = text[search..].find(marker) {
        let absolute = search + pos;
        let before = &text[..absolute];
        let Some(name_end) = before.rfind(|ch: char| ch.is_whitespace()) else {
            break;
        };
        let name_start = before[..name_end]
            .rfind(|ch: char| !ch.is_ascii_alphanumeric() && ch != '_')
            .map(|index| index + 1)
            .unwrap_or(0);
        let name = before[name_start..name_end].trim();
        if name.is_empty() {
            search = absolute + marker.len();
            continue;
        }
        let pattern = format!(", PRIMARY KEY ({name})");
        if let Some(found) = text.find(&pattern) {
            text.replace_range(found..found + pattern.len(), "");
        }
        search = absolute + marker.len();
    }
    text
}

fn sqlite_open(path: &str) -> AppError {
    AppError::new(
        format!("Cannot open the sqlite3 file {path}."),
        "check that the folder is writable, or set RUSTRBAC_SQLITE_FILE, then rust-rbac db-init",
    )
}

fn fail_statement() -> AppError {
    AppError::new("The database statement failed.", "rust-rbac db-init")
}
