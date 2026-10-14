**file**: docs/requirements/requirement-python-tui.md
**Status**: Retired (Version 1.2.0)
**Area**: python
**Key**: `requirement-python-tui`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-rust-tui`.

## 1. Purpose

The text menu is the front board plus nested boards. The first row is the working directory on the left and the local clock on the right. The bottom of the screen is a rounded input box. The status line under the box names the product, the version, and the board.

Empty arguments on a terminal ask for an account name and a password, then open this menu. They do not run pip. The same domain actions are typed verbs (`requirement-python-cli-interface`) and web pages (`requirement-web-interface`). The front board has no session row. `login`, `logout`, and `whoami` stay typed verbs.

### 1.1 Human-facing

**In one sentence:** On a terminal, `py-rbac` asks for an account and a password, then opens a numbered menu that has no session row.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The person at the keyboard | `py-rbac`, then the account name and the password |
| The other role | The path label | `Path` until a language pick says otherwise |
| Not this file | Permission checks | `requirement-rbac` |

## 2. Front board

| Number | Kind | Role |
|--------|------|------|
| 2 | access | users, roles, menus, departments, posts, numbers 21–33, Back 0 |
| 3 | system-log | view-log 31, clear-log 32, log-folder 33, Back 0 |
| 4 | language | codes 41–53 and Back 0 |
| 5 | setup | parameters, dictionaries, notices, numbers 71–77, Back 0 |
| 6 | audit | oper-log 61, login-log 62, online-list 63, online-logout 64, Back 0 |
| 7 | serve | start the web listener and show the URL |
| 8 | self-management | 82 version, 83 about, 84 version-check, 85 self-update, 86 self-uninstall, 87 self-install, Back 0 |
| 9 | Exit | leave with status 0 and no confirm |

`help` is a word on the open menu. It is not a numbered row. `language` is not an argv verb.

## 3. Normative rules

1. **Path and clock.** The path line **MUST** be the path label, a colon, the absolute working directory, and `HH:MM:SS` when both fit. The label comes from the language requirement. `LANG` **MUST NOT** select it. The directory and the clock are not translated.
2. **Columns.** Wide and Fullwidth characters count as two columns. Ambiguous characters, including the box glyphs and the block caret, count as one. A wide character that does not fit is dropped whole.
3. **Frame.** The input box uses `╭`, `╮`, `╰`, `╯`, `─`, and `│`. The caret is `█`. The hint is `Up/Down  •  Enter`.
4. **Clock layers.** The one-second clock refresh applies on front, access, setup, audit, system-log, language, and self-management. It does not apply on a result page, the sign-in question, or another question.
5. **Password.** A password question **MUST** mask the input with asterisks. Esc on a board question **MUST** cancel that question and stay on the board. Esc on the sign-in question **MUST** leave without opening the menu.
6. **Result.** A string returned by a row **MUST** be a result page. Up and Down scroll it when it overflows. Another key returns to the front board.
7. **Pip.** Rows 84–87 and the matching verbs use `python -m pip`. Empty arguments **MUST NOT** run them. `self-uninstall` on the command line **MUST** require `--force`.
8. **Inherited numbers.** System-log stays 3 and 31–33. Language stays 4 and 41–53. Self-management stays 8 and 82–87. Exit stays 9. Access stays 2. The front board **MUST NOT** include a session row, and number 1 **MUST NOT** be a row.
9. **Sign-in first.** Empty arguments on a terminal **MUST** prepare the database (`requirement-database`), then ask for `user_name` and a masked password, one field at a time. A wrong password **MUST** show the error and ask again. A match **MUST** open the front board. The menu **MUST NOT** appear before that match.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | Missing language file keeps the label `Path` while `LANG` is Traditional Chinese |
| AC-2 | Front numbers are 2 through 9, row 4 is language, and no row is session |
| AC-3 | Wide columns do not share a cell |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Front board | The menu opened after a successful sign-in |
| Path label | The word before the directory. Default `Path`. Traditional Chinese pick uses `路徑` |
| Leaf short | An English token such as `view-log` or `login` |
| Category short | A board name that follows the selected language when a translation exists |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-04 | Active 1.0.0 | Inherited frame, path, clock, and columns |
| 2026-10-11 | Active 1.1.0 | Session, access, setup, audit, and serve join the front board |
| 2026-10-11 | Active 1.2.0 | Sign-in before the menu. The session row is gone. Numbers 2–9 stay |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-rust-tui` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
