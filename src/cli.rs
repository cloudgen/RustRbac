//! Console verbs for rust-rbac.
//! requirement-rust-cli-interface — one product verb, or the sign-in menu on a terminal.
//! Lifecycle lines name cargo. This file does not run those commands.

use std::collections::HashMap;
use std::io::{self, BufRead, IsTerminal, Write};
use std::thread;
use std::time::Duration;

use crate::about::AboutPage;
use crate::desk::{Desk, ResumeError, VerbOptions};
use crate::error::AppError;
use crate::local_login::SignInNote;
use crate::log::StatusLog;
use crate::menu::{
    read_field, wait_action, FieldAnswer, MenuPainter, MenuSession, MenuStop, Screen, TermScreen,
};
use crate::{APP_NAME, CONSOLE_NAME, VERSION};

pub struct Cli;

impl Cli {
    pub const PRODUCT_VERBS: &'static [&'static str] = &[
        "help",
        "version",
        "about",
        "self-install",
        "version-check",
        "self-update",
        "self-uninstall",
        "login",
        "logout",
        "whoami",
        "serve",
        "db-init",
        "db-engine",
        "theme",
        "user-list",
        "user-add",
        "user-edit",
        "user-remove",
        "user-reset-password",
        "role-list",
        "role-grant",
        "menu-list",
        "menu-add",
        "dept-list",
        "dept-add",
        "post-list",
        "post-add",
        "config-list",
        "config-set",
        "config-add",
        "dict-list",
        "dict-add",
        "notice-list",
        "notice-add",
        "oper-log",
        "login-log",
        "online-list",
        "online-logout",
    ];

    pub const DOMAIN_VERBS: &'static [&'static str] = &[
        "login",
        "logout",
        "whoami",
        "db-init",
        "user-list",
        "user-add",
        "user-edit",
        "user-remove",
        "user-reset-password",
        "role-list",
        "role-grant",
        "menu-list",
        "menu-add",
        "dept-list",
        "dept-add",
        "post-list",
        "post-add",
        "config-list",
        "config-set",
        "config-add",
        "dict-list",
        "dict-add",
        "notice-list",
        "notice-add",
        "oper-log",
        "login-log",
        "online-list",
        "online-logout",
    ];

    pub fn opens_text_screen(argv: &[String], stdout_is_tty: bool) -> bool {
        if argv
            .iter()
            .any(|token| token == "--help" || token == "-h" || token == "--version")
        {
            return false;
        }
        if !stdout_is_tty {
            return false;
        }
        let mut verb = None;
        let mut saw_force = false;
        for token in argv {
            if token == "--force" {
                saw_force = true;
                continue;
            }
            if token.starts_with('-') {
                return false;
            }
            if verb.is_none() {
                verb = Some(token.as_str());
                continue;
            }
            return false;
        }
        if saw_force && verb != Some("self-uninstall") {
            return false;
        }
        verb == Some("about") || verb.is_none()
    }

    pub fn run(
        argv: &[String],
        stdout_is_tty: bool,
        environ: Option<HashMap<String, String>>,
    ) -> (i32, String, String) {
        match dispatch(argv, stdout_is_tty, environ, false, None) {
            Ok(text) => (0, finish_line(text), String::new()),
            Err(error) => (1, String::new(), error_text(&error)),
        }
    }

    pub fn interactive(environ: Option<HashMap<String, String>>, log: &StatusLog) -> i32 {
        match text_menu(environ, log) {
            Ok(code) => code,
            Err(error) => {
                log.log_message(
                    &format!("{} Next: {}", error.message, error.next),
                    "ERROR",
                    "main",
                );
                eprint!("{}", error_text(&error));
                1
            }
        }
    }
}

pub fn main_with(argv: &[String], log: &StatusLog) -> i32 {
    log.log_message("start", "INFO", "main");
    let tty = io::stdout().is_terminal();
    if Cli::opens_text_screen(argv, tty) {
        let verb = argv.iter().find(|token| !token.starts_with('-'));
        if verb.map(String::as_str) == Some("about") {
            println!("{}", AboutPage::text());
            return 0;
        }
        return Cli::interactive(None, log);
    }
    if parse(argv).ok().and_then(|parsed| parsed.verb).as_deref() == Some("serve")
        && !argv.iter().any(|token| token == "--help" || token == "-h")
    {
        return match serve_blocking(None) {
            Ok(text) => {
                println!("{}", text.trim_end());
                loop {
                    thread::sleep(Duration::from_secs(3600));
                }
            }
            Err(error) => fail(log, &error),
        };
    }
    let (code, out, err) = match dispatch(argv, tty, None, false, Some(log)) {
        Ok(text) => (0, finish_line(text), String::new()),
        Err(error) => (1, String::new(), error_text(&error)),
    };
    if !out.is_empty() {
        print!("{out}");
        let _ = io::stdout().flush();
    }
    if !err.is_empty() {
        eprint!("{err}");
        log.log_message(err.trim(), "ERROR", "main");
    }
    code
}

struct Parsed {
    verb: Option<String>,
    help: bool,
    version_flag: bool,
    force: bool,
    options: VerbOptions,
}

fn dispatch(
    argv: &[String],
    stdout_is_tty: bool,
    environ: Option<HashMap<String, String>>,
    block_serve: bool,
    log: Option<&StatusLog>,
) -> Result<String, AppError> {
    let parsed = parse(argv)?;
    if parsed.help || parsed.verb.as_deref() == Some("help") {
        return Ok(help_text());
    }
    if parsed.version_flag || parsed.verb.as_deref() == Some("version") {
        return Ok(version_line());
    }
    if parsed.force && parsed.verb.as_deref() != Some("self-uninstall") {
        return Err(AppError::new(
            "--force is only for self-uninstall.",
            format!("{CONSOLE_NAME} self-uninstall --force"),
        ));
    }
    let Some(verb) = parsed.verb else {
        if stdout_is_tty {
            return Ok(text_menu_notice());
        }
        return Ok(help_text());
    };
    if !Cli::PRODUCT_VERBS.contains(&verb.as_str()) {
        return Err(AppError::new(
            format!("Unknown verb '{verb}'."),
            format!(
                "{CONSOLE_NAME} help — verbs: {}",
                Cli::PRODUCT_VERBS.join(", ")
            ),
        ));
    }
    if verb == "about" {
        return Ok(AboutPage::text());
    }
    if matches!(
        verb.as_str(),
        "self-install" | "version-check" | "self-update" | "self-uninstall"
    ) {
        return lifecycle(&verb, parsed.force);
    }
    let desk = Desk::new(environ);
    if verb == "db-engine" {
        return engine_verb(&desk, &parsed.options.engine, stdout_is_tty);
    }
    if verb == "theme" {
        return theme_verb(&desk, &parsed.options.theme, stdout_is_tty);
    }
    if verb == "serve" {
        let text = desk.serve_background()?;
        if block_serve {
            loop {
                thread::sleep(Duration::from_secs(3600));
            }
        }
        return Ok(text);
    }
    let outcome = desk.execute(&verb, &parsed.options);
    record_note(log, desk.take_note());
    outcome
}

fn theme_verb(desk: &Desk, theme: &str, stdout_is_tty: bool) -> Result<String, AppError> {
    if !theme.trim().is_empty() {
        return desk.apply_theme(theme);
    }
    let mut lines = desk.theme_lines();
    if !stdout_is_tty || !io::stdin().is_terminal() {
        return Ok(lines.join("\n"));
    }
    let picked = prompt("Choice [vue3]: ", false);
    if let Some(picked) = picked {
        lines.push(desk.apply_theme(&picked)?);
    }
    Ok(lines.join("\n"))
}

fn engine_verb(desk: &Desk, engine: &str, stdout_is_tty: bool) -> Result<String, AppError> {
    if !engine.trim().is_empty() {
        return desk.apply_engine(engine);
    }
    let mut lines = desk.engine_choice_lines()?;
    if !stdout_is_tty || !io::stdin().is_terminal() {
        return Ok(lines.join("\n"));
    }
    let picked = prompt("Choice [keep]: ", false);
    if let Some(picked) = picked {
        lines.push(desk.apply_engine(&picked)?);
    }
    Ok(lines.join("\n"))
}

fn lifecycle(verb: &str, force: bool) -> Result<String, AppError> {
    match verb {
        "self-install" => Ok("cargo install --path .".to_string()),
        "self-update" => Ok("cargo install --path . --force".to_string()),
        "version-check" => Ok(format!("{}\ncargo search rust_rbac", version_line())),
        "self-uninstall" => {
            if !force {
                return Err(AppError::new(
                    "self-uninstall removes this package with cargo.",
                    format!("{CONSOLE_NAME} self-uninstall --force"),
                ));
            }
            Ok("cargo uninstall rust_rbac".to_string())
        }
        _ => Err(AppError::new(
            format!("Unknown verb '{verb}'."),
            format!("{CONSOLE_NAME} help"),
        )),
    }
}

fn version_line() -> String {
    format!("{APP_NAME} {VERSION}")
}

fn help_text() -> String {
    let verbs = Cli::PRODUCT_VERBS.join(", ");
    format!(
        "{CONSOLE_NAME} — online RBAC and centralized configuration.\n\
With no arguments on a terminal, uses the current engine,\n\
tries the saved sign-in, then opens the text menu.\n\
The default engine is the sqlite3 file. It creates missing tables first.\n\
When that sign-in is missing or rejected, it asks for the account name\n\
and a masked password. A wrong password asks again. Esc leaves without the menu.\n\
Row 9 Exit asks before it logs out.\n\
db-engine chooses sqlite3 or mysql and does not open the database.\n\
theme chooses vue3 or adminlte and does not open the database.\n\
Enter selects vue3. 2 selects adminlte. Server row 24 asks the same question.\n\
With no arguments and no terminal, prints this help and stops.\n\
Product verbs: {verbs}.\n\
The text menu is a full-screen board with a bottom input box.\n\
It has no session row. Server row 23 starts the site.\n\
Basic usage: {CONSOLE_NAME} login --user-name admin\n\
version-check prints: cargo search rust_rbac\n\
self-update prints: cargo install --path . --force\n\
self-install prints: cargo install --path .\n\
self-uninstall prints: cargo uninstall rust_rbac\n\
and needs --force. Empty arguments do not install or update.\n\
These lines are cargo commands. This program does not run them.\n\
--password is visible in the process list. The menu masks it.\n\
Change the seeded passwords after the first sign-in."
    )
}

fn text_menu_notice() -> String {
    let home = std::env::var("HOME").unwrap_or_default();
    let lang = std::env::var("RUSTRBAC_LANG").ok();
    let painter = MenuPainter::new(&home, lang.as_deref());
    let mut lines =
        vec!["Text menu: open a terminal and run rust-rbac with no arguments.".to_string()];
    lines.extend(painter.format_rows(&painter.display_rows("front")));
    lines.join("\n")
}

fn parse(argv: &[String]) -> Result<Parsed, AppError> {
    let mut parsed = Parsed {
        verb: None,
        help: false,
        version_flag: false,
        force: false,
        options: VerbOptions::default(),
    };
    let mut index = 0;
    while index < argv.len() {
        let token = argv[index].as_str();
        if token == "--help" || token == "-h" {
            parsed.help = true;
            index += 1;
            continue;
        }
        if token == "--version" {
            parsed.version_flag = true;
            index += 1;
            continue;
        }
        if token == "--force" {
            parsed.force = true;
            index += 1;
            continue;
        }
        if let Some(name) = token.strip_prefix("--") {
            let (name, inline) = match name.split_once('=') {
                Some((name, value)) => (name, Some(value.to_string())),
                None => (name, None),
            };
            let field = flag_field(name).ok_or_else(|| {
                AppError::new(
                    format!("Unknown option '--{name}'."),
                    format!("{CONSOLE_NAME} help"),
                )
            })?;
            let value = if let Some(value) = inline {
                value
            } else {
                index += 1;
                argv.get(index).cloned().ok_or_else(|| {
                    AppError::new(
                        format!("--{name} needs a value."),
                        format!("{CONSOLE_NAME} help"),
                    )
                })?
            };
            set_field(&mut parsed.options, field, value);
            index += 1;
            continue;
        }
        if token.starts_with('-') {
            return Err(AppError::new(
                format!("Unknown option '{token}'."),
                format!("{CONSOLE_NAME} help"),
            ));
        }
        if parsed.verb.is_some() {
            return Err(AppError::new(
                format!("Unexpected argument '{token}'."),
                format!("{CONSOLE_NAME} help"),
            ));
        }
        parsed.verb = Some(token.to_string());
        index += 1;
    }
    Ok(parsed)
}

fn flag_field(name: &str) -> Option<&'static str> {
    Some(match name {
        "user-name" => "user_name",
        "password" => "password",
        "nick-name" => "nick_name",
        "dept-id" => "dept_id",
        "role-id" => "role_id",
        "user-id" => "user_id",
        "status" => "status",
        "menu-ids" => "menu_ids",
        "data-scope" => "data_scope",
        "dept-ids" => "dept_ids",
        "menu-name" => "menu_name",
        "parent-id" => "parent_id",
        "menu-type" => "menu_type",
        "perms" => "perms",
        "path" => "path",
        "post-code" => "post_code",
        "post-name" => "post_name",
        "dept-name" => "dept_name",
        "config-key" => "config_key",
        "config-name" => "config_name",
        "config-value" => "config_value",
        "dict-type" => "dict_type",
        "dict-label" => "dict_label",
        "dict-value" => "dict_value",
        "notice-title" => "notice_title",
        "notice-type" => "notice_type",
        "content" => "content",
        "token" => "token",
        "engine" => "engine",
        "theme" => "theme",
        _ => return None,
    })
}

fn set_field(options: &mut VerbOptions, field: &str, value: String) {
    let slot = match field {
        "user_name" => &mut options.user_name,
        "password" => &mut options.password,
        "nick_name" => &mut options.nick_name,
        "dept_id" => &mut options.dept_id,
        "role_id" => &mut options.role_id,
        "user_id" => &mut options.user_id,
        "status" => &mut options.status,
        "menu_ids" => &mut options.menu_ids,
        "data_scope" => &mut options.data_scope,
        "dept_ids" => &mut options.dept_ids,
        "menu_name" => &mut options.menu_name,
        "parent_id" => &mut options.parent_id,
        "menu_type" => &mut options.menu_type,
        "perms" => &mut options.perms,
        "path" => &mut options.path,
        "post_code" => &mut options.post_code,
        "post_name" => &mut options.post_name,
        "dept_name" => &mut options.dept_name,
        "config_key" => &mut options.config_key,
        "config_name" => &mut options.config_name,
        "config_value" => &mut options.config_value,
        "dict_type" => &mut options.dict_type,
        "dict_label" => &mut options.dict_label,
        "dict_value" => &mut options.dict_value,
        "notice_title" => &mut options.notice_title,
        "notice_type" => &mut options.notice_type,
        "content" => &mut options.content,
        "token" => &mut options.token,
        "engine" => &mut options.engine,
        "theme" => &mut options.theme,
        _ => return,
    };
    *slot = value;
}

enum MenuStep {
    Stay,
    Leave(String),
    Interrupt,
}

fn text_menu(environ: Option<HashMap<String, String>>, log: &StatusLog) -> Result<i32, AppError> {
    let desk = Desk::new(environ);
    let state = desk.ensure_database()?;
    let label = desk.settings.lock().expect("settings").storage_label()?;
    let database_line = format!("Database {label}: {state}.");
    let mut screen = TermScreen::enter().map_err(|_| {
        AppError::new(
            "The text menu could not open the terminal.",
            format!("{CONSOLE_NAME} help"),
        )
    })?;
    let (height, width) = screen.size();
    if !MenuPainter::screen_can_hold(height, width) {
        screen.leave();
        return Err(AppError::new(
            "The terminal is too small for the text menu.",
            format!("{CONSOLE_NAME} help"),
        ));
    }
    let home = desk.settings.lock().expect("settings").home.clone();
    let lang = desk
        .settings
        .lock()
        .expect("settings")
        .environ
        .get("RUSTRBAC_LANG")
        .cloned();
    let mut session = MenuSession::new(MenuPainter::new(&home, lang.as_deref()));
    match sign_in_on_screen(&desk, log, &mut session, &mut screen, &database_line)? {
        MenuStep::Stay => {}
        MenuStep::Leave(text) => {
            screen.leave();
            if !text.is_empty() {
                println!("{text}");
            }
            return Ok(0);
        }
        MenuStep::Interrupt => {
            screen.leave();
            return Ok(130);
        }
    }
    let code = loop {
        match wait_action(&mut session.model, &mut screen) {
            Err(MenuStop::Leave) => break 0,
            Err(MenuStop::Interrupt) => break 130,
            Err(MenuStop::TooSmall) => {
                screen.leave();
                return Err(AppError::new(
                    "The terminal is too small for the text menu.",
                    format!("{CONSOLE_NAME} help"),
                ));
            }
            Ok(kind) => match menu_step(&kind, &mut session, &mut screen, &desk, log)? {
                MenuStep::Stay => {}
                MenuStep::Leave(text) => {
                    screen.leave();
                    if !text.is_empty() {
                        println!("{text}");
                    }
                    return Ok(0);
                }
                MenuStep::Interrupt => break 130,
            },
        }
    };
    screen.leave();
    Ok(code)
}

fn sign_in_on_screen(
    desk: &Desk,
    log: &StatusLog,
    session: &mut MenuSession,
    screen: &mut dyn Screen,
    database_line: &str,
) -> Result<MenuStep, AppError> {
    let mut rejected = String::new();
    match desk.resume_sign_in() {
        Ok(text) => {
            record_note(Some(log), desk.take_note());
            session.model.notice = format!("{text} {database_line}");
            return Ok(MenuStep::Stay);
        }
        Err(ResumeError::NeedAccount) => {
            record_note(Some(log), desk.take_note());
        }
        Err(ResumeError::Rejected(error)) => {
            record_note(Some(log), desk.take_note());
            rejected = error.message;
        }
        Err(ResumeError::Other(error)) => {
            record_note(Some(log), desk.take_note());
            return Err(error);
        }
    }
    loop {
        if !rejected.is_empty() {
            session.model.error = rejected.clone();
        }
        let account = read_field(
            &mut session.model,
            screen,
            "sign-in",
            &[database_line.to_string(), "Account".to_string()],
            false,
        );
        let account = match account {
            FieldAnswer::Text(text) => text.trim().to_string(),
            FieldAnswer::Esc | FieldAnswer::Eof => return Ok(MenuStep::Leave(String::new())),
            FieldAnswer::Interrupt => return Ok(MenuStep::Interrupt),
            FieldAnswer::TooSmall => {
                return Err(AppError::new(
                    "The terminal is too small for the text menu.",
                    format!("{CONSOLE_NAME} help"),
                ));
            }
        };
        if account.is_empty() {
            return Ok(MenuStep::Leave(String::new()));
        }
        let password = read_field(
            &mut session.model,
            screen,
            "sign-in",
            &[database_line.to_string(), "Password".to_string()],
            true,
        );
        let password = match password {
            FieldAnswer::Text(text) => text,
            FieldAnswer::Esc | FieldAnswer::Eof => return Ok(MenuStep::Leave(String::new())),
            FieldAnswer::Interrupt => return Ok(MenuStep::Interrupt),
            FieldAnswer::TooSmall => {
                return Err(AppError::new(
                    "The terminal is too small for the text menu.",
                    format!("{CONSOLE_NAME} help"),
                ));
            }
        };
        match desk.login(&account, &password) {
            Ok(text) => {
                record_note(Some(log), desk.take_note());
                session.model.notice = format!("{text} {database_line}");
                return Ok(MenuStep::Stay);
            }
            Err(error) => rejected = error.message,
        }
    }
}

fn menu_step(
    kind: &str,
    session: &mut MenuSession,
    screen: &mut dyn Screen,
    desk: &Desk,
    log: &StatusLog,
) -> Result<MenuStep, AppError> {
    if kind == "exit" {
        return Ok(exit_step(session, screen, desk, log)?);
    }
    if kind == "help" {
        session.model.show_result(&help_text());
        return Ok(MenuStep::Stay);
    }
    if kind.starts_with("set-") || kind == "version" || kind == "about" {
        let _ = session.take_action(Some(kind));
        return Ok(MenuStep::Stay);
    }
    if matches!(
        kind,
        "self-install" | "version-check" | "self-update" | "self-uninstall"
    ) {
        let force = if kind == "self-uninstall" {
            match read_field(
                &mut session.model,
                screen,
                "self-uninstall",
                &["Type force to confirm".to_string()],
                false,
            ) {
                FieldAnswer::Text(text) => text.trim() == "force",
                FieldAnswer::Esc => return Ok(MenuStep::Stay),
                FieldAnswer::Eof => return Ok(MenuStep::Leave(String::new())),
                FieldAnswer::Interrupt => return Ok(MenuStep::Interrupt),
                FieldAnswer::TooSmall => {
                    return Err(AppError::new(
                        "The terminal is too small for the text menu.",
                        format!("{CONSOLE_NAME} help"),
                    ));
                }
            }
        } else {
            false
        };
        match lifecycle(kind, force) {
            Ok(text) => session.model.show_result(&text),
            Err(error) => session.model.error = error.message,
        }
        return Ok(MenuStep::Stay);
    }
    if kind == "db-engine" {
        let mut lines = desk.engine_choice_lines()?;
        lines.push("Choice [keep]".to_string());
        let picked = match read_field(&mut session.model, screen, "db-engine", &lines, false) {
            FieldAnswer::Text(text) => text,
            FieldAnswer::Esc => return Ok(MenuStep::Stay),
            FieldAnswer::Eof => return Ok(MenuStep::Leave(String::new())),
            FieldAnswer::Interrupt => return Ok(MenuStep::Interrupt),
            FieldAnswer::TooSmall => {
                return Err(AppError::new(
                    "The terminal is too small for the text menu.",
                    format!("{CONSOLE_NAME} help"),
                ));
            }
        };
        match desk.apply_engine(&picked) {
            Ok(text) => session.model.show_result(&text),
            Err(error) => session.model.error = error.message,
        }
        return Ok(MenuStep::Stay);
    }
    if kind == "theme" {
        let mut lines = desk.theme_lines();
        lines.push("Choice [vue3]".to_string());
        let picked = match read_field(&mut session.model, screen, "theme", &lines, false) {
            FieldAnswer::Text(text) => text,
            FieldAnswer::Esc => return Ok(MenuStep::Stay),
            FieldAnswer::Eof => return Ok(MenuStep::Leave(String::new())),
            FieldAnswer::Interrupt => return Ok(MenuStep::Interrupt),
            FieldAnswer::TooSmall => {
                return Err(AppError::new(
                    "The terminal is too small for the text menu.",
                    format!("{CONSOLE_NAME} help"),
                ));
            }
        };
        match desk.apply_theme(&picked) {
            Ok(text) => session.model.show_result(&text),
            Err(error) => session.model.error = error.message,
        }
        return Ok(MenuStep::Stay);
    }
    if kind == "serve" {
        match desk.serve_background() {
            Ok(text) => session.model.show_result(&text),
            Err(error) => session.model.error = error.message,
        }
        return Ok(MenuStep::Stay);
    }
    if matches!(kind, "view-log" | "clear-log" | "log-folder") {
        session.model.show_result(&log_action(kind, log));
        return Ok(MenuStep::Stay);
    }
    let mut options = VerbOptions::default();
    for (field, label, secret) in fields_for(kind) {
        match read_field(
            &mut session.model,
            screen,
            kind,
            &[label.to_string()],
            *secret,
        ) {
            FieldAnswer::Text(value) => set_field(&mut options, field, value.trim().to_string()),
            FieldAnswer::Esc => return Ok(MenuStep::Stay),
            FieldAnswer::Eof => return Ok(MenuStep::Leave(String::new())),
            FieldAnswer::Interrupt => return Ok(MenuStep::Interrupt),
            FieldAnswer::TooSmall => {
                return Err(AppError::new(
                    "The terminal is too small for the text menu.",
                    format!("{CONSOLE_NAME} help"),
                ));
            }
        }
    }
    match desk.execute(kind, &options) {
        Ok(text) => session.model.show_result(&text),
        Err(error) => session.model.error = format!("ERROR: {}", error.message),
    }
    Ok(MenuStep::Stay)
}

fn exit_step(
    session: &mut MenuSession,
    screen: &mut dyn Screen,
    desk: &Desk,
    log: &StatusLog,
) -> Result<MenuStep, AppError> {
    let answer = read_field(
        &mut session.model,
        screen,
        "Exit",
        &["Log out and remove the saved sign-in?".to_string()],
        false,
    );
    Ok(match answer {
        FieldAnswer::Esc => MenuStep::Stay,
        FieldAnswer::Eof => MenuStep::Leave(String::new()),
        FieldAnswer::Interrupt => MenuStep::Interrupt,
        FieldAnswer::TooSmall => {
            return Err(AppError::new(
                "The terminal is too small for the text menu.",
                format!("{CONSOLE_NAME} help"),
            ));
        }
        FieldAnswer::Text(text) => match exit_choice(Some(&text)) {
            ExitChoice::Stay => MenuStep::Stay,
            ExitChoice::LeaveKeep => MenuStep::Leave(String::new()),
            ExitChoice::Logout => {
                let message = match desk.logout() {
                    Ok(text) => text,
                    Err(error) => error_text(&error),
                };
                record_note(Some(log), desk.take_note());
                MenuStep::Leave(message)
            }
        },
    })
}

fn fields_for(kind: &str) -> &'static [(&'static str, &'static str, bool)] {
    match kind {
        "login" => &[
            ("user_name", "Account", false),
            ("password", "Password", true),
        ],
        "user-add" => &[
            ("user_name", "Account", false),
            ("nick_name", "Display name", false),
            ("dept_id", "Department id", false),
            ("role_id", "Role id", false),
            ("password", "Password", true),
        ],
        "user-edit" => &[
            ("user_id", "User id", false),
            ("nick_name", "Display name", false),
            ("dept_id", "Department id", false),
            ("status", "Status 0 or 1", false),
        ],
        "user-remove" => &[("user_id", "User id", false)],
        "user-reset-password" => &[
            ("user_id", "User id", false),
            ("password", "Password", true),
        ],
        "role-grant" => &[
            ("role_id", "Role id", false),
            ("menu_ids", "Menu ids", false),
            ("data_scope", "Data scope", false),
            ("dept_ids", "Department ids", false),
        ],
        "menu-add" => &[
            ("menu_name", "Menu name", false),
            ("parent_id", "Parent id", false),
            ("menu_type", "Type M, C, or F", false),
            ("perms", "Permission", false),
            ("path", "Path", false),
        ],
        "dept-add" => &[
            ("dept_name", "Department name", false),
            ("parent_id", "Parent id", false),
        ],
        "post-add" => &[
            ("post_code", "Post code", false),
            ("post_name", "Post name", false),
        ],
        "config-set" => &[
            ("config_key", "Parameter key", false),
            ("config_value", "Value", false),
        ],
        "config-add" => &[
            ("config_key", "Parameter key", false),
            ("config_name", "Parameter name", false),
            ("config_value", "Value", false),
        ],
        "dict-add" => &[
            ("dict_type", "Dictionary type", false),
            ("dict_label", "Label", false),
            ("dict_value", "Value", false),
        ],
        "notice-add" => &[
            ("notice_title", "Title", false),
            ("notice_type", "Type 1 or 2", false),
            ("content", "Content", false),
        ],
        "online-logout" => &[("token", "Session token", false)],
        _ => &[],
    }
}

fn log_action(kind: &str, log: &StatusLog) -> String {
    let path = log.path();
    match kind {
        "log-folder" => {
            let folder = std::path::Path::new(path)
                .parent()
                .map(|item| item.display().to_string())
                .unwrap_or_default();
            if folder.is_empty() {
                "(none)".to_string()
            } else {
                folder
            }
        }
        "clear-log" => {
            if path.is_empty() {
                return "The log file is not set.".to_string();
            }
            match std::fs::write(path, "") {
                Ok(()) => format!("Cleared {path}."),
                Err(_) => "The log file could not be cleared.".to_string(),
            }
        }
        _ => {
            if path.is_empty() || !std::path::Path::new(path).is_file() {
                return "(none)".to_string();
            }
            std::fs::read_to_string(path).unwrap_or_else(|_| "(none)".to_string())
        }
    }
}

fn serve_blocking(environ: Option<HashMap<String, String>>) -> Result<String, AppError> {
    let desk = Desk::new(environ);
    let text = desk.serve_background()?;
    std::mem::forget(desk);
    Ok(text)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitChoice {
    Logout,
    LeaveKeep,
    Stay,
}

pub fn exit_choice(line: Option<&str>) -> ExitChoice {
    let Some(line) = line else {
        return ExitChoice::LeaveKeep;
    };
    let trimmed = line.trim();
    if trimmed.contains('\u{1b}') || trimmed.eq_ignore_ascii_case("esc") {
        return ExitChoice::Stay;
    }
    if trimmed.is_empty() || trimmed.eq_ignore_ascii_case("y") {
        return ExitChoice::Logout;
    }
    if trimmed.eq_ignore_ascii_case("n") {
        return ExitChoice::LeaveKeep;
    }
    ExitChoice::Stay
}

fn record_note(log: Option<&StatusLog>, note: SignInNote) {
    let Some(log) = log else {
        return;
    };
    if let Some(line) = note.line() {
        log.log_message(line, "INFO", "local-login");
    }
}

fn prompt(label: &str, secret: bool) -> Option<String> {
    print!("{label}");
    let _ = io::stdout().flush();
    let hidden = if secret { hide_echo() } else { None };
    let mut line = String::new();
    let read = io::stdin().lock().read_line(&mut line);
    if hidden.is_some() {
        show_echo();
        println!();
    }
    if read.ok()? == 0 && line.is_empty() {
        return None;
    }
    Some(line.trim().to_string())
}

fn hide_echo() -> Option<()> {
    if !io::stdin().is_terminal() {
        return None;
    }
    unsafe {
        let mut term = std::mem::zeroed();
        if libc::tcgetattr(0, &mut term) != 0 {
            return None;
        }
        term.c_lflag &= !libc::ECHO;
        if libc::tcsetattr(0, libc::TCSANOW, &term) != 0 {
            return None;
        }
    }
    Some(())
}

fn show_echo() {
    unsafe {
        let mut term = std::mem::zeroed();
        if libc::tcgetattr(0, &mut term) != 0 {
            return;
        }
        term.c_lflag |= libc::ECHO;
        let _ = libc::tcsetattr(0, libc::TCSANOW, &term);
    }
}

fn error_text(error: &AppError) -> String {
    format!("ERROR: {}\n   Next: {}\n", error.message, error.next)
}

fn fail(log: &StatusLog, error: &AppError) -> i32 {
    log.log_message(
        &format!("{} Next: {}", error.message, error.next),
        "ERROR",
        "main",
    );
    eprint!("{}", error_text(error));
    1
}

fn finish_line(text: String) -> String {
    if text.is_empty() || text.ends_with('\n') {
        text
    } else {
        format!("{text}\n")
    }
}
