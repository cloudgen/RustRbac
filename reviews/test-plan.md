# RustRbac test plan — 2026-10-11

Run `tests/run.sh` or `cargo test` from the repository root. The ship suite is `tests/product.rs`. Python `tests/test_*.py` is the previous edition and is not started by `tests/run.sh`.

The mysql case runs only when `~/.local/RustRbac/database` is mode 0600. It reads the password, connects as `rust_rbac` to `rust_rbac_test`, and drops only `sys_*` tables in that schema. It does not open `pyrbac`, `pyrbac_test`, `ruoyi`, `ruoyi_db`, or `ry`.

| ID | Proof | Result on 2026-10-11 |
|----|--------|----------------------|
| TP-ENV-01 | Mode 0600 file, empty environment does not win, non-empty environment wins, remote bind refused, denied database names | Pass in `password_file_mode_remote_and_denied_names` |
| TP-CLI-01 | `PRODUCT_VERBS` excludes `hello`, `join`, `list-videos`, and `language`. `db-engine` is not a domain verb. Empty argv opens the text screen only on a terminal | Pass in `cli_gate_and_front_rows` |
| TP-MENU-01 | Front numbers 1, 2, 3, 6, 7, and 9. Row 1 is rbac. Row 2 is server. Numbers 4, 5, and 8 are not rows. No session row. Setup Back returns to server. `access` opens rbac | Pass in `cli_gate_and_front_rows` on 2026-10-14 |
| TP-LANG-01 | Codes 71–83, reserved 70 and 84, leaf mode 0600, directory mode 0700, `RUSTRBAC_LANG` does not write, failed save keeps `zh-Hant` | Pass in `menu_language_save_and_path_width` on 2026-10-14 |
| TP-TUI-10 | Display width and the zh-Hans / zh-Hant front board. Self-management says cargo | Pass in `front_board_alignment_uses_cargo` |
| TP-ABOUT | Identity, host tools, star box, no `--json`, no `py-rbac`. Cache lines are the private leaf `cache/cache-RustRbac-…`. Persistence is `{home}/.local/RustRbac`. The page does not create that leaf | Pass in `about_page_and_docs_identity` and `cache_leaf_is_private_and_skips_the_parent` |
| TP-DOC-01 | README headings, badge `version-1.1.0`, absolute screenshot URLs, `StatusLog::open` in `src/main.rs` | Pass in `about_page_and_docs_identity` |
| TP-RBAC-01 | sqlite seed once, scopes 3, 4, and 5, protected user and role, wrong password stays out of home files | Pass in `sqlite_seed_scope_and_web` |
| TP-LOGIN-01 | Saved sign-in round trip hides the password. A bad check deletes `local-login` and does not open sqlite. Logout removes the file. A live token answers `login` with no password. A web sign-in leaves the file absent. Exit Enter/`y` logs out, `n` keeps it, Esc stays, end of input does not log out | Pass in `round_trip_hides_the_password_and_rejects_a_bad_check`, `rejected_saved_sign_in_deletes_the_ciphertext`, `exit_choice_asks_before_logout`, and `sqlite_seed_scope_and_web` |
| TP-WEB-01 | Cookie `rust_rbac_session`, CSRF 400, operator scope, roles 403, remote engine refusal, loopback engine change | Pass in `sqlite_seed_scope_and_web` and `loopback_may_post_an_engine_change` |
| TP-DB-02 | mysql `rust_rbac_test` seeds once, then reports present, then signs in `admin` | Pass in `mysql_test_database_when_the_password_file_is_present` |
| TP-HELP-01 | Help names `rust-rbac` and cargo, and does not name `py-rbac` or a registry publish. Version is `RustRbac 1.1.0`. `self-install` prints `cargo install --path .` | Pass in `help_mentions_cargo_and_not_a_publish` |
| TP-BUILD-01 | `./build.sh version` prints `1.1.0` from `Cargo.toml`. Unknown command exits 1 | Pass. `./build.sh help` lists setup, clean, build, test, and test-install |
| TP-BUILD-02 | Checks do not run `test-install` and do not run `cargo install` | Held. This run did not execute `test-install` |

`cargo test --offline` on 2026-10-11: library 1 passed; product 12 passed, 0 failed. The nine faster product tests finished in 20.81s. The three database tests finished in 301.02s. `sqlite_seed_scope_and_web` is the slow case because password hashing uses 260000 PBKDF2 iterations in an unoptimized build.

On 2026-10-14 the menu rows moved to the OpenRbac board. `cargo test --test product -- --skip sqlite_seed_scope_and_web --skip mysql_test_database` finished 11 passed, 0 failed. Those two database tests were not repeated. The review is `reviews/2026-10-14-tui-menu-review.md`.

A failure that names `ruoyi_db`, `ry`, `pyrbac`, or `pyrbac_test` means the test pointed at the wrong schema. Stop and fix the environment before repeating the write.
