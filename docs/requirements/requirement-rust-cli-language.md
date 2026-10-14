**file**: docs/requirements/requirement-rust-cli-language.md
**Status**: Active (Version 1.1.0)
**Area**: rust
**Key**: `requirement-rust-cli-language`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This file owns the menu language for RustRbac. The text menu can show its boards in one of thirteen languages. English is the default. The person picks a language on front row **7**. The pick is saved for the next run. The picture of the menu stays on `requirement-rust-tui`. `language` is not a product verb.

### 1.1 Human-facing

**In one sentence:** On the RustRbac menu, you type **7** to pick the language the menu uses, and the next time you open the menu it still uses that language.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The person at the keyboard | `rust-rbac`, then `7` |
| The other role | The saved leaf | `~/.local/RustRbac/language` |
| Not this file | Help, verbs, and the about page | Those stay English in this version |

| Includes | Excludes |
|----------|----------|
| Front row 7 and codes 71–83 | A `language` argument on the command line |
| The leaf under this login’s home | Reading `LANG` or `LC_ALL` to pick a language |
| The path word for the codes named below | Translating the folder path or the clock |

## 2. Normative rules

1. **Codes.** Assigned rows are **71** through **83**, in this order: `en`, `zh-Hans`, `zh-Hant`, `es`, `ar`, `fr`, `pt`, `ru`, `de`, `ja`, `ko`, `nl`, `el`. **70** and **84** through **89** **MUST NOT** be printed.
2. **Default.** The code **MUST** be `en` when the leaf is missing, empty, or not one of those thirteen codes. An invalid first line **MUST NOT** be rewritten.
3. **Locale.** `LANG`, `LANGUAGE`, `LC_ALL`, and `LC_MESSAGES` **MUST NOT** select a language. `RUSTRBAC_LANG`, when it is one of the thirteen codes, **MUST** win for this process and **MUST NOT** write the leaf. Any other value is ignored.
4. **Leaf.** The leaf is `${HOME}/.local/RustRbac/language`, unless `RUSTRBAC_HOME` replaces that home. The directory **MUST** be mode `0700`. The file **MUST** be mode `0600`. A menu pick writes one code and one newline. Back, Esc, and a reserved number **MUST NOT** write the leaf.
5. **Path labels.** The path word **MUST** be the value in this table for these codes. `LANG` **MUST NOT** select the path label.

| Code | Row | Path label |
|------|-----|------------|
| `en` | 71 | Path |
| `zh-Hans` | 72 | 路径 |
| `zh-Hant` | 73 | 路徑 |
| `es` | 74 | Ruta |
| `ja` | 80 | パス |
| `ko` | 81 | 경로 |

6. **Not a verb.** `language` **MUST NOT** appear in `PRODUCT_VERBS`.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | Front row 7 opens codes 71–83. Reserved numbers are not printed |
| AC-2 | A pick writes `~/.local/RustRbac/language` mode `0600` under a directory mode `0700` |
| AC-3 | `RUSTRBAC_LANG=zh-Hant` selects `路徑` and does not write the leaf |
| AC-4 | `LANG` and `LC_ALL` do not select a language or a path label |
| AC-5 | `language` is not a product verb |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Menu language | One of the thirteen codes on rows 71–83 |
| Leaf | `~/.local/RustRbac/language`, mode `0600` |
| Path label | The word before the working directory |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-11 | Active 1.0.0 | RustRbac menu language. Leaf under `~/.local/RustRbac`. `RUSTRBAC_LANG` does not write |
| 2026-10-14 | Active 1.1.0 | Language moves to front row 7. Codes are 71–83. Reserved block is 70–89 |

---

**Last Updated**: 2026-10-14
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
