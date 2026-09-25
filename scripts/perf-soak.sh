#!/bin/sh
# BrainRoot R1 soak: sidecar start/stop cycles + leftover check.
# Usage: sh scripts/perf-soak.sh [cycles] [port]
# Fast spike version of scripts/soak-linux.sh. No secrets logged.
set -eu

CYCLES="${1:-5}"
PORT="${2:-4099}"
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
completed=0
i=1
while [ "$i" -le "$CYCLES" ]; do
  PASS="r1-soak-$PPID-$i"
  OPENCODE_SERVER_PASSWORD="$PASS" opencode serve --port "$PORT" --hostname 127.0.0.1 --pure > /tmp/r1-soak.log 2>&1 &
  PID=$!
  sleep 2
  if kill -0 "$PID" 2>/dev/null; then
    CODE=$(curl -s -o /dev/null -w "%{http_code}" -u "opencode:$PASS" "http://127.0.0.1:$PORT/global/health" || echo 000)
    echo "[soak] cycle=$i health_http=$CODE"
    kill "$PID" 2>/dev/null || true
    sleep 1
    if kill -0 "$PID" 2>/dev/null; then
      echo "[soak] cycle=$i FAIL=leftover-process"
      kill -9 "$PID" 2>/dev/null || true
      break
    fi
    completed=$((completed + 1))
  else
    echo "[soak] cycle=$i FAIL=not-started"
    break
  fi
  i=$((i + 1))
done
rm -f /tmp/r1-soak.log
echo "[soak] completed=$completed cycles=$CYCLES"
[ "$completed" -eq "$CYCLES" ]
