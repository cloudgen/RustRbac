**file**: docs/requirements/requirement-python-cli-interface.md
**Status**: Retired (Version 1.2.0)
**Area**: python
**Key**: `requirement-python-cli-interface`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-rust-cli-interface`.

## 1. Purpose

`py-rbac` is the console script. With no arguments on a terminal it prepares a missing database, asks for an account and a password, and then opens the text menu. With no arguments and no terminal it prints help and returns 0. A product verb runs that verb and does not open the menu.

### 1.1 Human-facing

**In one sentence:** Type a verb when you already know the action, or run `py-rbac` with no arguments to sign in and then open the menu.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `py-rbac user-list` |
| The other role | Help | `py-rbac help` |
| Not this file | The row numbers | `requirement-python-tui` |

## 2. Verbs

Lifecycle: `help`, `version`, `about`, `self-install`, `version-check`, `self-update`, `self-uninstall`.

Domain: `login`, `logout`, `whoami`, `serve`, `db-init`, `user-list`, `user-add`, `user-edit`, `user-remove`, `user-reset-password`, `role-list`, `role-grant`, `menu-list`, `menu-add`, `dept-list`, `dept-add`, `post-list`, `post-add`, `config-list`, `config-set`, `config-add`, `dict-list`, `dict-add`, `notice-list`, `notice-add`, `oper-log`, `login-log`, `online-list`, `online-logout`.

`language` is not a verb. `hello`, `join`, and `list-videos` are not verbs.

## 3. Normative rules

1. **Gate.** `--help`, `-h`, `--version`, any flag, and any product verb **MUST NOT** open the menu. `about` on a terminal **MUST** open the about page on the text screen. Empty argv on a terminal **MUST** ask for a sign-in and **MUST** open the menu only after that sign-in succeeds. Esc on that question **MUST** leave without the menu.
2. **Uninstall.** `--force` **MUST** be accepted only with `self-uninstall`.
3. **Errors.** A failure **MUST** print `ERROR:` and `Next:` on stderr and return 1. The next step **MUST** tell the operator what to run.
4. **Password flag.** `--password` is allowed for a non-interactive test. Help **MUST** say that the process list can show it. The sign-in question and every later password question **MUST** ask one field at a time and mask the password.
5. **Serve.** The verb `serve` **MUST** print the URL and block. Menu row 7 **MUST** use the background listener.
6. **Identity.** The program name is `PyRbac`. The console script is `py-rbac`. The basic usage line is `py-rbac login --user-name admin`.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | Empty argv on a terminal is the sign-in gate, then the menu. `login` is not |
| AC-2 | `hello` is not a product verb |
| AC-3 | A domain failure prints ERROR and Next |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Product verb | One word from section 2 |
| Menu gate | Empty argv on a terminal, after the sign-in question |
| Next step | The line after `Next:` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-04 | Active 1.0.0 | Empty-argv menu and pip verbs |
| 2026-10-11 | Active 1.1.0 | RBAC verbs replace `hello` |
| 2026-10-11 | Active 1.2.0 | Empty argv signs in before the menu |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-rust-cli-interface` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
