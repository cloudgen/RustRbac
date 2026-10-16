**file**: docs/requirements/requirement-web-interface.md
**Status**: Active (Version 1.0.5)
**Area**: domain
**Key**: `requirement-web-interface`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The web interface is the Rust listener in the web module of RustRbac 1.1.1. The binary is `rust-rbac`. It has no JavaScript. Every list and every form calls the same access or catalog path as the text menu. The menus page is `requirement-rbac-menu`. The typed verb `serve` listens in the foreground. Server row 23 starts the listener in the background and prints the URL. The default bind is `127.0.0.1:8765`.

### 1.1 Human-facing

**In one sentence:** Open the printed URL, sign in, and the pages you see are the actions your role allows.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The browser on this computer | `http://127.0.0.1:8765/` |
| The other role | The cookie `rust_rbac_session` | HttpOnly, SameSite=Lax |
| Not this file | The permission strings | `requirement-rbac` |

| Includes | Excludes |
|----------|----------|
| Login, users, roles, menus, departments, posts, parameters, dictionaries, notices, both logs, online users | A Vue bundle or a JSON API clone |
| A hidden CSRF field on POST | A script that fetches the next page |
| Forms only when the account has that permission | Showing an Add button the account cannot use |

## 2. Normative rules

1. **Bind.** The listener **MUST** use `requirement-environment`. The default is `127.0.0.1:8765`. A non-loopback bind **MUST** be refused unless `RUSTRBAC_ALLOW_REMOTE=yes`. That switch is environment-only.
2. **Cookie.** A successful login **MUST** set `rust_rbac_session` with `HttpOnly`, `SameSite=Lax`, and `Path=/`, then redirect to `/`. The login page **MUST NOT** print the seed passwords.
3. **CSRF.** A POST other than login **MUST** include a hidden `csrf` field equal to the session row. A mismatch **MUST** be status 400.
4. **Auth.** A page other than `/login` without a session **MUST** redirect to `/login`. A missing permission **MUST** be status 403. The refusal **MUST** start with `Missing permission`. Another access error **MUST** be status 400.
5. **Headers.** Each HTML response **MUST** send `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, and `Content-Security-Policy: default-src 'none'; style-src 'self' 'unsafe-inline'; form-action 'self'`. The skins and the body size are `requirement-html-theme` and `requirement-html-body-font`. A page **MUST NOT** contain a script tag.
6. **Escape.** Text placed in HTML **MUST** be escaped. The online page **MUST** show an 8-character token prefix on the button and **MUST** send the full token only as a hidden field. The terminal `online-list` **MUST** show the full token so the operator can pass `--token`.
7. **Handler.** The listener **MUST** be Rust code in the web module. It **MUST** call the same access and catalog path as the menu. It **MUST NOT** keep a second permission list. It **MUST NOT** be a Python `ThreadingHTTPServer`.
8. **Nav.** A link **MUST** be omitted when the account lacks the matching list permission.
9. **No terminal files.** A web sign-in **MUST** write only the `sys_session` row and the cookie `rust_rbac_session`. It **MUST NOT** write `session`, `local-login`, or `local-login-key`. Those files are `requirement-local-login`.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | Admin login redirects and the users page includes Add a user |
| AC-2 | A POST with the wrong CSRF token is 400 |
| AC-3 | The operator users page omits Add a user and omits clerk |
| AC-4 | The operator roles page is 403 |
| AC-5 | A users request without a cookie redirects to `/login` |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Session cookie | `rust_rbac_session`, the token stored in `sys_session` |
| CSRF field | The hidden form value that must match the session row |
| Foreground serve | `rust-rbac serve`, which blocks until interrupted |
| Background serve | Server row 23, which lives until the process exits |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Loopback HTML site, cookie, and CSRF |
| 2026-10-11 | Active 1.0.1 | Cookie `rust_rbac_session`. Rust listener. 403 on a missing permission. Product 1.1.0 |
| 2026-10-11 | Active 1.0.2 | A web sign-in does not write `session`, `local-login`, or `local-login-key` |
| 2026-10-14 | Active 1.0.3 | Background serve is server row 23 |
| 2026-10-15 | Active 1.0.4 | The menus page is `requirement-rbac-menu` |
| 2026-10-16 | Active 1.0.5 | Stylesheets from this site. `style-src 'self' 'unsafe-inline'`. No script tag |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
