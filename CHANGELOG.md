# CHANGELOG

All notable changes to **RustRbac** are documented here.
Version SSOT: `Cargo.toml`.

## [1.1.0] - 2026-10-11

### Changed

- The ship unit is the Rust crate `rust_rbac`. The console script is `rust-rbac`.
- The home directory is `~/.local/RustRbac`. The sqlite file is `rust_rbac.sqlite3`. The mysql database is `rust_rbac`. Tests use `rust_rbac_test`.
- Environment names use the `RUSTRBAC_` prefix. The web cookie is `rust_rbac_session`.
- The product refuses to open `pyrbac`, `pyrbac_test`, `ruoyi`, `ruoyi_db`, and `ry`.
- Status logging is `StatusLog::open` in `src/main.rs`. Self-management copy names cargo.
- The earlier Python package remains on disk and is not the ship unit.
- The main menu and its questions use the full-screen board: path line, local clock, rounded input box, and the name and version on the status line. Typed verbs stay on the command line.
- 2026-10-14: The front board is 1 rbac, 2 server, 3 system-log, 6 self-management, 7 language, and 9 Exit. Server holds setup 21, audit 22, and serve 23. Rbac children are 11–19 and 110–113. Setup children are 211–218. Audit children are 221–224. Self-management children are 62–67. Language codes are 71–83. Self-management still says cargo.
- 2026-10-14: The README pictures are captures of this menu, including the server board. The public repository is https://github.com/cloudgen/RustRbac.

## [1.0.2] - 2026-10-11

### Added

- sqlite3 and mysql. The default is the sqlite3 file `~/.local/PyRbac/pyrbac.sqlite3`.
- The terminal, the text menu, and the web sign-in page each offer choice 1 sqlite3 and choice 2 mysql. `py-rbac db-engine` stores the choice and does not open the database. Setup row 78 and the Database page offer the same choice.

## [1.0.1] - 2026-10-11

### Changed

- A terminal with no arguments asks for an account name and a password before the menu. Esc leaves without the menu.
- The front board no longer has a session row. Numbers 2–9 stay. `login`, `logout`, and `whoami` remain typed verbs.
- The menu, `serve`, and every domain verb create missing tables and seed an empty `sys_user` before they continue. A later init still reports `present`.

## [1.0.0] - 2026-10-11

### Added

- Online RBAC and centralized configuration for the text menu, typed verbs, and a local web page.
- MariaDB tables shaped for users, roles, menus, departments, posts, parameters, dictionaries, notices, and audit logs.
- Data scope 1–4. Sessions in `sys_session`. Passwords as PBKDF2-HMAC-SHA256.
- Environment names `PYRBAC_DB_*`, `PYRBAC_BIND`, `PYRBAC_PORT`, `PYRBAC_ALLOW_REMOTE`, `PYRBAC_LANG`, and `PYRBAC_HOME`.
- ChronicleLogger constructed in `def main`.

### Changed

- Package name is **PyRbac**. Console script is `py-rbac`.
- Front row 1 is session. Rows 2, 5, 6, and 7 are access, setup, audit, and serve.
- The about page domain line is `Online RBAC and centralized configuration`.

## Earlier history

This tree was specialized from a HelloTui checkout. Join, file listing, and FFmpeg are not part of PyRbac.
