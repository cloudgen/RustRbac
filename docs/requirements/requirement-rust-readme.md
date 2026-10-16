**file**: docs/requirements/requirement-rust-readme.md
**Status**: Active (Version 1.0.2)
**Area**: rust
**Key**: `requirement-rust-readme`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The root `README.md` is the public user document for RustRbac. Its version badge matches `Cargo.toml` `1.1.1`. Every screenshot is an absolute `https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/<file>.png` link. The pictures are captures of the running text menu at 1.1.0.

### 1.1 Human-facing

**In one sentence:** The README shows the real menu, the seeded accounts, and how to change those passwords.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A reader on GitHub | The screenshot of the front board |
| The other role | The raw image URL | `https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/main-menu-en.png` |
| Not this file | A host vault path | Never written here |

## 2. Required headings

These headings **MUST** remain, in this order:

Features, Advantages, Quick Installation, Usage, Screenshots, Examples, Platform Compatibility, Related Projects, Contributing, License, Last Update.

## 3. Normative rules

1. **Identity.** The document **MUST** name RustRbac, the binary `rust-rbac`, and version `1.1.1`. It **MUST NOT** claim a crates.io publish. Self-management copy **MUST** say cargo, not pip. A screenshot caption **MAY** name the version printed in that capture.
2. **Images.** A picture **MUST** use `https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/<file>.png`. A relative link written as `](screenshots/` **MUST NOT** appear.
3. **Seed.** The document **MUST** list the four seed accounts from `requirement-domain-rust-rbac` and **MUST** tell the operator to change those passwords.
4. **Storage.** The document **MUST** say passwords are PBKDF2 `pbkdf2_sha256` with 260000 iterations rather than bcrypt, sessions are `sys_session` rather than Redis, and data scope is 1–4.
5. **Bind.** The document **MUST** say the site listens on `127.0.0.1` unless `RUSTRBAC_ALLOW_REMOTE=yes`. That switch is environment-only.
6. **Process list.** The document **MUST** say `--password` is visible in the process list and the menu masks the field.
7. **No secrets.** The document **MUST NOT** include a vault path, a private key, or the live database password.
8. **Sign-in.** The document **MUST** say that a terminal with no arguments tries the saved sign-in and asks for an account and a password only when that sign-in is missing or rejected, that the menu has no session row, and that a run creates missing tables before that question. The ciphertext rules stay on `requirement-local-login`.
9. **Homepage.** The document **MUST** name `https://github.com/cloudgen/RustRbac`.

## 4. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | The eleven headings in section 2 remain, in that order |
| AC-2 | Screenshot links are the absolute raw URL, and `](screenshots/` does not appear |
| AC-3 | The version badge is `1.1.1` and the binary is `rust-rbac` |
| AC-4 | The document does not claim a crates.io publish |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Raw URL | `https://raw.githubusercontent.com/cloudgen/RustRbac/main/screenshots/<file>.png` |
| Seed account | One of the four accounts created when `sys_user` is empty |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | RustRbac README. Absolute screenshot URLs. Version 1.1.0 |
| 2026-10-11 | Active 1.0.1 | Empty argv tries the saved sign-in and asks for the account only when that is missing or rejected |
| 2026-10-14 | Active 1.0.1 | README pictures are captures of the current menu, including the server board |
| 2026-10-16 | Active 1.0.2 | Version badge 1.1.1. The pictures stay the 1.1.0 captures. Server row 24 is named in the text |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
