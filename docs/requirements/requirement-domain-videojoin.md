**file**: docs/requirements/requirement-domain-videojoin.md
**Status**: Retired (Version 1.3.0)
**Area**: domain
**Key**: `requirement-domain-videojoin`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

The current product is RustRbac. This file stays retired.

This domain file is retired. RustRbac does not show a welcome line and does not join media.

The Active domain file is `requirement-domain-rust-rbac`. Do not treat a welcome sentence, `hello`, `join`, or `list-videos` as product law.

### 1.1 Human-facing

**In one sentence:** The welcome-line domain is retired, and online RBAC is the live domain.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | A reader of an older VideoJoin note | Open `requirement-domain-rust-rbac` |
| The other role | The retired welcome sentence | It is not a verb |
| Not this file | Users, roles, and the web page | `requirement-rbac`, `requirement-web-interface` |

| Includes | Excludes |
|----------|----------|
| A retired marker so an older note is not Active | Restoring `hello` or an encoder |

## 2. Normative rules

1. **MUST NOT** be the Active domain while `requirement-domain-rust-rbac` is Active.
2. **MUST NOT** restore `hello`, `join`, or `list-videos` without an explicit user order and a new Active domain revision.

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Retired domain | This file. It does not choose a menu row. |
| Active domain | `requirement-domain-rust-rbac` |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-08-09 | Active 1.0.0 | Domain SSOT for a two-file join |
| 2026-10-05 | Active 1.2.0 | The domain became a welcome line |
| 2026-10-11 | Retired 1.3.0 | PyRbac domain replaces this file |
| 2026-10-11 | Retired 1.3.0 | Current product is RustRbac. This file stays retired |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
