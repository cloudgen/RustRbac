**file**: docs/requirements/requirement-rust-build-script.md
**Status**: Active (Version 1.0.0)
**Area**: rust
**Key**: `requirement-rust-build-script`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

`./build.sh` is the maintainer build script for the RustRbac crate. It is not `rust-rbac`. It runs Cargo as the normal user. The command set below is closed. The product binary still runs when this script is absent.

Do not `cargo publish`. This product does not claim a crates.io publish.

### 1.1 Human-facing

**In one sentence:** From the repository root, `./build.sh` plus one command fetches, builds, or tests this checkout with Cargo.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The maintainer at the repository root | `./build.sh test` |
| The other role | Cargo | `cargo test`, `cargo build --release` |
| Not this file | The operator menu | `rust-rbac` with no verb |

| Includes | Excludes |
|----------|----------|
| `help`, `version`, `setup`, `clean`, `build`, `test`, `test-install` | `cargo publish`, twine, pip, `sudo` |
| `test-install` as a verb the maintainer types | Running `test-install` from this requirement's own checks |

## 2. Normative rules

1. **Entry.** The script lives at the repository root and is named `build.sh`. It **MUST** `cd` to its own directory before any command. It **MUST NOT** accept `rust-rbac` flags.
2. **Closed set.** The only commands are `help`, `version`, `setup`, `clean`, `build`, `test`, and `test-install`. No command, `-h`, and `--help` are `help`. An unknown token prints `ERROR: {message}` and `   Next: {next}` on stderr and exits 1.
3. **Cargo, not pip.** The commands **MUST** be the Cargo lines in the table. The script **MUST NOT** call `sudo`, `python3 -m pip`, or `cargo publish`.

| Command | Behavior |
|---------|----------|
| `help` | Print the closed set. Exit 0. |
| `version` | Print the `Cargo.toml` version `1.1.0`. |
| `setup` | `cargo fetch` |
| `clean` | Remove the Cargo `target/` directory. Do not remove `src/`, `reviews/`, or `screenshots/`. |
| `build` | `cargo build --release` |
| `test` | `cargo test` |
| `test-install` | `cargo install --path . --force` |

4. **Checks.** Acceptance checks for this requirement **MUST NOT** run `test-install` and **MUST NOT** run `cargo install`.
5. **Errors.** A failed command **MUST** print `ERROR: {message}` and `   Next: {next}` on stderr and exit 1. The next step names `./build.sh` and a command that repairs the failure.

## Under command line for normal user only

On Termux, Git Bash, Windows cmd, or the same class, the maintainer runs `./build.sh` as the normal user. **This requirement:** no sudo. Use `cargo build` and `cargo test`. Do not `cargo publish`. The verbs are `help`, `version`, `setup` (`cargo fetch`), `clean`, `build` (`cargo build --release`), `test` (`cargo test`), and `test-install` (`cargo install --path . --force`). `test-install` is not run by this requirement's own checks. Git Bash and Windows cmd do not call Termux `pkg`. Do not pipe a downloaded script into a shell.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `help` lists only the seven commands in section 2 |
| AC-2 | `setup` is `cargo fetch`, `build` is `cargo build --release`, and `test` is `cargo test` |
| AC-3 | `test-install` is `cargo install --path . --force` |
| AC-4 | The script does not call `sudo` or `cargo publish` |
| AC-5 | This requirement's checks do not run `test-install` |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Maintainer script | `./build.sh`, not the `rust-rbac` binary |
| test-install | `cargo install --path . --force`, not executed by this file's checks |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | Cargo verbs. No publish. `test-install` is not run by this file's checks |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
