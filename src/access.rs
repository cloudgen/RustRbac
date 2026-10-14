//! RBAC commands shared by the terminal and the web interface.
//! requirement-rbac — users, roles, menus, departments, posts, data scope.
//! requirement-domain-rust-rbac — one permission path.

use std::collections::HashMap;
use std::sync::Arc;

use base64::Engine;
use regex::Regex;

use crate::audit::Audit;
use crate::error::AppError;
use crate::password::PasswordHash;
use crate::store::{field, Arg, Store};

pub struct Access {
    pub store: Arc<Store>,
    audit: Arc<Audit>,
    passwords: PasswordHash,
    dummy: String,
    user_name: Regex,
}

impl Access {
    pub const PROTECTED_USER: i64 = 1;
    pub const PROTECTED_ROLE: i64 = 1;

    pub fn new(store: Arc<Store>, audit: Arc<Audit>) -> Self {
        let passwords = PasswordHash;
        let dummy = passwords.hash("not-a-real-password");
        Self {
            store,
            audit,
            passwords,
            dummy,
            user_name: Regex::new(r"^[A-Za-z][A-Za-z0-9_]{2,29}$").expect("user name pattern"),
        }
    }

    pub fn require(&self, actor: &HashMap<String, String>, perm: &str) -> Result<(), AppError> {
        if !self.allows(field(actor, "user_id"), perm)? {
            return Err(AppError::new(
                format!("Missing permission {perm}."),
                format!("ask an administrator to grant {perm}"),
            ));
        }
        Ok(())
    }

    pub fn allows(&self, user_id: &str, perm: &str) -> Result<bool, AppError> {
        if self.is_admin(user_id)? {
            return Ok(true);
        }
        Ok(self.perms_of(user_id)?.iter().any(|item| item == perm))
    }

    pub fn is_admin(&self, user_id: &str) -> Result<bool, AppError> {
        for role in self.roles_of(user_id)? {
            if field(&role, "role_key") == "admin" {
                return Ok(true);
            }
        }
        Ok(false)
    }

    pub fn roles_of(&self, user_id: &str) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.store.query_all(
            "SELECT r.role_id, r.role_key, r.role_name, r.data_scope, r.status
            FROM sys_role r
            JOIN sys_user_role ur ON ur.role_id = r.role_id
            WHERE ur.user_id = ? AND r.del_flag = '0' AND r.status = '0'",
            &[Arg::from(user_id)],
        )
    }

    fn perms_of(&self, user_id: &str) -> Result<Vec<String>, AppError> {
        if self.is_admin(user_id)? {
            return Ok(vec!["*:*:*".to_string()]);
        }
        let rows = self.store.query_all(
            "SELECT DISTINCT m.perms AS perms
            FROM sys_menu m
            JOIN sys_role_menu rm ON rm.menu_id = m.menu_id
            JOIN sys_user_role ur ON ur.role_id = rm.role_id
            JOIN sys_role r ON r.role_id = ur.role_id
            WHERE ur.user_id = ?
              AND r.del_flag = '0' AND r.status = '0'
              AND m.status = '0'
              AND m.perms IS NOT NULL AND m.perms <> ''",
            &[Arg::from(user_id)],
        )?;
        Ok(rows
            .iter()
            .map(|row| field(row, "perms").to_string())
            .collect())
    }

    pub fn user_by_id(&self, user_id: &str) -> Result<Option<HashMap<String, String>>, AppError> {
        self.store.query_one(
            "SELECT u.user_id, u.user_name, u.nick_name, u.dept_id, u.email,
                   u.phonenumber, u.sex, u.status, u.del_flag, d.dept_name
            FROM sys_user u
            LEFT JOIN sys_dept d ON d.dept_id = u.dept_id AND d.del_flag = '0'
            WHERE u.user_id = ? AND u.del_flag = '0'",
            &[Arg::from(user_id)],
        )
    }

    pub fn actor_from_token(
        &self,
        token: &str,
    ) -> Result<Option<HashMap<String, String>>, AppError> {
        let Some(row) = self.audit.session_row(token)? else {
            return Ok(None);
        };
        let Some(mut user) = self.user_by_id(field(&row, "user_id"))? else {
            return Ok(None);
        };
        if field(&user, "status") != "0" {
            return Ok(None);
        }
        user.insert("token".to_string(), field(&row, "token").to_string());
        user.insert("csrf".to_string(), field(&row, "csrf").to_string());
        self.audit.touch(token)?;
        Ok(Some(user))
    }

    pub fn login(
        &self,
        user_name: &str,
        password: &str,
        ipaddr: &str,
        surface: &str,
        user_agent: &str,
    ) -> Result<HashMap<String, String>, AppError> {
        let name = user_name.trim();
        let row = self.store.query_one(
            "SELECT user_id, user_name, nick_name, password, status, del_flag, dept_id
            FROM sys_user
            WHERE user_name = ? AND del_flag = '0'",
            &[Arg::from(name)],
        )?;
        let Some(row) = row else {
            let _ = self.passwords.verify(password, &self.dummy);
            self.audit
                .write_login(name, ipaddr, "1", "unknown account", "rust-rbac", "")?;
            return Err(AppError::new(
                "Unknown account or wrong password.",
                "rust-rbac login --user-name NAME --password SECRET",
            ));
        };
        if field(&row, "status") != "0" {
            let _ = self.passwords.verify(password, &self.dummy);
            self.audit
                .write_login(name, ipaddr, "1", "disabled", "rust-rbac", "")?;
            return Err(AppError::new(
                "This account is disabled.",
                "ask an administrator to enable it",
            ));
        }
        if !self.passwords.verify(password, field(&row, "password")) {
            self.audit
                .write_login(name, ipaddr, "1", "wrong password", "rust-rbac", "")?;
            return Err(AppError::new(
                "Unknown account or wrong password.",
                "rust-rbac login --user-name NAME --password SECRET",
            ));
        }
        let token = token_urlsafe(32);
        let csrf = token_urlsafe(24);
        let user_id = field(&row, "user_id").to_string();
        let Some(mut user) = self.user_by_id(&user_id)? else {
            return Err(AppError::new(
                "Unknown account or wrong password.",
                "rust-rbac login --user-name NAME --password SECRET",
            ));
        };
        self.audit
            .open_session(&token, &user, ipaddr, user_agent, &csrf, surface)?;
        self.store.execute(
            "UPDATE sys_user SET login_ip = ?, login_date = NOW() WHERE user_id = ?",
            &[Arg::from(ipaddr), Arg::from(user_id.as_str())],
        )?;
        self.audit
            .write_login(name, ipaddr, "0", "success", "rust-rbac", "")?;
        user.insert("token".to_string(), token);
        user.insert("csrf".to_string(), csrf);
        Ok(user)
    }

    pub fn logout(&self, token: &str) -> Result<i64, AppError> {
        self.audit.drop_session(token)
    }

    fn scope_clause(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<(String, Vec<Arg>), AppError> {
        let user_id = field(actor, "user_id");
        if self.is_admin(user_id)? {
            return Ok(("1=1".to_string(), Vec::new()));
        }
        let roles = self.roles_of(user_id)?;
        if roles.iter().any(|role| field(role, "data_scope") == "1") {
            return Ok(("1=1".to_string(), Vec::new()));
        }
        let dept_id = field(actor, "dept_id");
        let mut parts = Vec::new();
        let mut args = Vec::new();
        for role in &roles {
            match field(role, "data_scope") {
                "2" => {
                    let chosen = self.store.query_all(
                        "SELECT dept_id FROM sys_role_dept WHERE role_id = ?",
                        &[Arg::from(field(role, "role_id"))],
                    )?;
                    if chosen.is_empty() {
                        continue;
                    }
                    let marks = vec!["?"; chosen.len()].join(", ");
                    parts.push(format!("u.dept_id IN ({marks})"));
                    for item in chosen {
                        args.push(Arg::from(field(&item, "dept_id")));
                    }
                }
                "3" => {
                    if dept_id.is_empty() {
                        continue;
                    }
                    parts.push("u.dept_id = ?".to_string());
                    args.push(Arg::from(dept_id));
                }
                "4" => {
                    if dept_id.is_empty() {
                        continue;
                    }
                    parts.push("(u.dept_id = ? OR FIND_IN_SET(?, d.ancestors))".to_string());
                    args.push(Arg::from(dept_id));
                    args.push(Arg::from(dept_id));
                }
                _ => {}
            }
        }
        if parts.is_empty() {
            return Ok(("1=0".to_string(), Vec::new()));
        }
        Ok((format!("({})", parts.join(" OR ")), args))
    }

    pub fn user_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.require(actor, "system:user:list")?;
        let (clause, args) = self.scope_clause(actor)?;
        let sql = format!(
            "SELECT u.user_id, u.user_name, u.nick_name, u.dept_id,
                   d.dept_name, u.status
            FROM sys_user u
            LEFT JOIN sys_dept d ON d.dept_id = u.dept_id
            WHERE u.del_flag = '0' AND {clause}
            ORDER BY u.user_id
            LIMIT 200"
        );
        self.store.query_all(&sql, &args)
    }

    fn check_name(&self, user_name: &str) -> Result<(), AppError> {
        if !self.user_name.is_match(user_name) {
            return Err(AppError::new(
                "user_name must start with a letter and use 3 to 30 letters, digits, or underscores.",
                "pick another user_name",
            ));
        }
        Ok(())
    }

    fn dept_exists(&self, dept_id: i64) -> Result<(), AppError> {
        let row = self.store.query_one(
            "SELECT dept_id FROM sys_dept WHERE dept_id = ? AND del_flag = '0'",
            &[Arg::from(dept_id)],
        )?;
        if row.is_none() {
            return Err(AppError::new(
                format!("Unknown department {dept_id}."),
                "rust-rbac dept-list",
            ));
        }
        Ok(())
    }

    fn role_exists(&self, role_id: i64) -> Result<(), AppError> {
        let row = self.store.query_one(
            "SELECT role_id FROM sys_role WHERE role_id = ? AND del_flag = '0'",
            &[Arg::from(role_id)],
        )?;
        if row.is_none() {
            return Err(AppError::new(
                format!("Unknown role {role_id}."),
                "rust-rbac role-list",
            ));
        }
        Ok(())
    }

    fn init_password(&self) -> Result<String, AppError> {
        let row = self.store.query_one(
            "SELECT config_value FROM sys_config WHERE config_key = ?",
            &[Arg::from("sys.user.initPassword")],
        )?;
        if let Some(row) = row {
            let value = field(&row, "config_value");
            if !value.is_empty() {
                return Ok(value.to_string());
            }
        }
        Ok("123456".to_string())
    }

    pub fn user_add(
        &self,
        actor: &HashMap<String, String>,
        user_name: &str,
        nick_name: &str,
        dept_id: i64,
        role_id: i64,
        password: &str,
    ) -> Result<HashMap<String, String>, AppError> {
        self.require(actor, "system:user:add")?;
        self.check_name(user_name)?;
        let nick = nick_name.trim();
        if nick.is_empty() || nick.chars().count() > 30 {
            return Err(AppError::new(
                "nick_name is required and must be 30 characters or fewer.",
                "set --nick-name",
            ));
        }
        self.dept_exists(dept_id)?;
        self.role_exists(role_id)?;
        let existing = self.store.query_one(
            "SELECT user_id FROM sys_user WHERE user_name = ? AND del_flag = '0'",
            &[Arg::from(user_name)],
        )?;
        if existing.is_some() {
            return Err(AppError::new(
                "That user_name is already in use.",
                "pick another user_name",
            ));
        }
        let secret = if password.is_empty() {
            self.init_password()?
        } else {
            password.to_string()
        };
        if secret.chars().count() < 5 {
            return Err(AppError::new(
                "The password must be at least 5 characters.",
                "set --password",
            ));
        }
        let stored = self.passwords.hash(&secret);
        let done = self.store.execute(
            "INSERT INTO sys_user
              (dept_id, user_name, nick_name, password, status, del_flag,
               create_by, create_time, pwd_update_date)
            VALUES (?, ?, ?, ?, '0', '0', ?, NOW(), NOW())",
            &[
                Arg::from(dept_id),
                Arg::from(user_name),
                Arg::from(nick),
                Arg::from(stored.as_str()),
                Arg::from(field(actor, "user_name")),
            ],
        )?;
        self.store.execute(
            "INSERT INTO sys_user_role (user_id, role_id) VALUES (?, ?)",
            &[Arg::from(done.lastrowid), Arg::from(role_id)],
        )?;
        self.audit
            .write_oper(actor, "User", 1, "user_add", 0, "", user_name)?;
        self.user_by_id(&done.lastrowid.to_string())?
            .ok_or_else(|| AppError::new("Unknown user.", "rust-rbac user-list"))
    }

    pub fn user_edit(
        &self,
        actor: &HashMap<String, String>,
        user_id: i64,
        nick_name: Option<&str>,
        dept_id: Option<i64>,
        status: Option<&str>,
    ) -> Result<HashMap<String, String>, AppError> {
        self.require(actor, "system:user:edit")?;
        let id = user_id.to_string();
        if self.user_by_id(&id)?.is_none() {
            return Err(AppError::new(
                format!("Unknown user {user_id}."),
                "rust-rbac user-list",
            ));
        }
        if user_id == Self::PROTECTED_USER {
            if let Some(status) = status {
                if status != "0" {
                    return Err(AppError::new(
                        "The built-in administrator stays enabled.",
                        "leave status 0",
                    ));
                }
            }
        }
        if let Some(dept_id) = dept_id {
            self.dept_exists(dept_id)?;
        }
        if let Some(nick_name) = nick_name {
            let nick = nick_name.trim();
            if nick.is_empty() || nick.chars().count() > 30 {
                return Err(AppError::new(
                    "nick_name is required and must be 30 characters or fewer.",
                    "set --nick-name",
                ));
            }
            self.store.execute(
                "UPDATE sys_user SET nick_name = ?, update_by = ?, update_time = NOW() WHERE user_id = ?",
                &[
                    Arg::from(nick),
                    Arg::from(field(actor, "user_name")),
                    Arg::from(user_id),
                ],
            )?;
        }
        if let Some(dept_id) = dept_id {
            self.store.execute(
                "UPDATE sys_user SET dept_id = ?, update_by = ?, update_time = NOW() WHERE user_id = ?",
                &[
                    Arg::from(dept_id),
                    Arg::from(field(actor, "user_name")),
                    Arg::from(user_id),
                ],
            )?;
        }
        if let Some(status) = status {
            if status != "0" && status != "1" {
                return Err(AppError::new(
                    "status must be 0 or 1.",
                    "use 0 for normal or 1 for disabled",
                ));
            }
            self.store.execute(
                "UPDATE sys_user SET status = ?, update_by = ?, update_time = NOW() WHERE user_id = ?",
                &[
                    Arg::from(status),
                    Arg::from(field(actor, "user_name")),
                    Arg::from(user_id),
                ],
            )?;
        }
        self.audit
            .write_oper(actor, "User", 2, "user_edit", 0, "", &id)?;
        self.user_by_id(&id)?
            .ok_or_else(|| AppError::new("Unknown user.", "rust-rbac user-list"))
    }

    pub fn user_remove(
        &self,
        actor: &HashMap<String, String>,
        user_id: i64,
    ) -> Result<String, AppError> {
        self.require(actor, "system:user:remove")?;
        if user_id == Self::PROTECTED_USER {
            return Err(AppError::new(
                "The built-in administrator cannot be removed.",
                "disable another account instead",
            ));
        }
        if field(actor, "user_id") == user_id.to_string() {
            return Err(AppError::new(
                "You cannot remove the account you are using.",
                "sign in as another administrator",
            ));
        }
        let Some(user) = self.user_by_id(&user_id.to_string())? else {
            return Err(AppError::new(
                format!("Unknown user {user_id}."),
                "rust-rbac user-list",
            ));
        };
        self.store.execute(
            "UPDATE sys_user
            SET del_flag = '2', update_by = ?, update_time = NOW()
            WHERE user_id = ?",
            &[Arg::from(field(actor, "user_name")), Arg::from(user_id)],
        )?;
        self.store.execute(
            "DELETE FROM sys_user_role WHERE user_id = ?",
            &[Arg::from(user_id)],
        )?;
        self.store.execute(
            "DELETE FROM sys_user_post WHERE user_id = ?",
            &[Arg::from(user_id)],
        )?;
        let name = field(&user, "user_name").to_string();
        self.audit
            .write_oper(actor, "User", 3, "user_remove", 0, "", &name)?;
        Ok(name)
    }

    pub fn user_reset_password(
        &self,
        actor: &HashMap<String, String>,
        user_id: i64,
        password: &str,
    ) -> Result<String, AppError> {
        self.require(actor, "system:user:resetPwd")?;
        let Some(user) = self.user_by_id(&user_id.to_string())? else {
            return Err(AppError::new(
                format!("Unknown user {user_id}."),
                "rust-rbac user-list",
            ));
        };
        if password.chars().count() < 5 {
            return Err(AppError::new(
                "The password must be at least 5 characters.",
                "set --password",
            ));
        }
        let stored = self.passwords.hash(password);
        self.store.execute(
            "UPDATE sys_user
            SET password = ?, pwd_update_date = NOW(), update_by = ?, update_time = NOW()
            WHERE user_id = ?",
            &[
                Arg::from(stored.as_str()),
                Arg::from(field(actor, "user_name")),
                Arg::from(user_id),
            ],
        )?;
        let name = field(&user, "user_name").to_string();
        self.audit
            .write_oper(actor, "User", 2, "user_reset_password", 0, "", &name)?;
        Ok(name)
    }

    pub fn role_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.require(actor, "system:role:list")?;
        self.store.query_all(
            "SELECT role_id, role_name, role_key, data_scope, status
            FROM sys_role
            WHERE del_flag = '0'
            ORDER BY role_sort, role_id",
            &[],
        )
    }

    pub fn role_grant(
        &self,
        actor: &HashMap<String, String>,
        role_id: i64,
        menu_ids: &[i64],
        data_scope: Option<&str>,
        dept_ids: &[i64],
    ) -> Result<HashMap<String, String>, AppError> {
        self.require(actor, "system:role:edit")?;
        self.role_exists(role_id)?;
        if role_id == Self::PROTECTED_ROLE {
            return Err(AppError::new(
                "The built-in administrator role is not edited here.",
                "grant menus on another role",
            ));
        }
        if let Some(scope) = data_scope {
            if !matches!(scope, "1" | "2" | "3" | "4") {
                return Err(AppError::new(
                    "data_scope must be 1, 2, 3, or 4.",
                    "1 all, 2 custom departments, 3 this department, 4 this department and children",
                ));
            }
        }
        for item in menu_ids {
            let row = self.store.query_one(
                "SELECT menu_id FROM sys_menu WHERE menu_id = ?",
                &[Arg::from(*item)],
            )?;
            if row.is_none() {
                return Err(AppError::new(
                    format!("Unknown menu {item}."),
                    "rust-rbac menu-list",
                ));
            }
        }
        self.store.execute(
            "DELETE FROM sys_role_menu WHERE role_id = ?",
            &[Arg::from(role_id)],
        )?;
        for menu_id in menu_ids {
            self.store.execute(
                "INSERT INTO sys_role_menu (role_id, menu_id) VALUES (?, ?)",
                &[Arg::from(role_id), Arg::from(*menu_id)],
            )?;
        }
        if let Some(scope) = data_scope {
            self.store.execute(
                "UPDATE sys_role
                SET data_scope = ?, update_by = ?, update_time = NOW()
                WHERE role_id = ?",
                &[
                    Arg::from(scope),
                    Arg::from(field(actor, "user_name")),
                    Arg::from(role_id),
                ],
            )?;
        }
        let scope_now = if let Some(scope) = data_scope {
            scope.to_string()
        } else {
            let current = self.store.query_one(
                "SELECT data_scope FROM sys_role WHERE role_id = ?",
                &[Arg::from(role_id)],
            )?;
            current
                .as_ref()
                .map(|row| field(row, "data_scope").to_string())
                .unwrap_or_else(|| "1".to_string())
        };
        self.store.execute(
            "DELETE FROM sys_role_dept WHERE role_id = ?",
            &[Arg::from(role_id)],
        )?;
        if scope_now == "2" {
            for dept_id in dept_ids {
                self.dept_exists(*dept_id)?;
                self.store.execute(
                    "INSERT INTO sys_role_dept (role_id, dept_id) VALUES (?, ?)",
                    &[Arg::from(role_id), Arg::from(*dept_id)],
                )?;
            }
        }
        self.audit
            .write_oper(actor, "Role", 2, "role_grant", 0, "", &role_id.to_string())?;
        self.store
            .query_one(
                "SELECT role_id, role_name, role_key, data_scope, status FROM sys_role WHERE role_id = ?",
                &[Arg::from(role_id)],
            )?
            .ok_or_else(|| AppError::new("Unknown role.", "rust-rbac role-list"))
    }

    pub fn menu_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.require(actor, "system:menu:list")?;
        self.menus_unscoped()
    }

    pub fn menus_unscoped(&self) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.store.query_all(
            "SELECT menu_id, parent_id, menu_name, menu_type, perms
            FROM sys_menu
            WHERE status = '0'
            ORDER BY parent_id, order_num, menu_id",
            &[],
        )
    }

    pub fn menu_add(
        &self,
        actor: &HashMap<String, String>,
        menu_name: &str,
        parent_id: i64,
        menu_type: &str,
        perms: &str,
        path: &str,
    ) -> Result<i64, AppError> {
        self.require(actor, "system:menu:add")?;
        let name = menu_name.trim();
        if name.is_empty() || name.chars().count() > 50 {
            return Err(AppError::new("menu_name is required.", "set --menu-name"));
        }
        if !matches!(menu_type, "M" | "C" | "F") {
            return Err(AppError::new(
                "menu_type must be M, C, or F.",
                "M directory, C page, F button",
            ));
        }
        if parent_id != 0 {
            let parent = self.store.query_one(
                "SELECT menu_id FROM sys_menu WHERE menu_id = ?",
                &[Arg::from(parent_id)],
            )?;
            if parent.is_none() {
                return Err(AppError::new(
                    format!("Unknown parent menu {parent_id}."),
                    "rust-rbac menu-list",
                ));
            }
        }
        let done = self.store.execute(
            "INSERT INTO sys_menu
              (menu_name, parent_id, order_num, path, menu_type, visible, status,
               perms, create_by, create_time)
            VALUES (?, ?, 0, ?, ?, '0', '0', ?, ?, NOW())",
            &[
                Arg::from(name),
                Arg::from(parent_id),
                Arg::from(path),
                Arg::from(menu_type),
                Arg::from(perms),
                Arg::from(field(actor, "user_name")),
            ],
        )?;
        self.audit
            .write_oper(actor, "Menu", 1, "menu_add", 0, "", name)?;
        Ok(done.lastrowid)
    }

    pub fn dept_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.require(actor, "system:dept:list")?;
        self.store.query_all(
            "SELECT dept_id, parent_id, dept_name, ancestors, status
            FROM sys_dept
            WHERE del_flag = '0'
            ORDER BY parent_id, order_num, dept_id",
            &[],
        )
    }

    pub fn dept_add(
        &self,
        actor: &HashMap<String, String>,
        dept_name: &str,
        parent_id: i64,
    ) -> Result<i64, AppError> {
        self.require(actor, "system:dept:add")?;
        let name = dept_name.trim();
        if name.is_empty() || name.chars().count() > 30 {
            return Err(AppError::new("dept_name is required.", "set --dept-name"));
        }
        let ancestors = if parent_id == 0 {
            "0".to_string()
        } else {
            let parent = self.store.query_one(
                "SELECT dept_id, ancestors FROM sys_dept WHERE dept_id = ? AND del_flag = '0'",
                &[Arg::from(parent_id)],
            )?;
            let Some(parent) = parent else {
                return Err(AppError::new(
                    format!("Unknown parent department {parent_id}."),
                    "rust-rbac dept-list",
                ));
            };
            format!(
                "{},{}",
                field(&parent, "ancestors"),
                field(&parent, "dept_id")
            )
        };
        let done = self.store.execute(
            "INSERT INTO sys_dept
              (parent_id, ancestors, dept_name, order_num, status, del_flag,
               create_by, create_time)
            VALUES (?, ?, ?, 0, '0', '0', ?, NOW())",
            &[
                Arg::from(parent_id),
                Arg::from(ancestors.as_str()),
                Arg::from(name),
                Arg::from(field(actor, "user_name")),
            ],
        )?;
        self.audit
            .write_oper(actor, "Department", 1, "dept_add", 0, "", name)?;
        Ok(done.lastrowid)
    }

    pub fn post_list(
        &self,
        actor: &HashMap<String, String>,
    ) -> Result<Vec<HashMap<String, String>>, AppError> {
        self.require(actor, "system:post:list")?;
        self.store.query_all(
            "SELECT post_id, post_code, post_name, post_sort, status
            FROM sys_post
            ORDER BY post_sort, post_id",
            &[],
        )
    }

    pub fn post_add(
        &self,
        actor: &HashMap<String, String>,
        post_code: &str,
        post_name: &str,
    ) -> Result<i64, AppError> {
        self.require(actor, "system:post:add")?;
        let code = post_code.trim();
        let name = post_name.trim();
        if code.is_empty() || name.is_empty() {
            return Err(AppError::new(
                "post_code and post_name are required.",
                "set both",
            ));
        }
        let existing = self.store.query_one(
            "SELECT post_id FROM sys_post WHERE post_code = ?",
            &[Arg::from(code)],
        )?;
        if existing.is_some() {
            return Err(AppError::new(
                "That post_code is already in use.",
                "pick another post_code",
            ));
        }
        let done = self.store.execute(
            "INSERT INTO sys_post
              (post_code, post_name, post_sort, status, create_by, create_time)
            VALUES (?, ?, 0, '0', ?, NOW())",
            &[
                Arg::from(code),
                Arg::from(name),
                Arg::from(field(actor, "user_name")),
            ],
        )?;
        self.audit
            .write_oper(actor, "Post", 1, "post_add", 0, "", code)?;
        Ok(done.lastrowid)
    }

    pub fn describe(&self, actor: &HashMap<String, String>) -> Result<String, AppError> {
        let roles = self.roles_of(field(actor, "user_id"))?;
        let names = if roles.is_empty() {
            "(none)".to_string()
        } else {
            roles
                .iter()
                .map(|role| field(role, "role_key"))
                .collect::<Vec<_>>()
                .join(", ")
        };
        let admin = if self.is_admin(field(actor, "user_id"))? {
            "yes"
        } else {
            "no"
        };
        Ok(format!(
            "user_name={}\nnick_name={}\ndept_id={}\nroles={}\nadmin={}",
            field(actor, "user_name"),
            field(actor, "nick_name"),
            field(actor, "dept_id"),
            names,
            admin
        ))
    }
}

fn token_urlsafe(nbytes: usize) -> String {
    let mut bytes = vec![0u8; nbytes];
    getrandom::getrandom(&mut bytes).expect("random");
    base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
}
