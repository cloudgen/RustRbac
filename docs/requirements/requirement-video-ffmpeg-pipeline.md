**file**: docs/requirements/requirement-video-ffmpeg-pipeline.md
**Status**: Retired (Version 1.3.0)
**Area**: video
**Key**: `requirement-video-ffmpeg-pipeline`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

## 1. Purpose

This requirement is retired. RustRbac does not call FFmpeg, does not concatenate media, and does not publish a media file.

The current product is RustRbac. This file stays retired.

The welcome line is `requirement-domain-videojoin`. Do not treat the historical concat procedure as law.

### 1.1 Human-facing

**In one sentence:** There is no encoder step.

| Includes | Excludes |
|----------|----------|
| The record that stream-copy concat and re-encode fallback are gone | A command line that runs `ffmpeg` |
| | `shutil.move` of a joined media file. That publish path is gone with the join |

## 2. Core Rules / Requirements (Mandatory)

1. **MUST NOT** invoke `ffmpeg` or any other encoder from this product.
2. **MUST NOT** write a concat list, a media intermediate, or a joined output file.
3. **MUST NOT** restore a join module, or a Python package that concatenates media, without an explicit user order and a new Active revision of `requirement-domain-videojoin`.
4. A missing `ffmpeg` binary is not an error. The front board and `hello` do not check for it.

## 4. Protection Rule (Sacred)

**Future AI assistants, Grok, or maintainers MUST NOT** mark this file Active again in order to bring the concat pipeline back, unless the user orders that restoration and the domain file is revised in the same change.

## 5. Acceptance criteria

| ID | Criterion |
|----|-----------|
| AC-1 | Status is Retired |
| AC-2 | No product module imports or runs `ffmpeg` |
| AC-3 | `hello` and menu row 1 do not check for an encoder |

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-08-09 | Active 1.0.0 | Stream-copy concat, then re-encode, then `shutil.move` |
| 2026-10-04 | Active 1.2.0 | Fail-closed re-encode stayed |
| 2026-10-05 | Retired 1.3.0 | Encoder support removed with the join |
| 2026-10-11 | Retired 1.3.0 | Current product is RustRbac. This file stays retired |
| 2026-10-14 | Retired 1.3.0 | Rule 3 no longer names a Python path. The join stays retired |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`.
