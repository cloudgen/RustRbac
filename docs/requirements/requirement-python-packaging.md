**file**: docs/requirements/requirement-python-packaging.md
**Status**: Retired (Version 1.3.2)
**Area**: python
**Key**: `requirement-python-packaging`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-rust-packaging`.

## 1. Purpose

`pyproject.toml` is the install manifest. The package name is `PyRbac`, the version is `1.0.1`, and the console script is `py-rbac`.

### 1.1 Human-facing

**In one sentence:** pip installs PyRbac 1.0.1, the command `py-rbac`, ChronicleLogger, and PyMySQL.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The operator | `python -m pip install -e .` |
| The other role | The manifest | `pyproject.toml` |
| Not this file | MariaDB itself | `requirement-runtime-prerequisites` |

## 2. Normative rules

1. **Name and version.** `name` is `PyRbac`. `version` is `1.0.1`, the same string as `__version__`.
2. **Script.** `py-rbac = PyRbac.cli:main`.
3. **Dependencies.** `ChronicleLogger>=1.3.1` and `PyMySQL>=1.1.0` are required. The package **MUST NOT** re-export ChronicleLogger.
4. **Author.** Wong Chun Fai, `cloudgen.wong@gmail.com`. Homepage `https://github.com/cloudgen/PyRbac`.
5. **Python.** `requires-python` is `>=3.9`.
6. **Secrets.** The manifest **MUST NOT** contain a database password.
7. **Maintainer tool.** Commands of `./build.sh` are owned by `requirement-python-build-script`. This file owns the install manifest.

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Console script | `py-rbac` |
| Floor | The minimum version of a required dependency |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-04 | Active 1.2.2 | Earlier package identity |
| 2026-10-11 | Active 1.3.0 | PyRbac 1.0.0, `py-rbac`, PyMySQL |
| 2026-10-11 | Active 1.3.1 | Maintainer commands point at `requirement-python-build-script` |
| 2026-10-11 | Active 1.3.2 | Product version 1.0.1 |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-rust-packaging` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
