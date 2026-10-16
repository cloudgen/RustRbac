**file**: docs/requirements/requirement-environment.md
**Status**: Active (Version 1.0.3)
**Area**: domain
**Key**: `requirement-environment`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Process settings for RustRbac 1.1.1 come from the environment, then from a mode-0600 file, then from defaults. The binary is `rust-rbac`. Defaults never include a database password. The default engine is sqlite3. The web listener stays on `127.0.0.1:8765` unless the operator sets an explicit allow. `RUSTRBAC_ALLOW_REMOTE` is environment-only.

### 1.1 Human-facing

**In one sentence:** Put the database password in the environment or in a private file, and the web page stays on this computer unless you explicitly allow another address.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `export RUSTRBAC_DB_PASSWORD=…` |
| The other role | The mode-0600 file | `~/.local/RustRbac/database` |
| Not this file | The menu language leaf | `requirement-rust-cli-language` |

| Includes | Excludes |
|----------|----------|
| The names in the table below | A password baked into the package |
| Loopback unless `RUSTRBAC_ALLOW_REMOTE=yes` | Listening on every interface by default |
| `RUSTRBAC_ALLOW_REMOTE` from the environment only | The same switch stored in the database file |

## 2. Names

| Name | Default | Meaning |
|------|---------|---------|
| `RUSTRBAC_DB_ENGINE` | empty, means sqlite3 | `sqlite3` or `mysql`. Choice 1 is sqlite3. Choice 2 is mysql |
| `RUSTRBAC_DB_HOST` | `127.0.0.1` | Database host |
| `RUSTRBAC_DB_PORT` | `3306` | Database port |
| `RUSTRBAC_DB_NAME` | `rust_rbac` | Database name. Letters, digits, and underscore. Not a denied name |
| `RUSTRBAC_DB_USER` | `rust_rbac` | Database user |
| `RUSTRBAC_DB_PASSWORD` | empty | Required before a mysql connection. No default |
| `RUSTRBAC_SQLITE_FILE` | empty | Replaces `~/.local/RustRbac/rust_rbac.sqlite3` |
| `RUSTRBAC_BIND` | `127.0.0.1` | Web bind address |
| `RUSTRBAC_PORT` | `8765` | Web port. `0` asks for a free port |
| `RUSTRBAC_ALLOW_REMOTE` | unset | Environment only. Must be `yes` before a non-loopback bind |
| `RUSTRBAC_LANG` | unset | Menu language. Overrides the leaf and does not write it |
| `RUSTRBAC_HOME` | unset | When set, replaces `~/.local/RustRbac` for the session file, the saved sign-in files, the language leaf, and the database file |
| `RUSTRBAC_THEME` | empty, means vue3 | `vue3` or `adminlte`. Choice 1 is vue3. Choice 2 is adminlte. This name does not open the database |

## 3. Normative rules

1. **Order.** A non-empty environment value **MUST** win over the file. The file **MUST** win over a default. A password **MUST NOT** have a default.
2. **File.** The database file is `${RUSTRBAC_HOME:-$HOME/.local/RustRbac}/database`. The home directory **MUST** be mode `0700`. The program **MUST** ignore the file unless its mode is `0600`. Lines are `KEY=VALUE`. The sqlite file in that directory is `rust_rbac.sqlite3`, mode `0600`.
3. **Session file.** The terminal token file is `session` in that same directory, mode `0600`, after the directory is mode `0700`. A file that is not mode `0600` **MUST** be treated as signed out. The web cookie `rust_rbac_session` is not this file. The saved sign-in files `local-login` and `local-login-key` live in that same directory. Their rules are `requirement-local-login`. A saved-sign-in file that is not mode `0600` is ignored. This file does not define the ciphertext.
4. **Bind.** The default is `127.0.0.1:8765`. `127.0.0.1`, `localhost`, and `::1` are loopback. Any other address **MUST** be refused unless `RUSTRBAC_ALLOW_REMOTE` is `yes`. That name **MUST** be read from the environment only. A value in the database file **MUST NOT** allow a remote bind.
5. **Language.** `RUSTRBAC_LANG` **MUST** override the menu language for this process and **MUST NOT** write `~/.local/RustRbac/language`. `LANG` and `LC_ALL` **MUST NOT** select a language.
6. **Secrets in docs.** Versioned product documents **MUST NOT** contain a host vault path, a private key name, or a live password. The seeded initial passwords in the user document are the demo values the operator is told to change. The program **MUST NOT** call `sudo`.
7. **Theme.** `RUSTRBAC_THEME` **MUST** be empty, `vue3`, or `adminlte`. Empty means `vue3`. A non-empty environment value **MUST** win over the file. Reading it **MUST NOT** open the database. The page rules are `requirement-html-theme`.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | A mode-0644 database file does not supply the password |
| AC-2 | A mode-0600 file supplies it, and the environment wins |
| AC-3 | Bind `0.0.0.0` without `RUSTRBAC_ALLOW_REMOTE=yes` in the environment is refused |
| AC-4 | `RUSTRBAC_ALLOW_REMOTE=yes` in the database file does not allow a remote bind |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Database file | The mode-0600 `KEY=VALUE` file for connection settings |
| Loopback | `127.0.0.1`, `localhost`, or `::1` |
| Allow remote | The explicit `RUSTRBAC_ALLOW_REMOTE=yes` switch, environment only |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Environment names, 0600 file, loopback bind |
| 2026-10-11 | Active 1.0.1 | Prefix `RUSTRBAC_`. Home `~/.local/RustRbac`. Binary `rust-rbac`. Product 1.1.0. Allow-remote is environment-only |
| 2026-10-11 | Active 1.0.2 | Home also holds `local-login` and `local-login-key`. Cipher law stays on `requirement-local-login` |
| 2026-10-16 | Active 1.0.3 | `RUSTRBAC_THEME` is vue3 or adminlte. Empty means vue3. The environment wins |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
