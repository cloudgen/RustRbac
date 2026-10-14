**file**: docs/requirements/requirement-python-oop.md
**Status**: Retired (Version 1.1.0)
**Area**: python
**Key**: `requirement-python-oop`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-rust-modules`.

## 1. Purpose

Each module defines one class. An error type may sit beside the class that raises it. `def main` in `cli.py` is the function that constructs `ChronicleLogger`. Collaborators are created in `__init__`. There is no module-level helper that does product work.

### 1.1 Human-facing

**In one sentence:** Each behavior lives on one class, and `main` builds the logger before it builds the program.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A reader of `src/PyRbac` | `Desk` calls `Access` |
| The other role | The logger | Passed into `__init__` |
| Not this file | The permission table | `requirement-rbac` |

## 2. Class map

| Module | Class |
|--------|-------|
| `cli.py` | `Cli`, and `def main` |
| `tui.py` | `Tui` and `MenuScreenError` |
| `menu_painter.py` | `MenuPainter` |
| `menu_model.py` | `MenuModel` |
| `menu_session.py` | `MenuSession` |
| `menu_language.py` | `MenuLanguage` |
| `about_page.py` | `AboutPage` |
| `check_system.py` | `CheckSystem` |
| `self_management.py` | `SelfManage` |
| `system_log.py` | `SystemLog` |
| `desk.py` | `Desk` |
| `settings.py` | `Settings` and `SettingsError` |
| `store.py` | `Store` and `StoreError` |
| `schema.py` | `Schema` |
| `password_hash.py` | `PasswordHash` |
| `access.py` | `Access` and `AccessError` |
| `catalog.py` | `Catalog` |
| `audit_log.py` | `AuditLog` |
| `report_text.py` | `ReportText` |
| `session_file.py` | `SessionFile` |
| `web_app.py` | `WebApp` |

## 3. Normative rules

1. **Logger.** A class that accepts `logger` **MUST** log `instantiated` from `__init__` when the logger is present.
2. **Nested handler.** `WebApp.make_server` **MAY** define the standard-library request handler as a nested class because `ThreadingHTTPServer` constructs it. That handler **MUST** call `WebApp`. No other module **MAY** add a second product class.
3. **Version.** `__version__` in `__init__.py` is the only version string. It is `1.0.1`.
4. **No hello.** There is no `Tui.hello_text`.

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Class map | The table in section 2 |
| Nested handler | The one HTTP class inside `WebApp.make_server` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-04 | Active 1.0.0 | Menu and about classes |
| 2026-10-11 | Active 1.1.0 | Desk, RBAC, and web classes |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-rust-modules` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
