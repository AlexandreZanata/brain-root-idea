#!/bin/sh
# BrainRoot version consistency check.
#
# Verifies the single machine-readable version source, the changelog sections,
# and any present ecosystem copy (Cargo, npm, Tauri). Offline and deterministic.
set -eu

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
fi

[ -f "$ROOT/VERSION" ] || { printf 'FAIL: missing %s\n' "$ROOT/VERSION" >&2; exit 1; }
[ -f "$ROOT/CHANGELOG.md" ] || { printf 'FAIL: missing %s\n' "$ROOT/CHANGELOG.md" >&2; exit 1; }

python3 - "$ROOT" <<'PY'
import json
import os
import re
import sys

try:
    import tomllib
except ImportError:
    sys.exit("FAIL: python3.11+ with tomllib is required for version consistency validation")

root = sys.argv[1]
semver = re.compile(
    r"^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)"
    r"(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$"
)

with open(os.path.join(root, "VERSION"), encoding="utf-8") as handle:
    raw = handle.read()
lines = raw.splitlines()
if len(lines) != 1 or not lines[0] or lines[0].strip() != lines[0]:
    sys.exit("FAIL: VERSION must contain exactly one line without surrounding whitespace")
version = lines[0]
if version != "UNRELEASED" and not semver.match(version):
    sys.exit(f"FAIL: VERSION must be UNRELEASED or SemVer 2.0.0 (found {version!r})")
print(f"version source: {version}")

with open(os.path.join(root, "CHANGELOG.md"), encoding="utf-8") as handle:
    changelog = handle.read()
if not re.search(r"^## Unreleased\s*$", changelog, re.MULTILINE):
    sys.exit("FAIL: CHANGELOG.md is missing the '## Unreleased' section")
sections = ["Unreleased"]
if version != "UNRELEASED":
    if not re.search(rf"^## {re.escape(version)}\s*$", changelog, re.MULTILINE):
        sys.exit(f"FAIL: CHANGELOG.md is missing a '## {version}' section")
    sections.append(version)
print(f"changelog: {', '.join(sections)} section(s) present")

copies = []

cargo = os.path.join(root, "Cargo.toml")
if os.path.isfile(cargo):
    with open(cargo, "rb") as handle:
        data = tomllib.load(handle)
    value = data.get("package", {}).get("version")
    if value:
        copies.append(("Cargo.toml", value))

package = os.path.join(root, "package.json")
if os.path.isfile(package):
    with open(package, encoding="utf-8") as handle:
        data = json.load(handle)
    value = data.get("version")
    if value:
        copies.append(("package.json", value))

tauri = os.path.join(root, "src-tauri", "tauri.conf.json")
if os.path.isfile(tauri):
    with open(tauri, encoding="utf-8") as handle:
        data = json.load(handle)
    value = data.get("version")
    if value:
        copies.append(("src-tauri/tauri.conf.json", value))

for name, value in copies:
    if value != version:
        sys.exit(f"FAIL: {name} version {value!r} does not match VERSION {version!r}")

if copies:
    print(f"ecosystem copies: {len(copies)} checked and matching")
else:
    print("ecosystem copies: none present")
print("OK: version consistency verified")
PY
