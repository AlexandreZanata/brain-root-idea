#!/bin/sh
# BrainRoot accessibility and focus gate.
#
# Enforces the conversation UI's keyboard, focus, live-region, zoom, and motion
# invariants on the shell sources. Offline and deterministic; accepts an
# optional root for negative fixtures.
set -eu

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
fi

APP="$ROOT/src/App.svelte"
INDEX="$ROOT/index.html"
[ -f "$APP" ] || { printf 'FAIL: missing %s\n' "$APP" >&2; exit 1; }
[ -f "$INDEX" ] || { printf 'FAIL: missing %s\n' "$INDEX" >&2; exit 1; }

python3 - "$ROOT" "$APP" "$INDEX" <<'PY'
import re
import sys

root, app_path, index_path = sys.argv[1], sys.argv[2], sys.argv[3]
app = open(app_path, encoding="utf-8").read()
index = open(index_path, encoding="utf-8").read()
problems = []

for required, label in [
    (r'<html[^>]*\blang="en"', "index.html must declare lang=\"en\""),
    (r":focus-visible", "App.svelte must style :focus-visible"),
    (r"prefers-reduced-motion", "App.svelte must honor prefers-reduced-motion"),
    (r"<main\b", "App.svelte must use a main landmark"),
    (r'<label\s+for="prompt"', "the prompt control needs a bound label"),
    (r'role="alert"', "failures need a role=\"alert\" region"),
    (r'role="status"', "setup guidance needs a role=\"status\" region"),
    (r'aria-live="polite"', "the status line needs a polite live region"),
]:
    if not re.search(required, index if required.startswith("<html") else app):
        problems.append(f"missing: {label}")

if len(re.findall(r"aria-labelledby", app)) < 2:
    problems.append("missing: both sections need aria-labelledby headings")
if len(re.findall(r'aria-disabled=', app)) < 2:
    problems.append("missing: Send and Cancel must keep focus via aria-disabled")
if re.search(r"user-scalable", index):
    problems.append("zoom must not be disabled with user-scalable")

for tag in re.findall(r"<[^>]*conversation-history[^>]*>", app):
    if "aria-live" in tag:
        problems.append("the conversation history must not announce every streamed chunk")
if re.search(r"(?<!aria-)\bdisabled(?:=|\s|>)", app):
    problems.append("form controls must not use the disabled attribute (focus is dropped)")
if re.search(r"user-select\s*:\s*none", app):
    problems.append("text selection must not be blocked")
if re.search(r"outline\s*:\s*none", app):
    problems.append("focus outlines must not be removed")
if re.search(r"tabindex", app):
    problems.append("positive/negative tabindex must not override the natural order")

if problems:
    for problem in problems:
        print(f"FAIL: {problem}", file=sys.stderr)
    sys.exit(1)

print("accessibility gate ok: keyboard, focus, live regions, zoom, and motion invariants present")
PY
