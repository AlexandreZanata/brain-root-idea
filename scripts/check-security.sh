#!/bin/sh
# BrainRoot security and privacy gate.
#
# Enforces the release's static security invariants: documented destinations
# only, no frontend network surface, no telemetry, no risky Tauri permissions,
# no credentials in log lines, and the approved direct-dependency sets. Offline
# and deterministic; accepts an optional root for negative fixtures.
set -eu

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
fi

SRC="$ROOT/src-tauri/src"
[ -d "$SRC" ] || { printf 'FAIL: missing %s\n' "$SRC" >&2; exit 1; }

python3 - "$ROOT" <<'PY'
import json
import os
import re
import sys

root = sys.argv[1]
src_rust = os.path.join(root, "src-tauri", "src")
src_web = os.path.join(root, "src")

url_pattern = re.compile(r"https?://[A-Za-z0-9._~:/?#\[\]@!$&'()*+,;=%-]+")
url_exact = {"http://opencode.ai/zen/go/v1/chat/completions"}
url_prefixes = ("https://opencode.ai/zen/go/v1/", "https://example.com/")

network_api = re.compile(r"fetch\(|XMLHttpRequest|WebSocket|EventSource|sendBeacon")
telemetry_sdk = re.compile(
    r"(?i)\b(sentry|posthog|mixpanel|amplitude|bugsnag|datadog|newrelic|firebase|gtag|google-analytics)\b"
)
manifest_telemetry = re.compile(r"(?i)\b(telemetry|analytics)\b")
log_line = re.compile(r"(println!|eprintln!|log::)")
log_secret = re.compile(r"(?i)(authorization|bearer|credential|token|secret|api[_-]?key)")
capability_risk = re.compile(r'"(fs|shell|http|dialog|updater|process):')

problems = []


def rust_files(base):
    for dirpath, _dirnames, filenames in os.walk(base):
        for name in filenames:
            if name.endswith(".rs"):
                yield os.path.join(dirpath, name)


def web_files(base):
    for dirpath, _dirnames, filenames in os.walk(base):
        for name in filenames:
            if name.endswith((".svelte", ".ts", ".js", ".mjs", ".html")):
                yield os.path.join(dirpath, name)


for path in list(rust_files(src_rust)) + list(web_files(src_web)):
    text = open(path, encoding="utf-8").read()
    rel = os.path.relpath(path, root)
    for match in url_pattern.findall(text):
        url = match.rstrip('",);`')
        if url in url_exact or url.startswith(url_prefixes):
            continue
        problems.append(f"unapproved destination {url!r} in {rel}")

for path in web_files(src_web):
    text = open(path, encoding="utf-8").read()
    if network_api.search(text):
        problems.append(f"frontend network surface in {os.path.relpath(path, root)}")

for path in list(rust_files(src_rust)) + list(web_files(src_web)):
    text = open(path, encoding="utf-8").read()
    if telemetry_sdk.search(text):
        problems.append(f"telemetry marker in {os.path.relpath(path, root)}")

for name in ("src-tauri/Cargo.toml", "package.json", "src-tauri/tauri.conf.json", "index.html"):
    path = os.path.join(root, name)
    if not os.path.isfile(path):
        continue
    text = open(path, encoding="utf-8").read()
    if manifest_telemetry.search(text):
        problems.append(f"telemetry or analytics marker in {name}")

for path in rust_files(src_rust):
    for line in open(path, encoding="utf-8").read().splitlines():
        if log_line.search(line) and log_secret.search(line):
            problems.append(
                f"log line may print secret material in {os.path.relpath(path, root)}"
            )

capabilities = os.path.join(root, "src-tauri", "capabilities")
if os.path.isdir(capabilities):
    for name in sorted(os.listdir(capabilities)):
        if not name.endswith(".json"):
            continue
        text = open(os.path.join(capabilities, name), encoding="utf-8").read()
        if capability_risk.search(text):
            problems.append(f"risky Tauri permission in capabilities/{name}")

cargo = os.path.join(root, "src-tauri", "Cargo.toml")
rust_dependencies = set()
section = ""
for line in open(cargo, encoding="utf-8").read().splitlines():
    stripped = line.strip()
    if stripped.startswith("["):
        section = stripped.strip("[]")
        continue
    if section.endswith("dependencies") and "=" in stripped:
        rust_dependencies.add(stripped.split("=", 1)[0].strip())
allowed_rust = {
    "tauri",
    "serde",
    "serde_json",
    "ureq",
    "keyring",
    "uuid",
    "tauri-build",
    "gtk",
    "wry",
}
for extra in sorted(rust_dependencies - allowed_rust):
    problems.append(f"unapproved Rust dependency {extra!r}")

package = os.path.join(root, "package.json")
if os.path.isfile(package):
    data = json.loads(open(package, encoding="utf-8").read())
    web_dependencies = set(data.get("dependencies", {}).keys())
    for extra in sorted(web_dependencies - {"@tauri-apps/api"}):
        problems.append(f"unapproved frontend dependency {extra!r}")

if problems:
    for problem in problems:
        print(f"FAIL: {problem}", file=sys.stderr)
    sys.exit(1)

print(
    "security gate ok: destinations, frontend surface, telemetry, capabilities, logs, and dependencies clean"
)
print(f"rust dependencies: {', '.join(sorted(rust_dependencies))}")
PY
