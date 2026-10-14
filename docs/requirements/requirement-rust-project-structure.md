**file**: docs/requirements/requirement-rust-project-structure.md
**Status**: Active (Version 1.0.0)
**Area**: rust
**Key**: `requirement-rust-project-structure`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The ship tree is a Cargo crate. The library is `src/lib.rs`. The binary entry is `src/main.rs`. Rust tests are `tests/*.rs`. Product law that is committed lives in `docs/requirements/`. Public reviews live in `reviews/`. Menu captures live in `screenshots/`.

Python sources remain on disk under `src/PyRbac`. They are not the ship unit.

### 1.1 Human-facing

**In one sentence:** The program you ship is `src/main.rs` and `src/lib.rs`, and the public review notes are under `reviews/`.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A reader of the tree | `src/main.rs` |
| The other role | `Cargo.toml` | Version `1.1.0` |
| Not this file | The module map | `requirement-rust-modules` |

## 2. Normative rules

1. **Crate.** `src/lib.rs` and `src/main.rs` **MUST** be the library and the binary entry. The binary **MUST** be `rust-rbac`.
2. **Tests.** The ship suite **MUST** live in `tests/*.rs` and **MUST** be what `cargo test` runs.
3. **Law.** `docs/requirements/` **MUST** hold the requirement files in the registry. This file does not own their wording.
4. **Reviews.** A review, checklist, or test plan that must be in the public repository **MUST** live under `reviews/`.
5. **Screenshots.** Running-menu captures **MUST** live in `screenshots/` and **MUST** be tracked. The README links are `requirement-rust-readme`.
6. **Python tree.** `src/PyRbac` **MAY** remain on disk. It **MUST NOT** be the ship unit, the crate root, or the console script.
7. **Build script.** `./build.sh` is tracked at the repository root. Its commands are `requirement-rust-build-script`.
8. **Absent trees.** This product has no `sample-videos/` tree and no FFmpeg pipeline in the ship unit.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `src/lib.rs` and `src/main.rs` exist and the binary is `rust-rbac` |
| AC-2 | `docs/requirements/`, `reviews/`, and `screenshots/` are the homes named above |
| AC-3 | `src/PyRbac` is not the ship unit |
| AC-4 | `./build.sh` is at the repository root |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Ship unit | The Cargo crate that builds `rust-rbac` |
| Public review | A Markdown file under `reviews/` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Cargo layout. `src/PyRbac` remains and is not the ship unit |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
