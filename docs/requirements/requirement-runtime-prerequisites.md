**file**: docs/requirements/requirement-runtime-prerequisites.md
**Status**: Active (Version 1.2.2)
**Area**: runtime
**Key**: `requirement-runtime-prerequisites`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

RustRbac 1.1.1 is a Rust binary, `rust-rbac`. The operator needs `rustc` and `cargo` to build it. sqlite is acceptable as the bundled `rusqlite` feature inside the crate. The mysql engine needs a MySQL client and a MariaDB or MySQL server that already exists. The product mysql database is `rust_rbac`. Tests use `rust_rbac_test`.

The program does not install those with root and does not call sudo. ChronicleLogger is not a prerequisite. PyMySQL is not a prerequisite. FFmpeg is not a prerequisite.

### 1.1 Human-facing

**In one sentence:** Build with Cargo, keep sqlite inside the crate or on the machine, and provide MariaDB yourself when you choose mysql.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `cargo test` as the normal user |
| The other role | The mysql client | Used only when the engine is mysql |
| Not this file | The table list | `requirement-database` |

## 2. Normative rules

1. **Toolchain.** Building and testing **MUST** use `rustc` and `cargo`. The edition is 2021. The version source is `Cargo.toml` `1.1.1`.
2. **sqlite.** The default engine is sqlite3. A bundled sqlite inside the `rusqlite` crate is acceptable. The file is `rust_rbac.sqlite3`, mode `0600`.
3. **mysql.** When the engine is mysql, the operator **MUST** provide a MySQL client and a running MariaDB or MySQL server. The program **MUST NOT** install the server, create a system user, or call sudo. A connection failure **MUST** name the settings to check and `rust-rbac db-init`.
4. **Absent.** ChronicleLogger, PyMySQL, and FFmpeg **MUST NOT** be prerequisites. A missing FFmpeg binary is not an error.
5. **Privilege.** `cargo build` and `cargo test` run as the normal user. The product **MUST NOT** require sudo.

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Prerequisite | Something the operator provides before a build or a mysql verb can succeed |
| Bundled sqlite | sqlite compiled into the crate through `rusqlite`. It is acceptable |
| Next step | The sentence that names the setting to fix, including `rust-rbac db-init` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-04 | Active 1.1.2 | ChronicleLogger required. FFmpeg was a historical note |
| 2026-10-11 | Active 1.2.0 | MariaDB or MySQL and PyMySQL. No encoder |
| 2026-10-11 | Active 1.2.1 | Rustc and Cargo. Bundled sqlite is acceptable. MySQL client for mysql. No ChronicleLogger, PyMySQL, sudo, or FFmpeg |
| 2026-10-16 | Active 1.2.2 | The version source line names Cargo.toml 1.1.1 |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
