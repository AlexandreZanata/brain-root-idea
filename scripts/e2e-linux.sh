#!/bin/sh
# BrainRoot MVP-0 fake-provider end-to-end journey.
#
# The product journey and the rendered-output bounds are deterministic and run
# anywhere; the launch/readiness/close smoke needs a display and is skipped
# explicitly when none is available.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

echo "[e2e] fake-provider product journey (cargo test e2e)"
(cd "$ROOT/src-tauri" && cargo test e2e)

echo "[e2e] bounded rendered output (frontend unit tests)"
(cd "$ROOT" && pnpm run test:frontend)

if [ -n "${WAYLAND_DISPLAY:-}${DISPLAY:-}${BRAINROOT_E2E_SMOKE:-}" ]; then
  echo "[e2e] launch, readiness, and clean close (process smoke)"
  sh "$ROOT/scripts/smoke-linux.sh"
else
  echo "[e2e] skipped the process smoke (no display; set BRAINROOT_E2E_SMOKE=1 to force)"
fi

echo "[e2e] OK"
