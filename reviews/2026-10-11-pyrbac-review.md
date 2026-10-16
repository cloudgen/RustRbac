# PyRbac review — 2026-10-11

> Historical. The product is now RustRbac. The current review is `reviews/2026-10-11-rust-rbac-review.md`. This file keeps the PyRbac result. The Python sources and requirements named below were removed from the tree on 2026-10-14.

**Product:** PyRbac 1.0.1
**Scope:** Requirements, RBAC behavior, database isolation, web pages, text menu, and ChronicleLogger.
**Result:** Pass with the limits below.

## What was checked

| Check | Result |
|-------|--------|
| Sole Active domain is `requirement-domain-pyrbac` | Pass |
| `requirement-domain-videojoin` is Retired | Pass |
| Operator data scope 3 sees only `operator` | Pass in `tests/test_rbac.py` |
| Lead data scope 4 sees admin, clerk, and lead | Pass |
| Unknown scope 5 contributes no rows | Pass |
| Operator cannot add a user or edit a parameter | Pass |
| User 1 cannot be removed or disabled. Role 1 is not granted | Pass |
| Web login, CSRF mismatch 400, operator roles 403 | Pass |
| Loopback refusal and mode-0600 password file | Pass |
| ChronicleLogger construct, and a wrong password stays out of the log | Pass |
| Menu rows 2–9, no session row, language 41–53, path label ignores `LANG` | Pass |
| Sign-in question before the menu. A wrong password is asked again | Pass |
| A domain verb and serve initialize a database that has no tables, once | Pass |
| `ruoyi_db` and `ry` are not opened by the tests | Pass. The tests set `PYRBAC_DB_NAME=pyrbac_test` |

## Limits

- Captcha, export, import, and batch logout are not in 1.0.1.
- The front-board screenshots still show the earlier session row. The live menu does not.
- `user-add` and `role-grant` use sequential statements. A crash between them can leave a partial write. The seed itself is one transaction.
- The web grant form can read menu ids for an editor who has role edit without menu list. The seeded administrator has both.
- Screenshots are captures of the running text menu at a fixed size. They are not a browser capture of the HTML pages.

## Storage choices

- Passwords are PBKDF2-HMAC-SHA256, rather than bcrypt.
- Online sessions are `sys_session`, rather than Redis.
- Data scope stops at 4.
