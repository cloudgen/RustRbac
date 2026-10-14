//! Login log, operation log, and online sessions.
//! requirement-rbac — audit rows and online users.
//! requirement-rust-logging — this module is not StatusLog.

use std::collections::HashMap;
use std::sync::Arc;

use crate::error::AppError;
use crate::store::{field, Arg, Store};

pub struct Audit {
    store: Arc<Store>,
}

impl Audit {
    pub fn new(store: Arc<Store>) -> Self {
        Self { store }
    }

    pub fn write_login(
        &self,
        user_name: &str,
        ipaddr: &str,
        status: &str,
        message: &str,
        browser: &str,
        os_name: &str,
    ) -> Result<(), AppError> {
        self.store.execute(
            "INSERT INTO sys_logininfor
              (user_name, ipaddr, browser, os, status, msg, login_time)
            VALUES (?, ?, ?, ?, ?, ?, NOW())",
            &[
                Arg::from(user_name),
                Arg::from(ipaddr),
                Arg::from(browser),
                Arg::from(os_name),
                Arg::from(status),
                Arg::from(message),
            ],
        )?;
        Ok(())
    }

    pub fn write_oper(
        &self,
        actor: &HashMap<String, String>,
        title: &str,
        business_type: i64,
        method: &str,
        status: i64,
        error_msg: &str,
        param: &str,
    ) -> Result<(), AppError> {
        let name = actor.get("user_name").map(String::as_str).unwrap_or("");
        let dept = actor.get("dept_name").map(String::as_str).unwrap_or("");
        let param = clip(param, 2000);
        let error_msg = clip(error_msg, 2000);
        self.store.execute(
            "INSERT INTO sys_oper_log
              (title, business_type, method, request_method, operator_type,
               oper_name, dept_name, oper_param, status, error_msg, oper_time)
            VALUES (?, ?, ?, 'CALL', 1, ?, ?, ?, ?, ?, NOW())",
            &[
                Arg::from(title),
                Arg::from(business_type),
                Arg::from(method),
                Arg::from(name),
                Arg::from(dept),
                Arg::from(param.as_str()),
                Arg::from(status),
                Arg::from(error_msg.as_str()),
            ],
        )?;
        Ok(())
    }

    pub fn login_rows(&self, limit: i64) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.store.query_all(
            "SELECT info_id, user_name, ipaddr, status, msg, login_time
            FROM sys_logininfor
            ORDER BY info_id DESC
            LIMIT ?",
            &[Arg::from(limit)],
        )
    }

    pub fn oper_rows(&self, limit: i64) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.store.query_all(
            "SELECT oper_id, title, business_type, method, oper_name, status, oper_time
            FROM sys_oper_log
            ORDER BY oper_id DESC
            LIMIT ?",
            &[Arg::from(limit)],
        )
    }

    pub fn open_session(
        &self,
        token: &str,
        user: &HashMap<String, String>,
        ipaddr: &str,
        user_agent: &str,
        csrf: &str,
        surface: &str,
    ) -> Result<(), AppError> {
        let agent = clip(user_agent, 200);
        self.store.execute(
            "INSERT INTO sys_session
              (token, user_id, user_name, ipaddr, user_agent, csrf, surface,
               login_time, last_access)
            VALUES (?, ?, ?, ?, ?, ?, ?, NOW(), NOW())",
            &[
                Arg::from(token),
                Arg::from(field(user, "user_id")),
                Arg::from(field(user, "user_name")),
                Arg::from(ipaddr),
                Arg::from(agent.as_str()),
                Arg::from(csrf),
                Arg::from(surface),
            ],
        )?;
        Ok(())
    }

    pub fn touch(&self, token: &str) -> Result<(), AppError> {
        self.store.execute(
            "UPDATE sys_session SET last_access = NOW() WHERE token = ?",
            &[Arg::from(token)],
        )?;
        Ok(())
    }

    pub fn session_row(&self, token: &str) -> Result<Option<HashMap<String, String>>, AppError> {
        if token.is_empty() {
            return Ok(None);
        }
        self.store.query_one(
            "SELECT token, user_id, user_name, ipaddr, user_agent, csrf, surface,
                   login_time, last_access
            FROM sys_session
            WHERE token = ?",
            &[Arg::from(token)],
        )
    }

    pub fn online_rows(&self) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.store.query_all(
            "SELECT token, user_id, user_name, ipaddr, surface, login_time, last_access
            FROM sys_session
            ORDER BY last_access DESC",
            &[],
        )
    }

    pub fn drop_session(&self, token: &str) -> Result<i64, AppError> {
        let done = self.store.execute(
            "DELETE FROM sys_session WHERE token = ?",
            &[Arg::from(token)],
        )?;
        Ok(done.rowcount)
    }
}

fn clip(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}
