# Requirements

Authoritative specialized product law for **RustRbac** lives here.

| Fact | Value |
|------|-------|
| Product | RustRbac |
| Version SSOT | **`1.1.0`** (`Cargo.toml`) |
| Console script | `rust-rbac` |
| Crate / sqlite stem | `rust_rbac` |
| sqlite file | `rust_rbac.sqlite3`, mode `0600`, home directory `~/.local/RustRbac`, mode `0700` |
| Language / build | Rust, Cargo. Not a crates.io publish |
| Domain | `requirement-domain-rust-rbac` — online RBAC and centralized configuration. The only Active domain |
| Menu language | `requirement-rust-cli-language` — row 7, codes 71–83, leaf `~/.local/RustRbac/language` |
| Logger | `StatusLog::open` in `src/main.rs`. Not ChronicleLogger |
| Database | sqlite3 by default. Choice 2 is mysql, database `rust_rbac`, tests `rust_rbac_test` |
| Env prefix | `RUSTRBAC_` |
| Cookie | `rust_rbac_session` |
| Saved sign-in | `requirement-local-login` — `local-login` and `local-login-key`. Not the web cookie |
| Web | `127.0.0.1:8765` unless `RUSTRBAC_ALLOW_REMOTE=yes` (environment only) |
| Maintainer tool | `requirement-rust-build-script` — `./build.sh` |
| Python tree | `src/PyRbac` remains on disk and is not the ship unit. `requirement-python-*` is Retired |

The registry is [index.md](index.md).
