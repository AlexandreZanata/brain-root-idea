#!/bin/sh
# BrainRoot Linux performance baseline measurement.
#
# Follows docs/10-performance-budget.md: production build, 10 startup samples
# timed to the readiness marker, 5 settled steady-state windows with
# process-tree CPU/RSS, bundle sizes, and open/close cleanup cycles.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BINARY="$ROOT/src-tauri/target/release/brainroot"
OUT="$(mktemp)"
LOG="$(mktemp)"

exec 3>&1
exec >"$OUT" 2>&1

cleanup() {
  rm -f "$OUT" "$LOG"
}
trap cleanup EXIT INT TERM

if [ -z "${BRAINROOT_MEASURE_SKIP_BUILD:-}" ]; then
  echo "[measure] building the production binary (pnpm tauri build --no-bundle)"
  (cd "$ROOT" && pnpm tauri build --no-bundle)
fi

echo "[measure] environment"
printf 'commit=%s\n' "$(git -C "$ROOT" rev-parse HEAD)"
printf 'lockfile_sha256=%s\n' "$(sha256sum "$ROOT/src-tauri/Cargo.lock" | cut -d' ' -f1)"
printf 'lockfile_pnpm_sha256=%s\n' "$(sha256sum "$ROOT/pnpm-lock.yaml" | cut -d' ' -f1)"
printf 'os=%s\n' "$(. /etc/os-release && printf '%s %s' "$NAME" "$VERSION")"
printf 'kernel=%s\n' "$(uname -sr)"
printf 'architecture=%s\n' "$(uname -m)"
printf 'cpu=%s\n' "$(awk -F: '/model name/{print $2; exit}' /proc/cpuinfo | sed 's/^ //')"
printf 'cpu_count=%s\n' "$(nproc)"
printf 'memory_kib=%s\n' "$(awk '/MemTotal/{print $2}' /proc/meminfo)"
printf 'storage_root=%s\n' "$(df -h / | awk 'NR==2{print $2" total, "$4" free"}')"
printf 'display_session=%s\n' "${XDG_SESSION_TYPE:-unknown}"
printf 'governor=%s\n' "$(cat /sys/devices/system/cpu/cpu0/cpufreq/scaling_governor 2>/dev/null || echo unknown)"
printf 'ac_online=%s\n' "$(cat /sys/class/power_supply/ACAD/online 2>/dev/null || echo unknown)"
printf 'uptime=%s\n' "$(uptime -p)"
printf 'load_average=%s\n' "$(cut -d' ' -f1-3 /proc/loadavg)"
printf 'webkit2gtk=%s\n' "$(pkg-config --modversion webkit2gtk-4.1 2>/dev/null || echo unknown)"

echo "[measure] bundle sizes"
binary_bytes="$(stat -c%s "$BINARY")"
printf 'binary_bytes=%s\n' "$binary_bytes"
for asset in "$ROOT"/dist/index.html "$ROOT"/dist/assets/*; do
  [ -f "$asset" ] || continue
  uncompressed="$(stat -c%s "$asset")"
  gzipped="$(gzip -c "$asset" | wc -c)"
  printf 'asset path=%s bytes=%s gzip_bytes=%s\n' "${asset#"$ROOT/"}" "$uncompressed" "$gzipped"
done

echo "[measure] startup and steady-state sampling"
rc=0
python3 - "$BINARY" "$LOG" <<'PY' || rc=$?
import os
import signal
import statistics
import subprocess
import sys
import time

binary, log_path = sys.argv[1], sys.argv[2]
ready_marker = b"brainroot: health contract"
clock_ticks = os.sysconf("SC_CLK_TCK")
failed = False


def children_of(pid):
    try:
        with open(f"/proc/{pid}/task/{pid}/children", encoding="utf-8") as handle:
            return [int(value) for value in handle.read().split()]
    except (FileNotFoundError, ProcessLookupError, PermissionError):
        return []


def tree(pid):
    result = []
    stack = [pid]
    while stack:
        current = stack.pop()
        result.append(current)
        stack.extend(children_of(current))
    return result


def cpu_ticks(pid):
    try:
        with open(f"/proc/{pid}/stat", encoding="utf-8") as handle:
            data = handle.read()
        fields = data.rsplit(") ", 1)[1].split()
        return int(fields[11]) + int(fields[12])
    except (FileNotFoundError, ProcessLookupError, IndexError, ValueError):
        return None


def rss_bytes(pid):
    try:
        with open(f"/proc/{pid}/status", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("VmRSS:"):
                    return int(line.split()[1]) * 1024
    except (FileNotFoundError, ProcessLookupError, PermissionError):
        return None
    return None


def comm(pid):
    try:
        with open(f"/proc/{pid}/comm", encoding="utf-8") as handle:
            return handle.read().strip()
    except (FileNotFoundError, ProcessLookupError, PermissionError):
        return None


def pss_bytes(pid):
    try:
        with open(f"/proc/{pid}/smaps_rollup", encoding="utf-8") as handle:
            for line in handle:
                if line.startswith("Pss:"):
                    return int(line.split()[1]) * 1024
    except (FileNotFoundError, ProcessLookupError, PermissionError):
        return None
    return None


def launch():
    handle = open(log_path, "wb")
    process = subprocess.Popen([binary], stdout=handle, stderr=subprocess.STDOUT, start_new_session=True)
    return process


def wait_ready(process, timeout=15.0):
    start = time.monotonic()
    while time.monotonic() - start < timeout:
        if process.poll() is not None:
            return None
        try:
            with open(log_path, "rb") as handle:
                if ready_marker in handle.read():
                    return time.monotonic() - start
        except FileNotFoundError:
            pass
        time.sleep(0.02)
    return None


def close(process):
    try:
        os.killpg(process.pid, signal.SIGTERM)
    except ProcessLookupError:
        pass
    try:
        process.wait(timeout=15)
    except subprocess.TimeoutExpired:
        try:
            os.killpg(process.pid, signal.SIGKILL)
        except ProcessLookupError:
            pass
        process.wait(timeout=5)


def leftovers(pid):
    try:
        os.kill(pid, 0)
    except ProcessLookupError:
        return False
    # a zombie is reaped by wait()
    return True


def summarize(label, values):
    ordered = sorted(values)
    p95 = statistics.quantiles(ordered, n=100, method="inclusive")[94]
    print(f"[{label}-summary] n={len(ordered)} median={statistics.median(ordered):.3f} "
          f"p95={p95:.3f} min={ordered[0]:.3f} max={ordered[-1]:.3f}")


startup_samples = []
for run in range(1, 11):
    process = launch()
    elapsed = wait_ready(process)
    if elapsed is None:
        print(f"[startup] run={run} FAIL=readiness-not-reached")
        close(process)
        failed = True
        break
    startup_samples.append(elapsed)
    print(f"[startup] run={run} seconds={elapsed:.3f}")
    close(process)
    if leftovers(process.pid):
        print(f"[startup] run={run} FAIL=leftover-process")
        failed = True
        break

if startup_samples:
    summarize("startup", startup_samples)

window_summaries = []
class_samples = {}
for index in range(1, 6):
    process = launch()
    if wait_ready(process) is None:
        print(f"[window] index={index} FAIL=readiness-not-reached")
        close(process)
        failed = True
        break
    time.sleep(10)
    cpu_samples = []
    rss_samples = []
    pss_samples = []
    previous_ticks = {}
    previous_time = None
    window_end = time.monotonic() + 30
    while time.monotonic() < window_end:
        started = time.monotonic()
        pids = tree(process.pid)
        current = {pid: cpu_ticks(pid) for pid in pids}
        rss = 0
        pss = 0
        for pid in pids:
            value = rss_bytes(pid)
            shared = pss_bytes(pid)
            name = comm(pid)
            if value:
                rss += value
            if shared:
                pss += shared
            if name:
                if value:
                    class_samples.setdefault(("rss", name), []).append(value)
                if shared:
                    class_samples.setdefault(("pss", name), []).append(shared)
        if previous_time is not None:
            deltas = [current[pid] - previous_ticks[pid] for pid in current
                      if pid in previous_ticks
                      and current[pid] is not None
                      and previous_ticks[pid] is not None]
            if deltas:
                interval = max(started - previous_time, 0.001)
                cpu_samples.append(sum(deltas) / clock_ticks / interval * 100)
        rss_samples.append(rss)
        pss_samples.append(pss)
        previous_ticks = current
        previous_time = started
        time.sleep(max(0.0, 1.0 - (time.monotonic() - started)))
    close(process)
    if leftovers(process.pid):
        print(f"[window] index={index} FAIL=leftover-process")
        failed = True
        break
    median_rss_mb = statistics.median(rss_samples) / 1048576
    median_pss_mb = statistics.median(pss_samples) / 1048576
    mean_cpu = statistics.fmean(cpu_samples) if cpu_samples else 0.0
    window_summaries.append((mean_cpu, median_rss_mb, median_pss_mb))
    print(f"[window] index={index} mean_cpu_pct={mean_cpu:.3f} "
          f"median_rss_mb={median_rss_mb:.1f} median_pss_mb={median_pss_mb:.1f}")

if window_summaries:
    summarize("cpu", [value[0] for value in window_summaries])
    summarize("rss", [value[1] for value in window_summaries])
    summarize("pss", [value[2] for value in window_summaries])

for (metric, name) in sorted(class_samples):
    print(f"[{metric}-class] name={name} samples={len(class_samples[(metric, name)])} "
          f"median_mb={statistics.median(class_samples[(metric, name)]) / 1048576:.1f}")

cycles = 0
for index in range(1, 4):
    process = launch()
    if wait_ready(process) is None:
        print(f"[cycle] index={index} FAIL=readiness-not-reached")
        close(process)
        failed = True
        break
    close(process)
    if leftovers(process.pid):
        print(f"[cycle] index={index} FAIL=leftover-process")
        failed = True
        break
    cycles += 1
print(f"[cycles] completed={cycles}")

if failed:
    print("[measure] FAILED")
    sys.exit(1)
print("[measure] OK")
PY

cat "$OUT" >&3
if [ -n "${BRAINROOT_MEASURE_RAW:-}" ]; then
  cp "$OUT" "$BRAINROOT_MEASURE_RAW"
fi
exit "$rc"
