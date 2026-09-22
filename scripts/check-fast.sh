#!/bin/sh
# BrainRoot fast gate: Rust formatting and unit tests plus frontend types.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

echo "[check-fast] rust format (cargo fmt --check)"
(cd src-tauri && cargo fmt --all --check)

echo "[check-fast] rust unit tests (cargo test)"
(cd src-tauri && cargo test)

echo "[check-fast] frontend types (svelte-check)"
pnpm run check

echo "[check-fast] OK"
