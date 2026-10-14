**file**: docs/requirements/requirement-rust-modules.md
**Status**: Active (Version 1.0.1)
**Area**: rust
**Key**: `requirement-rust-modules`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The ship unit is a Rust crate. One concern lives in one file under `src/*.rs`. `src/lib.rs` declares the modules. `src/main.rs` is the binary entry and the `StatusLog::open` construct. Python sources under `src/PyRbac` are not modules of this crate.

### 1.1 Human-facing

**In one sentence:** Each Rust file does one job, and `main` opens the status log before it runs a verb.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A reader of `src/` | `store.rs` talks to sqlite3 or mysql |
| The other role | `src/lib.rs` | The module list |
| Not this file | Permission strings | `requirement-rbac` |

## 2. Module map

| File | Concern |
|------|---------|
| `src/main.rs` | Process entry. `StatusLog::open` |
| `src/lib.rs` | Crate root. Version and module declarations |
| `src/cli.rs` | Verb dispatch |
| `src/menu.rs` | Text menu |
| `src/texts.rs` | Menu language words |
| `src/about.rs` | About page |
| `src/desk.rs` | Shared service for the menu, the verbs, and the pages |
| `src/access.rs` | Users, roles, menus, departments, posts |
| `src/catalog.rs` | Parameters, dictionaries, notices |
| `src/audit.rs` | Login, operation, and session rows |
| `src/schema.rs` | Create-if-missing and seed-once |
| `src/schema_sql.rs` | `CREATE TABLE` text |
| `src/store.rs` | `rusqlite` and the `mysql` crate |
| `src/password.rs` | PBKDF2 |
| `src/session_file.rs` | Terminal session file |
| `src/local_login.rs` | Saved terminal sign-in |
| `src/settings.rs` | `RUSTRBAC_*` settings |
| `src/web.rs` | Local pages |
| `src/report.rs` | Text tables |
| `src/error.rs` | `AppError` |
| `src/log.rs` | `StatusLog` |

## 3. Normative rules

1. **One concern.** A file under `src/*.rs` **MUST** own one concern from the map. A second product concern **MUST NOT** be added beside it.
2. **Crate.** `src/lib.rs` **MUST** declare the modules the binary uses. The binary name is `rust-rbac`.
3. **Not the Python tree.** `src/PyRbac` **MUST NOT** be compiled as this crate. It remains on disk and is not the ship unit.
4. **Logger.** Only `src/main.rs` constructs `StatusLog::open`. `src/log.rs` defines `StatusLog` and does not open it as a second product entry.
5. **Errors.** `src/error.rs` defines `AppError` and does not dispatch verbs.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | Each mapped concern has its own `src/*.rs` file |
| AC-2 | `src/lib.rs` declares those modules |
| AC-3 | `src/PyRbac` is not a crate module |
| AC-4 | `StatusLog::open` is called from `src/main.rs` |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Concern | One row of the module map |
| Ship unit | The Cargo crate and the `rust-rbac` binary |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | One Rust concern per `src/*.rs` file. Python tree is not the crate |
| 2026-10-11 | Active 1.0.1 | `src/local_login.rs` owns the saved terminal sign-in. `StatusLog::open` stays in `src/main.rs` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
