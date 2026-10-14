//! Plain text tables shared by the terminal and the text menu.
//! requirement-domain-rust-rbac — the same rows on both surfaces.

use std::collections::HashMap;

use crate::store::field;

pub struct Report;

impl Report {
    pub fn table(&self, columns: &[&str], rows: &[HashMap<String, String>]) -> String {
        let mut lines = vec![columns.join("|")];
        if rows.is_empty() {
            lines.push("(none)".to_string());
            return lines.join("\n");
        }
        for row in rows {
            let cells: Vec<String> = columns
                .iter()
                .map(|name| field(row, name).replace('|', "/").replace('\n', " "))
                .collect();
            lines.push(cells.join("|"));
        }
        lines.join("\n")
    }

    pub fn users(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &[
                "user_id",
                "user_name",
                "nick_name",
                "dept_id",
                "dept_name",
                "status",
            ],
            rows,
        )
    }

    pub fn roles(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &["role_id", "role_name", "role_key", "data_scope", "status"],
            rows,
        )
    }

    pub fn menus(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &["menu_id", "parent_id", "menu_name", "menu_type", "perms"],
            rows,
        )
    }

    pub fn depts(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &["dept_id", "parent_id", "dept_name", "ancestors", "status"],
            rows,
        )
    }

    pub fn posts(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &["post_id", "post_code", "post_name", "post_sort", "status"],
            rows,
        )
    }

    pub fn configs(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &["config_key", "config_name", "config_value", "config_type"],
            rows,
        )
    }

    pub fn dicts(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(&["dict_type", "dict_label", "dict_value", "status"], rows)
    }

    pub fn notices(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &["notice_id", "notice_title", "notice_type", "status"],
            rows,
        )
    }

    pub fn login_log(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &[
                "info_id",
                "user_name",
                "ipaddr",
                "status",
                "msg",
                "login_time",
            ],
            rows,
        )
    }

    pub fn oper_log(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &[
                "oper_id",
                "title",
                "business_type",
                "oper_name",
                "status",
                "oper_time",
            ],
            rows,
        )
    }

    pub fn online(&self, rows: &[HashMap<String, String>]) -> String {
        self.table(
            &["token", "user_name", "ipaddr", "surface", "login_time"],
            rows,
        )
    }
}
