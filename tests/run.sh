#!/bin/sh
# RustRbac test runner. The ship suite is cargo test.
set -eu
ROOT="$(CDPATH= cd -- "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
cargo test -- --test-threads=8
