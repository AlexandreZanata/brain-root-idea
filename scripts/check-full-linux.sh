#!/bin/sh
# BrainRoot full Linux gate: fast checks plus lint, production build, release build,
# documentation links, version consistency, and license/secret checks.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
cd "$ROOT"

echo "[check-full-linux] rust format (cargo fmt --check)"
(cd src-tauri && cargo fmt --all --check)

echo "[check-full-linux] rust lint (cargo clippy -D warnings)"
(cd src-tauri && cargo clippy --all-targets -- -D warnings)

echo "[check-full-linux] rust unit tests (cargo test)"
(cd src-tauri && cargo test)

echo "[check-full-linux] frontend types (svelte-check)"
pnpm run check

echo "[check-full-linux] frontend production build (vite build)"
pnpm run build

echo "[check-full-linux] production build (pnpm tauri build --no-bundle)"
pnpm tauri build --no-bundle

echo "[check-full-linux] feature module structure (check-modules)"
sh scripts/check-modules.sh

echo "[check-full-linux] accessibility and focus (check-accessibility)"
sh "$ROOT/scripts/check-accessibility.sh"

echo "[check-full-linux] documentation links, required sections, secrets, and integrity"
sh scripts/check-docs.sh

echo "[check-full-linux] version consistency"
sh scripts/check-version-consistency.sh

echo "[check-full-linux] license and NOTICE artifacts"
sh scripts/check-license-artifacts.sh

echo "[check-full-linux] OK"
