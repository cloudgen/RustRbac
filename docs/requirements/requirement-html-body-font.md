**file**: docs/requirements/requirement-html-body-font.md
**Status**: Active (Version 1.0.0)
**Area**: domain
**Key**: `requirement-html-body-font`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

Both page skins set the document body to 13px. The skin choice is `requirement-html-theme`.

### 1.1 Human-facing

**In one sentence:** The page text is 13px in vue3 and in adminlte.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | The reader | The sign-in page |
| The other role | The stylesheet | `body { font-size: 13px; }` |
| Not this file | Which skin is selected | `requirement-html-theme` |

| Includes | Excludes |
|----------|----------|
| Body font-size 13px on both skins | A 16px body |
| The existing 16px heading class | A script that sets the size |

## 2. Normative rules

1. **Body.** `rustrbac.css` and `adminlte.css` **MUST** set `body` to `font-size: 13px`.
2. **Heading.** The class `.user-edit-head` **MAY** stay `16px`. That class is not the document body.
3. **No script.** The size **MUST** come from the stylesheet. A page **MUST NOT** set it with a script tag.

## 3. Acceptance checks

| ID | Check |
|----|-------|
| AC-1 | `/rustrbac.css` contains `font-size: 13px` on `body` |
| AC-2 | `/adminlte.css` contains `font-size: 13px` on `body` |

## Terminologies

| Term | Meaning in this file |
|------|----------------------|
| Body size | The `font-size` of the `body` rule |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-10-16 | Active 1.0.0 | Both skins set the document body to 13px |

---

**Last Updated**: 2026-10-16
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
