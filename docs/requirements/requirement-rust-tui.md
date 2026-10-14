**file**: docs/requirements/requirement-rust-tui.md
**Status**: Active (Version 1.1.0)
**Area**: rust
**Key**: `requirement-rust-tui`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The text menu is the front board plus nested boards for RustRbac. It is a full-screen board: the path and a local clock on the first row, the numbered rows under that, and a rounded input box along the bottom. The status line under the box shows the name and the version. Empty arguments on a terminal try the saved sign-in, and ask for an account name and a password only when that sign-in is missing or rejected, then open this menu. Those questions use the same bottom box. The front board is rows **1** rbac, **2** server, **3** system-log, **6** self-management, **7** language, and **9** Exit. It has no session row. Numbers **4**, **5**, and **8** are not rows.

The same domain actions are typed verbs (`requirement-rust-cli-interface`) and web pages (`requirement-web-interface`). A typed verb stays on the command line and does not draw this screen, except `about` on a terminal, which stays the about page. The path word is `requirement-rust-cli-language`. Self-management copy says cargo, not pip.

### 1.1 Human-facing

**In one sentence:** On a terminal, `rust-rbac` tries the saved sign-in, asks for an account and a password in the bottom box only when that is missing or rejected, then opens a full-screen numbered menu that has no session row.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The person at the keyboard | `rust-rbac`, then the account name and the password |
| The other role | The path label | `Path` until a language pick says otherwise |
| Not this file | Permission checks | `requirement-rbac` |

## 2. Front board

| Number | Kind | Role |
|--------|------|------|
| 1 | rbac | users, roles, menus, departments, posts. Children 11–19 and 110–113. Back 0 returns to the front board |
| 2 | server | setup 21, audit 22, serve 23. Back 0 returns to the front board |
| 3 | system-log | view-log 31, clear-log 32, log-folder 33, Back 0 |
| 6 | self-management | 62 version, 63 about, 64 version-check, 65 self-update, 66 self-uninstall, 67 self-install, Back 0 |
| 7 | language | codes 71–83 and Back 0 |
| 9 | Exit | ask first, then leave with status 0. Enter or `y` logs out. `n` keeps the saved sign-in. Esc stays |

Setup is server row 21. Its children are 211–218. Number 218 is `db-engine`. Back 0 returns to the server board. Audit is server row 22. Its children are 221–224. Back 0 returns to the server board. Serve is server row 23. It starts the web listener and shows the URL. It is not its own board. Numbers 4, 5, and 8 are not rows. The word `access` opens the rbac board.

`help` is a word on the open menu. It is not a numbered row. There is no session row.

## 3. Normative rules

1. **Rows.** The front board **MUST** show rows 1 rbac, 2 server, 3 system-log, 6 self-management, 7 language, and 9 Exit. It **MUST NOT** show a session row. Numbers 4, 5, and 8 **MUST NOT** be rows. Row 7 **MUST** be language. Row 9 **MUST** be Exit. Rbac children **MUST** be 11–19 and 110–113. Server **MUST** hold setup 21, audit 22, and serve 23. Setup children **MUST** be 211–218, and 218 **MUST** be `db-engine`. Audit children **MUST** be 221–224. Self-management children **MUST** be 62–67. System-log children stay 31–33. Back on setup and on audit **MUST** return to the server board. Back on rbac, server, system-log, language, and self-management **MUST** return to the front board. Esc on a nested board **MUST** return to the front board. The word `access` **MUST** open the rbac board.
2. **Sign-in first.** Empty arguments on a terminal **MUST** prepare the database with the current engine and **MUST NOT** ask for an engine. They **MUST** try the saved sign-in from `requirement-local-login` before any account question. When that sign-in is missing or rejected, they **MUST** ask for the account and a masked password, one field at a time. A wrong password **MUST** show the error and ask again. A match **MUST** open the front board. The menu **MUST NOT** appear before that match. Esc on the sign-in question **MUST** leave without the menu.
3. **Path label.** The path line **MUST** use the path label from `requirement-rust-cli-language`. `LANG` and `LC_ALL` **MUST NOT** select it. The directory and the clock are not translated.
4. **Display width.** An East Asian Wide or Fullwidth character (Unicode width class W or F) **MUST** count as 2 columns. An ambiguous character **MUST** count as 1. A wide character that does not fit **MUST** be dropped whole.
5. **Password.** A password question **MUST** mask the input. The program **MUST NOT** log the password or argv.
6. **Cargo.** Self-management rows 64–67 and the matching verbs **MUST** say cargo, not pip. Empty arguments **MUST NOT** run them.
7. **Serve.** Server row 23 **MUST** start the listener in the background and print the URL. The typed verb `serve` stays in the foreground.
8. **Exit.** Row 9 **MUST** ask `Log out and remove the saved sign-in?` before it leaves. Enter or `y` **MUST** log out, remove the saved sign-in, and leave with status 0. `n` **MUST** leave and keep the saved sign-in. Esc **MUST** stay on the front board. Any other answer **MUST** stay on the front board. End of input **MUST** leave without logging out.
9. **Full-screen board.** The front board and every nested board **MUST** be drawn by `MenuPainter` in `src/menu.rs` on one screen: the path line, the numbered rows, a three-row rounded box, and a status line. The box **MUST** use `╭` `╮` `╰` `╯` `─` `│`. ASCII `+`, `-`, `|`, and sharp corners **MUST NOT** stand in for them. Typed text **MUST** sit on the only inner line, after `> `. When that line is focused the caret **MUST** be `█`. The status line **MUST** show `RustRbac`, the version, the board title, and `Up/Down  •  Enter`. The product name and the version **MUST NOT** sit on the path line. Up and Down **MUST** walk the rows and the box. Enter **MUST** run the typed token, or the highlighted row when the box is empty. A sign-in question, an exit question, and each field for a chosen row **MUST** use this same box. The menu **MUST NOT** be a `Choice:` line. A result, including the serve URL, **MUST** stay on this screen until the next key returns to the board. While a board is showing, the clock **MUST** be drawn again at most one second later without a thread and without leaving the board. A question **MUST** wait with a blocking read. If the screen cannot hold the path line, one numbered row, the box, and the status line, the program **MUST** fail closed with a non-zero status and the next step `rust-rbac help`. Control-C **MUST** restore the terminal and return 130. It is not row 9.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | Front numbers are 1, 2, 3, 6, 7, and 9. Row 1 is rbac. Row 7 is language. No row is session |
| AC-2 | A missing language file keeps the label `Path` while `LANG` is set |
| AC-3 | East Asian W and F count as 2. Ambiguous counts as 1 |
| AC-4 | Self-management text says cargo, not pip |
| AC-5 | Enter or `y` logs out, `n` keeps the saved sign-in, Esc stays, and end of input does not log out |
| AC-6 | The front board paints `╭` `─` `╮`, `│ > `, `╰` `╯`, the path clock, and `RustRbac` plus the version on the status line, and it does not paint `Choice:` |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Front board | The menu opened after a successful sign-in |
| Path label | The word before the directory. Default `Path` |
| Display width | W and F count as 2. Ambiguous counts as 1 |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | RustRbac menu. Rows 2–9. No session row. Cargo, not pip |
| 2026-10-11 | Active 1.0.1 | Empty argv tries the saved sign-in. Row 9 Exit asks before logout |
| 2026-10-14 | Active 1.0.2 | The boards are the full-screen box from the HolaTui bootstrap. Questions use that box. Typed verbs stay on the command line |
| 2026-10-14 | Active 1.1.0 | Row 1 is rbac. Row 2 server holds setup, audit, and serve. Self-management is 6. Language is 7. Codes are 71–83 |

---

**Last Updated**: 2026-10-14
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
