//! About text for the terminal.
//! requirement-rust-about — identity, host check, private cache leaves, and the star box.
//! The page reports the leaf. It does not create it and it does not search a shared cache parent.

use std::path::Path;

use crate::{APP_NAME, CONSOLE_NAME, VERSION};

pub struct AboutPage;

/// Cache and persistence paths for one about page.
/// `user` empty omits the username segment on the preferred leaf.
/// `home` empty leaves the 2nd fallback and persistence blank.
pub struct CacheReport {
    pub used: String,
    pub preferred: String,
    pub first: String,
    pub second: String,
    pub persistence: String,
}

pub fn cache_report(
    app: &str,
    user: &str,
    pid: u32,
    home: &str,
    shm_is_dir: bool,
    tmp_is_dir: bool,
) -> CacheReport {
    let pid_text = pid.to_string();
    let preferred_leaf = if user.is_empty() {
        format!("cache-{app}-{pid_text}")
    } else {
        format!("cache-{app}-{user}-{pid_text}")
    };
    let second_leaf = format!("cache-{app}-{pid_text}");
    let preferred = format!("/dev/shm/cache/{preferred_leaf}");
    let first = format!("/tmp/cache/{preferred_leaf}");
    let second = if home.is_empty() {
        String::new()
    } else {
        format!("{home}/.cache/{second_leaf}")
    };
    let persistence = if home.is_empty() {
        String::new()
    } else {
        format!("{home}/.local/{app}")
    };
    let used = if shm_is_dir {
        preferred.clone()
    } else if tmp_is_dir {
        first.clone()
    } else {
        second.clone()
    };
    CacheReport {
        used,
        preferred,
        first,
        second,
        persistence,
    }
}

fn cache_user() -> String {
    for name in ["USER", "USERNAME"] {
        if let Ok(value) = std::env::var(name) {
            if !value.is_empty() {
                return value;
            }
        }
    }
    String::new()
}

fn cache_home() -> String {
    std::env::var("HOME")
        .ok()
        .filter(|value| !value.is_empty())
        .unwrap_or_default()
}

impl AboutPage {
    pub fn text() -> String {
        let user = std::env::var("USER").unwrap_or_else(|_| "unknown".to_string());
        let report = cache_report(
            APP_NAME,
            &cache_user(),
            std::process::id(),
            &cache_home(),
            Path::new("/dev/shm").is_dir(),
            Path::new("/tmp").is_dir(),
        );
        let host = [
            format!("{APP_NAME} {VERSION}"),
            "Domain: Online RBAC and centralized configuration".to_string(),
            "Runtime tools: MariaDB or MySQL and sqlite3".to_string(),
            format!("Entry points: {CONSOLE_NAME}"),
            format!(
                "Host: {} {} user {user}",
                std::env::consts::OS,
                std::env::consts::ARCH
            ),
            format!("Cache folder used: {}", report.used),
            format!("Cache folder (preferred): {}", report.preferred),
            format!("Cache folder (1st fallback): {}", report.first),
            format!("Cache folder (2nd fallback): {}", report.second),
            format!("Persistence storage: {}", report.persistence),
            String::new(),
        ];
        let boxed = star_box(&[
            format!("{APP_NAME} ({VERSION}) by Wong Chun Fai on 2026-10-11"),
            String::new(),
            "Basic Usage:".to_string(),
            format!("    {CONSOLE_NAME} login --user-name admin"),
            String::new(),
            "Please visit our homepage: ".to_string(),
            "    \"https://github.com/cloudgen/RustRbac\"".to_string(),
        ]);
        let mut lines = host.to_vec();
        lines.extend(boxed);
        lines.join("\n")
    }
}

fn star_box(rows: &[String]) -> Vec<String> {
    let width = rows
        .iter()
        .map(|row| row.chars().count())
        .max()
        .unwrap_or(0);
    let star = "*".repeat(width + 4);
    let blank = " ".repeat(width);
    let mut lines = vec![star.clone(), format!("* {blank} *")];
    for row in rows {
        let pad = " ".repeat(width - row.chars().count());
        lines.push(format!("* {row}{pad} *"));
    }
    lines.push(format!("* {blank} *"));
    lines.push(star);
    lines
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cache_leaf_is_private_and_skips_the_parent() {
        let report = cache_report("RustRbac", "ada", 99, "/home/ada", true, true);
        assert_eq!(report.preferred, "/dev/shm/cache/cache-RustRbac-ada-99");
        assert_eq!(report.first, "/tmp/cache/cache-RustRbac-ada-99");
        assert_eq!(report.second, "/home/ada/.cache/cache-RustRbac-99");
        assert_eq!(report.persistence, "/home/ada/.local/RustRbac");
        assert_eq!(report.used, report.preferred);
        assert!(report.second.ends_with("/.cache/cache-RustRbac-99"));
        assert!(!report.second.ends_with("cache-RustRbac-ada-99"));
        assert!(!report.persistence.contains("/.local/share/"));

        let no_user = cache_report("RustRbac", "", 99, "/home/ada", false, true);
        assert_eq!(no_user.preferred, "/dev/shm/cache/cache-RustRbac-99");
        assert_eq!(no_user.used, no_user.first);

        let no_dirs = cache_report("RustRbac", "ada", 7, "", false, false);
        assert_eq!(no_dirs.used, "");
        assert_eq!(no_dirs.second, "");
        assert_eq!(no_dirs.persistence, "");
        assert!(no_dirs.preferred.starts_with("/dev/shm/cache/cache-"));
    }
}
