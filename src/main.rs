//! rust-rbac process entry.
//! requirement-rust-logging — StatusLog::open is the construct. It stays in main.
//! requirement-rust-cli-interface — the console script is this binary.

fn main() {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let log = rust_rbac::log::StatusLog::open("", rust_rbac::APP_NAME);
    let code = rust_rbac::cli::main_with(&argv, &log);
    std::process::exit(code);
}
