# RustRbac review — 2026-10-11

**Product:** RustRbac 1.1.0
**Scope:** Requirements rename, database law, Rust CLI, menu, web, sqlite, and mysql.
**Result:** Pass with the limits below.
**Proof:** `cargo test --offline` on 2026-10-11 — library 1 passed; product 12 passed, 0 failed. The nine faster product tests finished in 20.81s. `sqlite_seed_scope_and_web`, `loopback_may_post_an_engine_change`, and `mysql_test_database_when_the_password_file_is_present` finished in 301.02s. `test-install` was not run.

## What was checked

| Check | Result |
|-------|--------|
| Sole Active domain is `requirement-domain-rust-rbac` | Pass. 20 Active requirement files. `requirement-local-login` is not a second domain |
| Python requirements and `requirement-domain-pyrbac` are Retired, not deleted | Pass |
| `requirement-database` Active 2.0.1 names `rust_rbac` and refuses the old schema names | Pass |
| Operator data scope 3 sees only `operator` | Pass |
| Lead data scope 4 sees admin, clerk, and lead | Pass |
| Unknown scope 5 contributes no rows | Pass |
| Operator cannot add a user or edit a parameter | Pass |
| User 1 cannot be removed or disabled. Role 1 is not granted | Pass |
| Web login, CSRF mismatch 400, operator `/roles` 403, unauthenticated redirect | Pass |
| A remote login cannot change the engine. Loopback can | Pass |
| Mode 0600 password file. Empty environment does not win. Non-empty environment wins | Pass |
| `StatusLog::open` is in `src/main.rs`. A wrong password stays out of the home files | Pass |
| Saved sign-in is `RUSTRBACLOCAL1`, mode 0600, and the password is not plain text. Logout removes it. A web sign-in does not write it | Pass |
| Menu rows 2–9, no session row, language 41–53, cargo on the self-management row | Pass |
| Help names cargo and `rust-rbac`, and does not name `py-rbac` or a registry publish | Pass |
| mysql `rust_rbac_test` seeds once and then reports present | Pass |
| `src/PyRbac` remains on disk | Pass |
| About cache lines are the private leaf `cache/cache-RustRbac-…`. Persistence is `~/.local/RustRbac`. The page does not create the leaf | Pass |

## Limits

- The text menu on a real terminal is a line menu. It is not a curses screen. `Cli::run` with a terminal and no arguments prints the front board and does not read stdin, so the test suite cannot block. `rust-rbac` with no arguments on a terminal uses the current engine, tries the saved sign-in, and asks for the account only when that sign-in is missing or rejected. Row 9 Exit asks before it logs out.
- `serve` from the binary prints the URL and stays up. `Cli::run` starts that listener and stops it when the desk is dropped.
- `user-add` writes the user and the role link as two statements. A crash between them can leave a user with no role. The seed itself is one transaction.
- Debug builds hash passwords slowly. The sqlite web case dominated the 301.02s database run.
- Screenshots are the existing 11 captures. They were not taken again in this run.
- `./build.sh test-install` was not executed.
- Captcha, export, import, and batch logout are not in 1.1.0.

## Storage choices

- Passwords are PBKDF2-HMAC-SHA256, 260000 iterations. There is no salt column.
- Online sessions are `sys_session`.
- A terminal sign-in is also saved as `local-login` plus `local-login-key`. The password is not plain text. A web sign-in does not write those files.
- The default engine is the sqlite file `rust_rbac.sqlite3`. Choice 2 is mysql database `rust_rbac`.
- Data scope stops at 4. Any other scope adds no rows.
