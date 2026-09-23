#!/bin/sh
# BrainRoot accessibility and focus gate.
#
# Enforces the shell's keyboard, focus, live-region, zoom, and motion
# invariants across the whole frontend source tree. Offline and deterministic;
# accepts an optional root for negative fixtures.
set -eu

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
fi

SRC="$ROOT/src"
INDEX="$ROOT/index.html"
[ -d "$SRC" ] || { printf 'FAIL: missing %s\n' "$SRC" >&2; exit 1; }
[ -f "$INDEX" ] || { printf 'FAIL: missing %s\n' "$INDEX" >&2; exit 1; }

python3 - "$ROOT" "$SRC" "$INDEX" <<'PY'
import os
import re
import sys

root, src, index_path = sys.argv[1], sys.argv[2], sys.argv[3]
frontend = []
for dirpath, _dirnames, filenames in os.walk(src):
    for name in filenames:
        if name.endswith((".svelte", ".ts", ".css", ".html")):
            frontend.append(os.path.join(dirpath, name))
frontend.append(index_path)

text = "\n".join(open(path, encoding="utf-8").read() for path in frontend)
index = open(index_path, encoding="utf-8").read()
problems = []

for pattern, label in [
    (r'<html[^>]*\blang="en"', 'index.html must declare lang="en"'),
    (r":focus-visible", "the frontend must style :focus-visible"),
    (r"prefers-reduced-motion", "the frontend must honor prefers-reduced-motion"),
    (r"<main\b", "the frontend must use a main landmark"),
    (r'<label\s+for="prompt"', "the prompt control needs a bound label"),
    (r'role="alert"', 'failures need a role="alert" region'),
    (r'role="status"', 'setup guidance needs a role="status" region'),
    (r'aria-live="polite"', "the status line needs a polite live region"),
]:
    if not re.search(pattern, index if pattern.startswith("<html") else text):
        problems.append(f"missing: {label}")

if len(re.findall(r"aria-labelledby", text)) < 2:
    problems.append("missing: both sections need aria-labelledby headings")
if len(re.findall(r'aria-disabled=', text)) < 2:
    problems.append("missing: disabled controls must keep focus via aria-disabled")
if re.search(r"user-scalable", index):
    problems.append("zoom must not be disabled with user-scalable")

for path in frontend:
    file_text = open(path, encoding="utf-8").read()
    rel = os.path.relpath(path, root)
    if re.search(r"(?<!aria-)\bdisabled(?:=|\s|>)", file_text):
        problems.append(f"controls must not use the disabled attribute (focus is dropped): {rel}")
    if re.search(r"user-select\s*:\s*none", file_text):
        problems.append(f"text selection must not be blocked: {rel}")
    if re.search(r"outline\s*:\s*none", file_text):
        problems.append(f"focus outlines must not be removed: {rel}")
    if re.search(r"tabindex", file_text):
        problems.append(f"positive/negative tabindex must not override the natural order: {rel}")
    for tag in re.findall(r"<[^>]*conversation-history[^>]*>", file_text):
        if "aria-live" in tag:
            problems.append(f"the conversation history must not announce every chunk: {rel}")

if problems:
    for problem in problems:
        print(f"FAIL: {problem}", file=sys.stderr)
    sys.exit(1)

print("accessibility gate ok: keyboard, focus, live regions, zoom, and motion invariants present")
PY
