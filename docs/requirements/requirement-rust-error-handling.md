**file**: docs/requirements/requirement-rust-error-handling.md
**Status**: Active (Version 1.0.0)
**Area**: rust
**Key**: `requirement-rust-error-handling`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

RustRbac reports an operator failure with `AppError`: a message and a next step. The process prints both on stderr and exits 1. The status file, once `main` has opened it, does not replace that sentence (`requirement-rust-logging`).

### 1.1 Human-facing

**In one sentence:** When an action cannot finish, RustRbac tells you why and names the next command, on stderr.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | You read `ERROR:` and then `   Next:` |
| The other role | `AppError` | `message` and `next` stay together |
| Not this file | The menu frame | `requirement-rust-tui` |

| Includes | Excludes |
|----------|----------|
| `ERROR: {message}` and `   Next: {next}` on stderr, exit 1 | A silent success after a failed action |
| `Result` and `AppError` on operator paths | `unwrap` that drops the next step |

## 2. Normative rules

1. **Shape.** A failure **MUST** write these two lines to stderr and **MUST** exit 1:

```text
ERROR: {message}
   Next: {next}
```

The second line begins with three spaces. `{message}` and `{next}` come from `AppError`.

2. **Type.** Operator paths **MUST** return `Result` and use `AppError` for the failure the operator sees. They **MUST NOT** `unwrap` or `expect` on that path.
3. **Secrets.** `{message}` and `{next}` **MUST NOT** contain a password or SQL text.
4. **Named sentences.** A missing permission **MUST** start with `Missing permission`. An unknown account or a wrong password **MUST** be `Unknown account or wrong password.` A signed-out action **MUST** be `Not signed in.` Those sentences are also `requirement-domain-rust-rbac`.
5. **Log.** The stderr lines **MUST** still appear when a status line was written. The log **MUST NOT** receive the password or argv.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | A domain failure prints `ERROR:` and `   Next:` on stderr and exits 1 |
| AC-2 | `AppError` carries both the message and the next step |
| AC-3 | The message does not contain a password |
| AC-4 | Operator paths do not `unwrap` away the next step |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| AppError | The Rust error with `message` and `next` |
| Next step | The stderr line `   Next: {next}` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Rust `AppError`. Stderr `ERROR:` and `   Next:`. Exit 1 |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
