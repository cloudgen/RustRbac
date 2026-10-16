# Review: RustRbac text menu follows OpenRbac

**Date:** 2026-10-14
**Product:** RustRbac 1.1.0
**Scope:** The text menu rows. The sibling is OpenRbac `requirement-python-tui` 1.6.0.
**Suite:** `cargo test --test product -- --skip sqlite_seed_scope_and_web --skip mysql_test_database` on 2026-10-14. 11 passed, 0 failed. The two database tests were not repeated.

## What changed

The front board is 1 rbac, 2 server, 3 system-log, 6 self-management, 7 language, and 9 Exit. Numbers 4, 5, and 8 are not rows. There is no session row.

Server holds setup 21, audit 22, and serve 23. Rbac children are 11–19 and 110–113. Setup children are 211–218. Number 218 is `db-engine`. Audit children are 221–224. Self-management children are 62–67. Language codes are 71–83. Reserved numbers are 70 and 84–89.

Back on setup and audit returns to the server board. Back on the other nested boards returns to the front board. Esc on a nested board returns to the front board. The word `access` opens the rbac board.

Self-management copy still says cargo. Empty arguments still use the current engine and do not ask for one. Enter on the engine choice still keeps the current engine. That part does not follow OpenRbac, which selects sqlite3 on an empty choice.

## Checks

| Check | Result |
|-------|--------|
| Front numbers are 1, 2, 3, 6, 7, and 9. Row 1 is rbac. Row 7 is language. No session row | Pass in `cli_gate_and_front_rows` |
| Number 4 is rejected. Row 2 opens server. Row 21 opens setup. Back returns to server. Esc returns to the front board. `access` opens rbac. Audit Back returns to server | Pass in `cli_gate_and_front_rows` |
| Codes 71–83. 70 and 84 do not write the leaf. 73 saves `zh-Hant` | Pass in `menu_language_save_and_path_width` |
| zh-Hans and zh-Hant front lines name rbac, server, and cargo | Pass in `front_board_alignment_uses_cargo` |
| The frame still paints the rounded box and `RustRbac 1.1.0` | Pass in `front_board_paints_the_tui_frame` |
| Help names server row 23 and cargo, and does not name `py-rbac` | Pass in `help_mentions_cargo_and_not_a_publish` |

## Left as they were

- The 2026-10-11 reviews record the previous board. They were not rewritten.
- The retired Python package and `tests/test_*.py` were removed later on 2026-10-14. This menu review did not move their numbers. `tests/run.sh` starts `cargo test` only.
- Screenshot PNG files were recaptured from the running menu on 2026-10-14. The README captions name the words on those pictures. The server board is `screenshots/server-menu.png`. The old session board file is not part of this menu.
- `sqlite_seed_scope_and_web` and `mysql_test_database_when_the_password_file_is_present` were not run again.

## Verdict

Pass for the menu move.
