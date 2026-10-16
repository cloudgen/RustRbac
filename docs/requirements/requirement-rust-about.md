**file**: docs/requirements/requirement-rust-about.md
**Status**: Active (Version 1.0.3)
**Area**: rust
**Key**: `requirement-rust-about`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This file is the about page for RustRbac. Menu action **about** and the verb `rust-rbac about` return one text page: an identity block, a host check, and a star box. The page does not claim `--json`.

The screen that shows the page stays on `requirement-rust-tui`. The domain sentence stays on `requirement-domain-rust-rbac`. The version string is `1.1.1` from `Cargo.toml` (`requirement-rust-packaging`).

### 1.1 Human-facing

**In one sentence:** Choose **about** and the page names this build, this computer, and where `rust-rbac` is running from.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The person reading the page | Menu row **63 about**, or `rust-rbac about` |
| The other role | The star box | Author, date, and the homepage |
| Not this file | The permission table | `requirement-rbac` |

| Includes | Excludes |
|----------|----------|
| Identity lines, a host check, the private cache leaf, and a star box | `--json`, a second about layout |
| Persistence `{home}/.local/RustRbac` | A cache folder that is only `/dev/shm`, `/tmp`, or `{home}/.cache` |
| MariaDB or MySQL, and sqlite3, named as runtime tools | FFmpeg, ChronicleLogger, or PyMySQL |
| Entry `rust-rbac` | `python -m` or a pip install line |

## 2. Normative rules

1. **One page.** `about` **MUST** be three blocks in this order: identity, host check, star box. A blank line **MUST** separate the blocks. The page **MUST NOT** be `--version` and **MUST NOT** be JSON. There is no `--json`.
2. **Identity.** The first lines **MUST** be, in order: `RustRbac 1.1.1`, `Domain: Online RBAC and centralized configuration`, `Runtime tools: MariaDB or MySQL, and sqlite3`, `Entry: rust-rbac`. The version **MUST** be the `Cargo.toml` version. The page **MUST NOT** name FFmpeg.
3. **Host check.** The check **MUST** name this computer after the identity block. Building it **MUST NOT** call `sudo`, open a socket, or spawn a package manager.
4. **Cache leaf.** After the computer line, the host check **MUST** print these labels in order: `Cache folder used`, `Cache folder (preferred)`, `Cache folder (1st fallback)`, `Cache folder (2nd fallback)`, `Persistence storage`. The username is `USER`, then `USERNAME`, then empty. The pid is this process id. Home is `HOME` when that value is non-empty.
    - Preferred, when the username is non-empty: `/dev/shm/cache/cache-RustRbac-{user}-{pid}`. When the username is empty: `/dev/shm/cache/cache-RustRbac-{pid}`.
    - 1st fallback: the same leaf under `/tmp/cache/`.
    - 2nd fallback: `{home}/.cache/cache-RustRbac-{pid}`. That leaf has no username. When home is empty the value is empty and the label stays.
    - Used: the preferred path when `/dev/shm` is a directory, otherwise the 1st fallback when `/tmp` is a directory, otherwise the 2nd fallback.
    - Persistence: `{home}/.local/RustRbac` when home is set. When home is empty the value is empty and the label stays. The value **MUST NOT** be `{home}/.local/share/RustRbac`.
    - The page **MUST NOT** create those directories. It **MUST NOT** create a probe file in `/dev/shm`, `/tmp`, or `{home}/.cache`. It **MUST NOT** report `/dev/shm`, `/tmp`, or `{home}/.cache` as the cache folder.
5. **Star box.** The box **MUST** name `RustRbac`, version `1.1.1`, author Wong Chun Fai, date `2026-10-11`, the usage line `rust-rbac login --user-name admin`, and the homepage `https://github.com/cloudgen/RustRbac`. The live page **MUST NOT** contain `curl -fsSL` and **MUST NOT** claim a crates.io publish.
6. **Author mail.** The about identity **MAY** name `cloudgen.wong@gmail.com`. It **MUST NOT** name a database password.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | The identity lines match rule 2, including the domain sentence |
| AC-2 | A host check follows the identity block, and a star box follows the host check |
| AC-3 | The page has no `--json`, no FFmpeg line, and no crates.io publish claim |
| AC-4 | The star box names the homepage `https://github.com/cloudgen/RustRbac` |
| AC-5 | The cache lines name `cache/cache-RustRbac-…` and persistence is `{home}/.local/RustRbac`. A parent-only path does not pass |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Identity line | `RustRbac` plus the Cargo version |
| Host check | The block that names the computer, the private cache leaf, and persistence |
| Cache leaf | `cache/cache-RustRbac-{user}-{pid}`, or without `{user}` when the username is empty. The 2nd fallback omits `{user}` |
| Star box | The bordered block with author, usage, and homepage |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | RustRbac about page. Entry `rust-rbac`. No `--json` |
| 2026-10-11 | Active 1.0.1 | Private cache leaf restored from the Python about rule. The page does not search a shared cache parent |
| 2026-10-14 | Active 1.0.2 | About on the menu is self-management row 63. Cargo rows are 64–67 |
| 2026-10-16 | Active 1.0.3 | Identity and star box use Cargo.toml 1.1.1. The star date stays 2026-10-11 |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
