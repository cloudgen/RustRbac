**file**: docs/requirements/requirement-local-login.md
**Status**: Active (Version 1.0.0)
**Area**: domain
**Key**: `requirement-local-login`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

A terminal sign-in keeps a saved account apart from the session token. The saved sign-in is the pair `local-login` and `local-login-key` under the application home. The home folder and the `session` file stay on `requirement-environment`. This file is not a second domain. The sole Active domain remains `requirement-domain-rust-rbac`.

The password is not stored as plain text. Reading `local-login` alone does not show it. Reading both files does. A web sign-in does not write these files.

### 1.1 Human-facing

**In one sentence:** The next `rust-rbac` on a terminal can reopen the menu from the saved sign-in, and the password is not sitting in that file as plain text.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator at the terminal | `rust-rbac` with no arguments |
| The other role | The key file | `local-login-key`, 32 bytes, mode `0600` |
| Not this file | The home folder and the `session` file | `requirement-environment` |

| Includes | Excludes |
|----------|----------|
| Encrypted account name and password for the next terminal start | A second password column in the database |
| Logout and row 9 removing that ciphertext | The web cookie `rust_rbac_session` |

## 2. Files

The home is `$RUSTRBAC_HOME` when that name is set, and `~/.local/RustRbac` otherwise. The folder is mode `0700`. Each file below is mode `0600`. A file that is not mode `0600` is ignored and is not deleted by a read.

| What | Path | What it holds |
|------|------|----------------|
| Saved sign-in | `local-login` and `local-login-key` | Encrypted account name and password |
| Terminal session | `session` | The token for this terminal. Owned by `requirement-environment` |
| Online session | `sys_session` | The same token as a row. The default sqlite file is `rust_rbac.sqlite3` in that home |
| Web session | Cookie `rust_rbac_session` | The browser token. Flags are `HttpOnly`, `SameSite=Lax`, and `Path=/` |

`local-login` is four text lines:

1. `RUSTRBACLOCAL1`
2. Standard base64 of a 16-byte nonce
3. Standard base64 of HMAC-SHA256(key, nonce concatenated with the raw ciphertext bytes)
4. Standard base64 of the ciphertext

`local-login-key` is 32 random bytes used only to read that file. The plaintext is the account name, one `0` byte, then the password. The account name has no `0` byte. The password may contain newlines. The ciphertext is the plaintext XORed with a keystream. Each keystream block is HMAC-SHA256(key, nonce concatenated with the big-endian `u32` block counter), and the counter starts at 0.

A save reuses a mode-`0600` key of 32 bytes. Otherwise it writes a new key before the ciphertext. Writes are atomic and mode `0600`.

## 3. Normative rules

1. **Current engine.** On a terminal with no arguments, `rust-rbac` **MUST** use the current engine and **MUST NOT** ask for an engine. The default is sqlite3 and the default sqlite file. A non-empty `RUSTRBAC_DB_ENGINE` still wins. It **MUST** create missing tables, then try the previous sign-in.
2. **Resume.** A live `session` token **MUST** open the menu with no account question and **MUST NOT** hash the password again. Otherwise the program decodes `local-login` and tries that account. A match **MUST** open the menu. A failed try **MUST** delete `local-login` and **MUST** ask for the account name, then a masked password. A wrong typed password **MUST** show the error and ask again. Esc **MUST** leave without the menu. End of input on the account question **MUST** leave without the menu.
3. **Typed login.** The verb `login` with no account and no password **MUST** use the same store: the token file first, then the ciphertext. A successful password login from the terminal **MUST** save `local-login`. One flag without the other **MUST** still fail as a missing flag.
4. **Logout.** `logout` **MUST** remove `local-login` even when the token file is already gone. A missing token **MUST** still report `Not signed in.` The status line `sign-in cleared` is recorded only when that ciphertext file existed.
5. **Exit.** Row 9 Exit **MUST** ask `Log out and remove the saved sign-in?` first. Enter or `y` logs out, removes the saved sign-in, and leaves with status 0. `n` leaves and keeps the saved sign-in. Esc stays on the front board. Any other answer stays on the front board. End of input leaves without asking and does not log out.
6. **Web.** A web sign-in **MUST** write only the `sys_session` row and the cookie `rust_rbac_session`. It **MUST NOT** write `session`, `local-login`, or `local-login-key`.
7. **Log.** The status log **MUST** record only `sign-in saved`, `sign-in cleared`, or `sign-in rejected`. The line **MUST NOT** include the account or the password. A rejected saved sign-in is a bad check or a ciphertext login that fails. A typed wrong password **MUST NOT** be written to the log. `StatusLog::open` stays in `src/main.rs`.
8. **Ignore.** A `local-login` or `local-login-key` that is missing, not mode `0600`, or a key that is not 32 bytes **MUST** be ignored. That is not a rejected sign-in and **MUST NOT** delete the other file. Bad magic, a bad check, or a failed ciphertext login **MUST** delete `local-login` only.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `local-login` starts with `RUSTRBACLOCAL1` and does not contain the password as plain text. The key file is 32 bytes, mode `0600` |
| AC-2 | Both files together restore the account and the password. A bad check is a rejected sign-in and deletes `local-login` |
| AC-3 | A mode other than `0600` is ignored and is not deleted by a read |
| AC-4 | `logout` removes `local-login` and a second logout with no token reports `Not signed in.` |
| AC-5 | A live session token answers `login` with no password flag. A web sign-in leaves `local-login` absent |
| AC-6 | Enter or `y` is logout, `n` keeps the file, Esc stays, and end of input leaves without logout |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Saved sign-in | The pair `local-login` and `local-login-key` |
| Failed try | Bad magic, a bad check, or a ciphertext login that does not match |
| Live token | A mode-`0600` `session` file whose `sys_session` row is still signed in |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Adopted from the login note. PyRbac names translated. Magic `RUSTRBACLOCAL1`. Home and `session` stay on `requirement-environment` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
