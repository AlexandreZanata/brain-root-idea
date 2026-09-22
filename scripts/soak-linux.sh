#!/bin/sh
# BrainRoot MVP-0 performance soak.
#
# Runs the ignored release soak for the conversation core and, when a display
# exists or BRAINROOT_SOAK_OPEN_CLOSE=1, repeats open/close cycles through the
# existing smoke. Measurements, not a CI gate.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
CYCLES="${BRAINROOT_SOAK_CYCLES:-5}"
CYCLE_LOG="$(mktemp)"
trap 'rm -f "$CYCLE_LOG"' EXIT INT TERM

echo "[soak] conversation core soak (cargo test --release soak -- --ignored)"
(cd "$ROOT/src-tauri" && cargo test --release soak -- --ignored --nocapture)

if [ -n "${WAYLAND_DISPLAY:-}${DISPLAY:-}${BRAINROOT_SOAK_OPEN_CLOSE:-}" ]; then
  BINARY="${BRAINROOT_SMOKE_BINARY:-$ROOT/src-tauri/target/release/brainroot}"
  if [ ! -x "$BINARY" ]; then
    echo "[soak] building the production binary (pnpm tauri build --no-bundle)"
    (cd "$ROOT" && pnpm tauri build --no-bundle)
  fi
  cycle=1
  while [ "$cycle" -le "$CYCLES" ]; do
    echo "[soak] open/close cycle $cycle/$CYCLES"
    if ! BRAINROOT_SMOKE_BINARY="$BINARY" sh "$ROOT/scripts/smoke-linux.sh" >"$CYCLE_LOG" 2>&1; then
      tail -20 "$CYCLE_LOG" >&2
      printf 'FAIL: open/close cycle %s failed\n' "$cycle" >&2
      exit 1
    fi
    cycle=$((cycle + 1))
  done
  echo "[soak] open/close cycles completed=$CYCLES"
else
  echo "[soak] skipped open/close cycles (no display; set BRAINROOT_SOAK_OPEN_CLOSE=1 to force)"
fi

echo "[soak] OK"
