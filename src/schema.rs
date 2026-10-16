//! Tables and the one-time seed.
//! requirement-database — the same columns on sqlite3 and on MySQL.
//! requirement-rbac — seed accounts, roles, menus, and permission strings.
//! requirement-rbac-menu — menu 1022 is edit and menu 1023 is remove.

use std::sync::Arc;

use crate::error::AppError;
use crate::password::PasswordHash;
use crate::schema_sql::MYSQL_STATEMENTS;
use crate::store::{sqlite_statement, Arg, Store, Tx};

pub struct Schema {
    store: Arc<Store>,
    passwords: PasswordHash,
}

impl Schema {
    pub const INITIAL_ADMIN_PASSWORD: &'static str = "admin123";
    pub const INITIAL_OPERATOR_PASSWORD: &'static str = "operator123";
    pub const INITIAL_LEAD_PASSWORD: &'static str = "lead123";
    pub const INITIAL_CLERK_PASSWORD: &'static str = "clerk123";

    pub fn new(store: Arc<Store>, passwords: PasswordHash) -> Self {
        Self { store, passwords }
    }

    pub fn statements(&self) -> Result<Vec<String>, AppError> {
        let engine = {
            let settings_arc = self.store.settings();
            let mut settings = settings_arc.lock().expect("settings");
            settings.engine()?
        };
        if engine == "sqlite3" {
            Ok(MYSQL_STATEMENTS
                .iter()
                .map(|sql| sqlite_statement(sql))
                .collect())
        } else {
            Ok(MYSQL_STATEMENTS
                .iter()
                .map(|sql| (*sql).to_string())
                .collect())
        }
    }

    pub fn ensure(&self) -> Result<String, AppError> {
        for statement in self.statements()? {
            self.store.execute(&statement, &[])?;
        }
        let row = self
            .store
            .query_one("SELECT COUNT(*) AS n FROM sys_user", &[])?;
        let count = row
            .as_ref()
            .and_then(|item| crate::store::field(item, "n").parse::<i64>().ok())
            .unwrap_or(0);
        if count > 0 {
            return Ok("present".to_string());
        }
        self.seed()?;
        Ok("seeded".to_string())
    }

    fn seed(&self) -> Result<(), AppError> {
        let admin = self.passwords.hash(Self::INITIAL_ADMIN_PASSWORD);
        let operator = self.passwords.hash(Self::INITIAL_OPERATOR_PASSWORD);
        let lead = self.passwords.hash(Self::INITIAL_LEAD_PASSWORD);
        let clerk = self.passwords.hash(Self::INITIAL_CLERK_PASSWORD);
        self.store
            .transaction(|tx| seed_work(tx, &admin, &operator, &clerk, &lead))
    }
}

fn seed_work(
    tx: &mut Tx,
    admin: &str,
    operator: &str,
    clerk: &str,
    lead: &str,
) -> Result<(), AppError> {
    let now_row = tx.run("SELECT NOW() AS t", &[])?;
    let now = now_row
        .rows
        .first()
        .map(|row| crate::store::field(row, "t").to_string())
        .unwrap_or_default();
    let depts = [
        (100_i64, 0_i64, "0", "Group", 0_i64),
        (101, 100, "0,100", "Engineering", 1),
        (102, 100, "0,100", "Operations", 2),
        (103, 101, "0,100,101", "Platform", 3),
    ];
    for item in depts {
        tx.run(
            "INSERT INTO sys_dept (dept_id, parent_id, ancestors, dept_name, order_num, status, del_flag, create_by, create_time) VALUES (?, ?, ?, ?, ?, '0', '0', 'schema', ?)",
            &[
                Arg::from(item.0),
                Arg::from(item.1),
                Arg::from(item.2),
                Arg::from(item.3),
                Arg::from(item.4),
                Arg::from(now.as_str()),
            ],
        )?;
    }
    let users = [
        (1_i64, 101_i64, "admin", "Administrator", admin),
        (2, 102, "operator", "Operator", operator),
        (3, 103, "clerk", "Clerk", clerk),
        (4, 101, "lead", "Lead", lead),
    ];
    for item in users {
        tx.run(
            "INSERT INTO sys_user (user_id, dept_id, user_name, nick_name, user_type, password, status, del_flag, create_by, create_time, pwd_update_date) VALUES (?, ?, ?, ?, '00', ?, '0', '0', 'schema', ?, ?)",
            &[
                Arg::from(item.0),
                Arg::from(item.1),
                Arg::from(item.2),
                Arg::from(item.3),
                Arg::from(item.4),
                Arg::from(now.as_str()),
                Arg::from(now.as_str()),
            ],
        )?;
    }
    let posts = [
        (1_i64, "ceo", "Chief", 1_i64),
        (2, "se", "Engineer", 2),
        (3, "hr", "People", 3),
        (4, "user", "Staff", 4),
    ];
    for item in posts {
        tx.run(
            "INSERT INTO sys_post (post_id, post_code, post_name, post_sort, status, create_by, create_time) VALUES (?, ?, ?, ?, '0', 'schema', ?)",
            &[
                Arg::from(item.0),
                Arg::from(item.1),
                Arg::from(item.2),
                Arg::from(item.3),
                Arg::from(now.as_str()),
            ],
        )?;
    }
    let roles = [
        (
            1_i64,
            "Super administrator",
            "admin",
            1_i64,
            "1",
            "super administrator",
        ),
        (2, "Common", "common", 2, "3", "this department"),
        (3, "Lead", "lead", 3, "4", "this department and children"),
    ];
    for item in roles {
        tx.run(
            "INSERT INTO sys_role (role_id, role_name, role_key, role_sort, data_scope, menu_check_strictly, dept_check_strictly, status, del_flag, create_by, create_time, remark) VALUES (?, ?, ?, ?, ?, 1, 1, '0', '0', 'schema', ?, ?)",
            &[
                Arg::from(item.0),
                Arg::from(item.1),
                Arg::from(item.2),
                Arg::from(item.3),
                Arg::from(item.4),
                Arg::from(now.as_str()),
                Arg::from(item.5),
            ],
        )?;
    }
    for menu in MENUS {
        tx.run(
            "INSERT INTO sys_menu (menu_id, menu_name, parent_id, order_num, path, menu_type, visible, status, perms, create_by, create_time) VALUES (?, ?, ?, ?, ?, ?, '0', '0', ?, 'schema', ?)",
            &[
                Arg::from(menu.0),
                Arg::from(menu.1),
                Arg::from(menu.2),
                Arg::from(menu.3),
                Arg::from(menu.4),
                Arg::from(menu.5),
                Arg::from(menu.6),
                Arg::from(now.as_str()),
            ],
        )?;
    }
    for (user_id, role_id) in [(1_i64, 1_i64), (2, 2), (3, 2), (4, 3)] {
        tx.run(
            "INSERT INTO sys_user_role (user_id, role_id) VALUES (?, ?)",
            &[Arg::from(user_id), Arg::from(role_id)],
        )?;
    }
    for menu in MENUS {
        tx.run(
            "INSERT INTO sys_role_menu (role_id, menu_id) VALUES (1, ?)",
            &[Arg::from(menu.0)],
        )?;
    }
    for menu_id in COMMON_MENUS {
        tx.run(
            "INSERT INTO sys_role_menu (role_id, menu_id) VALUES (2, ?)",
            &[Arg::from(*menu_id)],
        )?;
    }
    for menu_id in LEAD_MENUS {
        tx.run(
            "INSERT INTO sys_role_menu (role_id, menu_id) VALUES (3, ?)",
            &[Arg::from(*menu_id)],
        )?;
    }
    let configs = [
        (1_i64, "Skin name", "sys.index.skinName", "skin-blue", "Y"),
        (
            2,
            "Initial password",
            "sys.user.initPassword",
            "123456",
            "Y",
        ),
        (3, "Captcha", "sys.account.captchaEnabled", "false", "Y"),
        (4, "Registration", "sys.account.registerUser", "false", "Y"),
    ];
    for item in configs {
        tx.run(
            "INSERT INTO sys_config (config_id, config_name, config_key, config_value, config_type, create_by, create_time) VALUES (?, ?, ?, ?, ?, 'schema', ?)",
            &[
                Arg::from(item.0),
                Arg::from(item.1),
                Arg::from(item.2),
                Arg::from(item.3),
                Arg::from(item.4),
                Arg::from(now.as_str()),
            ],
        )?;
    }
    let types = [
        (1_i64, "User sex", "sys_user_sex"),
        (2, "Normal or disabled", "sys_normal_disable"),
        (3, "Notice type", "sys_notice_type"),
    ];
    for item in types {
        tx.run(
            "INSERT INTO sys_dict_type (dict_id, dict_name, dict_type, status, create_by, create_time) VALUES (?, ?, ?, '0', 'schema', ?)",
            &[
                Arg::from(item.0),
                Arg::from(item.1),
                Arg::from(item.2),
                Arg::from(now.as_str()),
            ],
        )?;
    }
    let data_rows = [
        (1_i64, 1_i64, "Male", "0", "sys_user_sex", "Y"),
        (2, 2, "Female", "1", "sys_user_sex", "N"),
        (3, 3, "Unknown", "2", "sys_user_sex", "N"),
        (4, 1, "Normal", "0", "sys_normal_disable", "Y"),
        (5, 2, "Disabled", "1", "sys_normal_disable", "N"),
        (6, 1, "Notice", "1", "sys_notice_type", "Y"),
        (7, 2, "Announcement", "2", "sys_notice_type", "N"),
    ];
    for item in data_rows {
        tx.run(
            "INSERT INTO sys_dict_data (dict_code, dict_sort, dict_label, dict_value, dict_type, is_default, status, create_by, create_time) VALUES (?, ?, ?, ?, ?, ?, '0', 'schema', ?)",
            &[
                Arg::from(item.0),
                Arg::from(item.1),
                Arg::from(item.2),
                Arg::from(item.3),
                Arg::from(item.4),
                Arg::from(item.5),
                Arg::from(now.as_str()),
            ],
        )?;
    }
    tx.run(
        "INSERT INTO sys_notice (notice_id, notice_title, notice_type, notice_content, status, create_by, create_time) VALUES (1, 'Welcome', '2', 'RustRbac is ready.', '0', 'schema', ?)",
        &[Arg::from(now.as_str())],
    )?;
    Ok(())
}

/// menu_id, name, parent, order, path, type, perms
const MENUS: &[(i64, &str, i64, i64, &str, &str, &str)] = &[
    (1, "System", 0, 1, "system", "M", ""),
    (100, "User", 1, 1, "user", "C", "system:user:list"),
    (1001, "User query", 100, 1, "", "F", "system:user:query"),
    (1002, "User add", 100, 2, "", "F", "system:user:add"),
    (1003, "User edit", 100, 3, "", "F", "system:user:edit"),
    (1004, "User remove", 100, 4, "", "F", "system:user:remove"),
    (
        1005,
        "Reset password",
        100,
        5,
        "",
        "F",
        "system:user:resetPwd",
    ),
    (101, "Role", 1, 2, "role", "C", "system:role:list"),
    (1011, "Role query", 101, 1, "", "F", "system:role:query"),
    (1012, "Role edit", 101, 2, "", "F", "system:role:edit"),
    (102, "Menu", 1, 3, "menu", "C", "system:menu:list"),
    (1021, "Menu add", 102, 1, "", "F", "system:menu:add"),
    (1022, "Menu edit", 102, 2, "", "F", "system:menu:edit"),
    (1023, "Menu remove", 102, 3, "", "F", "system:menu:remove"),
    (103, "Department", 1, 4, "dept", "C", "system:dept:list"),
    (1031, "Department add", 103, 1, "", "F", "system:dept:add"),
    (104, "Post", 1, 5, "post", "C", "system:post:list"),
    (1041, "Post add", 104, 1, "", "F", "system:post:add"),
    (2, "Setup", 0, 2, "setup", "M", ""),
    (110, "Parameter", 2, 1, "config", "C", "system:config:list"),
    (
        1101,
        "Parameter edit",
        110,
        1,
        "",
        "F",
        "system:config:edit",
    ),
    (1102, "Parameter add", 110, 2, "", "F", "system:config:add"),
    (111, "Dictionary", 2, 2, "dict", "C", "system:dict:list"),
    (1111, "Dictionary add", 111, 1, "", "F", "system:dict:add"),
    (112, "Notice", 2, 3, "notice", "C", "system:notice:list"),
    (1121, "Notice add", 112, 1, "", "F", "system:notice:add"),
    (3, "Audit", 0, 3, "audit", "M", ""),
    (
        120,
        "Operation log",
        3,
        1,
        "operlog",
        "C",
        "monitor:operlog:list",
    ),
    (
        121,
        "Login log",
        3,
        2,
        "logininfor",
        "C",
        "monitor:logininfor:list",
    ),
    (122, "Online", 3, 3, "online", "C", "monitor:online:list"),
    (
        1221,
        "Force logout",
        122,
        1,
        "",
        "F",
        "monitor:online:forceLogout",
    ),
];

const COMMON_MENUS: &[i64] = &[1, 100, 1001, 2, 110, 112, 3, 120, 121];
const LEAD_MENUS: &[i64] = &[1, 100, 1001];
