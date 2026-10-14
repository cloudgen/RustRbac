# Tests

The ship suite is `cargo test`, which `tests/run.sh` starts.

`tests/product.rs` covers the RustRbac identity, the sqlite seed, data scope, the web cookie, the menu language, and the path line. A mysql case runs only when `~/.local/RustRbac/database` is mode 0600 and names `rust_rbac_test`. It does not open `pyrbac`, `pyrbac_test`, `ruoyi`, `ruoyi_db`, or `ry`.

The Python files `test_*.py` are the previous edition. They are not the ship suite.
