#!/bin/sh
# BrainRoot R1 RSS snapshot: process-tree RSS/PSS for one PID.
# Usage: sh scripts/measure-rss.sh <pid>
# No build, no network, no secrets. Read-only /proc scan.
set -eu

if [ $# -ne 1 ]; then
  echo "usage: sh scripts/measure-rss.sh <pid>" >&2
  exit 2
fi

TARGET="$1"
if ! kill -0 "$TARGET" 2>/dev/null; then
  echo "measure-rss: pid $TARGET not running" >&2
  exit 1
fi

children_of() {
  # $1 = pid; prints space-separated children or nothing.
  if [ -r "/proc/$1/task/$1/children" ]; then
    cat "/proc/$1/task/$1/children" 2>/dev/null || true
  fi
}

# Breadth-first tree walk without bash arrays.
PIDS="$TARGET"
QUEUE="$TARGET"
while [ -n "$QUEUE" ]; do
  HEAD="${QUEUE%% *}"
  case "$QUEUE" in
    *" "*) QUEUE="${QUEUE#* }" ;;
    *) QUEUE="" ;;
  esac
  for child in $(children_of "$HEAD"); do
    case " $PIDS " in
      *" $child "*) ;;
      *) PIDS="$PIDS $child"; QUEUE="$QUEUE $child" ;;
    esac
  done
done

total_rss=0
total_pss=0
for pid in $PIDS; do
  if [ -r "/proc/$pid/status" ]; then
    rss=$(awk '/^VmRSS:/{print $2}' "/proc/$pid/status" 2>/dev/null || echo 0)
    total_rss=$((total_rss + ${rss:-0}))
  fi
  if [ -r "/proc/$pid/smaps_rollup" ]; then
    pss=$(awk '/^Pss:/{print $2}' "/proc/$pid/smaps_rollup" 2>/dev/null || echo 0)
    total_pss=$((total_pss + ${pss:-0}))
  fi
  if [ -r "/proc/$pid/comm" ]; then
    comm=$(cat "/proc/$pid/comm" 2>/dev/null || echo unknown)
    echo "proc pid=$pid comm=$comm"
  fi
done

echo "tree pids=$PIDS"
echo "total_rss_kb=$total_rss"
echo "total_pss_kb=$total_pss"
