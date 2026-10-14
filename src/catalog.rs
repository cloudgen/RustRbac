//! Centralized parameters, dictionaries, and notices.
//! requirement-domain-rust-rbac — setup surface.

use std::collections::HashMap;
use std::sync::Arc;

use crate::access::Access;
use crate::audit::Audit;
use crate::error::AppError;
use crate::store::{field, Arg, Store};

pub struct Catalog {
    store: Arc<Store>,
    pub access: Arc<Access>,
    audit: Arc<Audit>,
}

impl Catalog {
    pub fn new(store: Arc<Store>, access: Arc<Access>, audit: Arc<Audit>) -> Self {
        Self {
            store,
            access,
            audit,
        }
    }

    pub fn config_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.access.require(actor, "system:config:list")?;
        self.store.query_all(
            "SELECT config_key, config_name, config_value, config_type
            FROM sys_config
            ORDER BY config_id",
            &[],
        )
    }

    pub fn config_set(
        &self,
        actor: &HashMap<String, String>,
        config_key: &str,
        config_value: &str,
    ) -> Result<String, AppError> {
        self.access.require(actor, "system:config:edit")?;
        let row = self.store.query_one(
            "SELECT config_id, config_key FROM sys_config WHERE config_key = ?",
            &[Arg::from(config_key)],
        )?;
        if row.is_none() {
            return Err(AppError::new(
                format!("Unknown parameter {config_key}."),
                "rust-rbac config-list",
            ));
        }
        if config_value.chars().count() > 500 {
            return Err(AppError::new(
                "config_value must be 500 characters or fewer.",
                "shorten the value",
            ));
        }
        self.store.execute(
            "UPDATE sys_config
            SET config_value = ?, update_by = ?, update_time = NOW()
            WHERE config_key = ?",
            &[
                Arg::from(config_value),
                Arg::from(field(actor, "user_name")),
                Arg::from(config_key),
            ],
        )?;
        self.audit
            .write_oper(actor, "Parameter", 2, "config_set", 0, "", config_key)?;
        Ok(config_key.to_string())
    }

    pub fn config_add(
        &self,
        actor: &HashMap<String, String>,
        config_key: &str,
        config_name: &str,
        config_value: &str,
    ) -> Result<String, AppError> {
        self.access.require(actor, "system:config:add")?;
        let key = config_key.trim();
        if key.is_empty() || key.chars().count() > 100 {
            return Err(AppError::new("config_key is required.", "set --config-key"));
        }
        let existing = self.store.query_one(
            "SELECT config_id FROM sys_config WHERE config_key = ?",
            &[Arg::from(key)],
        )?;
        if existing.is_some() {
            return Err(AppError::new(
                "That parameter already exists.",
                format!("rust-rbac config-set --config-key {key}"),
            ));
        }
        let name = clip(
            if config_name.is_empty() {
                key
            } else {
                config_name
            },
            100,
        );
        let value = clip(config_value, 500);
        self.store.execute(
            "INSERT INTO sys_config
              (config_name, config_key, config_value, config_type, create_by, create_time)
            VALUES (?, ?, ?, 'N', ?, NOW())",
            &[
                Arg::from(name.as_str()),
                Arg::from(key),
                Arg::from(value.as_str()),
                Arg::from(field(actor, "user_name")),
            ],
        )?;
        self.audit
            .write_oper(actor, "Parameter", 1, "config_add", 0, "", key)?;
        Ok(key.to_string())
    }

    pub fn dict_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.access.require(actor, "system:dict:list")?;
        self.store.query_all(
            "SELECT d.dict_type, d.dict_label, d.dict_value, d.status
            FROM sys_dict_data d
            ORDER BY d.dict_type, d.dict_sort, d.dict_code",
            &[],
        )
    }

    pub fn dict_add(
        &self,
        actor: &HashMap<String, String>,
        dict_type: &str,
        dict_label: &str,
        dict_value: &str,
    ) -> Result<String, AppError> {
        self.access.require(actor, "system:dict:add")?;
        let kind = self.store.query_one(
            "SELECT dict_type FROM sys_dict_type WHERE dict_type = ? AND status = '0'",
            &[Arg::from(dict_type)],
        )?;
        if kind.is_none() {
            return Err(AppError::new(
                format!("Unknown dictionary type {dict_type}."),
                "rust-rbac dict-list",
            ));
        }
        let label = dict_label.trim();
        let value = dict_value.trim();
        if label.is_empty() || value.is_empty() {
            return Err(AppError::new(
                "dict_label and dict_value are required.",
                "set both",
            ));
        }
        self.store.execute(
            "INSERT INTO sys_dict_data
              (dict_sort, dict_label, dict_value, dict_type, is_default, status,
               create_by, create_time)
            VALUES (0, ?, ?, ?, 'N', '0', ?, NOW())",
            &[
                Arg::from(clip(label, 100).as_str()),
                Arg::from(clip(value, 100).as_str()),
                Arg::from(dict_type),
                Arg::from(field(actor, "user_name")),
            ],
        )?;
        self.audit
            .write_oper(actor, "Dictionary", 1, "dict_add", 0, "", dict_type)?;
        Ok(label.to_string())
    }

    pub fn notice_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.access.require(actor, "system:notice:list")?;
        self.store.query_all(
            "SELECT notice_id, notice_title, notice_type, status
            FROM sys_notice
            ORDER BY notice_id",
            &[],
        )
    }

    pub fn notice_add(
        &self,
        actor: &HashMap<String, String>,
        title: &str,
        notice_type: &str,
        content: &str,
    ) -> Result<String, AppError> {
        self.access.require(actor, "system:notice:add")?;
        let name = title.trim();
        if name.is_empty() || name.chars().count() > 50 {
            return Err(AppError::new(
                "The notice title is required.",
                "set --title",
            ));
        }
        if notice_type != "1" && notice_type != "2" {
            return Err(AppError::new(
                "notice_type must be 1 (notice) or 2 (announcement).",
                "set --notice-type 1",
            ));
        }
        self.store.execute(
            "INSERT INTO sys_notice
              (notice_title, notice_type, notice_content, status, create_by, create_time)
            VALUES (?, ?, ?, '0', ?, NOW())",
            &[
                Arg::from(name),
                Arg::from(notice_type),
                Arg::from(content),
                Arg::from(field(actor, "user_name")),
            ],
        )?;
        self.audit
            .write_oper(actor, "Notice", 1, "notice_add", 0, "", name)?;
        Ok(name.to_string())
    }

    pub fn oper_log(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.access.require(actor, "monitor:operlog:list")?;
        self.audit.oper_rows(100)
    }

    pub fn login_log(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.access.require(actor, "monitor:logininfor:list")?;
        self.audit.login_rows(100)
    }

    pub fn online_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.access.require(actor, "monitor:online:list")?;
        self.audit.online_rows()
    }

    pub fn online_logout(
        &self,
        actor: &HashMap<String, String>,
        token: &str,
    ) -> Result<i64, AppError> {
        self.access.require(actor, "monitor:online:forceLogout")?;
        if token.is_empty() {
            return Err(AppError::new(
                "A session token is required.",
                "rust-rbac online-list",
            ));
        }
        let count = self.audit.drop_session(token)?;
        if count == 0 {
            return Err(AppError::new(
                "That session is not online.",
                "rust-rbac online-list",
            ));
        }
        let prefix: String = token.chars().take(8).collect();
        self.audit
            .write_oper(actor, "Online", 3, "online_logout", 0, "", &prefix)?;
        Ok(count)
    }
}

fn clip(text: &str, limit: usize) -> String {
    text.chars().take(limit).collect()
}
