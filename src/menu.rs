//! Text menu rows, path line, language leaf, and the full-screen board.
//! requirement-rust-tui — path line, clock, rounded input box, status line.
//! requirement-rust-cli-language — row 7 and the language leaf.

use std::collections::VecDeque;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::Path;
use std::time::Duration;

use crossterm::cursor::{Hide, MoveTo, Show};
use crossterm::event::{self, Event, KeyCode, KeyModifiers};
use crossterm::style::{Attribute, Print, SetAttribute};
use crossterm::terminal::{
    self, disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
};
use crossterm::ExecutableCommand;
use unicode_width::UnicodeWidthChar;

use crate::texts::text;

pub const KEY_DOWN: i32 = 258;
pub const KEY_UP: i32 = 259;
pub const KEY_LEFT: i32 = 260;
pub const KEY_RIGHT: i32 = 261;
pub const KEY_HOME: i32 = 262;
pub const KEY_BACKSPACE: i32 = 263;
pub const KEY_END: i32 = 360;

// =============================================================================
// CIAO-Lite Protection Zone
// Do NOT simplify, refactor, or remove without explicit user instruction.
// The frame glyphs are the bottom input box. requirement-rust-tui.md
// =============================================================================
pub const INPUT_MARK: &str = "> ";
pub const INPUT_HINT: &str = "Up/Down  •  Enter";
pub const FRAME_TOP_LEFT: &str = "╭";
pub const FRAME_TOP_RIGHT: &str = "╮";
pub const FRAME_BOTTOM_LEFT: &str = "╰";
pub const FRAME_BOTTOM_RIGHT: &str = "╯";
pub const FRAME_HORIZ: &str = "─";
pub const FRAME_VERT: &str = "│";
pub const CARET_BLOCK: &str = "█";
pub const FRAME_ROWS: u16 = 3;
pub const FOOTER_ROWS: u16 = 1;
pub const CHROME_ROWS: u16 = FRAME_ROWS + FOOTER_ROWS;
pub const MIN_MENU_ABOVE: u16 = 3;
pub const MIN_HEIGHT: u16 = MIN_MENU_ABOVE + CHROME_ROWS;
pub const MIN_PLACEABLE: u16 = 6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Up,
    Down,
    Left,
    Right,
    Home,
    End,
    Enter,
    Backspace,
    Esc,
    Eof,
    Interrupt,
    Char(char),
    Other,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Style {
    pub bold: bool,
    pub italic: bool,
    pub reverse: bool,
    pub underline: bool,
}

pub trait Screen {
    fn size(&self) -> (u16, u16);
    fn erase(&mut self);
    fn put(&mut self, y: u16, x: u16, text: &str, style: Style);
    fn refresh(&mut self);
    fn move_to(&mut self, y: u16, x: u16);
    fn read_key(&mut self) -> Key;
    fn poll_key(&mut self, timeout_ms: u64) -> Option<Key>;
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MenuStop {
    Leave,
    Interrupt,
    TooSmall,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FieldAnswer {
    Text(String),
    Esc,
    Eof,
    Interrupt,
    TooSmall,
}

const CODES: &[(&str, &str, i32)] = &[
    ("en", "English", 71),
    ("zh-Hans", "简体中文", 72),
    ("zh-Hant", "繁體中文", 73),
    ("es", "Español", 74),
    ("ar", "العربية", 75),
    ("fr", "Français", 76),
    ("pt", "Português", 77),
    ("ru", "Русский", 78),
    ("de", "Deutsch", 79),
    ("ja", "日本語", 80),
    ("ko", "한국어", 81),
    ("nl", "Nederlands", 82),
    ("el", "Ελληνικά", 83),
];

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MenuRow {
    pub number: i32,
    pub short: String,
    pub explain: String,
    pub kind: String,
}

impl MenuRow {
    fn new(number: i32, short: &str, explain: &str, kind: &str) -> Self {
        Self {
            number,
            short: short.to_string(),
            explain: explain.to_string(),
            kind: kind.to_string(),
        }
    }
}

pub struct MenuLanguage {
    home: String,
    code: String,
}

impl MenuLanguage {
    pub fn with_env(home: &str, lang: Option<&str>) -> Self {
        let mut language = Self {
            home: home.to_string(),
            code: "en".to_string(),
        };
        if let Some(lang) = lang {
            if CODES.iter().any(|(code, _, _)| *code == lang) {
                language.code = lang.to_string();
                return language;
            }
        }
        language.load();
        language
    }

    pub fn code(&self) -> &str {
        &self.code
    }

    pub fn path_label(&self) -> String {
        self.text_key("path")
    }

    pub fn leaf_path(&self) -> String {
        if self.home.is_empty() {
            return String::new();
        }
        Path::new(&self.home)
            .join(".local")
            .join("RustRbac")
            .join("language")
            .to_string_lossy()
            .into_owned()
    }

    pub fn load(&mut self) {
        self.code = self.read_leaf();
    }

    fn read_leaf(&self) -> String {
        let path = self.leaf_path();
        if path.is_empty() || !Path::new(&path).is_file() {
            return "en".to_string();
        }
        let Ok(body) = fs::read_to_string(&path) else {
            return "en".to_string();
        };
        let line = body
            .lines()
            .next()
            .unwrap_or("")
            .replace('\r', "")
            .trim()
            .to_string();
        if CODES.iter().any(|(code, _, _)| *code == line) {
            line
        } else {
            "en".to_string()
        }
    }

    pub fn save(&mut self, code: &str) -> String {
        if !CODES.iter().any(|(item, _, _)| *item == code) {
            return self.text_key("save_failed");
        }
        let path = self.leaf_path();
        if path.is_empty() {
            return self.text_key("save_failed");
        }
        let folder = Path::new(&path).parent().map(|item| item.to_path_buf());
        let wrote = (|| {
            let folder = folder.ok_or(())?;
            fs::create_dir_all(&folder).map_err(|_| ())?;
            let _ = fs::set_permissions(&folder, fs::Permissions::from_mode(0o700));
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .truncate(true)
                .mode(0o600)
                .open(&path)
                .map_err(|_| ())?;
            file.write_all(format!("{code}\n").as_bytes())
                .map_err(|_| ())?;
            let _ = fs::set_permissions(&path, fs::Permissions::from_mode(0o600));
            Ok::<(), ()>(())
        })();
        if wrote.is_err() {
            return self.text_key("save_failed");
        }
        self.code = code.to_string();
        self.text_key("saved")
    }

    pub fn text_key(&self, key: &str) -> String {
        text(&self.code, key).to_string()
    }

    pub fn row_short(&self, kind: &str, english: &str) -> String {
        match short_key(kind) {
            Some(key) => self.text_key(key),
            None => english.to_string(),
        }
    }

    pub fn row_long(&self, kind: &str, english: &str) -> String {
        match long_key(kind) {
            Some(key) => self.text_key(key),
            None => english.to_string(),
        }
    }

    pub fn lang_long(&self, endonym: &str) -> String {
        self.text_key("lang_long").replace("{name}", endonym)
    }
}

fn short_key(kind: &str) -> Option<&'static str> {
    match kind {
        "language" => Some("language_short"),
        "system-log" => Some("system_log_short"),
        "self-management" => Some("self_short"),
        "exit" => Some("exit_short"),
        "back" | "back-server" => Some("back_short"),
        _ => None,
    }
}

fn long_key(kind: &str) -> Option<&'static str> {
    match kind {
        "language" => Some("language_long"),
        "system-log" => Some("system_log_long"),
        "self-management" => Some("self_long"),
        "exit" => Some("exit_long"),
        "back" => Some("back_long"),
        "back-server" => Some("back_server_long"),
        "view-log" => Some("view_long"),
        "clear-log" => Some("clear_long"),
        "log-folder" => Some("folder_long"),
        "version" => Some("version_long"),
        "about" => Some("about_long"),
        "version-check" => Some("version_check_long"),
        "self-update" => Some("self_update_long"),
        "self-uninstall" => Some("self_uninstall_long"),
        "self-install" => Some("self_install_long"),
        _ => None,
    }
}

fn board_alias(token: &str) -> Option<&'static str> {
    match token {
        "rbac" | "access" => Some("rbac"),
        "server" => Some("server"),
        "setup" => Some("setup"),
        "audit" => Some("audit"),
        "serve" => Some("serve"),
        "system-log" => Some("system-log"),
        "self-management" => Some("self-management"),
        "language" => Some("language"),
        _ => None,
    }
}

pub struct MenuPainter {
    pub language: MenuLanguage,
    clock: Option<String>,
}

impl MenuPainter {
    pub const PATH_LABEL_EN: &'static str = "Path";
    pub const PATH_LABEL_ZH_HANT: &'static str = "路徑";

    pub fn new(home: &str, lang: Option<&str>) -> Self {
        Self {
            language: MenuLanguage::with_env(home, lang),
            clock: None,
        }
    }

    pub fn set_clock(&mut self, clock: &str) {
        self.clock = Some(clock.to_string());
    }

    pub fn clock_text(&self) -> String {
        if let Some(clock) = &self.clock {
            return clock.clone();
        }
        local_clock()
    }

    pub fn path_label(&self) -> String {
        self.language.path_label()
    }

    pub fn display_width(text: &str) -> usize {
        text.chars().map(|ch| ch.width().unwrap_or(0)).sum()
    }

    pub fn clip_columns(text: &str, columns: usize) -> String {
        if columns == 0 {
            return String::new();
        }
        let mut kept = String::new();
        let mut used = 0;
        for ch in text.chars() {
            let width = ch.width().unwrap_or(0);
            if used + width > columns {
                break;
            }
            kept.push(ch);
            used += width;
        }
        kept
    }

    pub fn clip_columns_right(text: &str, columns: usize) -> String {
        if columns == 0 {
            return String::new();
        }
        let mut kept: Vec<char> = Vec::new();
        let mut used = 0;
        for ch in text.chars().rev() {
            let width = ch.width().unwrap_or(0);
            if used + width > columns {
                break;
            }
            kept.push(ch);
            used += width;
        }
        kept.reverse();
        kept.into_iter().collect()
    }

    pub fn path_line(&self, placeable: Option<usize>) -> String {
        let prefix = format!("{}: ", self.path_label());
        let current = std::env::current_dir()
            .map(|path| path.display().to_string())
            .unwrap_or_else(|_| ".".to_string());
        let left = format!("{prefix}{current}");
        let right = self.clock_text();
        let gap = 2;
        let left_width = Self::display_width(&left);
        let right_width = Self::display_width(&right);
        let prefix_width = Self::display_width(&prefix);
        let Some(placeable) = placeable else {
            return format!("{left}{}{right}", " ".repeat(gap));
        };
        if placeable >= left_width + gap + right_width {
            let spaces = placeable - left_width - right_width;
            return format!("{left}{}{right}", " ".repeat(spaces));
        }
        if placeable >= left_width {
            return left;
        }
        if placeable <= prefix_width {
            return Self::clip_columns(&prefix, placeable);
        }
        format!(
            "{prefix}{}",
            Self::clip_columns_right(&current, placeable - prefix_width)
        )
    }

    pub fn rows_for(&self, layer: &str) -> Vec<MenuRow> {
        match layer {
            "self" => vec![
                MenuRow::new(62, "version", "show the installed version", "version"),
                MenuRow::new(63, "about", "version and this computer", "about"),
                MenuRow::new(
                    64,
                    "version-check",
                    "compare this install with cargo",
                    "version-check",
                ),
                MenuRow::new(
                    65,
                    "self-update",
                    "upgrade this package with cargo",
                    "self-update",
                ),
                MenuRow::new(
                    66,
                    "self-uninstall",
                    "remove this package with cargo",
                    "self-uninstall",
                ),
                MenuRow::new(
                    67,
                    "self-install",
                    "install this package with cargo",
                    "self-install",
                ),
                MenuRow::new(0, "Back", "return to the main menu", "back"),
            ],
            "log" => vec![
                MenuRow::new(31, "view-log", "list a log file and show it", "view-log"),
                MenuRow::new(32, "clear-log", "empty one log file", "clear-log"),
                MenuRow::new(33, "log-folder", "show the log folder", "log-folder"),
                MenuRow::new(0, "Back", "return to the main menu", "back"),
            ],
            "rbac" => vec![
                MenuRow::new(
                    11,
                    "user-list",
                    "accounts you are allowed to see",
                    "user-list",
                ),
                MenuRow::new(12, "user-add", "add an account", "user-add"),
                MenuRow::new(
                    13,
                    "user-edit",
                    "change a name, department, or status",
                    "user-edit",
                ),
                MenuRow::new(14, "user-remove", "mark an account deleted", "user-remove"),
                MenuRow::new(
                    15,
                    "user-reset-password",
                    "set a new password",
                    "user-reset-password",
                ),
                MenuRow::new(16, "role-list", "roles and data scope", "role-list"),
                MenuRow::new(
                    17,
                    "role-grant",
                    "menus and data scope for one role",
                    "role-grant",
                ),
                MenuRow::new(18, "menu-list", "menu and permission strings", "menu-list"),
                MenuRow::new(
                    19,
                    "menu-add",
                    "add a directory, page, or button",
                    "menu-add",
                ),
                MenuRow::new(110, "dept-list", "departments", "dept-list"),
                MenuRow::new(111, "dept-add", "add a department", "dept-add"),
                MenuRow::new(112, "post-list", "posts", "post-list"),
                MenuRow::new(113, "post-add", "add a post", "post-add"),
                MenuRow::new(0, "Back", "return to the main menu", "back"),
            ],
            "server" => vec![
                MenuRow::new(
                    21,
                    "setup",
                    "parameters, dictionaries, and notices",
                    "setup",
                ),
                MenuRow::new(
                    22,
                    "audit",
                    "operation log, login log, and online users",
                    "audit",
                ),
                MenuRow::new(
                    23,
                    "serve",
                    "start the web interface on this computer",
                    "serve",
                ),
                MenuRow::new(0, "Back", "return to the main menu", "back"),
            ],
            "setup" => vec![
                MenuRow::new(211, "config-list", "parameters", "config-list"),
                MenuRow::new(212, "config-set", "change one parameter", "config-set"),
                MenuRow::new(213, "config-add", "add a parameter", "config-add"),
                MenuRow::new(214, "dict-list", "dictionary values", "dict-list"),
                MenuRow::new(215, "dict-add", "add a dictionary value", "dict-add"),
                MenuRow::new(216, "notice-list", "notices", "notice-list"),
                MenuRow::new(217, "notice-add", "add a notice", "notice-add"),
                MenuRow::new(218, "db-engine", "sqlite3 file or mysql", "db-engine"),
                MenuRow::new(0, "Back", "return to the server menu", "back-server"),
            ],
            "audit" => vec![
                MenuRow::new(221, "oper-log", "operation log", "oper-log"),
                MenuRow::new(222, "login-log", "login log", "login-log"),
                MenuRow::new(
                    223,
                    "online-list",
                    "sessions that are signed in",
                    "online-list",
                ),
                MenuRow::new(
                    224,
                    "online-logout",
                    "close one online session",
                    "online-logout",
                ),
                MenuRow::new(0, "Back", "return to the server menu", "back-server"),
            ],
            _ => vec![
                MenuRow::new(
                    1,
                    "rbac",
                    "users, roles, menus, departments, and posts",
                    "rbac",
                ),
                MenuRow::new(2, "server", "setup, audit, and the web listener", "server"),
                MenuRow::new(
                    3,
                    "system-log",
                    "view, clear, and the log folder",
                    "system-log",
                ),
                MenuRow::new(
                    6,
                    "self-management",
                    "version, about, and cargo lifecycle",
                    "self-management",
                ),
                MenuRow::new(7, "language", "display language for this menu", "language"),
                MenuRow::new(9, "Exit", "leave", "exit"),
            ],
        }
    }

    pub fn display_rows(&self, layer: &str) -> Vec<MenuRow> {
        if layer == "lang" {
            let mut rows = Vec::new();
            for (code, endonym, number) in CODES {
                rows.push(MenuRow::new(
                    *number,
                    endonym,
                    &self.language.lang_long(endonym),
                    &format!("set-{code}"),
                ));
            }
            rows.push(MenuRow::new(
                0,
                &self.language.row_short("back", "Back"),
                &self.language.row_long("back", "return to the main menu"),
                "back",
            ));
            return rows;
        }
        self.rows_for(layer)
            .into_iter()
            .map(|row| MenuRow {
                short: self.language.row_short(&row.kind, &row.short),
                explain: self.language.row_long(&row.kind, &row.explain),
                ..row
            })
            .collect()
    }

    pub fn row_parts(&self, rows: &[MenuRow]) -> Vec<(String, String, String, String)> {
        let mut number_width = 1;
        let mut verb_width = 0;
        for row in rows {
            let text = row.number.to_string();
            number_width = number_width.max(text.len());
            verb_width = verb_width.max(Self::display_width(&row.short));
        }
        rows.iter()
            .map(|row| {
                let text = row.number.to_string();
                let pad = number_width - text.len();
                let number_field = format!("{}{text}. ", " ".repeat(pad));
                let verb_pad = " ".repeat(verb_width - Self::display_width(&row.short));
                (
                    number_field,
                    row.short.clone(),
                    verb_pad,
                    row.explain.clone(),
                )
            })
            .collect()
    }

    pub fn format_rows(&self, rows: &[MenuRow]) -> Vec<String> {
        self.row_parts(rows)
            .into_iter()
            .map(|(number_field, short, verb_pad, explain)| {
                format!("{number_field}{short}{verb_pad}: {explain}")
            })
            .collect()
    }

    pub fn board_title(&self, phase_result: bool, layer: &str) -> String {
        if phase_result {
            return "result".to_string();
        }
        let key = match layer {
            "self" => "title_self",
            "log" => "title_log",
            "lang" => "title_language",
            "rbac" | "access" => "title_rbac",
            "server" => "title_server",
            "setup" => "title_setup",
            "audit" => "title_audit",
            _ => "title_main",
        };
        self.language.text_key(key)
    }

    pub fn screen_can_hold(height: u16, width: u16) -> bool {
        height >= MIN_HEIGHT && width >= MIN_PLACEABLE
    }

    pub fn result_room(height: u16, line_count: usize) -> usize {
        if line_count > height.saturating_sub(3) as usize {
            (height.saturating_sub(4) as usize).max(1)
        } else {
            (height.saturating_sub(3) as usize).max(1)
        }
    }
}

pub struct MenuModel {
    pub painter: MenuPainter,
    pub layer: String,
    pub index: usize,
    pub buffer: String,
    pub cursor: usize,
    pub focus: String,
    pub error: String,
    pub notice: String,
    pub phase: String,
    pub result_text: String,
    pub result_offset: usize,
    pub mask: bool,
}

impl MenuModel {
    pub fn new(painter: MenuPainter) -> Self {
        Self {
            painter,
            layer: "front".to_string(),
            index: 0,
            buffer: String::new(),
            cursor: 0,
            focus: "list".to_string(),
            error: String::new(),
            notice: String::new(),
            phase: "board".to_string(),
            result_text: String::new(),
            result_offset: 0,
            mask: false,
        }
    }

    pub fn rows(&self) -> Vec<MenuRow> {
        self.painter.display_rows(&self.layer)
    }

    pub fn show_front(&mut self) {
        self.phase = "board".to_string();
        self.layer = "front".to_string();
        self.index = 0;
        self.buffer.clear();
        self.cursor = 0;
        self.focus = "list".to_string();
        self.error.clear();
        self.result_offset = 0;
    }

    pub fn show_result(&mut self, text: &str) {
        self.result_text = text.to_string();
        self.result_offset = 0;
        self.phase = "result".to_string();
        self.layer = "front".to_string();
        self.index = 0;
        self.buffer.clear();
        self.cursor = 0;
        self.focus = "list".to_string();
        self.error.clear();
    }

    pub fn apply_key(&mut self, key: i32, screen_height: i32) -> Option<String> {
        self.notice.clear();
        if key == 27 || (key == -1 && (self.phase == "result" || self.layer != "front")) {
            self.show_front();
            return None;
        }
        if key == -1 {
            return Some("exit".to_string());
        }
        if self.phase == "result" {
            let count = self.result_text.lines().count().max(1);
            let height = screen_height.max(0) as u16;
            let overflow = count > height.saturating_sub(3) as usize;
            if overflow && (key == KEY_UP || key == b'k' as i32) {
                self.scroll_result(-1, height);
                return None;
            }
            if overflow && (key == KEY_DOWN || key == b'j' as i32) {
                self.scroll_result(1, height);
                return None;
            }
            self.show_front();
            return None;
        }
        if key == KEY_UP
            || key == KEY_DOWN
            || (self.focus == "list" && (key == b'k' as i32 || key == b'j' as i32))
        {
            self.arrow(key);
            return None;
        }
        if key == KEY_LEFT || key == KEY_RIGHT || key == KEY_HOME || key == KEY_END {
            self.slide(key);
            return None;
        }
        if key == KEY_BACKSPACE || key == 127 || key == 8 {
            self.backspace();
            return None;
        }
        if key == 10 || key == 13 || key == 343 {
            return self.commit();
        }
        if (32..127).contains(&key) {
            self.insert(char::from_u32(key as u32).unwrap_or(' '));
            return None;
        }
        self.error = self.painter.language.text_key("invalid");
        None
    }

    fn arrow(&mut self, key: i32) {
        let last = self.rows().len().saturating_sub(1);
        let down = key == KEY_DOWN || key == b'j' as i32;
        if self.focus == "input" {
            self.focus = "list".to_string();
            self.index = if down { 0 } else { last };
            return;
        }
        if down {
            if self.index < last {
                self.index += 1;
            } else {
                self.focus = "input".to_string();
                self.cursor = self.buffer.chars().count();
            }
            return;
        }
        if self.index > 0 {
            self.index -= 1;
            return;
        }
        self.focus = "input".to_string();
        self.cursor = self.buffer.chars().count();
    }

    fn slide(&mut self, key: i32) {
        if self.focus != "input" {
            return;
        }
        let len = self.buffer.chars().count();
        if key == KEY_LEFT {
            self.cursor = self.cursor.saturating_sub(1);
        } else if key == KEY_RIGHT {
            self.cursor = (self.cursor + 1).min(len);
        } else if key == KEY_HOME {
            self.cursor = 0;
        } else if key == KEY_END {
            self.cursor = len;
        }
    }

    fn insert(&mut self, ch: char) {
        if self.focus != "input" {
            self.focus = "input".to_string();
            self.cursor = self.buffer.chars().count();
        }
        let cursor = self.cursor.min(self.buffer.chars().count());
        let byte = self
            .buffer
            .char_indices()
            .nth(cursor)
            .map(|(index, _)| index)
            .unwrap_or(self.buffer.len());
        self.buffer.insert(byte, ch);
        self.cursor = cursor + 1;
    }

    fn backspace(&mut self) {
        if self.focus != "input" {
            self.focus = "input".to_string();
            self.cursor = self.buffer.chars().count();
        }
        if self.cursor == 0 || self.buffer.is_empty() {
            self.cursor = 0;
            return;
        }
        let cursor = self.cursor.min(self.buffer.chars().count());
        let start = self
            .buffer
            .char_indices()
            .nth(cursor - 1)
            .map(|(index, _)| index)
            .unwrap_or(0);
        let end = self
            .buffer
            .char_indices()
            .nth(cursor)
            .map(|(index, _)| index)
            .unwrap_or(self.buffer.len());
        self.buffer.replace_range(start..end, "");
        self.cursor = cursor - 1;
    }

    fn commit(&mut self) -> Option<String> {
        let token = self.buffer.trim().to_string();
        self.buffer.clear();
        self.cursor = 0;
        if token.is_empty() {
            let rows = self.rows();
            let row = rows.get(self.index)?;
            return self.activate(row.number, row.kind.clone());
        }
        if token.chars().all(|ch| ch.is_ascii_digit()) {
            let number: i32 = token.parse().unwrap_or(-1);
            return self.activate_number(number);
        }
        let rows = self.rows();
        if let Some(row) = rows.iter().find(|row| row.short == token) {
            return self.activate(row.number, row.kind.clone());
        }
        if let Some(kind) = board_alias(&token) {
            return self.activate(0, kind.to_string());
        }
        if token == "help" {
            self.error.clear();
            self.focus = "list".to_string();
            return Some("help".to_string());
        }
        self.error = self.painter.language.text_key("invalid");
        self.focus = "input".to_string();
        None
    }

    fn scroll_result(&mut self, delta: i32, screen_height: u16) {
        let count = self.result_text.lines().count().max(1);
        let room = MenuPainter::result_room(screen_height, count);
        let max_offset = count.saturating_sub(room);
        let next = self.result_offset as i32 + delta;
        self.result_offset = next.clamp(0, max_offset as i32) as usize;
    }

    pub fn banner(&self) -> String {
        if !self.notice.is_empty() {
            self.notice.clone()
        } else {
            self.error.clone()
        }
    }

    pub fn paint_board(&mut self, screen: &mut dyn Screen, app_name: &str, version: &str) {
        let (height, width) = screen.size();
        let phase_result = self.phase == "result";
        let layer = self.layer.clone();
        let title = self.painter.board_title(phase_result, &layer);
        let path = self.painter.path_line(Some(width as usize));
        let rows = self.painter.display_rows(&layer);
        let parts = self.painter.row_parts(&rows);
        self.paint_collected(
            screen, app_name, version, &title, &path, &parts, height, width,
        );
    }

    pub fn paint_prompt(
        &mut self,
        screen: &mut dyn Screen,
        title: &str,
        lines: &[String],
        app_name: &str,
        version: &str,
    ) {
        let (height, width) = screen.size();
        screen.erase();
        let italic = Style {
            italic: true,
            ..Style::default()
        };
        put_at(
            screen,
            0,
            0,
            app_name,
            Style {
                bold: true,
                ..Style::default()
            },
            width,
        );
        let mut cursor = MenuPainter::display_width(app_name) as u16;
        put_at(screen, 0, cursor, " (", Style::default(), width);
        cursor += 2;
        put_at(screen, 0, cursor, version, italic, width);
        cursor += MenuPainter::display_width(version) as u16;
        put_at(
            screen,
            0,
            cursor,
            &format!(") — {title}"),
            Style::default(),
            width,
        );
        let box_top = height.saturating_sub(CHROME_ROWS);
        let banner = self.banner();
        let mut stop = box_top;
        if !banner.is_empty() && box_top >= 4 {
            stop = box_top - 1;
        }
        let room = stop.saturating_sub(2) as usize;
        let shown = visible_lines(lines, room, true);
        let mut y = 2u16;
        for line in shown {
            if y >= stop {
                break;
            }
            put_at(screen, y, 0, &line, Style::default(), width);
            y += 1;
        }
        if !banner.is_empty() && box_top >= 4 {
            put_at(
                screen,
                box_top - 1,
                0,
                &banner,
                Style {
                    bold: true,
                    ..Style::default()
                },
                width,
            );
        }
        self.paint_box(screen, box_top, app_name, version, title, width);
        screen.refresh();
    }

    fn paint_collected(
        &mut self,
        screen: &mut dyn Screen,
        app_name: &str,
        version: &str,
        title: &str,
        path: &str,
        parts: &[(String, String, String, String)],
        height: u16,
        width: u16,
    ) {
        screen.erase();
        if self.phase == "result" {
            self.paint_result(screen, app_name, version, title, height, width);
            return;
        }
        put_at(screen, 0, 0, path, Style::default(), width);
        let box_top = height.saturating_sub(CHROME_ROWS);
        let banner = self.banner();
        let mut stop = box_top;
        if !banner.is_empty() && box_top >= 4 {
            stop = box_top - 1;
        }
        let mut y = 2u16;
        if y < stop {
            for (idx, (number_field, short, verb_pad, explain)) in parts.iter().enumerate() {
                if y >= stop {
                    break;
                }
                let selected = idx == self.index;
                let attr = if selected && self.focus == "list" {
                    Style {
                        reverse: true,
                        ..Style::default()
                    }
                } else if selected {
                    Style {
                        underline: true,
                        ..Style::default()
                    }
                } else {
                    Style::default()
                };
                put_at(screen, y, 0, number_field, attr, width);
                let verb_x = MenuPainter::display_width(number_field) as u16;
                let mut verb_style = attr;
                verb_style.bold = true;
                put_at(screen, y, verb_x, short, verb_style, width);
                let colon_x = verb_x + MenuPainter::display_width(short) as u16;
                let colon = format!("{verb_pad}: ");
                put_at(screen, y, colon_x, &colon, attr, width);
                let explain_x = colon_x + MenuPainter::display_width(&colon) as u16;
                let mut explain_style = attr;
                explain_style.italic = true;
                put_at(screen, y, explain_x, explain, explain_style, width);
                y += 1;
            }
        }
        if !banner.is_empty() && box_top >= 4 {
            put_at(
                screen,
                box_top - 1,
                0,
                &banner,
                Style {
                    bold: true,
                    ..Style::default()
                },
                width,
            );
        }
        self.paint_box(screen, box_top, app_name, version, title, width);
        screen.refresh();
    }

    fn paint_result(
        &mut self,
        screen: &mut dyn Screen,
        app_name: &str,
        version: &str,
        title: &str,
        height: u16,
        width: u16,
    ) {
        let italic = Style {
            italic: true,
            ..Style::default()
        };
        put_at(
            screen,
            0,
            0,
            app_name,
            Style {
                bold: true,
                ..Style::default()
            },
            width,
        );
        let mut cursor = MenuPainter::display_width(app_name) as u16;
        put_at(screen, 0, cursor, " (", Style::default(), width);
        cursor += 2;
        put_at(screen, 0, cursor, version, italic, width);
        cursor += MenuPainter::display_width(version) as u16;
        put_at(
            screen,
            0,
            cursor,
            &format!(") — {title}"),
            Style::default(),
            width,
        );
        let lines: Vec<&str> = if self.result_text.is_empty() {
            vec![""]
        } else {
            self.result_text.lines().collect()
        };
        let lines = if lines.is_empty() { vec![""] } else { lines };
        let room = MenuPainter::result_room(height, lines.len());
        let max_offset = lines.len().saturating_sub(room);
        if self.result_offset > max_offset {
            self.result_offset = max_offset;
        }
        let end = self.result_offset.saturating_add(room).min(lines.len());
        let window = &lines[self.result_offset..end];
        let mut y = 2u16;
        for line in window {
            if y >= height.saturating_sub(1) {
                break;
            }
            put_at(screen, y, 0, line, Style::default(), width);
            y += 1;
        }
        if lines.len() > height.saturating_sub(3) as usize {
            put_at(
                screen,
                height.saturating_sub(2),
                0,
                "Up/Down scrolls this page.",
                italic,
                width,
            );
            put_at(
                screen,
                height.saturating_sub(1),
                0,
                "Press a key to return to the main menu.",
                italic,
                width,
            );
        } else {
            let mut hint_y = y + 1;
            if hint_y >= height {
                hint_y = height.saturating_sub(1);
            }
            put_at(
                screen,
                hint_y,
                0,
                "Press a key to return to the main menu.",
                italic,
                width,
            );
        }
        screen.refresh();
    }

    fn paint_box(
        &self,
        screen: &mut dyn Screen,
        top: u16,
        app_name: &str,
        version: &str,
        title: &str,
        width: u16,
    ) {
        let placeable = width as usize;
        if placeable < 2 {
            return;
        }
        let inner = placeable - 2;
        let (field, caret_at) = input_field(self, inner);
        let lines = [
            format!(
                "{FRAME_TOP_LEFT}{}{FRAME_TOP_RIGHT}",
                FRAME_HORIZ.repeat(inner)
            ),
            format!("{FRAME_VERT}{field}{FRAME_VERT}"),
            format!(
                "{FRAME_BOTTOM_LEFT}{}{FRAME_BOTTOM_RIGHT}",
                FRAME_HORIZ.repeat(inner)
            ),
        ];
        for (offset, line) in lines.iter().enumerate() {
            put_at(
                screen,
                top + offset as u16,
                0,
                line,
                Style::default(),
                width,
            );
        }
        put_at(
            screen,
            top + FRAME_ROWS,
            0,
            &format!("  {app_name} {version}  {FRAME_VERT}  {title}  {FRAME_VERT}  {INPUT_HINT}"),
            Style::default(),
            width,
        );
        if let Some(caret_at) = caret_at {
            screen.move_to(top + 1, 1 + caret_at as u16);
        }
    }

    fn activate_number(&mut self, number: i32) -> Option<String> {
        let rows = self.rows();
        if let Some(row) = rows.iter().find(|row| row.number == number) {
            return self.activate(number, row.kind.clone());
        }
        self.error = self.painter.language.text_key("invalid");
        self.focus = "input".to_string();
        None
    }

    fn activate(&mut self, _number: i32, kind: String) -> Option<String> {
        self.error.clear();
        self.focus = "list".to_string();
        self.cursor = 0;
        if kind == "exit" {
            return Some("exit".to_string());
        }
        if kind == "back" {
            self.layer = "front".to_string();
            self.index = 0;
            return None;
        }
        if kind == "back-server" {
            self.layer = "server".to_string();
            self.index = 0;
            return None;
        }
        if matches!(
            kind.as_str(),
            "self-management"
                | "system-log"
                | "language"
                | "rbac"
                | "access"
                | "server"
                | "setup"
                | "audit"
        ) {
            self.layer = match kind.as_str() {
                "self-management" => "self",
                "system-log" => "log",
                "language" => "lang",
                "access" => "rbac",
                other => other,
            }
            .to_string();
            self.index = 0;
            self.buffer.clear();
            self.cursor = 0;
            return None;
        }
        Some(kind)
    }
}

pub struct MenuSession {
    pub model: MenuModel,
    on_version: String,
    on_about: String,
}

impl MenuSession {
    pub fn new(painter: MenuPainter) -> Self {
        Self {
            model: MenuModel::new(painter),
            on_version: crate::VERSION.to_string(),
            on_about: crate::about::AboutPage::text(),
        }
    }

    pub fn take_action(&mut self, action: Option<&str>) -> String {
        match action {
            Some("exit") => "exit".to_string(),
            Some(action) if action.starts_with("set-") => {
                let message = self.model.painter.language.save(&action[4..]);
                self.model.show_front();
                self.model.notice = message;
                "stay".to_string()
            }
            Some("version") => {
                let text = self.on_version.clone();
                self.model.show_result(&text);
                "stay".to_string()
            }
            Some("about") => {
                let text = self.on_about.clone();
                self.model.show_result(&text);
                "stay".to_string()
            }
            Some(action) if !action.is_empty() => {
                self.model.show_result(action);
                "stay".to_string()
            }
            _ => "stay".to_string(),
        }
    }
}

pub fn wait_action(model: &mut MenuModel, screen: &mut dyn Screen) -> Result<String, MenuStop> {
    loop {
        let (height, width) = screen.size();
        if !MenuPainter::screen_can_hold(height, width) {
            return Err(MenuStop::TooSmall);
        }
        model.paint_board(screen, crate::APP_NAME, crate::VERSION);
        let key = if model.phase == "result" {
            screen.read_key()
        } else {
            match screen.poll_key(1000) {
                Some(key) => key,
                None => continue,
            }
        };
        match key {
            Key::Interrupt => return Err(MenuStop::Interrupt),
            Key::Eof if model.phase != "result" && model.layer == "front" => {
                return Err(MenuStop::Leave);
            }
            Key::Esc if model.phase != "result" && model.layer == "front" => {
                return Err(MenuStop::Leave);
            }
            other => {
                if let Some(action) = apply_menu_key(model, other, height) {
                    return Ok(action);
                }
            }
        }
    }
}

pub fn read_field(
    model: &mut MenuModel,
    screen: &mut dyn Screen,
    title: &str,
    lines: &[String],
    secret: bool,
) -> FieldAnswer {
    model.buffer.clear();
    model.cursor = 0;
    model.focus = "input".to_string();
    model.mask = secret;
    let answer = loop {
        let (height, width) = screen.size();
        if !MenuPainter::screen_can_hold(height, width) {
            break FieldAnswer::TooSmall;
        }
        model.paint_prompt(screen, title, lines, crate::APP_NAME, crate::VERSION);
        match screen.read_key() {
            Key::Interrupt => break FieldAnswer::Interrupt,
            Key::Eof => break FieldAnswer::Eof,
            Key::Esc => break FieldAnswer::Esc,
            Key::Enter => break FieldAnswer::Text(model.buffer.clone()),
            Key::Backspace => {
                model.backspace();
                model.error.clear();
            }
            Key::Left => {
                let _ = model.apply_key(KEY_LEFT, height as i32);
            }
            Key::Right => {
                let _ = model.apply_key(KEY_RIGHT, height as i32);
            }
            Key::Home => {
                let _ = model.apply_key(KEY_HOME, height as i32);
            }
            Key::End => {
                let _ = model.apply_key(KEY_END, height as i32);
            }
            Key::Char(ch) if !ch.is_control() => {
                model.insert(ch);
                model.error.clear();
            }
            _ => {}
        }
    };
    model.buffer.clear();
    model.cursor = 0;
    model.mask = false;
    model.focus = "list".to_string();
    model.error.clear();
    answer
}

fn apply_menu_key(model: &mut MenuModel, key: Key, height: u16) -> Option<String> {
    let height = height as i32;
    match key {
        Key::Up => model.apply_key(KEY_UP, height),
        Key::Down => model.apply_key(KEY_DOWN, height),
        Key::Left => model.apply_key(KEY_LEFT, height),
        Key::Right => model.apply_key(KEY_RIGHT, height),
        Key::Home => model.apply_key(KEY_HOME, height),
        Key::End => model.apply_key(KEY_END, height),
        Key::Enter => model.apply_key(10, height),
        Key::Backspace => model.apply_key(KEY_BACKSPACE, height),
        Key::Esc => model.apply_key(27, height),
        Key::Eof => model.apply_key(-1, height),
        Key::Char(ch) if ch.is_ascii() && !ch.is_control() => model.apply_key(ch as i32, height),
        Key::Char(ch) if !ch.is_control() => {
            model.insert(ch);
            None
        }
        Key::Interrupt | Key::Other | Key::Char(_) => {
            model.error = model.painter.language.text_key("invalid");
            None
        }
    }
}

fn input_field(model: &MenuModel, inner: usize) -> (String, Option<usize>) {
    let prefix = format!(" {INPUT_MARK}");
    let focused = model.focus != "list";
    let chars: Vec<char> = model
        .buffer
        .chars()
        .map(|ch| if model.mask { '*' } else { ch })
        .collect();
    let cursor = model.cursor.min(chars.len());
    let reserve = prefix.chars().count() + if focused { 1 } else { 0 };
    let text_room = inner.saturating_sub(reserve);
    let mut start = 0usize;
    if text_room > 0 && cursor > text_room {
        start = cursor - text_room;
    }
    if text_room > 0 && start > chars.len().saturating_sub(text_room) {
        start = chars.len().saturating_sub(text_room);
    }
    let shown: String = if text_room == 0 {
        String::new()
    } else {
        chars.iter().skip(start).take(text_room).collect()
    };
    let rel = cursor.saturating_sub(start).min(shown.chars().count());
    let (body, caret_at) = if focused && inner >= reserve {
        let mut body = String::new();
        for (index, ch) in shown.chars().enumerate() {
            if index == rel {
                body.push_str(CARET_BLOCK);
            }
            body.push(ch);
        }
        if rel == shown.chars().count() {
            body.push_str(CARET_BLOCK);
        }
        (body, Some(prefix.chars().count() + rel))
    } else {
        (shown, None)
    };
    let mut field = prefix + &body;
    let field_len = field.chars().count();
    if field_len < inner {
        field.push_str(&" ".repeat(inner - field_len));
    } else if field_len > inner {
        field = field.chars().take(inner).collect();
    }
    let caret_at = caret_at.filter(|at| *at < inner);
    (field, caret_at)
}

fn visible_lines(lines: &[String], room: usize, pin_last: bool) -> Vec<String> {
    if room < 1 {
        return Vec::new();
    }
    if lines.len() <= room {
        return lines.to_vec();
    }
    if !pin_last {
        return lines[lines.len() - room..].to_vec();
    }
    if room == 1 {
        return vec![lines[lines.len() - 1].clone()];
    }
    let mut out = lines[..room - 2].to_vec();
    out.push("…".to_string());
    out.push(lines[lines.len() - 1].clone());
    out
}

fn put_at(screen: &mut dyn Screen, y: u16, x: u16, text: &str, style: Style, width: u16) {
    if x >= width {
        return;
    }
    let room = (width - x) as usize;
    let clipped = MenuPainter::clip_columns(text, room);
    if !clipped.is_empty() {
        screen.put(y, x, &clipped, style);
    }
}

pub struct MemoryScreen {
    height: u16,
    width: u16,
    rows: Vec<String>,
    keys: VecDeque<Key>,
}

impl MemoryScreen {
    pub fn new(height: u16, width: u16) -> Self {
        Self {
            height,
            width,
            rows: vec![" ".repeat(width as usize); height as usize],
            keys: VecDeque::new(),
        }
    }

    pub fn push(&mut self, key: Key) {
        self.keys.push_back(key);
    }

    pub fn plain(&self) -> String {
        self.rows.join("\n")
    }
}

impl Screen for MemoryScreen {
    fn size(&self) -> (u16, u16) {
        (self.height, self.width)
    }

    fn erase(&mut self) {
        self.rows = vec![" ".repeat(self.width as usize); self.height as usize];
    }

    fn put(&mut self, y: u16, x: u16, text: &str, _style: Style) {
        let y = y as usize;
        let x = x as usize;
        if y >= self.rows.len() || x >= self.width as usize {
            return;
        }
        let mut chars: Vec<char> = self.rows[y].chars().collect();
        let mut col = x;
        for ch in text.chars() {
            if col >= chars.len() {
                break;
            }
            chars[col] = ch;
            col += 1;
        }
        self.rows[y] = chars.into_iter().collect();
    }

    fn refresh(&mut self) {}

    fn move_to(&mut self, _y: u16, _x: u16) {}

    fn read_key(&mut self) -> Key {
        self.keys.pop_front().unwrap_or(Key::Eof)
    }

    fn poll_key(&mut self, _timeout_ms: u64) -> Option<Key> {
        Some(self.read_key())
    }
}

pub struct TermScreen {
    stdout: io::Stdout,
    active: bool,
}

impl TermScreen {
    pub fn enter() -> io::Result<Self> {
        let mut stdout = io::stdout();
        enable_raw_mode()?;
        stdout.execute(EnterAlternateScreen)?;
        stdout.execute(Hide)?;
        let _ = stdout.execute(terminal::DisableLineWrap);
        Ok(Self {
            stdout,
            active: true,
        })
    }

    pub fn leave(&mut self) {
        if !self.active {
            return;
        }
        self.active = false;
        let _ = self.stdout.execute(terminal::EnableLineWrap);
        let _ = self.stdout.execute(Show);
        let _ = self.stdout.execute(LeaveAlternateScreen);
        let _ = disable_raw_mode();
        let _ = self.stdout.flush();
    }
}

impl Drop for TermScreen {
    fn drop(&mut self) {
        self.leave();
    }
}

impl Screen for TermScreen {
    fn size(&self) -> (u16, u16) {
        terminal::size().map(|(w, h)| (h, w)).unwrap_or((24, 80))
    }

    fn erase(&mut self) {
        let _ = self
            .stdout
            .execute(terminal::Clear(terminal::ClearType::All));
        let _ = self.stdout.execute(MoveTo(0, 0));
    }

    fn put(&mut self, y: u16, x: u16, text: &str, style: Style) {
        let _ = self.stdout.execute(MoveTo(x, y));
        apply_style(&mut self.stdout, style);
        let _ = self.stdout.execute(Print(text));
        let _ = self.stdout.execute(SetAttribute(Attribute::Reset));
    }

    fn refresh(&mut self) {
        let _ = self.stdout.flush();
    }

    fn move_to(&mut self, y: u16, x: u16) {
        let _ = self.stdout.execute(MoveTo(x, y));
    }

    fn read_key(&mut self) -> Key {
        loop {
            match event::read() {
                Ok(event) => {
                    if let Some(key) = map_event(event) {
                        return key;
                    }
                }
                Err(_) => return Key::Eof,
            }
        }
    }

    fn poll_key(&mut self, timeout_ms: u64) -> Option<Key> {
        match event::poll(Duration::from_millis(timeout_ms)) {
            Ok(true) => match event::read() {
                Ok(event) => map_event(event),
                Err(_) => Some(Key::Eof),
            },
            Ok(false) => None,
            Err(_) => Some(Key::Eof),
        }
    }
}

fn apply_style(stdout: &mut io::Stdout, style: Style) {
    let _ = stdout.execute(SetAttribute(Attribute::Reset));
    if style.bold {
        let _ = stdout.execute(SetAttribute(Attribute::Bold));
    }
    if style.italic {
        let _ = stdout.execute(SetAttribute(Attribute::Italic));
    }
    if style.reverse {
        let _ = stdout.execute(SetAttribute(Attribute::Reverse));
    }
    if style.underline {
        let _ = stdout.execute(SetAttribute(Attribute::Underlined));
    }
}

fn map_event(event: Event) -> Option<Key> {
    match event {
        Event::Resize(_, _) => None,
        Event::Key(key) => Some(map_key(key.code, key.modifiers)),
        _ => None,
    }
}

fn map_key(code: KeyCode, modifiers: KeyModifiers) -> Key {
    if modifiers.contains(KeyModifiers::CONTROL)
        && matches!(code, KeyCode::Char('c') | KeyCode::Char('C'))
    {
        return Key::Interrupt;
    }
    match code {
        KeyCode::Up => Key::Up,
        KeyCode::Down => Key::Down,
        KeyCode::Left => Key::Left,
        KeyCode::Right => Key::Right,
        KeyCode::Home => Key::Home,
        KeyCode::End => Key::End,
        KeyCode::Enter => Key::Enter,
        KeyCode::Backspace => Key::Backspace,
        KeyCode::Esc => Key::Esc,
        KeyCode::Char(ch) => Key::Char(ch),
        _ => Key::Other,
    }
}

fn local_clock() -> String {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|value| value.as_secs() as i64)
        .unwrap_or(0);
    unsafe {
        let mut tm = std::mem::zeroed();
        libc::localtime_r(&now, &mut tm);
        format!("{:02}:{:02}:{:02}", tm.tm_hour, tm.tm_min, tm.tm_sec)
    }
}
