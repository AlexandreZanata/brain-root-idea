#!/bin/sh
# BrainRoot fast gate: Rust formatting/unit tests plus frontend tests and types.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

echo "[check-fast] rust format (cargo fmt --check)"
(cd src-tauri && cargo fmt --all --check)

echo "[check-fast] rust unit tests (cargo test)"
(cd src-tauri && cargo test)

echo "[check-fast] frontend unit tests (node --test)"
pnpm run test:frontend

echo "[check-fast] security and privacy (check-security)"
sh "$ROOT/scripts/check-security.sh"

echo "[check-fast] feature module structure (check-modules)"
sh "$ROOT/scripts/check-modules.sh"

echo "[check-fast] accessibility and focus (check-accessibility)"
sh "$ROOT/scripts/check-accessibility.sh"

echo "[check-fast] frontend types (svelte-check)"
pnpm run check

echo "[check-fast] OK"
