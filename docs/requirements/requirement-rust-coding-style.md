**file**: docs/requirements/requirement-rust-coding-style.md
**Status**: Active (Version 1.0.0)
**Area**: rust
**Key**: `requirement-rust-coding-style`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This file is how RustRbac is written. The language is Rust 2021. Operator paths return `Result` and `AppError`. They do not `unwrap`. The module map stays on `requirement-rust-modules`. The stderr shape stays on `requirement-rust-error-handling`.

### 1.1 Human-facing

**In one sentence:** A failure the operator can hit returns `AppError`, and the code does not panic instead of naming the next step.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A maintainer changing a verb | Return `Err(AppError::new(...))` |
| The other role | The module map | One concern in one file |
| Not this file | The menu picture | `requirement-rust-tui` |

| Includes | Excludes |
|----------|----------|
| Edition 2021, `Result`, and `AppError` | `unwrap` or `expect` on an operator path |
| Rust as the ship language | Python as the ship language |

## 2. Normative rules

1. **Edition.** Product Rust **MUST** be edition 2021, the edition in `Cargo.toml`.
2. **Results.** An operator path **MUST** return `Result<T, AppError>` or propagate that error. It **MUST NOT** call `unwrap` or `expect` on a value the operator can fail.
3. **Messages.** The `AppError` message and next step **MUST** follow `requirement-rust-error-handling`. They **MUST NOT** include a password.
4. **Modules.** New behavior **MUST** go in the file that already owns that concern. **MUST NOT** add a second concern to that file.
5. **Ship language.** New product code **MUST** be Rust. `src/PyRbac` is not the ship unit and **MUST NOT** be cited as the live entry.
6. **Privilege.** Product code **MUST NOT** call `sudo` or require a dedicated system user.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `Cargo.toml` says edition 2021 |
| AC-2 | Operator paths use `Result` and `AppError` |
| AC-3 | An operator path does not `unwrap` or `expect` |
| AC-4 | The ship entry is `src/main.rs`, not `src/PyRbac` |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Operator path | Code that can fail for the person running `rust-rbac` |
| AppError | The error type with a message and a next step |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Rust 2021. `Result` and `AppError`. No `unwrap` on operator paths |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
