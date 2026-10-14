**file**: docs/requirements/requirement-domain-rust-rbac.md
**Status**: Active (Version 1.0.2)
**Area**: domain
**Key**: `requirement-domain-rust-rbac`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This requirement is the domain surface for RustRbac. The product is online role-based access control and centralized configuration. The same actions are available from the text menu, from typed verbs, and from the local web pages.

RustRbac owns users, roles, menus, departments, posts, dictionaries, parameters, notices, and audit records. The ship unit is the Rust binary `rust-rbac`. Python sources under `src/PyRbac` are not the ship unit.

This file is the sole Active `requirement-domain-*`.

### 1.1 Human-facing

**In one sentence:** RustRbac lets an administrator manage accounts and settings from a menu, a command, or a page on this computer, and each surface enforces the same permissions.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `rust-rbac login --user-name admin` |
| The other role | A signed-in account limited by its roles | An operator sees only that operator's department |
| Not this file | The rounded menu frame, the SQL tables, and the HTTP headers | TUI, database, and web requirements |

| Includes | Excludes |
|----------|----------|
| Access, setup, audit, and serve as one product, after a sign-in | Code generation, AI chat, plugins, file storage, scheduled jobs |
| Scope 1 through 4, and no rows for any other scope | A scope 5 that still returns rows |
| One seed when `sys_user` is empty | A second seed that duplicates admin |
| English seed labels written for this product | Seed text copied from another product |

| Surface | What you open | What for |
|---------|---------------|----------|
| Text menu | `rust-rbac` on a terminal | Sign in, then row 1 rbac and row 2 server |
| Typed verb | `rust-rbac user-list` | The same list the menu shows |
| Web page | the URL printed by `serve` | The same list, with cookie `rust_rbac_session` |

## 2. Normative rules

1. **Domain sentence.** The about page **MUST** say `Domain: Online RBAC and centralized configuration`.
2. **Scope.** Data scope **MUST** mean 1 all, 2 custom departments, 3 this department, and 4 this department and children. Any other scope, including 5, **MUST** add no rows.
3. **Admin bypass.** A role whose `role_key` is `admin` **MUST** bypass the permission check and the data scope.
4. **Seed once.** When `sys_user` is empty, init **MUST** insert the rows below and **MUST NOT** insert them again. Departments are Group 100, Engineering 101, Operations 102, and Platform 103.

| Account | Initial password | Department | Role key | Scope |
|---------|------------------|------------|----------|-------|
| admin | admin123 | 101 Engineering | admin | 1 |
| operator | operator123 | 102 Operations | common | 3 |
| clerk | clerk123 | 103 Platform | common | 3, the same common role |
| lead | lead123 | 101 Engineering | lead | 4 |

The seeded notice text **MUST** be `RustRbac is ready.`

5. **Common menus.** `COMMON_MENUS` **MUST NOT** include user add, parameter edit, or force logout.
6. **Refusal.** A missing permission **MUST** start with `Missing permission`.
7. **Protected rows.** User 1 **MUST NOT** be removed or disabled. Role 1 **MUST NOT** be edited through role-grant.
8. **Sign-in sentences.** An unknown account or a wrong password **MUST** be `Unknown account or wrong password.` A signed-out action **MUST** be `Not signed in.`
9. **whoami.** The `whoami` text for an account whose role key is `admin` **MUST** contain `admin=yes`.
10. **Sole domain.** `requirement-domain-pyrbac`, `requirement-domain-videojoin`, and `requirement-video-ffmpeg-pipeline` stay Retired.
11. **Saved sign-in.** The encrypted terminal sign-in is `requirement-local-login`. This file does not define that ciphertext.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | About text contains the domain sentence in this file |
| AC-2 | Scope 5 adds no rows. `role_key` `admin` bypasses permission and scope |
| AC-3 | An empty `sys_user` seeds the four accounts and the four departments once. The notice is `RustRbac is ready.` |
| AC-4 | Common menus omit user add, parameter edit, and force logout |
| AC-5 | Removing or disabling user 1 fails. Role-grant on role 1 fails |
| AC-6 | A bad password is `Unknown account or wrong password.` A signed-out action is `Not signed in.` |
| AC-7 | `whoami` for admin contains `admin=yes` |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Online RBAC | Accounts, roles, menus, departments, and posts checked on each action |
| Centralized configuration | Parameters, dictionary values, and notices in this product's database |
| Data scope | 1 all, 2 custom departments, 3 this department, 4 this department and children |
| Common menus | The menu ids granted to role `common`. They omit user add, parameter edit, and force logout |
| Surface | The text menu, a typed verb, or the web page |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | RustRbac domain. The PyRbac domain file is Retired |
| 2026-10-11 | Active 1.0.1 | Saved terminal sign-in points at `requirement-local-login`. This file stays the sole Active domain |
| 2026-10-14 | Active 1.0.2 | The text menu opens rbac on row 1 and server on row 2 |

---

**Last Updated**: 2026-10-14
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
