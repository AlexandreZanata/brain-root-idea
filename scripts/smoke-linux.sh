#!/bin/sh
# BrainRoot release smoke test: launch, typed readiness, clean shutdown, cleanup.
#
# Runs on a session with a display; not used in CI. Counts only the application's
# direct children so unrelated WebKit processes cannot corrupt the result.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BINARY="${BRAINROOT_SMOKE_BINARY:-$ROOT/src-tauri/target/release/brainroot}"
READY_MARKER="${BRAINROOT_SMOKE_READY_MARKER:-brainroot: health contract}"
READY_TIMEOUT="${BRAINROOT_SMOKE_READY_TIMEOUT:-30}"
EXIT_TIMEOUT="${BRAINROOT_SMOKE_EXIT_TIMEOUT:-15}"

LOG="$(mktemp)"
APP_PID=""
CHILD_PIDS=""

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

is_alive() {
  state="$(ps -o stat= -p "$1" 2>/dev/null || true)"
  [ -n "$state" ] && [ "${state#Z}" = "$state" ]
}

child_command() {
  tr '\0' ' ' <"/proc/$1/cmdline" 2>/dev/null || true
}

existing_release_pid() {
  target="$(readlink -f "$BINARY" 2>/dev/null || true)"
  for pid in $(pgrep -x brainroot 2>/dev/null || true); do
    exe="$(readlink -f "/proc/$pid/exe" 2>/dev/null || true)"
    if [ -n "$target" ] && [ "$exe" = "$target" ]; then
      printf '%s\n' "$pid"
      return 0
    fi
  done
  return 1
}

cleanup() {
  if [ -n "$APP_PID" ]; then
    kill -s TERM -- "-$APP_PID" 2>/dev/null || true
  fi
  for pid in $CHILD_PIDS; do
    kill -TERM "$pid" 2>/dev/null || true
  done
  rm -f "$LOG"
}
trap cleanup EXIT INT TERM

echo "[smoke] baseline: checking for an existing release process"
if existing_release_pid >/dev/null; then
  fail "a release brainroot process is already running; close it first"
fi

if [ -z "${BRAINROOT_SMOKE_BINARY:-}" ]; then
  echo "[smoke] building the production binary (pnpm tauri build --no-bundle)"
  pnpm tauri build --no-bundle
fi

echo "[smoke] launching $BINARY"
setsid "$BINARY" >"$LOG" 2>&1 &
APP_PID=$!

waited=0
ready=no
while [ "$waited" -lt "$READY_TIMEOUT" ]; do
  if ! is_alive "$APP_PID"; then
    fail "the application exited before readiness"
  fi
  if grep -qF "$READY_MARKER" "$LOG" 2>/dev/null; then
    ready=yes
    break
  fi
  sleep 1
  waited=$((waited + 1))
done
[ "$ready" = yes ] || fail "readiness marker not observed within ${READY_TIMEOUT}s"
echo "[smoke] readiness marker observed: $(grep -m1 -F "$READY_MARKER" "$LOG")"

web=0
network=0
for pid in $(pgrep -P "$APP_PID" 2>/dev/null || true); do
  command_line="$(child_command "$pid")"
  case "$command_line" in
    *WebKitWebProcess*)
      web=$((web + 1))
      CHILD_PIDS="$CHILD_PIDS $pid"
      ;;
    *WebKitNetworkProcess*)
      network=$((network + 1))
      CHILD_PIDS="$CHILD_PIDS $pid"
      ;;
  esac
done
[ "$web" -eq 1 ] || fail "expected exactly 1 WebKitWebProcess child, found $web"
[ "$network" -eq 1 ] || fail "expected exactly 1 WebKitNetworkProcess child, found $network"
echo "[smoke] child processes: $web web, $network network"

echo "[smoke] closing with SIGTERM"
kill -TERM "$APP_PID" 2>/dev/null || true

waited=0
while [ "$waited" -lt "$EXIT_TIMEOUT" ]; do
  remaining=0
  if is_alive "$APP_PID"; then
    remaining=1
  fi
  for pid in $CHILD_PIDS; do
    if is_alive "$pid"; then
      remaining=1
    fi
  done
  [ "$remaining" -eq 0 ] && break
  sleep 1
  waited=$((waited + 1))
done

if is_alive "$APP_PID"; then
  fail "the main process survived ${EXIT_TIMEOUT}s"
fi
for pid in $CHILD_PIDS; do
  if is_alive "$pid"; then
    fail "child process $pid survived ${EXIT_TIMEOUT}s"
  fi
done
echo "[smoke] main and child processes exited"

if ss -tlnp 2>/dev/null | grep -qiE 'brainroot|WebKit'; then
  fail "a listener owned by the application remained"
fi
echo "[smoke] no listeners remained"

wait "$APP_PID" 2>/dev/null || true
APP_PID=""
CHILD_PIDS=""
echo "[smoke] OK"
