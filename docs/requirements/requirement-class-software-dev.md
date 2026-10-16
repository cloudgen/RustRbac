**file**: docs/requirements/requirement-class-software-dev.md
**Status**: Active (Version 1.1.8 – RustRbac software-development class law + residual stack)
**Area**: class
**Key**: `requirement-class-software-dev`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Declare this workspace as a software-development project class and hold the residual collection of software-engineering stack facts not already owned by more specific Active peer requirements: primary language, toolchain policy, package and build tooling, and runtime OS family.

This file is class law plus the residual SSOT. It is not a second copy of the domain surface, the CLI, the text menu, the logger construct, the module map, packaging tables, or error-handling tables. Those stay on peer requirements. FFmpeg concat stays retired.

### 1.1 Human-facing

**In one sentence:** This file records that RustRbac is Rust, built with Cargo, and it points at the requirement that owns each other topic.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | Maintainer checking which file owns a topic | The menu picture is the TUI requirement. The language row stays here |
| The other role | Peer requirements | CLI, TUI, logging, domain, packaging |
| Not this file | The frame glyphs and the cargo command strings | Their own requirements. This file only points |

| Includes | Excludes |
|----------|----------|
| Language Rust, Cargo, and the residual pointer table | A second copy of the menu, the logger construct, or the module map |
| Shell install and root elevation marked absent | A cargo lifecycle marked absent. That lifecycle is owned by the CLI and the TUI |
| One human operator. No dest approver | A new actor-requirement file |

| Surface | What you open | What for |
|---------|---------------|----------|
| This file | Residual table | Which requirement owns the topic |
| `Cargo.toml` | Package tool facts | The concrete manifest. Packaging owns the tables. Version **1.1.1** |
| `rust-rbac` | The program | You run it as the normal user |

| You do… | What it means | What you type |
|---------|---------------|---------------|
| Find the menu law | Open the TUI requirement. Do not expect the picture in this file. | `rust-rbac` |
| Find the logger law | Open the logging requirement. `StatusLog::open` is constructed in `src/main.rs`. | `rust-rbac version` |
| Build | Use Cargo. There is no shell installer and no crates.io publish claim. | `cargo build --release` |

## 2. Core Rules (Mandatory)

### 2.0 Project class membership

1. **MUST** treat this workspace as software-development (shippable software), not genesis-template and not server-maintenance.
2. **MUST** use basename `requirement-class-software-dev.md` as the sole Active class-law file for this class.
3. **MUST NOT** register an Active `requirement-class-server-maintenance.md` while the class is software-development.
4. **MUST** retain portable harness knowledge when present. Specialized product knowledge lives in this and peer `requirement-*.md` files.
5. **MUST** apply the software-development SSOT and gate posture when claimed (identity, package ship surface, and a pre-commit check when git is used — as applicable).
6. **MUST NOT** invent hollow product docs solely to look specialized.

### 2.1 Residual collection principle (SSOT hygiene)

7. **MUST** treat this file as the default home for software-stack facts not owned by another Active requirement.
8. **MUST NOT** duplicate full normative tables that already live in a more specific Active requirement. Prefer a one-line pointer to the peer requirement key.
9. When a new specialized requirement takes ownership of a topic previously only listed here, **MUST** update this file in the same change: shrink the residual entry and point to the new owner.
10. **MUST NOT** leave contradictory stack facts across this file and peer requirements.

### 2.2 Programming language(s)

11. **MUST** declare at least one primary programming language for the ship unit.
12. **SHOULD** list secondary languages only when they are real product law.
13. **MUST** state whether the product is primarily interpreted, compiled, polyglot, or package-multi-language.
14. **MUST NOT** freeze a marketing product name as if it were the language name.

### 2.3 Compilers, interpreters, and toolchains

15. **MUST** declare the target toolchain class used to build or run the product.
16. **MUST** state version policy as one of: unconstrained, minimum version, range, or pinned.
17. **SHOULD** record whether cross-compilation is in scope.
18. **MUST** fail closed in docs claims: do not claim “supports all interpreters” without tests or an explicit unconstrained policy.

### 2.4 Project / package / build tools

19. **MUST** declare the primary project or package tool used for dependencies and builds.
20. **MUST** declare how dependencies are resolved when the ecosystem supports lockfiles.
21. **SHOULD** name the test runner and linter or formatter classes when they are project law.
22. **MUST NOT** require a secret token or a private registry password in this file.

### 2.5 Runtime and platform (residual)

23. **MUST** declare the intended primary runtime and OS family when not fully owned by another architecture requirement.
24. **SHOULD** declare minimum CPU or architecture support only when it is real product law.
25. **MUST** separate developer-machine toolchain requirements from end-user runtime requirements when they differ.

### 2.6 No-hardcode / dual policy (class file)

26. **MUST NOT** hard-code a single product brand, one organization’s production hostname, or a personal owner identity as universal core law.
27. **MUST** put the live product name, the repo slug, and the concrete stack choices in Implementation Notes.
28. **MUST NOT** store secrets, PATs, or toy credentials in this file.

### 2.7 Implementation Notes (this project)

| Field | Value (RustRbac) |
|-------|---------------------|
| **Project display name** | RustRbac |
| **Project class** | software-development |
| **Class requirement basename** | `requirement-class-software-dev.md` |
| **Primary language(s)** | Rust |
| **Language role** | Rust is the ship language. This tree has no Python package |
| **Execution model** | compiled binary |
| **Toolchain** | `rustc` and Cargo. Edition 2021 |
| **Toolchain version policy** | edition 2021 as declared in `Cargo.toml`. Do not advertise an untested rustc minor |
| **Cross-compile in scope?** | no |
| **Primary project/package tool** | Cargo. `Cargo.toml` is the version source |
| **Lockfile policy** | `Cargo.lock` is a local build input. This product does not publish to crates.io |
| **Test runner** | `cargo test` |
| **Linter/formatter** | none as project law |
| **Primary runtime / OS family** | the `rust-rbac` binary (documented focus: Linux; other hosts when the Rust binary runs) |
| **Architectures supported** | architectures where this Rust binary is built |
| **Git surface** | used — remote `https://github.com/cloudgen/RustRbac` |
| **Ship surface** | Cargo binary `rust-rbac`. Not a crates.io publish |
| **Product version SSOT** | `Cargo.toml` `version` (current: **1.1.1**) |
| **Install mode** | Cargo on this checkout. Not a shell online-install product. Self-management says cargo, not pip |
| **Type 1 elevation** | intentionally absent. No root or sudo product surface |
| **Actor / role / subject / approver** | **Considered — no dest approver.** The human operator of `rust-rbac` is the only actor. No dest approval machine. **None** is valid. Do not add an actor requirement file |
| **Dest fence conditions** | **Considered — no dest fence conditions.** No dest inbound, approve, or reject queue |
| **Author contact (non-secret)** | Wong Chun Fai · `cloudgen.wong@gmail.com` |

**Residual ownership table:**

| Topic | Owner | Notes |
|-------|-------|--------|
| Project class membership | **this file** | Fixed |
| Primary language + toolchain policy | **this file** | Rust, `rustc`, and Cargo. Not Python as the ship language |
| Package/build tool + version | **this file** + `requirement-rust-packaging` | `Cargo.toml` version **1.1.1**. No crates.io publish claim |
| Maintainer `./build.sh` commands | `requirement-rust-build-script` | Cargo verbs. The CLI does not require this script to run |
| Project layout | `requirement-rust-project-structure` | `src/lib.rs`, `src/main.rs`, `tests/*.rs` |
| Modules | `requirement-rust-modules` | One concern per `src/*.rs` file |
| CLI entry / typed verbs | `requirement-rust-cli-interface` | Empty argv and the `PRODUCT_VERBS` summary |
| Text menu | `requirement-rust-tui` | Front rows 1 rbac, 2 server, 3 system-log, 6 self-management, 7 language, 9 Exit. No session row |
| Menu language | `requirement-rust-cli-language` | Front row 7, codes 71–83, leaf `~/.local/RustRbac/language` |
| Status logger | `requirement-rust-logging` | `StatusLog::open` in `src/main.rs`. Not ChronicleLogger |
| Saved terminal sign-in | `requirement-local-login` | Files `local-login` and `local-login-key`. Not the web cookie |
| Domain surface | `requirement-domain-rust-rbac` | The sole Active domain. Online RBAC and centralized configuration |
| About page | `requirement-rust-about` | Identity, host check, and the star box. No `--json` |
| FFmpeg concat / re-encode | `requirement-video-ffmpeg-pipeline` | Retired. Not a prerequisite |
| Coding style | `requirement-rust-coding-style` | Rust 2021, `Result`, `AppError`. No `unwrap` on operator paths |
| Error / fail-closed user messaging | `requirement-rust-error-handling` | stderr `ERROR:` and `   Next:` |
| Host runtime deps | `requirement-runtime-prerequisites` | `rustc`, `cargo`, sqlite, mysql client for mysql. No PyMySQL. No sudo. No FFmpeg |
| Shell online install / shell self-update / Type O | **intentionally absent** | Not a shell channel |
| Cargo lifecycle (`version-check`, `self-update`, `self-install`, `self-uninstall`) | `requirement-rust-cli-interface` and `requirement-rust-tui` | cargo, not pip. Not a shell channel. Do not `cargo publish` |
| Type 1 sudoers / root elevation | **intentionally absent** | No elevation law |
| Python package | **absent** | `pyproject.toml`, `src/PyRbac`, the Python tests, and `requirement-python-*` are removed. Database names `pyrbac` and `pyrbac_test` stay refused |
| Actor / role / subject / approver | **this file (residual)** | Considered — no dest approver |
| Dest fence conditions | **this file (residual)** | Considered — no dest fence conditions |

## 3. Why This Requirement Exists (Direct CIAO Alignment)

- **CIAO Principle 2 – Intentional**: Class and stack choices are explicit.
- **CIAO Principle 5 – SSOT**: Residual facts stay here until a peer takes the topic. The menu, the logger, and the class map have already moved.
- **CIAO Principle 1 – Caution**: This file does not invent a shell installer.
- **CIAO Principle 21 – Dual Policies**: Pointers stay short. Concrete values sit in Implementation Notes.
- **CIAO Principle 4 (O) + Principle 20**: One class file. No second stack SSOT.

## 4. Design Principles (CIAO / CIAO-Lite)

- **Caution**: FFmpeg is not a prerequisite. This file does not install it.
- **Intentional**: The residual table points. It does not copy peer bodies.
- **Anti-fragile**: Packaging identity stays in `Cargo.toml`.
- **Over-protect**: Shell Type O stays absent while cargo lifecycle is claimed on the CLI and the TUI. Do not `cargo publish`.

## 5. Protection Rule (Sacred)

**Future AI assistants, Grok, or maintainers MUST NOT**:

1. Delete this file while the workspace remains software-development with other Active product requirements.
2. Rename the specialized basename away from `requirement-class-software-dev.md` without an explicit class-model change.
3. Hard-code secrets, personal tokens, or production host names into core rules as universal law.
4. Duplicate full peer requirement bodies into this residual section.
5. Leave Implementation Notes hollow while Status is Active.
6. Introduce a shell online-install channel, a `curl|sh` installer, or shell Type O empty-argv install-ensure. Cargo lifecycle owned by the CLI and TUI requirements is not that channel. Do not `cargo publish`.
7. Introduce Type 1 sudoers or root elevation without an explicit user order and an elevation allowlist.
8. Treat this file as server-maintenance allowlist law, or register an Active server-maintenance class file in parallel.
9. Invent a second primary language that contradicts the Rust requirements, or call Python the ship language.
10. Add an actor-requirement file while the residual says no dest approver.
11. Mark cargo lifecycle absent, mark shell Type O present, or claim a crates.io publish.

**Violating any of these is a critical regression.**

## 6. Acceptance criteria

| ID | Criterion |
|----|-----------|
| AC-1 | The sole Active class file is `requirement-class-software-dev.md` |
| AC-2 | Primary ship language is Rust. The toolchain is `rustc` and Cargo |
| AC-3 | The package tool is Cargo. `Cargo.toml` is the version source, **1.1.1** |
| AC-4 | The residual table points at the Rust CLI, TUI, logging, modules, and domain peers without copying their bodies |
| AC-5 | Shell Type O and Type 1 elevation are absent. Cargo lifecycle is owned by the CLI and the TUI. No sudo |
| AC-6 | Registered in `docs/requirements/index.md` with Area `class` |
| AC-7 | Actor residual stays “no dest approver”. No actor requirement file |

## 7. Related requirements (peer keys only)

| Key | Relationship |
|-----|----------------|
| `requirement-rust-packaging` | Manifest, binary `rust-rbac`, version 1.1.1 |
| `requirement-rust-build-script` | Maintainer `./build.sh` commands |
| `requirement-rust-project-structure` | Layout |
| `requirement-rust-modules` | One concern per file |
| `requirement-rust-cli-interface` | Typed verbs and empty argv |
| `requirement-rust-tui` | Text menu |
| `requirement-rust-logging` | `StatusLog::open` in `src/main.rs` |
| `requirement-local-login` | Saved terminal sign-in. Not the web cookie |
| `requirement-domain-rust-rbac` | Sole Active domain |
| `requirement-video-ffmpeg-pipeline` | Retired. Not a prerequisite |
| `requirement-rust-coding-style` | Rust 2021 and `AppError` |
| `requirement-rust-error-handling` | Console errors |
| `requirement-runtime-prerequisites` | `rustc`, `cargo`, sqlite, mysql client |
| `docs/requirements/index.md` | Registry |

## Design-time verification

| TP family / ID | Suite | Status | Note |
|----------------|-------|--------|------|
| n/a | packaging and structure smoke | deferred | Class residual is exercised by TP-PKG, TP-STRUCT, TP-LOG, TP-TUI, and TP-OOP when those rows leave todo |

**Matrix:** `docs/reviews/requirement-test-matrix.md`
**Map:** `docs/reviews/test-plan.md`

## Under command line for normal user only

On Termux, Git Bash, Windows cmd, or the same class, RustRbac runs as the normal user. **This requirement:** do not use administrator privilege, `sudo`, `apt`, or a dedicated system user. Do not pipe a downloaded script into a shell. Type 1 and Type 2 are unused on Termux, Git Bash, and Windows cmd. Git Bash and Windows cmd do not call Termux `pkg`. Cargo lifecycle, when the operator asks for it, says cargo, not pip, and does not `cargo publish`.

## 8. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-08-09 | Active 1.0.0 | Initial software-dev class law for PyRbac |
| 2026-10-04 | Active 1.1.0 | Residual pointers for the text menu, the logger, the class map, and pip lifecycle. Shell Type O stays absent |
| 2026-10-04 | Active 1.1.1 | Menu language points at `requirement-python-cli-language`. No new actor file |
| 2026-10-04 | Active 1.1.2 | Residual row for the about page. Product version **1.0.5** |
| 2026-10-11 | Active 1.1.3 | Maintainer `./build.sh` points at `requirement-python-build-script` |
| 2026-10-11 | Active 1.1.4 | Ship language is Rust. Cargo. Binary `rust-rbac`. Version 1.1.0. No sudo |
| 2026-10-11 | Active 1.1.5 | Residual row for the saved terminal sign-in. The body stays on `requirement-local-login` |
| 2026-10-14 | Active 1.1.6 | The menu-language pointer names front row 7 |
| 2026-10-14 | Active 1.1.7 | The retired Python package and its requirements are removed |
| 2026-10-16 | Active 1.1.8 | Product version **1.1.1**. Page skins stay on `requirement-html-theme` |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
