**file**: docs/requirements/requirement-domain-pyrbac.md
**Status**: Retired (Version 1.2.0)
**Area**: domain
**Key**: `requirement-domain-pyrbac`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-domain-rust-rbac`.

## 1. Purpose

This requirement is the domain surface for PyRbac. The product is online role-based access control and centralized configuration. The same actions are available from the text menu, from typed verbs, and from the local web pages.

PyRbac owns users, roles, menus, departments, posts, dictionaries, parameters, notices, and audit records.

This file is the sole Active `requirement-domain-*`.

### 1.1 Human-facing

**In one sentence:** PyRbac lets an administrator manage accounts and settings from a menu, a command, or a page on this computer, and each surface enforces the same permissions.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `py-rbac login --user-name admin` |
| The other role | A signed-in account limited by its roles | An operator sees only that operator's department |
| Not this file | The rounded menu frame, the SQL tables, and the HTTP headers | TUI, database, and web requirements |

| Includes | Excludes |
|----------|----------|
| Access, setup, audit, and serve as one product, after a sign-in | Code generation, AI chat, plugins, file storage, scheduled jobs |
| One service layer for the menu, the verbs, and the pages | A second permission check that the web page skips |
| English seed labels written for this product | Seed text copied from another product |

| Surface | What you open | What for |
|---------|---------------|----------|
| Text menu | `py-rbac` on a terminal | Sign in, then rows 2, 5, 6, and 7 |
| Typed verb | `py-rbac user-list` | The same list the menu shows |
| Web page | the URL printed by `serve` | The same list, with a cookie |

## 2. Normative rules

1. **Domain sentence.** The about page **MUST** say `Domain: Online RBAC and centralized configuration`.
2. **Shared service.** Class `Desk` **MUST** be the command path for the terminal and the text menu. Class `WebApp` **MUST** call `Access` and `Catalog` for the same facts.
3. **Non-goals.** The product **MUST NOT** ship code generation, an AI feature, plugins, a file manager, scheduled jobs, a Redis cache, a Vue build, or transport encryption of the page body. The front board **MUST NOT** include a session row.
4. **Storage.** Passwords are PBKDF2-HMAC-SHA256, rather than a bcrypt string. Online sessions are rows in `sys_session`, rather than Redis keys. Both facts **MUST** stay visible in the user document.
5. **Sole domain.** `requirement-domain-videojoin` stays Retired.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | About text contains the domain sentence in this file |
| AC-2 | `hello`, `join`, and `list-videos` are not product verbs |
| AC-3 | A menu row and the matching verb call the same Desk method |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Online RBAC | Accounts, roles, menus, departments, and posts checked on each action |
| Centralized configuration | Parameters, dictionary values, and notices in this product's database |
| Surface | The text menu, a typed verb, or the web page |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | PyRbac domain. Welcome-line domain retired |
| 2026-10-11 | Active 1.1.0 | Sign-in before the menu. No session row |
| 2026-10-11 | Active 1.2.0 | This product does not depend on another admin framework |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-domain-rust-rbac` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
