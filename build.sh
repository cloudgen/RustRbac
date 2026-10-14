#!/bin/sh
# requirement-rust-build-script
# Maintainer verbs only. rust-rbac flags are not accepted here.
# This script does not publish to crates.io.
set -eu

ROOT=$(CDPATH= cd -- "$(dirname "$0")" && pwd)
cd "$ROOT"

PROJECT="RustRbac"

out_err() {
    printf '%s\n' "$1" >&2
}

read_version() {
    awk '
        /^version[[:space:]]*=/ {
            gsub(/"/, "", $3)
            print $3
            exit
        }
    ' "$ROOT/Cargo.toml"
}

VERSION=$(read_version 2>/dev/null || true)

need_version() {
    if [ -z "$VERSION" ]; then
        out_err "ERROR: Cannot read the package version from Cargo.toml."
        out_err "   Next: set version in Cargo.toml, then ./build.sh version"
        exit 1
    fi
}

if [ -n "$VERSION" ]; then
    printf '%s\n' "$PROJECT build tool (v$VERSION)"
else
    printf '%s\n' "$PROJECT build tool (version unread)"
fi
printf '%s\n' "========================================"

show_help() {
    cat << EOF
Usage: ./build.sh <command>

Operational commands:
  help          Show this list (also: no command, -h, --help)
  version       Print the package version from Cargo.toml
  setup         cargo fetch
  clean         cargo clean
  build         cargo build --release
  test          cargo test
  test-install  cargo install --path . --force

test-install is listed for the operator. The checks in this repository do not run it.
This script does not publish to crates.io.

Examples:
  ./build.sh version
  ./build.sh build
  ./build.sh test
EOF
}

do_setup() {
    cargo fetch
}

do_clean() {
    cargo clean
}

do_build() {
    cargo build --release
}

do_test() {
    cargo test
}

do_test_install() {
    cargo install --path . --force
}

COMMAND=${1:-help}
case "$COMMAND" in
    help|-h|--help|"")
        show_help
        ;;
    version)
        need_version
        printf '%s\n' "$VERSION"
        ;;
    setup)
        do_setup
        ;;
    clean)
        do_clean
        ;;
    build)
        do_build
        ;;
    test)
        do_test
        ;;
    test-install)
        do_test_install
        ;;
    *)
        out_err "ERROR: Unknown build command '$COMMAND'."
        out_err "   Next: ./build.sh help"
        exit 1
        ;;
esac
