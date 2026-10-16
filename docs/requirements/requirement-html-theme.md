**file**: docs/requirements/requirement-html-theme.md
**Status**: Active (Version 1.0.0)
**Area**: domain
**Key**: `requirement-html-theme`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The local pages use one of two skins, `vue3` or `adminlte`. The default is `vue3`. The form method, the form action, and the field names stay the same. The pages have no script. The body size is `requirement-html-body-font`. The setting name is `requirement-environment`.

### 1.1 Human-facing

**In one sentence:** Pick vue3 or adminlte, and the same forms come back in that skin.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `rust-rbac theme` |
| The other role | The settings file | `RUSTRBAC_THEME=adminlte` |
| Not this file | The permission check | `requirement-rbac` |

| Includes | Excludes |
|----------|----------|
| Skins `vue3` and `adminlte` | A third skin |
| Same-origin stylesheets | A script tag, a CDN, or a Vue bundle |
| Server row 24 and the verb `theme` | Opening the database to read the theme |

## 2. Normative rules

1. **Values.** The only themes are `vue3` and `adminlte`. Empty, `1`, and `vue3` select `vue3`. `2` and `adminlte` select `adminlte`. Any other token **MUST** fail. The error **MUST** say `RUSTRBAC_THEME must be vue3 or adminlte.` when the stored or environment value is not one of those two.
2. **Storage.** `RUSTRBAC_THEME` lives in the environment or in the mode-0600 settings file. A non-empty environment value **MUST** win and **MUST NOT** be overwritten by the menu. Enter on the question stores `vue3`. The theme read **MUST NOT** open sqlite or mysql. Each page **MUST** read the file again.
3. **Command.** `theme` is a product verb. It is not a domain verb. It **MUST NOT** open the database. Server row 24 asks the same question. The front-board server explain stays `setup, audit, and the web listener`.
4. **Markup.** `vue3` links `/element-plus.css` and `/rustrbac.css` and uses the `el-form`, `el-input__inner`, `el-button`, `el-radio`, and `el-menu` class names. `adminlte` links only `/adminlte.css` and uses `form-horizontal`, `form-control`, `btn btn-primary`, and `nav nav-sidebar`. The login page is `div.login`. The sign-in button is large. The side column is `aside.sidebar` or `aside.main-sidebar`. It **MUST NOT** sit inside `el-scrollbar`.
5. **Sheets.** `GET /element-plus.css`, `GET /rustrbac.css`, and `GET /adminlte.css` **MUST** answer `text/css; charset=utf-8` without a session. `.main-sidebar` **MUST** set `height: 100vh` and `overflow: auto`.
6. **No script.** A page **MUST NOT** contain a script tag. The content security policy is `requirement-web-interface`.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `theme --theme 2` stores adminlte and does not create the sqlite file |
| AC-2 | `GET /login` after that choice links `/adminlte.css` and has no script tag |
| AC-3 | `GET /adminlte.css` is `text/css` and contains `.main-sidebar`, `height: 100vh`, and `overflow: auto` |
| AC-4 | `RUSTRBAC_THEME=neon` makes `GET /login` status 400 and names the settings error |
| AC-5 | A later write of `RUSTRBAC_THEME=vue3` in the settings file is what the next `GET /login` uses |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Theme | `vue3` or `adminlte` |
| Settings file | The mode-0600 `KEY=VALUE` file from `requirement-environment` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-16 | Active 1.0.0 | vue3 and adminlte. Verb `theme`. Server row 24. No database open. No script |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
