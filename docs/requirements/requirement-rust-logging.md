**file**: docs/requirements/requirement-rust-logging.md
**Status**: Active (Version 1.0.1)
**Area**: rust
**Key**: `requirement-rust-logging`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

RustRbac keeps an in-process status log. `StatusLog::open` is constructed in `src/main.rs`. The logger is not ChronicleLogger. It is not a second process. It is not `sys_oper_log`.

Operator sentences stay on `requirement-rust-error-handling`. The text menu stays on `requirement-rust-tui`.

### 1.1 Human-facing

**In one sentence:** `src/main.rs` opens `StatusLog`, and that file does not receive your password or the argument list.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The person running `rust-rbac` | A failure is still explained on stderr |
| The other role | `StatusLog::open` | One status file for this process |
| Not this file | Permission text and the menu frame | Domain and TUI requirements |

| Includes | Excludes |
|----------|----------|
| One `StatusLog::open` in `main` | ChronicleLogger, PyMySQL, or a second logger |
| Status lines with no password and no argv | Painting the menu with log lines |

## 2. Normative rules

1. **Construct.** `fn main` in `src/main.rs` **MUST** call `StatusLog::open` before it dispatches a verb or opens the menu. The construct **MUST NOT** be ChronicleLogger.
2. **Secrets.** The log **MUST NOT** contain a password. The log **MUST NOT** contain argv.
3. **Not the audit table.** A status line **MUST NOT** be treated as `sys_oper_log` or `sys_logininfor`. An operation parameter **MUST NOT** be a password.
4. **Failure.** A log write that cannot finish **MUST NOT** replace the stderr `ERROR:` / `   Next:` sentence, and **MUST NOT** hide a non-zero exit.
5. **Privilege.** Opening the log **MUST NOT** call `sudo` or require a dedicated system user.
6. **Saved sign-in.** A saved, cleared, or rejected terminal sign-in **MUST** be recorded only as `sign-in saved`, `sign-in cleared`, or `sign-in rejected`. That line **MUST NOT** include the account or the password. A typed wrong password **MUST NOT** be written to the log. The file rules are `requirement-local-login`. `StatusLog::open` stays in `src/main.rs`.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `src/main.rs` calls `StatusLog::open` |
| AC-2 | The product does not construct ChronicleLogger |
| AC-3 | A logged line does not contain a password or argv |
| AC-4 | A log failure still leaves the operator sentence on stderr |
| AC-5 | A saved, cleared, or rejected sign-in is one of those three facts and names neither the account nor the password |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Status log | The in-process file opened by `StatusLog::open` |
| Construct | The `StatusLog::open` call in `src/main.rs` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | `StatusLog::open` in `src/main.rs`. No ChronicleLogger. No password or argv |
| 2026-10-11 | Active 1.0.1 | Saved, cleared, and rejected sign-ins are those facts only |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
