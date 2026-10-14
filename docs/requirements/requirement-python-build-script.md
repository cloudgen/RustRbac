**file**: docs/requirements/requirement-python-build-script.md
**Status**: Retired (Version 1.0.0)
**Area**: python
**Key**: `requirement-python-build-script`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-rust-build-script`.

## 1. Purpose

`./build.sh` is the maintainer build script for this Python package. It is not `py-rbac`. It reads the package version through `python3`, and it runs pip, the `build` package, and twine through `python3`. The command set below is closed. The product CLI still runs when this script is absent.

### 1.1 Human-facing

**In one sentence:** From the repository root, `./build.sh` plus one command packs, installs, tests, or publishes this checkout.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The maintainer at the repository root | `./build.sh test-install` |
| The other role | This checkout's `python3` | `python3 -m pip`, `python3 -m build`, `python3 -m twine` |
| Not this file | The operator menu | `py-rbac` with no verb |

| Includes | Excludes |
|----------|----------|
| `help`, `version`, `setup`, `clean`, `build`, `upload`, `git`, `tag`, `release`, `all`, `test`, `test-install` | `py-rbac` verbs, the text menu, `db-init`, a database password, `sudo` |

| Surface | What you open | What for |
|---------|---------------|----------|
| `./build.sh` | Repository root | One maintainer command |
| `tests/run.sh` | Called by `./build.sh test` | The product suite |
| `dist/` | Created by `build` | The sdist and the wheel |

| You do… | What it means | What you type |
|---------|---------------|---------------|
| See the list | No command, `help`, `-h`, and `--help` print the same list, then `Done.` | `./build.sh` |
| Install this checkout | Remove an existing `PyRbac` install for this `python3`, then install this directory. Later edits under `src/` stay out of that copy until you run it again. | `./build.sh test-install` |
| Pack | Write an sdist and a wheel into `dist/`. This does not upload and does not tag. | `./build.sh build` |
| Send the cartons | Upload whatever is already in `dist/`. This does not build first. | `./build.sh upload` |
| Release | Clean, then build, then upload, then tag. `all` is the same chain. | `./build.sh release` |

## 2. Core Rules / Requirements (Mandatory)

1. **Entry.** The script lives at the repository root and is named `build.sh`. It **MUST** `cd` to its own directory before any command. It **MUST NOT** accept `py-rbac` flags.
2. **Banner.** Before the command, print `PyRbac build tool (v` plus `__version__` plus `)` when `src/PyRbac/__init__.py` imports. When that read fails, print `PyRbac build tool (version unread)`. Then print a line of forty `=` characters.
3. **Closed set.** The only commands are `help`, `version`, `setup`, `clean`, `build`, `upload`, `git`, `tag`, `test`, `test-install`, `release`, and `all`. No command, `-h`, and `--help` are `help`. An unknown token prints `ERROR: Unknown command: ` plus the token on stderr, prints the help list, and exits `1` without printing `Done.`
4. **Extra tokens.** `test` and `test-install` reject any extra token and exit `1`. Every other command ignores trailing tokens.
5. **Done.** A command that exits `0` prints `Done.` as its last line. A command that exits non-zero does not print `Done.`
6. **Errors.** Failures print `ERROR: …` and `   Next: …` on stderr. The next-step line names `./build.sh` and the command that repairs the failure.
7. **Interpreter.** Package version, project name, pip, `build`, and twine run as `python3` on `PATH`. The script **MUST NOT** create a virtual environment and **MUST NOT** call `sudo`.

### 2.1 Commands

| Command | Behavior |
|---------|----------|
| `help` | Print the usage list in §2.2. Exit `0`. |
| `version` | If `__version__` was unread, exit `1` with `ERROR: Cannot read the package version from src/PyRbac.` and `   Next: set __version__ in src/PyRbac/__init__.py to match pyproject.toml, then ./build.sh version`. Otherwise print `PyRbac build tool (v` plus that version plus `)` again. |
| `setup` | `python3 -m pip install --upgrade build twine`. |
| `clean` | Remove `build/`, `dist/`, `.eggs/`, `.pytest_cache/`, `*.egg-info` in the root and under `src/`, every `__pycache__` directory, and every file named `._*`. Print `Clean complete`. Do not remove the source tree, `reviews/`, or `screenshots/`. |
| `build` | `python3 -m build --sdist --wheel --outdir dist/`. On failure exit `1` with `ERROR: Package build failed.` and `   Next: ./build.sh setup`. On success print `Build complete -> dist/` and a long listing of `dist/`. Do not clean first. Do not upload. Do not tag. |
| `upload` | `python3 -m twine upload dist/*`. Do not build first. On failure exit `1` with `ERROR: Upload failed.` and `   Next: ./build.sh build`. On success print `Upload finished for PyRbac v` plus the version string read at startup (empty when that read failed). Do not print index credentials. |
| `git` | When stdin is a terminal, print `Enter commit message:` and read one line. Otherwise read one line from stdin. An empty line exits `1` with `ERROR: Commit message is empty.` and `   Next: ./build.sh git`. Otherwise run `git add .`, `git commit -m` with that one line, then `git push` with no flags. The push uses the current branch's configured remote. Do not pass `--force`. |
| `tag` | Require a readable version, as `version` does. The tag name is `v` plus that version. Print `Creating and pushing tag: ` plus the name. Create an annotated tag with message `Release ` plus the tag name, then `git push origin` plus that tag. |
| `test` | No extra arguments. If `tests/run.sh` is missing, exit `1` with `ERROR: tests/run.sh is not in this checkout.` and `   Next: add the suite runner, then ./build.sh test`. Otherwise run `./tests/run.sh`. |
| `test-install` | No extra arguments. Read `[project].name` from `pyproject.toml` with the stdlib `tomllib` module. If the name cannot be read, exit `1` with `ERROR: Cannot read the project name from pyproject.toml.` and `   Next: set [project].name, then ./build.sh test-install`. When `python3 -m pip show` finds that name, print `Removing previous install of ` plus the name and run `python3 -m pip uninstall -y` for it. Uninstall failure exits `1` with `ERROR: Could not remove the installed copy.` and `   Next: ./build.sh test-install`. When pip does not show the name, print `No previous install of ` plus the name. Then print `Installing this checkout` and run `python3 -m pip install` on this repository directory. That install is not editable. Failure exits `1` with `ERROR: Local install failed.` and `   Next: ./build.sh test-install`. |
| `release` | Require a readable version, then run `clean`, `build`, `upload`, and `tag` in that order. Stop at the first failure. |
| `all` | Same chain as `release`. |

### 2.2 Help list

The help text **MUST** name the usage line `Usage: ./build.sh <command>` and **MUST** keep `test` under the heading `Test command:`. The operational lines are `help`, `version`, `setup`, `clean`, `build`, `upload`, `git`, `tag`, `release`, `all`, and `test-install`, with the same one-line glosses the script prints today:

- `help` — Show this list (also: no command, `-h`, `--help`)
- `version` — Print the package version from this checkout
- `setup` — Install or upgrade build and twine for this python3
- `clean` — Remove build, dist, egg-info, and caches
- `build` — Build an sdist and a wheel into dist/
- `upload` — Upload dist/* . Does not build first
- `git` — Ask for one commit message, then stage, commit, and push
- `tag` — Create and push annotated tag `v` plus the version, or `vunread` when the version was unread
- `release` — clean, then build, then upload, then tag
- `all` — Same chain as release
- `test-install` — Remove this project's pip install, then install this checkout
- `test` — Run tests/run.sh. No extra arguments.

The examples **MUST** include `./build.sh version`, `./build.sh build`, `./build.sh test`, `./build.sh test-install`, and `./build.sh release`.

### 2.3 Implementation Notes (this project)

| Field | Value |
|-------|-------|
| Script | `./build.sh` |
| Package | `PyRbac` |
| Version read | `src/PyRbac/__init__.py` → `__version__` (`1.0.1`) |
| Pip name | `[project].name` in `pyproject.toml` (`PyRbac`) |
| `test-install` target | The repository directory that contains `build.sh` |
| `test` target | `tests/run.sh` |
| Tag remote | `origin` |
| `git` remote | Whatever `git push` uses for the current branch |
| Index credentials | Not stored in this script or in this requirement |
| Product CLI | Unchanged. `py-rbac` does not gain these commands |

`tomllib` is in the Python standard library from 3.11. `test-install` therefore needs that interpreter even though the package declares `requires-python` `>=3.9`. The other commands do not import `tomllib`.

### 2.4 Why This Requirement Exists (Direct CIAO Alignment)

- **CIAO Principle 2 – Intentional** (https://github.com/cloudgen/ciao): Each maintainer command has one order of steps.
- **CIAO Principle 5 – SSOT**: This file owns the build-script commands. Packaging owns the manifest.
- **CIAO Principle 1 – Caution**: `upload` does not invent a fresh `dist/`, and `git` does not force-push.
- **CIAO Principle 9 – Least privilege**: The script runs as the invoking login. It does not elevate.

## Under command line for normal user only

On Termux, Git Bash, Windows cmd, or the same class, the maintainer runs `./build.sh` as the normal user. **This requirement:** do not call `sudo`, `apt`, or Termux `pkg`, and do not create a dedicated system user. Git Bash and Windows cmd do not call Termux `pkg`. `setup` and `test-install` use `python3 -m pip` only.

## 3. Design Principles (CIAO / CIAO-Lite)

- **Caution**: A failed `build` names `setup` as the next step. A failed `upload` names `build`.
- **Intentional**: `release` and `all` are the only commands that chain clean, build, upload, and tag.
- **Anti-fragile**: The banner still prints when the version cannot be read, and commands that need the version then stop.
- **Over-protect**: `test` and `test-install` refuse extra tokens so a mistyped flag cannot change the suite or the install.
- **Principle 16**: `git` prompts only when stdin is a terminal. A pipe supplies the one message line and does not wait.

## 4. Protection Rule (Sacred)

**Future AI assistants or maintainers MUST NOT**:

1. Add a `py-rbac` verb to this script, or route `login`, `serve`, or `db-init` through it.
2. Drop, rename, or merge a command in §2.1. `all` stays an alias of `release`.
3. Make `upload` build first, or make `build` upload or tag.
4. Make `release` skip `clean`.
5. Add `--force` to `git push` or `git push origin` for the tag.
6. Print a token, a password, or an index credential.
7. Call `sudo` from this script.
8. Turn `test-install` into an editable install.
9. Treat this file as the product CLI requirement.

## 5. Related artifacts (versioned surface only)

| Artifact | Role |
|----------|------|
| `docs/requirements/index.md` | Registry |
| `docs/requirements/requirement-python-packaging.md` | Manifest and console script |
| `docs/requirements/requirement-python-project-structure.md` | Where `build.sh` sits |
| `./build.sh` | The script this requirement describes |
| `reviews/test-plan.md` | `TP-BUILD-*` rows |

## Design-time verification

| TP family / ID | Suite | Status | Note |
|----------------|-------|--------|------|
| TP-BUILD-01 | `tests/` | todo | `help` with no arguments lists every command in §2.1 |
| TP-BUILD-02 | `tests/` | todo | `test` and `test-install` reject an extra token |
| TP-BUILD-03 | `tests/` | todo | `version` prints the same `__version__` as `src/PyRbac/__init__.py` |
| TP-BUILD-04 | `tests/` | todo | `clean` removes `dist/` and does not remove `src/PyRbac` |

The suite does not invoke `./build.sh` yet. Those rows stay todo until a test runs the script.

## Terminologies

### PyPI build

**Definition:** A PyPI build produces the source distribution and wheel for a Python package with the `build` package (`python -m build`, typically `python -m build --sdist --wheel`). When `./build.sh` exists, the steps are `clean` then `build`. Clean removes the previous `dist/`, `build/`, and egg-info so a foreign wheel is not left in `dist/`. A PyPI build does not upload and does not create a git tag. Generated `dist/` is not source.

**Human daily-life explanation:** A PyPI build is packing the finished goods into the shipping cartons. You empty the old cartons first, then pack this product only. Packing is not handing the cartons to the warehouse, and it is not sticking a new luggage tag on the git history.

**Daily-life example:** You run the pack step. The table that held yesterday’s other brand’s carton is cleared. Today’s source carton and wheel carton are the only ones left. Nothing has been sent to the public index yet.

On this product, `./build.sh build` packs and does not clean. `./build.sh release` is the chain that cleans first. That split is §2.1, not a second meaning of the word.

### PyPI upload

**Definition:** A PyPI upload publishes the sdist and wheel from a PyPI build to the package index with the `twine` package (`python -m twine upload` / `twine upload`). It runs only after a proper project version bump, a fresh push of that bump (the remote contains the bump commit), and a local version strictly newer than the probed PyPI version (`info.version` for `[project].name`). When `./build.sh` exists, the steps are `clean`, `build`, then `upload`. `release` is not this term unless the user asked to tag. Agents MUST NOT upload a dirty `dist/` that holds another package’s artifacts, and MUST NOT print index credentials.

**Human daily-life explanation:** A PyPI upload is handing this edition’s cartons to the public warehouse only after three checks: the edition number was raised on purpose, that raised number is already on the public shelf of the source history, and the warehouse’s current catalog number is older. You do not hand over yesterday’s other brand, and you do not read the warehouse password aloud.

**Daily-life example:** The catalog already lists edition `1.2.0`. The books on the table say `1.2.1`, and that `1.2.1` commit is already on the remote. You pack, then send. You do not send a carton whose label is `1.2.0` again, and you do not announce a luggage tag unless someone asked for a tag.

On this product, `./build.sh upload` sends `dist/*` and does not perform those three checks itself. `./build.sh release` cleans, builds, uploads, and tags because the operator asked for the release chain.

## 6. Status history

| Date | Status | Notes |
|------|--------|-------|
| 2026-10-11 | Active 1.0.0 | Command set taken from the live `./build.sh` |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-rust-build-script` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
