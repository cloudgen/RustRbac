**file**: docs/requirements/requirement-python-project-structure.md
**Status**: Retired (Version 1.1.1)
**Area**: python
**Key**: `requirement-python-project-structure`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-rust-project-structure`.

## 1. Purpose

The installable package is `src/PyRbac`. Tests live in `tests/`. Product law that is committed lives in `docs/requirements/`. Public reviews live in `reviews/` because the rest of `docs/` is gitignored.

### 1.1 Human-facing

**In one sentence:** The code is under `src/PyRbac`, and the public review notes are under `reviews/`.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A reader of the tree | `src/PyRbac/cli.py` |
| The other role | gitignore | `docs/**` ignored except `docs/requirements/**` |
| Not this file | The class list | `requirement-python-oop` |

## 2. Normative rules

1. **Package.** Import name `PyRbac`. There is no `src/HelloTui` and no `src/VideoJoin` in this product.
2. **Tracked docs.** `docs/requirements/**` is tracked. Other `docs/` trees stay local.
3. **Reviews.** A review, checklist, or test plan that must be in the public repository **MUST** live under `reviews/`.
4. **Screenshots.** Running-menu captures live in `screenshots/` and are tracked.
5. **Samples.** This product has no `sample-videos/` tree.
6. **Build script.** `./build.sh` is tracked at the repository root. Its commands are owned by `requirement-python-build-script`.

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Package root | `src/PyRbac` |
| Public review | A Markdown file under `reviews/` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-04 | Active 1.0.0 | Earlier package path |
| 2026-10-11 | Active 1.1.0 | `src/PyRbac` and `reviews/` |
| 2026-10-11 | Active 1.1.1 | `./build.sh` is tracked. Commands live on `requirement-python-build-script` |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-rust-project-structure` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
