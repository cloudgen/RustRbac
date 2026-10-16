# Requirements index

**Product:** RustRbac (Rust CLI — text menu and local web pages for online RBAC)
**Workspace state:** Specialized product law; **software-development** class; **Cargo** build. Binary `rust-rbac`.
**Product version:** **1.1.1** (`Cargo.toml` is the version source)
**Updated:** 2026-10-16

| ID / key | Title | Area | Status | Path | Updated |
|----------|-------|------|--------|------|---------|
| requirement-class-software-dev | Software-development class law + residual stack (Rust, Cargo) | class | Active | `requirement-class-software-dev.md` | 2026-10-14 |
| requirement-domain-rust-rbac | Domain surface: online RBAC and centralized configuration. Sole Active domain | domain | Active | `requirement-domain-rust-rbac.md` | 2026-10-14 |
| requirement-rbac | RBAC system map, permissions, data scope 1–4, seed on an empty database, protected rows | domain | Active | `requirement-rbac.md` | 2026-10-15 |
| requirement-rbac-menu | Menu list, add, and page edit, remove, and save sort. Types M, C, and F. Rows 18 and 19 stay the text menu | domain | Active | `requirement-rbac-menu.md` | 2026-10-15 |
| requirement-database | Table structure, `rust_rbac` schema, seed-once, sqlite3 and mysql | domain | Active | `requirement-database.md` | 2026-10-14 |
| requirement-environment | `RUSTRBAC_*` names, mode-0600 file, loopback bind, `RUSTRBAC_THEME` | domain | Active | `requirement-environment.md` | 2026-10-16 |
| requirement-html-theme | Page skins `vue3` and `adminlte`. Verb `theme`. Server row 24. No script | domain | Active | `requirement-html-theme.md` | 2026-10-16 |
| requirement-html-body-font | Both skins set the document body to 13px | domain | Active | `requirement-html-body-font.md` | 2026-10-16 |
| requirement-local-login | Saved terminal sign-in `local-login` and `local-login-key`. Not the web cookie | domain | Active | `requirement-local-login.md` | 2026-10-11 |
| requirement-web-interface | Local HTML site, cookie `rust_rbac_session`, CSRF, shared service. Stylesheets from this site. The menus page is `requirement-rbac-menu` | domain | Active | `requirement-web-interface.md` | 2026-10-16 |
| requirement-rust-about | About page: identity, host check, private cache leaf, star box. No `--json` | rust | Active | `requirement-rust-about.md` | 2026-10-14 |
| requirement-rust-cli-interface | CLI entry `rust-rbac`, `PRODUCT_VERBS`, sign-in then menu on a terminal. `theme` does not open the database. Menu edit, remove, and save sort are not verbs | rust | Active | `requirement-rust-cli-interface.md` | 2026-10-16 |
| requirement-rust-logging | `StatusLog::open` in `src/main.rs`. No passwords or argv | rust | Active | `requirement-rust-logging.md` | 2026-10-11 |
| requirement-rust-tui | Text menu rows 1 rbac, 2 server, 3 system-log, 6 self-management, 7 language, 9 Exit. Server row 24 is theme. Menu rows 18 and 19 only | rust | Active | `requirement-rust-tui.md` | 2026-10-16 |
| requirement-rust-cli-language | Menu language: front row 7, codes 71–83, leaf `~/.local/RustRbac/language` | rust | Active | `requirement-rust-cli-language.md` | 2026-10-14 |
| requirement-rust-modules | One concern per file under `src/*.rs` | rust | Active | `requirement-rust-modules.md` | 2026-10-14 |
| requirement-rust-coding-style | Rust 2021. `Result` and `AppError`. No `unwrap` on operator paths | rust | Active | `requirement-rust-coding-style.md` | 2026-10-14 |
| requirement-rust-packaging | `Cargo.toml` version 1.1.1, edition 2021, bin `rust-rbac`, MIT | rust | Active | `requirement-rust-packaging.md` | 2026-10-16 |
| requirement-rust-build-script | Maintainer `./build.sh` Cargo commands, including test-install | rust | Active | `requirement-rust-build-script.md` | 2026-10-11 |
| requirement-rust-project-structure | `src/lib.rs`, `src/main.rs`, `tests/*.rs`, requirements, reviews, screenshots | rust | Active | `requirement-rust-project-structure.md` | 2026-10-14 |
| requirement-rust-readme | Root user document: sections, absolute screenshot URLs | rust | Active | `requirement-rust-readme.md` | 2026-10-14 |
| requirement-rust-error-handling | Fail-closed errors. stderr `ERROR:` and `   Next:` | rust | Active | `requirement-rust-error-handling.md` | 2026-10-11 |
| requirement-runtime-prerequisites | `rustc`, `cargo`, sqlite, mysql client for mysql. No sudo | runtime | Active | `requirement-runtime-prerequisites.md` | 2026-10-11 |
| requirement-domain-videojoin | Retired welcome-line domain | domain | Retired | `requirement-domain-videojoin.md` | 2026-10-11 |
| requirement-video-ffmpeg-pipeline | Retired. No encoder, no concat, no media publish | video | Retired | `requirement-video-ffmpeg-pipeline.md` | 2026-10-14 |

Version column for the files that carry one: database **2.0.2**. Class **1.1.8**. RBAC **1.3.3**. Menu **1.0.0**. Environment **1.0.3**. HTML theme **1.0.0**. HTML body font **1.0.0**. Web **1.0.5**. Local login **1.0.0**. Runtime **1.2.2**. Domain `requirement-domain-rust-rbac` **1.0.3**. Rust TUI **1.1.2**. Rust CLI language **1.1.0**. Rust about **1.0.3**. Rust CLI interface **1.0.4**. Rust logging **1.0.1**. Rust readme **1.0.2**. Rust modules **1.0.2**. Rust project structure **1.0.2**. Rust coding style **1.0.1**. Rust packaging **1.0.1**. Rust build script and error handling **1.0.0**.

## Intentionally absent (by design)

| Surface | Status on RustRbac |
|---------|----------------------|
| Shell online install / `curl\|sh` | **Absent** |
| Type 1 sudoers / root elevation inside the program | **Absent** |
| Code generation, AI, plugins, jobs, Redis, Vue bundle | **Absent** for 1.1.1. The page skins are static CSS |
| Second Active `requirement-domain-*` | **Forbidden**. The only Active domain is `requirement-domain-rust-rbac` |
| `hello`, `join`, `list-videos`, FFmpeg | **Absent** |
| `--json` | **Absent** |
| ChronicleLogger, PyMySQL, a crates.io publish | **Absent** |
| Python package, `pyproject.toml`, `requirement-python-*` | **Absent**. Removed 2026-10-14 |
| `sudo` as a product requirement | **Absent** |

**Install mode:** Cargo. Ship binary `rust-rbac`. Do not claim a crates.io publish. Self-management says cargo, not pip.

**Logger:** `StatusLog::open` constructed in `src/main.rs`. In-process. Not ChronicleLogger.

**Database:** Default engine sqlite3. File `rust_rbac.sqlite3`, mode `0600`, home directory `~/.local/RustRbac`, mode `0700`. Choice 2 is mysql, database `rust_rbac`, tests `rust_rbac_test`. Users `rust_rbac`@localhost and `rust_rbac`@127.0.0.1, granted only on `rust_rbac.*` and `rust_rbac_test.*`. No password default.

**Denied databases:** the product must not open or write `pyrbac`, `pyrbac_test`, `ruoyi`, `ruoyi_db`, or `ry`.

**Rules for agents:**

1. Treat Active rows above as the live product-law inventory for RustRbac.
2. Do not invent additional `requirement-*.md` paths without a registry row in the same change.
3. Product source comments cite only these live requirement files.
4. Keep Status and Path in sync with each file’s header.
5. Exactly one Active `requirement-domain-*` (`requirement-domain-rust-rbac`).
6. Do not open or write `pyrbac`, `pyrbac_test`, `ruoyi`, `ruoyi_db`, or `ry`.
7. This tree has no Python package, no `pyproject.toml`, and no `requirement-python-*`. The ship unit is the Rust crate. Database names `pyrbac` and `pyrbac_test` stay refused.

When adding a requirement: append a row, create the file under `docs/requirements/`, keep Status in sync with the file header.
