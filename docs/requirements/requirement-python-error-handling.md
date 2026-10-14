**file**: docs/requirements/requirement-python-error-handling.md
**Status**: Retired (Version 1.2.0)
**Area**: python
**Key**: `requirement-python-error-handling`
**Philosophy**: CIAO **v2.10.2** / CIAO-Lite (Caution • Intentional • Anti-fragile • Over-engineered / Over-protect)

> **Retired.** The product is RustRbac. The Active replacement is `requirement-rust-error-handling`.

## 1. Purpose

Define how PyRbac detects, reports, and recovers from errors during the text menu, the `join` verb, and FFmpeg processing without destroying the user’s source media.

Console sentences in this file stay required. Durable status, once the logger exists, is `requirement-python-cli-logging`. ChronicleLogger is required. It is not optional.

### 1.1 Human-facing

**In one sentence:** When a join cannot finish, PyRbac tells you why on the screen and leaves both source videos where they are.

| Box | Meaning | Example |
|-----|---------|---------|
| You / this login | Person who picked a bad index or has no FFmpeg | You see the reason and can try again or stop |
| The other role | The status file | The same fact is written by ChronicleLogger after `main` has constructed it |
| Not this file | The menu frame, and the concat command order | `requirement-python-tui`, `requirement-video-ffmpeg-pipeline` |

| Includes | Excludes |
|----------|----------|
| A readable failure sentence. Sources left intact. Temps cleaned best-effort | A silent success after FFmpeg failed |
| Ask again when the index is not in the list | Encoding that invalid index |
| The missing-library sentence, printed before any logger exists | Sending that one line through `log_message` |

| Surface | What you open | What for |
|---------|---------------|----------|
| Join questions | Menu region above the box | The reason, without leaving the front board when the board is already open |
| `py-rbac join` with one video | Message, non-zero exit | Nothing is concatenated |
| Console before the logger exists | Missing ChronicleLogger | The pip next step |

| You do… | What it means | What you type |
|---------|---------------|---------------|
| Pick a number that is not in the list | The question stays. FFmpeg does not run. | A bad index, then a real one |
| Join with fewer than two videos | You see why, and the sources stay. | `py-rbac join` |
| Run without the logger library | The console names the pip next step and the program returns non-zero. | `py-rbac` |

## 2. Core Rules (Mandatory)

### 2.1 Fail-closed principles

1. **MUST NOT** silently ignore a failed join when the product claims success.
2. **MUST NOT** treat invalid user input as success.
3. **MUST** prefer clear human-readable messages over stack traces for expected user mistakes.
4. **MUST** leave both source videos intact on all failure paths.

### 2.2 Required error categories

| Category | Detection | Required action |
|----------|-----------|-----------------|
| Fewer than two eligible videos | Discovery returns fewer than 2 | Message. Do not run FFmpeg. Direct `join` exits non-zero. An open front board stays open |
| Invalid selection index | Out of range or non-numeric | Ask again. Do not proceed |
| FFmpeg missing | `ffmpeg` is not on PATH | Actionable message: install FFmpeg. `join` exits non-zero. The front board may still open |
| Stream-copy failure | primary FFmpeg non-zero | Attempt fallback re-encode (pipeline peer). Do not claim lossless success |
| Fallback failure | re-encode FFmpeg non-zero or exception | Report failure. Do not claim success |
| Temp cleanup failure | unlink errors | Best-effort. Do not mask the original error |
| ChronicleLogger missing | import fails inside `def main` | Console next step from `requirement-python-cli-logging`. Return non-zero. That line cannot use `log_message` |
| `join` with no terminal | stdout is not a terminal | Non-zero. Tell the operator to use a terminal. Do not wait |
| Empty argv with no terminal | stdout is not a terminal | Help text. Return 0. This is not a failure |

### 2.3 Cleanup on failure

5. **MUST** attempt to remove the concat list temp file after failed or successful demuxer-path jobs.
6. **MUST NOT** delete the final user output path solely because a later optional step failed, unless the partial final is known corrupt — then remove only that corrupt final and say so.
7. **MUST NOT** delete either source media as cleanup.
8. An unfinished temp is not the result. Discard it, and log that discard before the unlink (`requirement-python-cli-logging`).

### 2.4 Logging and the console

9. Once `def main` has constructed ChronicleLogger, durable status **MUST** go through that logger. The logger is required (`requirement-python-packaging`).
10. **MUST** still show the user-visible failure reason. On the text screen it sits in the menu region. Off the text screen it is a console line. A quiet logger mirror is not a substitute for that sentence.
11. **MUST NOT** log secrets. This product has none.
12. **MUST NOT** hang on `input()` when stdout is not a terminal.

### 2.5 Implementation Notes (this project)

| Item | Value |
|------|--------|
| **Primary FFmpeg check** | `subprocess.run` on the stream-copy path. Success only if return code 0 and the intermediate file is present |
| **Fallback FFmpeg** | second `subprocess.run`. Success only if return code 0 and the intermediate is present. Otherwise a message and a non-zero exit from the join |
| **Fewer than two videos** | Message. Direct `join` exits non-zero. Open menu stays open |
| **Invalid index** | Ask again inside the join questions |
| **Temp cleanup** | Unique list and media temps unlinked best-effort. Log the discard first once the logger exists |
| **FFmpeg missing** | Checked when `join` runs. Not a reason to refuse the front board |
| **ChronicleLogger** | Required. Missing import is a console sentence and a non-zero return, before any `log_message` |
| **No terminal** | Empty argv prints help and returns 0. Direct `join` exits non-zero |
| **Running tree** | `cli.py` still uses `input()` and treats the logger as absent. That is not the allowed end state |

### 2.6 Why This Requirement Exists (CIAO)

- **Principle 1 – Caution**: Fail closed on encode errors and missing tools.
- **Principle 11 – Temps**: Cleanup without destroying sources.
- **Principle 12 – Traceability**: The operator still sees the failure when the log mirror is quiet.

## Under command line for normal user only

On Termux, Git Bash, Windows cmd, or the same class, failure sentences are for the normal user who started `py-rbac`. **This requirement:** do not use administrator privilege, `sudo`, `apt`, or a dedicated system user to report an error or to clean a temp. Do not pipe a downloaded script into a shell. Type 1 and Type 2 are unused on Termux, Git Bash, and Windows cmd. Git Bash and Windows cmd do not call Termux `pkg`.

## 3. Design Principles (CIAO / CIAO-Lite)

- **Caution:** An invalid selection never reaches FFmpeg.
- **Intentional:** The category table is the failure contract.
- **Anti-fragile:** Cleanup is best-effort and does not hide the first error.
- **Over-protect:** Source media stays in place.

## 4. Protection Rule (Sacred)

**Future AI assistants, Grok, or maintainers MUST NOT**:

1. Swallow FFmpeg errors without a user-visible failure.
2. Delete source media on error.
3. Claim success after a failed join.
4. Replace clear messages with a silent pass.
5. Log credentials or tokens.
6. Call the logger optional, or send the missing-library line through `log_message`.
7. Hang on a prompt when there is no terminal.

**Violating this rule is a critical safety regression.**

## 5. Acceptance criteria

| ID | Criterion |
|----|-----------|
| AC-1 | An invalid selection does not encode |
| AC-2 | Missing FFmpeg on `join` surfaces an actionable error |
| AC-3 | Source files remain after failure |
| AC-4 | The temp list is cleaned best-effort |
| AC-5 | Fewer than two videos does not encode |
| AC-6 | A failed join does not present itself as success |
| AC-7 | Missing ChronicleLogger is a console next step and a non-zero return |

## 6. Related requirements (peer keys only)

| Key | Relationship |
|-----|----------------|
| `requirement-video-ffmpeg-pipeline` | Stages that can fail |
| `requirement-python-cli-interface` | Verb exits |
| `requirement-python-tui` | Where the sentence sits on the screen |
| `requirement-python-cli-logging` | Durable status after the construct |
| `requirement-runtime-prerequisites` | Missing FFmpeg and the logger floor |
| `requirement-domain-videojoin` | Session outcomes |
| `docs/requirements/index.md` | Registry |

## Design-time verification

| TP family / ID | Suite | Status | Note |
|----------------|-------|--------|------|
| **TP-ERR-01** | `tests/test_errors.py` | todo | No FFmpeg on `join` → message, non-zero |
| **TP-ERR-02** | `tests/test_errors.py` | todo | One video only → message, no encode |
| **TP-ERR-03** | `tests/test_errors.py` | todo | Sources intact after a failed encode |
| **TP-FFMPEG-05** | `tests/test_ffmpeg_pipeline.py` | todo | Fallback fail-closed |
| **TP-LOG-01** | `tests/test_logging.py` | todo | Peer: missing library is a console line, not `log_message` |

**Matrix:** `docs/reviews/requirement-test-matrix.md`
**Map:** `docs/reviews/test-plan.md`

## 7. Status history

| Date | Status | Note |
|------|--------|------|
| 2026-08-09 | Active 1.0.0 | Initial error-handling law for PyRbac |
| 2026-08-09 | Active 1.1.0 | Fail-closed fallback and unique temp cleanup notes |
| 2026-10-04 | Active 1.2.0 | ChronicleLogger is required for durable status. Console sentences stay. Missing library is a console line. Empty argv off a terminal is help, return 0 |
| 2026-10-11 | Retired | Product is RustRbac. Active replacement is `requirement-rust-error-handling` |

---

**Last Updated**: 2026-10-11
**Owner**: project maintainers
**Alignment**: Registry `docs/requirements/index.md`; **CIAO** (https://github.com/cloudgen/ciao); CIAO-Lite (https://github.com/cloudgen/ciao-lite).
