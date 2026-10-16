**file**: docs/requirements/requirement-rust-cli-interface.md
**Status**: Active (Version 1.0.4)
**Area**: rust
**Key**: `requirement-rust-cli-interface`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

`rust-rbac` is the console script. It is the Cargo binary, not a Python module. With no arguments on a terminal it prepares a missing database, tries the saved sign-in, and then opens the text menu. It asks for an account and a password only when that sign-in is missing or rejected. With no arguments and no terminal it prints help and returns 0. A product verb runs that verb and does not open the menu.

Self-management copy says cargo, not pip. The maintainer script is `requirement-rust-build-script`. Failure sentences are `requirement-rust-error-handling`.

### 1.1 Human-facing

**In one sentence:** Type a verb when you already know the action, or run `rust-rbac` with no arguments to try the saved sign-in and then open the menu.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `rust-rbac user-list` |
| The other role | Help | `rust-rbac help` |
| Not this file | The row numbers | `requirement-rust-tui` |

## 2. Verbs

The summary of `PRODUCT_VERBS` is the closed list below. `language` is not a verb. `hello`, `join`, and `list-videos` are not verbs.

Lifecycle: `help`, `version`, `about`, `self-install`, `version-check`, `self-update`, `self-uninstall`.

Domain: `login`, `logout`, `whoami`, `serve`, `db-init`, `db-engine`, `theme`, `user-list`, `user-add`, `user-edit`, `user-remove`, `user-reset-password`, `role-list`, `role-grant`, `menu-list`, `menu-add`, `dept-list`, `dept-add`, `post-list`, `post-add`, `config-list`, `config-set`, `config-add`, `dict-list`, `dict-add`, `notice-list`, `notice-add`, `oper-log`, `login-log`, `online-list`, `online-logout`.

Self-management verbs say cargo, not pip. `theme` is a product verb and is not a domain verb. It does not open the database. `menu-edit`, `menu-remove`, and `menu-sort` are not verbs. Those actions are the menus page in `requirement-rbac-menu`.

## 3. Normative rules

1. **Gate.** `--help`, `-h`, `--version`, any flag, and any product verb **MUST NOT** open the menu. `about` on a terminal **MUST** open the about page. Empty argv on a terminal **MUST** use the current engine, **MUST NOT** ask for an engine, **MUST** try the saved sign-in in `requirement-local-login`, and **MUST** open the menu only after a sign-in succeeds. When that sign-in is missing or rejected, it **MUST** ask for the account name and a masked password. A wrong password **MUST** show the error and ask again. Esc on that question **MUST** leave without the menu.
2. **Uninstall.** `--force` **MUST** be accepted only with `self-uninstall`.
3. **Errors.** A failure **MUST** print `ERROR: {message}` and `   Next: {next}` on stderr and exit 1. The next step **MUST** tell the operator what to run. The sentences are `requirement-rust-error-handling`.
4. **Password flag.** `--password` is allowed for a non-interactive test. Help **MUST** say that the process list can show it. The program **MUST NOT** log the password or argv.
5. **Serve.** The verb `serve` **MUST** print the URL and block. Server row 23 **MUST** use the background listener.
6. **Identity.** The program name is `RustRbac`. The console script is `rust-rbac`. The basic usage line is `rust-rbac login --user-name admin`. The version is `1.1.1` from `Cargo.toml`.
7. **Theme.** `theme` **MUST** choose `vue3` or `adminlte` and **MUST NOT** open the database. Enter selects `vue3`. `2` selects `adminlte`. Server row 24 asks the same question. The skin rules are `requirement-html-theme`.
8. **Saved sign-in.** The verb `login` with no account and no password **MUST** try the token file, then the ciphertext, as `requirement-local-login` states. `logout` **MUST** remove that ciphertext even when the token file is already gone. A missing token **MUST** still report `Not signed in.`

## Under command line for normal user only

On Termux, Git Bash, Windows cmd, or the same class, the person runs `rust-rbac` as the normal user. **This requirement:** do not use administrator privilege, `sudo`, `apt`, or a dedicated system user. Do not pipe a downloaded script into a shell. Git Bash and Windows cmd do not call Termux `pkg`. Building and testing this checkout are `cargo build` and `cargo test` on `requirement-rust-build-script`. Do not `cargo publish`.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | Empty argv on a terminal tries the saved sign-in, then the menu, and does not ask for an engine |
| AC-2 | `hello`, `join`, `list-videos`, and `language` are not in `PRODUCT_VERBS` |
| AC-5 | `menu-edit`, `menu-remove`, and `menu-sort` are not in `PRODUCT_VERBS` |
| AC-3 | A domain failure prints `ERROR:` and `   Next:` on stderr and exits 1 |
| AC-4 | Self-management text says cargo, not pip |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Product verb | One word from the `PRODUCT_VERBS` summary in section 2 |
| Menu gate | Empty argv on a terminal, after the saved sign-in or the account question |
| Next step | The stderr line `   Next: {next}` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Rust binary `rust-rbac`. Verbs from the `PRODUCT_VERBS` summary. Cargo, not pip |
| 2026-10-11 | Active 1.0.1 | Empty argv tries the saved sign-in and does not ask for an engine. Logout removes the ciphertext |
| 2026-10-14 | Active 1.0.2 | The background listener is server row 23 |
| 2026-10-15 | Active 1.0.3 | Menu edit, remove, and save sort are not verbs |
| 2026-10-16 | Active 1.0.4 | Verb `theme`. Version 1.1.1. Theme does not open the database |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
