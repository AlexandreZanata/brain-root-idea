#!/bin/sh
# BrainRoot R1 fast baseline: no rebuild, redacted, tolerant headless.
# Usage: BRAINROOT_BASELINE_RAW=/tmp/r1-baseline.log sh scripts/perf-baseline.sh
# Records env + binary/asset sizes + one opencode sidecar probe with an
# ephemeral password. Never prints keys, tokens, or response bodies.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
OUT="${BRAINROOT_BASELINE_RAW:-/tmp/r1-baseline.log}"
BINARY="$ROOT/src-tauri/target/release/brainroot"

{
  echo "[baseline] commit=$(git -C "$ROOT" rev-parse HEAD)"
  echo "[baseline] os=$(. /etc/os-release && printf '%s %s' "$NAME" "$VERSION")"
  echo "[baseline] kernel=$(uname -sr) arch=$(uname -m)"
  echo "[baseline] cpu=$(awk -F: '/model name/{print $2; exit}' /proc/cpuinfo | sed 's/^ //') cores=$(nproc)"
  echo "[baseline] opencode_version=$(opencode --version 2>/dev/null || echo unknown)"

  if [ -f "$BINARY" ]; then
    echo "[baseline] brainroot_bytes=$(stat -c%s "$BINARY")"
  else
    echo "[baseline] brainroot_bytes=UNKNOWN (no release binary)"
  fi
  for asset in "$ROOT"/dist/index.html "$ROOT"/dist/assets/*; do
    [ -f "$asset" ] || continue
    echo "[baseline] asset path=${asset#"$ROOT/"} bytes=$(stat -c%s "$asset") gzip_bytes=$(gzip -c "$asset" | wc -c)"
  done

  PORT=4099
  PASS="r1-ephemeral-$PPID"
  OPENCODE_SERVER_PASSWORD="$PASS" opencode serve --port "$PORT" --hostname 127.0.0.1 --pure > /tmp/r1-probe.log 2>&1 &
  SIDECAR_PID=$!
  sleep 3
  if kill -0 "$SIDECAR_PID" 2>/dev/null; then
    HEALTH_CODE=$(curl -s -o /dev/null -w "%{http_code}" -u "opencode:$PASS" "http://127.0.0.1:$PORT/global/health" || echo 000)
    DOC_BYTES=$(curl -s -u "opencode:$PASS" "http://127.0.0.1:$PORT/doc" -o /dev/null -w "%{size_download}" || echo 0)
    echo "[baseline] sidecar pid=$SIDECAR_PID health_http=$HEALTH_CODE doc_bytes=$DOC_BYTES"
    echo "[baseline] sidecar_rss_kb=$(awk '/^VmRSS:/{print $2}' /proc/"$SIDECAR_PID"/status 2>/dev/null || echo UNKNOWN)"
    sh "$ROOT/scripts/measure-rss.sh" "$SIDECAR_PID" | grep -E "^(total_rss_kb|total_pss_kb)" | sed 's/^/[baseline] sidecar_/'
    kill "$SIDECAR_PID" 2>/dev/null || true
    sleep 1
    if kill -0 "$SIDECAR_PID" 2>/dev/null; then
      echo "[baseline] sidecar_cleanup=FAIL (leftover)"
    else
      echo "[baseline] sidecar_cleanup=OK"
    fi
  else
    echo "[baseline] sidecar=UNKNOWN (failed to start; see /tmp/r1-probe.log head, no secrets)"
  fi
  rm -f /tmp/r1-probe.log
  echo "[baseline] OK (redacted, no keys logged)"
} | tee "$OUT"
