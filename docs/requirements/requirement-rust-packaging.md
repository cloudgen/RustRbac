**file**: docs/requirements/requirement-rust-packaging.md
**Status**: Active (Version 1.0.0)
**Area**: rust
**Key**: `requirement-rust-packaging`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

`Cargo.toml` is the package manifest and the version source. The crate name is `rust_rbac`, the version is `1.1.0`, the edition is 2021, the license is MIT, and the binary is `rust-rbac`. This product does not claim a crates.io publish.

Maintainer commands of `./build.sh` are `requirement-rust-build-script`.

### 1.1 Human-facing

**In one sentence:** Cargo builds RustRbac 1.1.0 as the binary `rust-rbac`. It is not a published crates.io crate in this revision.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The maintainer | `cargo build --release` |
| The other role | `Cargo.toml` | version `1.1.0`, bin `rust-rbac` |
| Not this file | MariaDB itself | `requirement-runtime-prerequisites` |

## 2. Normative rules

1. **Manifest.** `Cargo.toml` **MUST** set `version` to `1.1.0`, `edition` to `2021`, and `license` to `MIT`. The package name **MUST** be `rust_rbac`.
2. **Binary.** `[[bin]]` **MUST** name `rust-rbac` and **MUST** point at `src/main.rs`. There is no Python console script in the ship unit.
3. **Author.** The authors entry **MUST** name Wong Chun Fai, `cloudgen.wong@gmail.com`. The homepage **MUST** be `https://github.com/cloudgen/RustRbac`.
4. **No publish claim.** Docs and the about page **MUST NOT** say this crate has been published to crates.io. `cargo publish` is not a product step.
5. **Dependencies.** sqlite may be the bundled `rusqlite` feature. mysql access uses the `mysql` crate. The manifest **MUST NOT** require ChronicleLogger, PyMySQL, or FFmpeg.
6. **Secrets.** The manifest **MUST NOT** contain a database password.
7. **Self-management.** Copy that tells the operator how to install or update this checkout **MUST** say cargo, not pip.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `Cargo.toml` version is `1.1.0`, edition `2021`, license `MIT` |
| AC-2 | The binary name is `rust-rbac` and the path is `src/main.rs` |
| AC-3 | No requirement and no about line claims a crates.io publish |
| AC-4 | The manifest has no database password |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Version source | `Cargo.toml` `version`, currently `1.1.0` |
| Binary | `rust-rbac`, built from `src/main.rs` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Cargo.toml 1.1.0, edition 2021, bin `rust-rbac`, MIT. No crates.io claim |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
