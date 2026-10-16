**file**: docs/requirements/requirement-rust-project-structure.md
**Status**: Active (Version 1.0.2)
**Area**: rust
**Key**: `requirement-rust-project-structure`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The ship tree is a Cargo crate. The library is `src/lib.rs`. The binary entry is `src/main.rs`. Rust tests are `tests/*.rs`. Product law that is committed lives in `docs/requirements/`. Public reviews live in `reviews/`. Menu captures live in `screenshots/`. There is no Python package in this tree.

### 1.1 Human-facing

**In one sentence:** The program you ship is `src/main.rs` and `src/lib.rs`, and the public review notes are under `reviews/`.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A reader of the tree | `src/main.rs` |
| The other role | `Cargo.toml` | Version `1.1.1` |
| Not this file | The module map | `requirement-rust-modules` |

## 2. Normative rules

1. **Crate.** `src/lib.rs` and `src/main.rs` **MUST** be the library and the binary entry. The binary **MUST** be `rust-rbac`.
2. **Tests.** The ship suite **MUST** live in `tests/*.rs` and **MUST** be what `cargo test` runs. **MUST NOT** keep Python `tests/test_*.py`.
3. **Law.** `docs/requirements/` **MUST** hold the requirement files in the registry. This file does not own their wording.
4. **Reviews.** A review, checklist, or test plan that must be in the public repository **MUST** live under `reviews/`.
5. **Screenshots.** Running-menu captures **MUST** live in `screenshots/` and **MUST** be tracked. The README links are `requirement-rust-readme`.
6. **No Python package.** This tree **MUST NOT** contain `pyproject.toml`, `src/PyRbac`, or a Python console script. The ship unit is the Cargo crate. The console script is `rust-rbac`.
7. **Stylesheets.** `src/html/` holds `element-plus.css`, `rustrbac.css`, `adminlte.css`, and the Element Plus license. The web module includes those bytes. They are not a Vue bundle.
8. **Build script.** `./build.sh` is tracked at the repository root. Its commands are `requirement-rust-build-script`.
9. **Absent trees.** This product has no `sample-videos/` tree and no FFmpeg pipeline in the ship unit.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `src/lib.rs` and `src/main.rs` exist and the binary is `rust-rbac` |
| AC-2 | `docs/requirements/`, `reviews/`, and `screenshots/` are the homes named above |
| AC-3 | There is no `pyproject.toml` and no `src/PyRbac`. The ship unit is the Cargo crate |
| AC-4 | `./build.sh` is at the repository root |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Ship unit | The Cargo crate that builds `rust-rbac` |
| Public review | A Markdown file under `reviews/` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Cargo layout. The Python tree was still present and was not the ship unit |
| 2026-10-14 | Active 1.0.1 | The Python package, `pyproject.toml`, and the Python tests are removed |
| 2026-10-16 | Active 1.0.2 | `src/html/` holds the page stylesheets. Version example is 1.1.1 |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
