//! RustRbac ship suite. Injected settings. No process-environment writes.

use std::collections::HashMap;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rust_rbac::about::AboutPage;
use rust_rbac::cli::{exit_choice, Cli, ExitChoice};
use rust_rbac::desk::{Desk, ResumeError, VerbOptions};
use rust_rbac::local_login::{LocalLogin, SavedBody, SignInNote};
use rust_rbac::menu::{
    wait_action, Key, MemoryScreen, MenuLanguage, MenuModel, MenuPainter, MenuRow, MenuSession,
    MenuStop,
};
use rust_rbac::password::PasswordHash;
use rust_rbac::schema::Schema;
use rust_rbac::settings::Settings;
use rust_rbac::web::WebApp;

fn temp_dir(prefix: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "{prefix}{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("clock")
            .as_nanos()
    ));
    fs::create_dir_all(&path).expect("temp");
    path
}

fn mode_of(path: &std::path::Path) -> u32 {
    fs::metadata(path).expect("meta").permissions().mode() & 0o777
}

fn commit(model: &mut MenuModel, token: &str) -> Option<String> {
    model.buffer = token.to_string();
    model.cursor = token.chars().count();
    model.focus = "input".to_string();
    model.apply_key(10, 24)
}

fn names(text: &str) -> Vec<String> {
    let mut found = Vec::new();
    for line in text.lines().skip(1) {
        if line == "(none)" {
            return Vec::new();
        }
        let mut parts = line.split('|');
        let _id = parts.next();
        found.push(parts.next().unwrap_or("").to_string());
    }
    found
}

struct SqliteHome {
    home: PathBuf,
    file: PathBuf,
}

impl SqliteHome {
    fn new() -> Self {
        let home = temp_dir("rustrbac-home-");
        let file = home.join("rust_rbac.sqlite3");
        Self { home, file }
    }

    fn env(&self) -> HashMap<String, String> {
        let mut map = HashMap::new();
        map.insert("RUSTRBAC_HOME".to_string(), self.home.display().to_string());
        map.insert("HOME".to_string(), self.home.display().to_string());
        map.insert("RUSTRBAC_DB_ENGINE".to_string(), "sqlite3".to_string());
        map.insert(
            "RUSTRBAC_SQLITE_FILE".to_string(),
            self.file.display().to_string(),
        );
        map.insert("RUSTRBAC_BIND".to_string(), "127.0.0.1".to_string());
        map.insert("RUSTRBAC_PORT".to_string(), "0".to_string());
        map
    }
}

fn run(env: &HashMap<String, String>, argv: &[&str]) -> (i32, String, String) {
    let args: Vec<String> = argv.iter().map(|item| (*item).to_string()).collect();
    Cli::run(&args, false, Some(env.clone()))
}

#[test]
fn password_round_trip() {
    let hasher = PasswordHash;
    let stored = hasher.hash("admin123");
    assert!(stored.starts_with("pbkdf2_sha256$"));
    assert!(hasher.verify("admin123", &stored));
    assert!(!hasher.verify("admin123", "not-this-scheme"));
    assert!(!hasher.verify("other", &stored));
}

#[test]
fn password_file_mode_remote_and_denied_names() {
    let home = temp_dir("rustrbac-env-");
    let path = home.join("database");
    fs::write(&path, "RUSTRBAC_DB_PASSWORD=from-file\n").expect("write");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644)).expect("mode");
    let mut loose = HashMap::new();
    loose.insert("RUSTRBAC_HOME".to_string(), home.display().to_string());
    loose.insert("HOME".to_string(), home.display().to_string());
    loose.insert("RUSTRBAC_DB_PASSWORD".to_string(), String::new());
    let home_text = home.display().to_string();
    assert_eq!(
        Settings::from_map(loose.clone(), home_text.clone()).password(),
        ""
    );
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("mode");
    assert_eq!(
        Settings::from_map(loose.clone(), home_text.clone()).password(),
        "from-file"
    );
    let mut env_wins = loose.clone();
    env_wins.insert("RUSTRBAC_DB_PASSWORD".to_string(), "from-env".to_string());
    assert_eq!(
        Settings::from_map(env_wins, home_text.clone()).password(),
        "from-env"
    );
    let mut remote = HashMap::new();
    remote.insert("RUSTRBAC_BIND".to_string(), "0.0.0.0".to_string());
    remote.insert("RUSTRBAC_PORT".to_string(), "8765".to_string());
    let error = Settings::from_map(remote.clone(), home_text.clone())
        .bind()
        .expect_err("remote");
    assert!(error.to_string().contains("Refusing to listen"));
    remote.insert("RUSTRBAC_ALLOW_REMOTE".to_string(), "yes".to_string());
    assert_eq!(
        Settings::from_map(remote, home_text.clone())
            .bind()
            .expect("allow")
            .0,
        "0.0.0.0"
    );
    assert_eq!(
        Settings::from_map(HashMap::new(), home_text.clone())
            .bind()
            .expect("default"),
        ("127.0.0.1".to_string(), 8765)
    );
    for denied in ["pyrbac", "pyrbac_test", "ruoyi", "ruoyi_db", "ry"] {
        let mut map = HashMap::new();
        map.insert("RUSTRBAC_DB_NAME".to_string(), denied.to_string());
        map.insert("RUSTRBAC_DB_PASSWORD".to_string(), "unused".to_string());
        let error = Settings::from_map(map, home_text.clone())
            .database()
            .expect_err("denied");
        assert!(
            error.to_string().contains("does not open"),
            "{denied} {}",
            error
        );
    }
}

#[test]
fn cli_gate_and_front_rows() {
    assert!(Cli::PRODUCT_VERBS.contains(&"login"));
    assert!(Cli::PRODUCT_VERBS.contains(&"serve"));
    assert!(Cli::PRODUCT_VERBS.contains(&"db-init"));
    assert!(!Cli::PRODUCT_VERBS.contains(&"hello"));
    assert!(!Cli::PRODUCT_VERBS.contains(&"join"));
    assert!(!Cli::PRODUCT_VERBS.contains(&"list-videos"));
    assert!(!Cli::PRODUCT_VERBS.contains(&"language"));
    assert!(Cli::opens_text_screen(&[], true));
    assert!(Cli::opens_text_screen(&["about".to_string()], true));
    assert!(!Cli::opens_text_screen(&["login".to_string()], true));
    assert!(!Cli::opens_text_screen(
        &[
            "login".to_string(),
            "--user-name".to_string(),
            "admin".to_string()
        ],
        true
    ));
    assert!(!Cli::opens_text_screen(&["serve".to_string()], true));
    assert!(!Cli::opens_text_screen(&["help".to_string()], true));
    assert!(!Cli::opens_text_screen(&["version".to_string()], true));
    assert!(!Cli::opens_text_screen(&["self-update".to_string()], true));
    assert!(!Cli::opens_text_screen(&[], false));
    assert!(!Cli::DOMAIN_VERBS.contains(&"db-engine"));
    let home = temp_dir("rustrbac-menu-");
    let painter = MenuPainter::new(home.to_str().unwrap(), None);
    let rows = painter.display_rows("front");
    let kinds: Vec<&str> = rows.iter().map(|row| row.kind.as_str()).collect();
    assert!(!kinds.contains(&"session"));
    assert_eq!(
        rows[0],
        MenuRow {
            number: 1,
            short: "rbac".to_string(),
            explain: "users, roles, menus, departments, and posts".to_string(),
            kind: "rbac".to_string(),
        }
    );
    assert_eq!(rows[1].kind, "server");
    assert_eq!(rows[2].kind, "system-log");
    assert_eq!(rows[3].number, 6);
    assert_eq!(rows[3].kind, "self-management");
    assert!(rows[3].explain.contains("cargo"));
    assert!(!rows[3].explain.contains("pip"));
    assert_eq!(rows[4].number, 7);
    assert_eq!(rows[4].kind, "language");
    assert_eq!(rows[5].number, 9);
    assert_eq!(rows[5].kind, "exit");
    assert_eq!(
        rows.iter().map(|row| row.number).collect::<Vec<_>>(),
        vec![1, 2, 3, 6, 7, 9]
    );
    let rbac = painter.rows_for("rbac");
    assert_eq!(rbac[0].number, 11);
    assert_eq!(rbac[9].number, 110);
    assert_eq!(
        painter
            .rows_for("server")
            .iter()
            .map(|row| row.number)
            .collect::<Vec<_>>(),
        vec![21, 22, 23, 0]
    );
    let setup = painter.rows_for("setup");
    assert_eq!(setup[setup.len() - 2].number, 218);
    assert_eq!(setup[setup.len() - 2].kind, "db-engine");
    assert_eq!(setup.last().expect("back").kind, "back-server");
    assert_eq!(painter.rows_for("audit")[0].number, 221);
    assert_eq!(painter.rows_for("self")[0].number, 62);
    assert_eq!(painter.display_rows("log")[0].number, 31);
    assert_eq!(painter.display_rows("lang")[0].number, 71);
    let mut model = MenuModel::new(painter);
    assert!(model.apply_key(b'4' as i32, 24).is_none());
    assert!(model.apply_key(10, 24).is_none());
    assert_eq!(model.layer, "front");
    assert!(model.error.contains("not on this list"));
    model.error.clear();
    assert!(model.apply_key(b'2' as i32, 24).is_none());
    assert!(model.apply_key(10, 24).is_none());
    assert_eq!(model.layer, "server");
    assert!(model.apply_key(b'2' as i32, 24).is_none());
    assert!(model.apply_key(b'1' as i32, 24).is_none());
    assert!(model.apply_key(10, 24).is_none());
    assert_eq!(model.layer, "setup");
    assert!(model.apply_key(27, 24).is_none());
    assert_eq!(model.layer, "front");
    assert!(commit(&mut model, "access").is_none());
    assert_eq!(model.layer, "rbac");
    assert!(commit(&mut model, "0").is_none());
    assert_eq!(model.layer, "front");
    assert!(commit(&mut model, "2").is_none());
    assert_eq!(model.layer, "server");
    assert!(commit(&mut model, "22").is_none());
    assert_eq!(model.layer, "audit");
    assert!(commit(&mut model, "0").is_none());
    assert_eq!(model.layer, "server");
    assert!(model.apply_key(b'0' as i32, 24).is_none());
    assert!(model.apply_key(10, 24).is_none());
    assert_eq!(model.layer, "front");
}

#[test]
fn menu_language_save_and_path_width() {
    let home = temp_dir("rustrbac-lang-");
    let mut session = MenuSession::new(MenuPainter::new(home.to_str().unwrap(), None));
    let leaf = session.model.painter.language.leaf_path();
    assert!(!home.join(".local").exists());
    assert!(commit(&mut session.model, "7").is_none());
    assert!(commit(&mut session.model, "0").is_none());
    assert_eq!(session.model.layer, "front");
    assert!(!PathBuf::from(&leaf).exists());
    assert!(commit(&mut session.model, "7").is_none());
    assert!(commit(&mut session.model, "70").is_none());
    assert_eq!(session.model.layer, "lang");
    assert!(session.model.error.contains("not on this list"));
    assert!(!PathBuf::from(&leaf).exists());
    assert!(commit(&mut session.model, "84").is_none());
    assert!(!PathBuf::from(&leaf).exists());
    let action = commit(&mut session.model, "73");
    assert_eq!(action.as_deref(), Some("set-zh-Hant"));
    assert_eq!(session.take_action(action.as_deref()), "stay");
    assert_eq!(session.model.layer, "front");
    assert_eq!(session.model.notice, "選單語言是繁體中文");
    assert_eq!(session.model.painter.path_label(), "路徑");
    assert_eq!(session.model.painter.language.code(), "zh-Hant");
    assert!(leaf.starts_with(home.to_str().unwrap()));
    assert_eq!(fs::read_to_string(&leaf).expect("leaf"), "zh-Hant\n");
    assert_eq!(mode_of(std::path::Path::new(&leaf)), 0o600);
    assert_eq!(
        mode_of(std::path::Path::new(&leaf).parent().expect("dir")),
        0o700
    );
    let front = session.model.painter.display_rows("front");
    assert_eq!(front[4].short, "語言");
    assert_eq!(front[4].kind, "language");
    assert_eq!(front[0].short, "rbac");
    assert_eq!(front[0].kind, "rbac");
    assert_eq!(front[1].kind, "server");
    assert!(front.iter().all(|row| row.kind != "session"));
    assert_eq!(
        session.model.painter.display_rows("log")[0].short,
        "view-log"
    );
    assert_eq!(
        session.model.painter.display_rows("lang")[0].short,
        "English"
    );
    assert!(session
        .model
        .painter
        .format_rows(&front)
        .iter()
        .any(|line| line.contains("cargo")));
    assert!(session
        .model
        .painter
        .format_rows(&front)
        .iter()
        .all(|line| !line.contains("pip")));

    let leaf_path = MenuLanguage::with_env(home.to_str().unwrap(), None).leaf_path();
    fs::write(&leaf_path, "nope\nzh-Hant\n").expect("bad leaf");
    let loaded = MenuLanguage::with_env(home.to_str().unwrap(), None);
    assert_eq!(loaded.code(), "en");
    assert_eq!(loaded.path_label(), "Path");
    assert_eq!(
        fs::read_to_string(&leaf_path).expect("same"),
        "nope\nzh-Hant\n"
    );
    let forced = MenuLanguage::with_env(home.to_str().unwrap(), Some("zh-Hant"));
    assert_eq!(forced.code(), "zh-Hant");
    assert_eq!(forced.path_label(), MenuPainter::PATH_LABEL_ZH_HANT);
    assert_eq!(
        fs::read_to_string(&leaf_path).expect("same"),
        "nope\nzh-Hant\n"
    );
    fs::write(&leaf_path, "es\r\n").expect("es");
    let mut cr = MenuLanguage::with_env(home.to_str().unwrap(), None);
    cr.load();
    assert_eq!(cr.code(), "es");
    assert_eq!(cr.path_label(), "Ruta");

    let mut empty = MenuLanguage::with_env("", None);
    assert_eq!(empty.save("es"), "Could not save the menu language");
    assert_eq!(empty.code(), "en");
    let mut kept = MenuLanguage::with_env(home.to_str().unwrap(), None);
    kept.save("zh-Hant");
    let local = home.join(".local");
    let parked = home.join(".local.kept");
    fs::rename(&local, &parked).expect("rename");
    fs::write(&local, "x").expect("block");
    let message = kept.save("es");
    assert_eq!(kept.code(), "zh-Hant");
    assert_eq!(message, "無法儲存選單語言");

    assert_eq!(MenuPainter::display_width("╭─╮"), 3);
    assert_eq!(MenuPainter::display_width("█"), 1);
    assert_eq!(
        MenuPainter::display_width("路径"),
        MenuPainter::display_width("Path")
    );
    assert_eq!(MenuPainter::display_width("路徑"), 4);
    assert_eq!(MenuPainter::display_width("パス"), 4);
    assert_eq!(MenuPainter::display_width("경로"), 4);
    let mut labeled = MenuPainter::new(home.to_str().unwrap(), Some("zh-Hans"));
    labeled.set_clock("14:05:09");
    assert_eq!(labeled.path_line(Some(1)), "");
    assert_eq!(labeled.path_line(Some(2)), "路");
    assert_eq!(labeled.path_line(Some(3)), "路");
    assert_eq!(labeled.path_line(Some(5)), "路径:");
    let wide = labeled.path_line(None);
    assert!(wide.ends_with("14:05:09"));
    assert!(wide.starts_with("路径: "));
    let mut english = MenuPainter::new(home.to_str().unwrap(), None);
    english.set_clock("14:05:09");
    let logical = english.path_line(None);
    assert!(logical.starts_with("Path: "));
    assert!(logical.ends_with("14:05:09"));
    assert!(!logical.contains("路徑"));
}

#[test]
fn front_board_alignment_uses_cargo() {
    let home = temp_dir("rustrbac-board-");
    let cases = [
        (
            "zh-Hans",
            [
                "1. rbac    : users, roles, menus, departments, and posts",
                "2. server  : setup, audit, and the web listener",
                "3. 系统日志: 查看、清空，以及日志文件夹",
                "6. 自我管理: 版本、关于，以及 cargo 生命周期",
                "7. 语言    : 此菜单的显示语言",
                "9. 离开    : 离开",
            ],
        ),
        (
            "zh-Hant",
            [
                "1. rbac    : users, roles, menus, departments, and posts",
                "2. server  : setup, audit, and the web listener",
                "3. 系統日誌: 查看、清空，以及日誌資料夾",
                "6. 自我管理: 版本、關於，以及 cargo 生命週期",
                "7. 語言    : 這個選單的顯示語言",
                "9. 離開    : 離開",
            ],
        ),
    ];
    for (code, expected) in cases {
        let painter = MenuPainter::new(home.to_str().unwrap(), Some(code));
        let lines = painter.format_rows(&painter.display_rows("front"));
        assert_eq!(lines, expected, "{code}");
    }
}

#[test]
fn front_board_paints_the_tui_frame() {
    let home = temp_dir("rustrbac-frame-");
    let mut session = MenuSession::new(MenuPainter::new(home.to_str().unwrap(), None));
    session.model.painter.set_clock("14:05:09");
    let mut screen = MemoryScreen::new(24, 80);
    session
        .model
        .paint_board(&mut screen, rust_rbac::APP_NAME, rust_rbac::VERSION);
    let text = screen.plain();
    assert!(text.contains('╭'));
    assert!(text.contains('╮'));
    assert!(text.contains('╰'));
    assert!(text.contains('╯'));
    assert!(text.contains("│ > "));
    assert!(text.contains("RustRbac 1.1.0"));
    assert!(text.contains("main menu"));
    assert!(text.contains("14:05:09"));
    assert!(text.contains("1. rbac"));
    assert!(text.contains("9."));
    assert!(text.contains("Exit"));
    assert!(!text.contains("Choice:"));
    assert!(!text.contains('█'));

    session.model.focus = "input".to_string();
    session
        .model
        .paint_board(&mut screen, rust_rbac::APP_NAME, rust_rbac::VERSION);
    assert!(screen.plain().contains('█'));

    let mut driven = MemoryScreen::new(24, 80);
    driven.push(Key::Char('9'));
    driven.push(Key::Enter);
    let action = wait_action(&mut session.model, &mut driven).expect("exit row");
    assert_eq!(action, "exit");
    assert!(driven.plain().contains('╭'));
    assert!(!driven.plain().contains("Choice:"));

    let mut leave = MemoryScreen::new(12, 40);
    leave.push(Key::Esc);
    assert_eq!(
        wait_action(&mut session.model, &mut leave).unwrap_err(),
        MenuStop::Leave
    );
    let mut small = MemoryScreen::new(4, 20);
    assert_eq!(
        wait_action(&mut session.model, &mut small).unwrap_err(),
        MenuStop::TooSmall
    );
}

#[test]
fn about_page_and_docs_identity() {
    let cache_user = std::env::var("USER")
        .ok()
        .filter(|value| !value.is_empty())
        .or_else(|| {
            std::env::var("USERNAME")
                .ok()
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_default();
    let cache_home = std::env::var("HOME").unwrap_or_default();
    let report = rust_rbac::about::cache_report(
        "RustRbac",
        &cache_user,
        std::process::id(),
        &cache_home,
        std::path::Path::new("/dev/shm").is_dir(),
        std::path::Path::new("/tmp").is_dir(),
    );
    let preferred_before = std::path::Path::new(&report.preferred).exists();
    let text = AboutPage::text();
    assert!(text.contains(&format!("RustRbac {}", rust_rbac::VERSION)));
    assert!(text.contains("Domain: Online RBAC and centralized configuration"));
    assert!(text.contains("sqlite3"));
    assert!(text.contains("MariaDB") || text.contains("MySQL"));
    assert!(text.contains("rust-rbac"));
    assert!(text.contains('*'));
    assert!(!text.contains("--json"));
    assert!(!text.contains("py-rbac"));
    assert!(text.contains(&format!("Cache folder used: {}", report.used)));
    assert!(text.contains(&format!("Cache folder (preferred): {}", report.preferred)));
    assert!(text.contains(&format!("Cache folder (1st fallback): {}", report.first)));
    assert!(text.contains(&format!("Cache folder (2nd fallback): {}", report.second)));
    assert!(text.contains(&format!("Persistence storage: {}", report.persistence)));
    assert!(report.preferred.contains("/dev/shm/cache/cache-RustRbac-"));
    assert!(report.first.contains("/tmp/cache/cache-RustRbac-"));
    assert!(!report.persistence.contains("/.local/share/"));
    assert!(!report.used.ends_with("/dev/shm"));
    assert!(!report.used.ends_with("/.cache"));
    if !cache_home.is_empty() {
        assert!(report.persistence.ends_with("/.local/RustRbac"));
        assert!(report
            .second
            .contains(&format!("/.cache/cache-RustRbac-{}", std::process::id())));
        assert!(!report.second.contains(&format!(
            "cache-RustRbac-{cache_user}-{}",
            std::process::id()
        )));
    }
    if !preferred_before {
        assert!(!std::path::Path::new(&report.preferred).exists());
    }
    let readme = fs::read_to_string("README.md").expect("readme");
    assert!(readme.contains("version-1.1.0"));
    assert_eq!(rust_rbac::VERSION, "1.1.0");
    for heading in [
        "Features",
        "Advantages",
        "Quick Installation",
        "Usage",
        "Screenshots",
        "Examples",
        "Platform Compatibility",
        "Related Projects",
        "Contributing",
        "License",
        "Last Update",
    ] {
        assert!(readme.contains(&format!("\n## {heading}\n")), "{heading}");
    }
    for name in [
        "main-menu-en.png",
        "login-prompt.png",
        "access-menu.png",
        "setup-menu.png",
        "audit-menu.png",
        "system-log.png",
        "language-menu.png",
        "self-management.png",
        "main-menu-zh-hant.png",
        "main-menu-zh-hans.png",
        "main-menu-ja.png",
    ] {
        let url =
            format!("https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/{name}");
        assert!(readme.contains(&url), "{name}");
        assert!(std::path::Path::new("screenshots").join(name).is_file());
    }
    assert!(!readme.contains("](screenshots/"));
    assert!(readme.contains("no session row"));
    assert!(readme.contains("asks for the account name"));
    assert!(readme.contains("creates the tables when they are missing"));
    assert!(readme.contains("admin123"));
    assert!(readme.contains("operator123"));
    assert!(readme.contains("clerk123"));
    assert!(readme.contains("lead123"));
    assert!(readme.contains("PBKDF2"));
    assert!(readme.contains("sys_session"));
    assert!(readme.contains("RUSTRBAC_ALLOW_REMOTE"));
    assert!(!readme.contains("join-first.png"));
    assert!(!readme.contains("vaults/"));
    let main_rs = fs::read_to_string("src/main.rs").expect("main");
    assert!(main_rs.contains("StatusLog::open"));
}

#[test]
fn sqlite_seed_scope_and_web() {
    let fixture = SqliteHome::new();
    let env = fixture.env();
    let (code, out, err) = run(&env, &["db-init"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains(&fixture.file.display().to_string()), "{out}");
    assert!(out.contains("seeded"), "{out}");
    let (code, out, err) = run(&env, &["db-init"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("present"), "{out}");
    assert!(!out.contains("seeded"), "{out}");

    let (code, out, err) = run(
        &env,
        &[
            "login",
            "--user-name",
            "admin",
            "--password",
            Schema::INITIAL_ADMIN_PASSWORD,
        ],
    );
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Signed in as admin"), "{out}");
    let (code, out, err) = run(&env, &["login"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Signed in as admin"), "{out}");
    let login_file = fixture.home.join("local-login");
    let body = fs::read_to_string(&login_file).expect("local-login");
    assert!(body.starts_with("RUSTRBACLOCAL1\n"), "{body}");
    assert!(!body.contains(Schema::INITIAL_ADMIN_PASSWORD));
    assert_eq!(mode_of(&login_file), 0o600);
    let key_file = fixture.home.join("local-login-key");
    assert_eq!(mode_of(&key_file), 0o600);
    assert_eq!(fs::metadata(&key_file).expect("key").len(), 32);
    match LocalLogin::new(fixture.home.display().to_string()).load() {
        SavedBody::Ready {
            user_name,
            password,
        } => {
            assert_eq!(user_name, "admin");
            assert_eq!(password, Schema::INITIAL_ADMIN_PASSWORD);
        }
        other => panic!("saved sign-in {other:?}"),
    }
    let (code, out, err) = run(&env, &["whoami"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("admin=yes"), "{out}");
    let (code, out, err) = run(&env, &["user-list"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(names(&out), vec!["admin", "operator", "clerk", "lead"]);

    let (code, _out, err) = run(
        &env,
        &[
            "login",
            "--user-name",
            "operator",
            "--password",
            Schema::INITIAL_OPERATOR_PASSWORD,
        ],
    );
    assert_eq!(code, 0, "{err}");
    let (code, out, err) = run(&env, &["user-list"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(names(&out), vec!["operator"]);
    let (code, _out, err) = run(
        &env,
        &[
            "user-add",
            "--user-name",
            "extra",
            "--nick-name",
            "Extra",
            "--dept-id",
            "102",
            "--role-id",
            "2",
        ],
    );
    assert_eq!(code, 1);
    assert!(err.contains("Missing permission"), "{err}");
    let (code, _out, err) = run(
        &env,
        &[
            "config-set",
            "--config-key",
            "sys.user.initPassword",
            "--config-value",
            "nope",
        ],
    );
    assert_eq!(code, 1);
    assert!(err.contains("Missing permission"), "{err}");

    let (code, _out, err) = run(
        &env,
        &[
            "login",
            "--user-name",
            "lead",
            "--password",
            Schema::INITIAL_LEAD_PASSWORD,
        ],
    );
    assert_eq!(code, 0, "{err}");
    let (code, out, err) = run(&env, &["user-list"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(names(&out), vec!["admin", "clerk", "lead"]);

    let (code, _out, err) = run(
        &env,
        &[
            "login",
            "--user-name",
            "clerk",
            "--password",
            Schema::INITIAL_CLERK_PASSWORD,
        ],
    );
    assert_eq!(code, 0, "{err}");
    let (code, out, err) = run(&env, &["user-list"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(names(&out), vec!["clerk"]);

    let desk = Desk::new(Some(env.clone()));
    desk.store
        .execute(
            "UPDATE sys_role SET data_scope = '5' WHERE role_id = 2",
            &[],
        )
        .expect("scope");
    let (code, out, err) = run(&env, &["user-list"]);
    assert_eq!(code, 0, "{err}");
    assert_eq!(names(&out), Vec::<String>::new());
    desk.store
        .execute(
            "UPDATE sys_role SET data_scope = '3' WHERE role_id = 2",
            &[],
        )
        .expect("restore");

    let (code, _out, err) = run(
        &env,
        &[
            "login",
            "--user-name",
            "admin",
            "--password",
            Schema::INITIAL_ADMIN_PASSWORD,
        ],
    );
    assert_eq!(code, 0, "{err}");
    let (code, _out, err) = run(&env, &["user-remove", "--user-id", "1"]);
    assert_eq!(code, 1);
    assert!(err.contains("cannot be removed"), "{err}");
    let (code, _out, err) = run(&env, &["user-edit", "--user-id", "1", "--status", "1"]);
    assert_eq!(code, 1);
    assert!(err.contains("stays enabled"), "{err}");
    let (code, _out, err) = run(
        &env,
        &[
            "role-grant",
            "--role-id",
            "1",
            "--menu-ids",
            "1",
            "--data-scope",
            "3",
        ],
    );
    assert_eq!(code, 1);
    assert!(err.contains("not edited"), "{err}");
    let (code, out, err) = run(&env, &["logout"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("Signed out"), "{out}");
    assert!(!fixture.home.join("local-login").exists());
    let (code, _out, err) = run(&env, &["logout"]);
    assert_eq!(code, 1);
    assert!(err.contains("Not signed in"), "{err}");
    let secret = "not-the-password";
    let (code, _out, err) = run(
        &env,
        &["login", "--user-name", "missing", "--password", secret],
    );
    assert_eq!(code, 1);
    assert!(err.contains("Unknown account or wrong password"), "{err}");
    assert!(!err.contains(secret));
    let (code, _out, err) = run(&env, &["notice-list"]);
    assert_eq!(code, 1);
    assert!(err.contains("Not signed in"), "{err}");
    for entry in fs::read_dir(&fixture.home).unwrap().flatten() {
        if entry.path().is_file() {
            let body = fs::read_to_string(entry.path()).unwrap_or_default();
            assert!(!body.contains(secret), "{}", entry.path().display());
        }
    }

    let (code, out, err) = run(&env, &["db-engine", "--engine", "mysql"]);
    assert_eq!(code, 1, "{out} {err}");
    assert!(err.contains("RUSTRBAC_DB_ENGINE is set"), "{err}");

    let engine_home = temp_dir("rustrbac-engine-");
    let mut engine_env = HashMap::new();
    engine_env.insert(
        "RUSTRBAC_HOME".to_string(),
        engine_home.display().to_string(),
    );
    engine_env.insert("HOME".to_string(), engine_home.display().to_string());
    let sqlite = engine_home.join("absent.sqlite3");
    engine_env.insert(
        "RUSTRBAC_SQLITE_FILE".to_string(),
        sqlite.display().to_string(),
    );
    let (code, out, err) = run(&engine_env, &["db-engine", "--engine", "mysql"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("mysql"), "{out}");
    assert!(!sqlite.exists());

    let desk = Desk::new(Some(env.clone()));
    let web = WebApp::new(
        Arc::clone(&desk.settings),
        Arc::clone(&desk.access),
        Arc::clone(&desk.catalog),
        Arc::clone(&desk.schema),
    );
    let admin = sign_in(&web, "admin", Schema::INITIAL_ADMIN_PASSWORD, "127.0.0.1");
    assert!(!fixture.home.join("local-login").exists());
    let page = web.handle(
        "GET",
        "/users",
        &[("Cookie".to_string(), format!("rust_rbac_session={admin}"))],
        "",
        "127.0.0.1",
    );
    assert_eq!(page.status, 200);
    assert!(page.body.contains("Add a user"));
    assert!(page.body.contains(">clerk<"));
    let csrf = csrf_of(&page.body);
    let denied = web.handle(
        "POST",
        "/users/add",
        &[
            ("Cookie".to_string(), format!("rust_rbac_session={admin}")),
            (
                "Content-Type".to_string(),
                "application/x-www-form-urlencoded".to_string(),
            ),
        ],
        &format!("user_name=x&csrf=nope&nick_name=n&dept_id=101&role_id=2"),
        "127.0.0.1",
    );
    assert_eq!(denied.status, 400);
    assert!(denied.body.contains("form token"));
    let _ = csrf;
    let operator = sign_in(
        &web,
        "operator",
        Schema::INITIAL_OPERATOR_PASSWORD,
        "127.0.0.1",
    );
    let scoped = web.handle(
        "GET",
        "/users",
        &[(
            "Cookie".to_string(),
            format!("rust_rbac_session={operator}"),
        )],
        "",
        "127.0.0.1",
    );
    assert_eq!(scoped.status, 200);
    assert!(!scoped.body.contains("Add a user"));
    assert!(scoped.body.contains(">operator<"));
    assert!(!scoped.body.contains(">clerk<"));
    assert!(!scoped.body.contains(">lead<"));
    let blocked = web.handle(
        "GET",
        "/roles",
        &[(
            "Cookie".to_string(),
            format!("rust_rbac_session={operator}"),
        )],
        "",
        "127.0.0.1",
    );
    assert_eq!(blocked.status, 403);
    assert!(blocked.body.starts_with("<!DOCTYPE") || blocked.body.contains("Missing permission"));
    assert!(blocked.body.contains("Missing permission"));

    let remote = web.handle(
        "POST",
        "/login",
        &[(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        "user_name=admin&password=admin123&engine=mysql",
        "203.0.113.9",
    );
    assert!(remote.body.contains("can be changed from this computer") || remote.status == 400);
    assert!(remote.body.contains("can be changed from this computer"));
    let mut settings = desk.settings.lock().expect("settings");
    assert_eq!(settings.engine().expect("engine"), "sqlite3");
    drop(settings);

    let bare = web.handle("GET", "/users", &[], "", "127.0.0.1");
    assert!(bare.status == 302 || bare.status == 303);
    assert!(bare
        .headers
        .iter()
        .any(|(key, value)| key.eq_ignore_ascii_case("location") && value == "/login"));
    let login_page = web.handle("GET", "/login", &[], "", "127.0.0.1");
    assert!(login_page.body.contains("name=\"engine\""));
    assert!(login_page.body.contains("sqlite3"));
    assert!(login_page.body.contains("mysql"));
    assert!(
        login_page.body.contains("nosniff")
            || login_page
                .headers
                .iter()
                .any(|(k, v)| k == "X-Content-Type-Options" && v == "nosniff")
    );
}

fn sign_in(web: &WebApp, user: &str, password: &str, ip: &str) -> String {
    let reply = web.handle(
        "POST",
        "/login",
        &[(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        &format!("user_name={user}&password={password}&engine=sqlite3"),
        ip,
    );
    assert!(
        reply.status == 302 || reply.status == 303,
        "{} {}",
        reply.status,
        reply.body
    );
    reply
        .headers
        .iter()
        .find(|(key, _)| key.eq_ignore_ascii_case("set-cookie"))
        .and_then(|(_, value)| {
            value
                .split(';')
                .next()
                .and_then(|pair| pair.split_once('='))
                .map(|(_, token)| token.to_string())
        })
        .expect("cookie")
}

fn csrf_of(body: &str) -> String {
    let marker = "name=\"csrf\" value=\"";
    let start = body.find(marker).expect("csrf") + marker.len();
    let rest = &body[start..];
    rest.split('"').next().unwrap_or("").to_string()
}

#[test]
fn loopback_may_post_an_engine_change() {
    let fixture = SqliteHome::new();
    let mut env = fixture.env();
    env.remove("RUSTRBAC_DB_ENGINE");
    let desk = Desk::new(Some(env));
    desk.db_init().expect("seed");
    let web = WebApp::new(
        Arc::clone(&desk.settings),
        Arc::clone(&desk.access),
        Arc::clone(&desk.catalog),
        Arc::clone(&desk.schema),
    );
    let reply = web.handle(
        "POST",
        "/login",
        &[(
            "Content-Type".to_string(),
            "application/x-www-form-urlencoded".to_string(),
        )],
        "user_name=admin&password=admin123&engine=mysql",
        "127.0.0.1",
    );
    assert!(!reply.body.contains("can be changed from this computer"));
    let mut settings = desk.settings.lock().expect("settings");
    assert_eq!(settings.engine().expect("engine"), "mysql");
}

#[test]
fn mysql_test_database_when_the_password_file_is_present() {
    let Ok(unix_home) = std::env::var("HOME") else {
        return;
    };
    let file = PathBuf::from(unix_home).join(".local/RustRbac/database");
    if !file.is_file() || mode_of(&file) != 0o600 {
        return;
    }
    let text = fs::read_to_string(&file).expect("database file");
    let mut password = String::new();
    for line in text.lines() {
        if let Some(value) = line.strip_prefix("RUSTRBAC_DB_PASSWORD=") {
            password = value.to_string();
        }
    }
    if password.is_empty() {
        return;
    }
    let home = temp_dir("rustrbac-mysql-");
    let mut env = HashMap::new();
    env.insert("RUSTRBAC_HOME".to_string(), home.display().to_string());
    env.insert("HOME".to_string(), home.display().to_string());
    env.insert("RUSTRBAC_DB_ENGINE".to_string(), "mysql".to_string());
    env.insert("RUSTRBAC_DB_HOST".to_string(), "127.0.0.1".to_string());
    env.insert("RUSTRBAC_DB_PORT".to_string(), "3306".to_string());
    env.insert("RUSTRBAC_DB_USER".to_string(), "rust_rbac".to_string());
    env.insert("RUSTRBAC_DB_NAME".to_string(), "rust_rbac_test".to_string());
    env.insert("RUSTRBAC_DB_PASSWORD".to_string(), password);
    let desk = Desk::new(Some(env.clone()));
    for name in [
        "sys_user_role",
        "sys_user_post",
        "sys_role_menu",
        "sys_role_dept",
        "sys_session",
        "sys_oper_log",
        "sys_logininfor",
        "sys_notice",
        "sys_dict_data",
        "sys_dict_type",
        "sys_config",
        "sys_user",
        "sys_role",
        "sys_menu",
        "sys_post",
        "sys_dept",
    ] {
        let _ = desk
            .store
            .execute(&format!("DROP TABLE IF EXISTS {name}"), &[]);
    }
    let init = desk.db_init().expect("mysql seed");
    assert!(init.contains("rust_rbac_test"), "{init}");
    assert!(init.contains("seeded"), "{init}");
    let again = desk.db_init().expect("mysql present");
    assert!(again.contains("present"), "{again}");
    assert!(!again.contains("seeded"), "{again}");
    let options = VerbOptions {
        user_name: "admin".to_string(),
        password: Schema::INITIAL_ADMIN_PASSWORD.to_string(),
        ..VerbOptions::default()
    };
    let signed = desk.execute("login", &options).expect("login");
    assert!(signed.contains("Signed in as admin"));
}

#[test]
fn exit_choice_asks_before_logout() {
    assert_eq!(exit_choice(Some("")), ExitChoice::Logout);
    assert_eq!(exit_choice(Some("y")), ExitChoice::Logout);
    assert_eq!(exit_choice(Some("Y")), ExitChoice::Logout);
    assert_eq!(exit_choice(Some("n")), ExitChoice::LeaveKeep);
    assert_eq!(exit_choice(Some("esc")), ExitChoice::Stay);
    assert_eq!(exit_choice(Some("\u{1b}")), ExitChoice::Stay);
    assert_eq!(exit_choice(Some("later")), ExitChoice::Stay);
    assert_eq!(exit_choice(None), ExitChoice::LeaveKeep);
}

#[test]
fn rejected_saved_sign_in_deletes_the_ciphertext() {
    let home = temp_dir("rustrbac-reject-");
    let saved = LocalLogin::new(home.display().to_string());
    saved.save("admin", "secret-value").expect("save");
    let raw = fs::read_to_string(home.join("local-login")).expect("body");
    let mut lines: Vec<String> = raw.lines().map(str::to_string).collect();
    lines[2] = "AAAA".to_string();
    fs::write(home.join("local-login"), format!("{}\n", lines.join("\n"))).expect("tamper");
    fs::set_permissions(home.join("local-login"), fs::Permissions::from_mode(0o600)).expect("mode");
    let mut env = HashMap::new();
    env.insert("RUSTRBAC_HOME".to_string(), home.display().to_string());
    env.insert("HOME".to_string(), home.display().to_string());
    env.insert("RUSTRBAC_DB_ENGINE".to_string(), "sqlite3".to_string());
    env.insert(
        "RUSTRBAC_SQLITE_FILE".to_string(),
        home.join("absent.sqlite3").display().to_string(),
    );
    let desk = Desk::new(Some(env));
    match desk.resume_sign_in() {
        Err(ResumeError::Rejected(error)) => {
            assert!(error.message.contains("rejected"), "{}", error.message);
        }
        other => panic!("expected a rejected sign-in, got {other:?}"),
    }
    assert!(!home.join("local-login").exists());
    assert!(home.join("local-login-key").exists());
    assert!(!home.join("absent.sqlite3").exists());
    assert_eq!(desk.take_note(), SignInNote::Rejected);
    assert_eq!(SignInNote::Rejected.line(), Some("sign-in rejected"));
}

#[test]
fn help_mentions_cargo_and_not_a_publish() {
    let (code, out, err) = run(&HashMap::new(), &["help"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("rust-rbac"));
    assert!(out.contains("cargo"));
    assert!(!out.contains("py-rbac"));
    assert!(!out.contains("crates.io"));
    let (code, out, err) = run(&HashMap::new(), &["version"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("RustRbac 1.1.0"), "{out}");
    let (code, out, err) = run(&HashMap::new(), &["self-install"]);
    assert_eq!(code, 0, "{err}");
    assert!(out.contains("cargo install --path ."), "{out}");
}

#[allow(dead_code)]
fn _settings_arc_type(settings: Arc<Mutex<Settings>>) -> Arc<Mutex<Settings>> {
    settings
}
