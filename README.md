# RustRbac - online role-based access control and centralized configuration

![Version 1.1.0](https://img.shields.io/badge/version-1.1.0-blue)
![License MIT](https://img.shields.io/badge/license-MIT-green)
[![CIAO](https://img.shields.io/badge/CIAO-cloudgen-black)](https://github.com/cloudgen/ciao)
[![Stars](https://img.shields.io/github/stars/cloudgen/RustRbac?style=flat-square)](https://github.com/cloudgen/RustRbac)
[![Rust](https://img.shields.io/badge/rust-1.89%2B-blue)](https://www.rust-lang.org/)

RustRbac is online role-based access control and centralized configuration. The text menu, the typed commands, and the local web pages call the same service.

The web pages are plain HTML. There is no JavaScript bundle. This crate is not published on crates.io.

**Domain:** Online RBAC and centralized configuration.

## Features

- A terminal with no arguments uses the current engine, tries the saved sign-in, and opens the menu. It asks for the account name only when that sign-in is missing or rejected.
- Front board rows: 1 rbac, 2 server, 3 system-log, 6 self-management, 7 language, and 9 Exit. Server holds setup, audit, and serve. There is no session row. Exit asks before it leaves. Enter or y logs out. n keeps the saved sign-in. Esc stays.
- The same user, role, menu, department, post, parameter, dictionary, notice, and audit actions on the menu, on the command line, and on the web page.
- Data scope 1 all, 2 custom departments, 3 this department, 4 this department and children. Any other scope shows no rows.
- sqlite3 or MariaDB or MySQL. The default is the sqlite3 file `~/.local/RustRbac/rust_rbac.sqlite3`. mysql uses the database `rust_rbac`.
- An in-process status log. Passwords and the argument vector are not written there.
- The site listens on `127.0.0.1` unless you set `RUSTRBAC_ALLOW_REMOTE=yes`.

## Advantages

- One permission check for every surface. A button that your role cannot use is not shown.
- The menu keeps the path line, the local clock, and the rounded input box.
- Menu language is row 7, codes 71–83. `RUSTRBAC_LANG` overrides it and does not write the language file.
- `rust-rbac` with no arguments on a terminal tries the saved sign-in, then opens the menu, and does not run cargo. It creates the tables when they are missing, before any account question.

## Quick Installation

The default database is a sqlite3 file. You do not install a database server for that choice. mysql is optional: you provide MariaDB or MySQL yourself. Cargo builds this crate. It does not install a database server, and this program does not call sudo.

From a checkout:

```bash
cargo build --release
```

The binary is `target/release/rust-rbac`. A local install, when you want one, is `cargo install --path .`. That command is not a crates.io publish.

The default engine is sqlite3. `rust-rbac db-init` creates `~/.local/RustRbac/rust_rbac.sqlite3` when that file is missing. A password is not required for sqlite3.

```bash
rust-rbac db-init
```

The first init prints `seeded`. A later init prints `present` and does not add another administrator.

Choose the engine with `rust-rbac db-engine`, from setup row 218, or from the web sign-in page. A terminal with no arguments keeps the current engine. Choice 1 is the sqlite3 file. Choice 2 is mysql. Enter on those choices keeps the current engine. `rust-rbac db-engine` does not open the database.

```bash
rust-rbac db-engine --engine sqlite3
rust-rbac db-engine --engine mysql
```

mysql reads the password from the environment or from `~/.local/RustRbac/database` when that file is mode `0600`. The sqlite3 data file is also mode `0600`. The product mysql database is `rust_rbac`. Tests use `rust_rbac_test`. The program refuses to open `pyrbac`, `pyrbac_test`, `ruoyi`, `ruoyi_db`, and `ry`.

```bash
export RUSTRBAC_DB_PASSWORD='the-password-you-chose'
rust-rbac db-engine --engine mysql
rust-rbac db-init
```

On a terminal, `rust-rbac` asks for the account when no saved sign-in is present:

```text
**RustRbac**(*1.1.0*) — sign-in

Password
```

The password field is masked. After a match, the main menu is:

```text
**RustRbac**(*1.1.0*) │ main menu

1. **rbac**: *users, roles, menus, departments, and posts*
2. **server**: *setup, audit, and the web listener*
3. **system-log**: *view, clear, and the log folder*
6. **self-management**: *version, about, and cargo lifecycle*
7. **language**: *display language for this menu*
9. **Exit**: *leave*
```

Choose a number and press Enter. `9` leaves. `0` on a nested board goes back.

Rbac:

```text
**RustRbac**(*1.1.0*) │ rbac

11. **user-list**: *accounts you are allowed to see*
12. **user-add**: *add an account*
13. **user-edit**: *change a name, department, or status*
14. **user-remove**: *mark an account deleted*
15. **user-reset-password**: *set a new password*
16. **role-list**: *roles and data scope*
17. **role-grant**: *menus and data scope for one role*
18. **menu-list**: *menu and permission strings*
19. **menu-add**: *add a directory, page, or button*
110. **dept-list**: *departments*
111. **dept-add**: *add a department*
112. **post-list**: *posts*
113. **post-add**: *add a post*
0. **Back**: *return to the main menu*
```

Server:

```text
**RustRbac**(*1.1.0*) │ server

21. **setup**: *parameters, dictionaries, and notices*
22. **audit**: *operation log, login log, and online users*
23. **serve**: *start the web interface on this computer*
0. **Back**: *return to the main menu*
```

Setup:

```text
**RustRbac**(*1.1.0*) │ setup

211. **config-list**: *parameters*
212. **config-set**: *change one parameter*
213. **config-add**: *add a parameter*
214. **dict-list**: *dictionary values*
215. **dict-add**: *add a dictionary value*
216. **notice-list**: *notices*
217. **notice-add**: *add a notice*
218. **db-engine**: *sqlite3 file or mysql*
0. **Back**: *return to the server menu*
```

Audit:

```text
**RustRbac**(*1.1.0*) │ audit

221. **oper-log**: *operation log*
222. **login-log**: *login log*
223. **online-list**: *sessions that are signed in*
224. **online-logout**: *close one online session*
0. **Back**: *return to the server menu*
```

System log:

```text
**RustRbac**(*1.1.0*) │ system-log

31. **view-log**: *list a log file and show it*
32. **clear-log**: *empty one log file*
33. **log-folder**: *show the log folder*
0. **Back**: *return to the main menu*
```

Self-management:

```text
**RustRbac**(*1.1.0*) │ self-management

62. **version**: *show the installed version*
63. **about**: *version and this computer*
64. **version-check**: *compare this install with cargo*
65. **self-update**: *upgrade this package with cargo*
66. **self-uninstall**: *remove this package with cargo*
67. **self-install**: *install this package with cargo*
0. **Back**: *return to the main menu*
```

Language:

```text
**RustRbac**(*1.1.0*) │ language

71. **English**: *use English for this menu*
72. **简体中文**: *use 简体中文 for this menu*
73. **繁體中文**: *use 繁體中文 for this menu*
74. **Español**: *use Español for this menu*
75. **العربية**: *use العربية for this menu*
76. **Français**: *use Français for this menu*
77. **Português**: *use Português for this menu*
78. **Русский**: *use Русский for this menu*
79. **Deutsch**: *use Deutsch for this menu*
80. **日本語**: *use 日本語 for this menu*
81. **한국어**: *use 한국어 for this menu*
82. **Nederlands**: *use Nederlands for this menu*
83. **Ελληνικά**: *use Ελληνικά for this menu*
0. **Back**: *return to the main menu*
```

Traditional Chinese front board:

```text
**RustRbac**(*1.1.0*) │ 主選單

1. **rbac**: *users, roles, menus, departments, and posts*
2. **server**: *setup, audit, and the web listener*
3. **系統日誌**: *查看、清空，以及日誌資料夾*
6. **自我管理**: *版本、關於，以及 cargo 生命週期*
7. **語言**: *這個選單的顯示語言*
9. **離開**: *離開*
```

Simplified Chinese front board:

```text
**RustRbac**(*1.1.0*) │ 主菜单

1. **rbac**: *users, roles, menus, departments, and posts*
2. **server**: *setup, audit, and the web listener*
3. **系统日志**: *查看、清空，以及日志文件夹*
6. **自我管理**: *版本、关于，以及 cargo 生命周期*
7. **语言**: *此菜单的显示语言*
9. **离开**: *离开*
```

Japanese front board:

```text
**RustRbac**(*1.1.0*) │ メインメニュー

1. **rbac**: *users, roles, menus, departments, and posts*
2. **server**: *setup, audit, and the web listener*
3. **システムログ**: *表示、消去、ログフォルダ*
6. **自己管理**: *バージョン、概要、cargo のライフサイクル*
7. **言語**: *このメニューの表示言語*
9. **終了**: *終了*
```

## Usage

On a terminal, `rust-rbac` uses the current engine, creates the tables when they are missing, and tries the saved sign-in. It asks for the account name, then the password, only when that sign-in is missing or rejected. A wrong password shows the error and asks again. After a match it opens the menu. Esc on the sign-in question leaves without the menu. Row 9 Exit asks before it logs out and removes the saved sign-in. Setup row 218 asks for the engine choice. `rust-rbac help` lists the verbs. `rust-rbac serve` prints a URL and listens until you interrupt it. Server row 23 starts that site in the background for as long as the process stays open. The web sign-in page has the same two engine choices. A signed-in administrator can also open Database.

`login`, `logout`, and `whoami` are typed verbs. They are not rows on the menu.

`--password` is visible in the process list. The sign-in question and the menu mask the password field.

### Seeded accounts

Change these after the first sign-in.

| Account | Password | What this account can see |
|---------|----------|---------------------------|
| admin | admin123 | Every account and every action. Department Engineering. |
| operator | operator123 | Only the Operations department. List and query. No add, no parameter edit, no force logout. |
| clerk | clerk123 | Only the Platform department. Same common role as operator. |
| lead | lead123 | Engineering and its child department Platform. Not Operations. User list and query only. |

A suggested password for a new account is the parameter `sys.user.initPassword`, seeded as `123456`.

### Passwords and sessions

- Passwords are stored as PBKDF2-HMAC-SHA256 (`pbkdf2_sha256$…`) with 260000 iterations, rather than bcrypt.
- A terminal sign-in can be saved as `local-login` plus `local-login-key` under the application home. The password is not stored as plain text. A web sign-in does not write those files or the terminal `session` file.
- Signed-in sessions are rows in `sys_session`, rather than Redis.
- Data scope uses 1, 2, 3, and 4. Scope 5 is not granted.
- Captcha, export, import, scheduled jobs, code generation, and a Vue application are not in this version.

### Environment

| Name | Default |
|------|---------|
| `RUSTRBAC_DB_ENGINE` | `sqlite3`. The other value is `mysql` |
| `RUSTRBAC_SQLITE_FILE` | `~/.local/RustRbac/rust_rbac.sqlite3` |
| `RUSTRBAC_DB_HOST` | `127.0.0.1` |
| `RUSTRBAC_DB_PORT` | `3306` |
| `RUSTRBAC_DB_NAME` | `rust_rbac` |
| `RUSTRBAC_DB_USER` | `rust_rbac` |
| `RUSTRBAC_DB_PASSWORD` | none |
| `RUSTRBAC_BIND` | `127.0.0.1` |
| `RUSTRBAC_PORT` | `8765` |
| `RUSTRBAC_ALLOW_REMOTE` | unset. Set `yes` before binding a non-loopback address |
| `RUSTRBAC_LANG` | unset. One of the thirteen menu codes. It does not write the language file |
| `RUSTRBAC_HOME` | unset. Overrides `~/.local/RustRbac` |

## Screenshots

These are captures of the running text menu.

![Front board](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/main-menu-en.png)

*Front board after sign-in. Rows are 1 rbac, 2 server, 3 system-log, 6 self-management, 7 language, and 9 Exit. There is no session row. The status line says RustRbac 1.1.0 and main menu.*

![Login prompt](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/login-prompt.png)

*Sign-in. The heading is RustRbac (1.1.0) — sign-in. The question is Password. The field shows one asterisk.*

![Access](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/access-menu.png)

*Rbac. The rows run from user-list through post-add. Back says return to the main menu. The status title is rbac.*

![Server](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/server-menu.png)

*Server. The rows are setup, audit, and serve. Back says return to the main menu. The status title is server.*

![Setup](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/setup-menu.png)

*Setup. The rows run from config-list through db-engine. db-engine says sqlite3 file or mysql. Back says return to the server menu.*

![Audit](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/audit-menu.png)

*Audit. The rows are oper-log, login-log, online-list, and online-logout. Back says return to the server menu.*

![System log](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/system-log.png)

*System log. The rows are view-log, clear-log, and log-folder. The status title is system-log.*

![Language](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/language-menu.png)

*Language. The codes are 71–83. Each short name is that language's own name, including العربية. The status title is language.*

![Self-management](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/self-management.png)

*Self-management. The rows are version, about, version-check, self-update, self-uninstall, and self-install. The explains name cargo.*

![Traditional Chinese](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/main-menu-zh-hant.png)

*Traditional Chinese front board. The path label is 路徑. The status title is 主選單. Category rows include 系統日誌, 自我管理, 語言, and 離開.*

![Simplified Chinese](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/main-menu-zh-hans.png)

*Simplified Chinese front board. The path label is 路径. The status title is 主菜单.*

![Japanese](https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/main-menu-ja.png)

*Japanese front board. The path label is パス. The status title is メインメニュー.*

## Examples

```bash
rust-rbac
rust-rbac db-init
rust-rbac login --user-name admin
rust-rbac user-list
rust-rbac whoami
rust-rbac logout
rust-rbac serve
```

On a terminal, after sign-in, `user-list` is row 11 on the rbac board. Setup is row 21 on the server board.

## Platform Compatibility

Linux is the primary host. The menu needs a terminal. sqlite3 is built into the crate. mysql talks to MariaDB or MySQL with the `mysql` crate. The web pages need a browser on this computer.

## Related Projects

- https://github.com/cloudgen/RustRbac
- https://github.com/cloudgen/ciao
- https://github.com/cloudgen/ciao-lite
- https://github.com/cloudgen/safe-rm

## Contributing

Open an issue or a pull request on https://github.com/cloudgen/RustRbac/issues. Do not send a database password or a session token in a public issue.

## License

MIT. Copyright (c) 2026 Wong Chun Fai, aka: Cloudgen (cloudgen.wong@gmail.com). See [LICENSE.md](LICENSE.md).

## Last Update

2026-10-14
