# RustRbac checklist — 2026-10-11

Use this list before treating the Rust edition as ready. The earlier PyRbac checks are historical in `reviews/2026-10-11-pyrbac-review.md`.

## Requirements

- [x] Exactly one Active domain file: `requirement-domain-rust-rbac`
- [x] On 2026-10-11, `requirement-domain-pyrbac` and `requirement-python-*` were Retired and still on disk. On 2026-10-14 those files, `pyproject.toml`, and `src/PyRbac` were removed. Database names `pyrbac` and `pyrbac_test` stay refused
- [x] `requirement-database` is Active 2.0.1 for `rust_rbac` / `RUSTRBAC_*`
- [x] Independent `requirement-rust-*` files are Active, including the CLI and the build script
- [x] Those two files contain the section "Under command line for normal user only"
- [x] Index status matches the file headers. 20 Active requirements
- [x] `requirement-local-login` is Active. The home folder and the `session` file stay on `requirement-environment`

## Behavior

- [x] Front rows are access, system-log, language, setup, audit, serve, self-management, Exit. There is no session row
- [x] Language stays on row 4, codes 41–53, leaf `~/.local/RustRbac/language`
- [x] Self-management copy says cargo. Lifecycle verbs print the cargo line and do not run it
- [x] Data scope 1–4. Unknown scope shows no users
- [x] Web bind defaults to loopback. Cookie is `rust_rbac_session`. CSRF mismatch is 400
- [x] A wrong password is not written into the files directly under the application home
- [x] A terminal sign-in writes `local-login` (magic `RUSTRBACLOCAL1`, mode 0600) and a 32-byte `local-login-key`. The password is not plain text in that file
- [x] `logout` removes `local-login`. A second logout with no token reports `Not signed in.`
- [x] A web sign-in does not write `local-login`. Row 9 Exit asks before logout

## Database

- [x] Product schema is `rust_rbac`. Tests use `rust_rbac_test`
- [x] `db-init` seeds only when `sys_user` is empty
- [x] Denied names `pyrbac`, `pyrbac_test`, `ruoyi`, `ruoyi_db`, and `ry` are refused
- [x] The mysql proof drops only `sys_*` tables inside `rust_rbac_test`

## Document

- [x] README screenshots use absolute raw GitHub URLs for the 11 existing captures
- [x] Seeded passwords are documented with a change-them warning
- [x] No vault path or live database password in versioned docs
- [x] `Cargo.toml` version is 1.1.0. `./build.sh version` prints 1.1.0

## Not done in this run

- [ ] `./build.sh test-install` (`cargo install --path . --force`) is listed and was not executed
- [ ] No git commit and no push
- [ ] Screenshots were not recaptured
