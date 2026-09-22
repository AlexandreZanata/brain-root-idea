#!/bin/sh
# BrainRoot documentation, secret-pattern, and repository-integrity gate.
#
# Validates required files, local Markdown links, ADR required sections,
# secret-like patterns, and repository cleanliness. Offline and deterministic.
set -eu

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
fi

python3 - "$ROOT" <<'PY'
import os
import re
import sys

root = sys.argv[1]
problems = []

required_files = [
    "README.md",
    "AGENTS.md",
    "LICENSE",
    "NOTICE",
    "VERSION",
    "CHANGELOG.md",
    "docs/18-mvp-execution-plan.md",
    "docs/19-release-and-versioning.md",
    "docs/20-project-history-and-wiki.md",
    ".github/PULL_REQUEST_TEMPLATE.md",
    ".github/ISSUE_TEMPLATE/microstep.yml",
    ".github/workflows/ci.yml",
    "scripts/check-license-artifacts.sh",
    "scripts/check-governance-templates.sh",
    "scripts/check-version-consistency.sh",
    "scripts/check-docs.sh",
]
for rel in required_files:
    if not os.path.isfile(os.path.join(root, rel)):
        problems.append(f"missing required file: {rel}")

pruned = {".git", "node_modules", "target"}


def walk():
    for dirpath, dirnames, filenames in os.walk(root):
        dirnames[:] = sorted(name for name in dirnames if name not in pruned)
        for filename in sorted(filenames):
            yield os.path.join(dirpath, filename)


link_pattern = re.compile(r"\[[^\]]*\]\(([^)]+)\)")
markdown_files = 0
links_checked = 0
for path in walk():
    if not path.endswith(".md"):
        continue
    markdown_files += 1
    text = open(path, encoding="utf-8").read()
    if os.path.dirname(path) == root and not re.search(r"^# ", text, re.MULTILINE):
        problems.append(f"missing H1 title: {os.path.relpath(path, root)}")
    for raw_target in link_pattern.findall(text):
        target = raw_target.strip()
        if target.startswith("<") or target.startswith(("http://", "https://", "mailto:", "#")):
            continue
        if " " in target:
            target = target.split(" ", 1)[0]
        target = target.split("#", 1)[0].split("?", 1)[0]
        if not target:
            continue
        links_checked += 1
        candidate = os.path.normpath(os.path.join(os.path.dirname(path), target))
        if os.path.exists(candidate) or os.path.exists(candidate + ".md"):
            continue
        problems.append(f"broken link in {os.path.relpath(path, root)}: {raw_target}")

adr_sections = [
    "## Context",
    "## Decision",
    "## Alternatives considered",
    "## Consequences",
    "## Performance implications",
    "## Security implications",
    "## Reversibility",
    "## References",
]
adr_files = 0
adr_dir = os.path.join(root, "docs", "adr")
for name in sorted(os.listdir(adr_dir)):
    if not name.endswith(".md") or name == "README.md":
        continue
    adr_files += 1
    text = open(os.path.join(adr_dir, name), encoding="utf-8").read()
    if not re.search(r"^(\*\*Status:\*\*|## Status)", text, re.MULTILINE):
        problems.append(f"ADR missing status: docs/adr/{name}")
    for section in adr_sections:
        if section not in text:
            problems.append(f"ADR missing section {section!r}: docs/adr/{name}")

secret_patterns = [
    r"-----BEGIN [A-Z ]*PRIVATE KEY-----",
    r"gh[pousr]_[A-Za-z0-9]{36}",
    r"github_pat_[A-Za-z0-9_]{22,}",
    r"AKIA[0-9A-Z]{16}",
    r"sk-[A-Za-z0-9]{20,}",
    r"(?i:authorization:\s*bearer\s+[A-Za-z0-9._-]{20,})",
]
secret_pattern = re.compile("|".join(secret_patterns))
scanned_files = 0
for path in walk():
    try:
        text = open(path, encoding="utf-8").read()
    except (UnicodeDecodeError, IsADirectoryError):
        continue
    scanned_files += 1
    if secret_pattern.search(text):
        problems.append(f"secret-like pattern in {os.path.relpath(path, root)}")

for path in walk():
    name = os.path.basename(path)
    if name == ".DS_Store" or name.endswith((".orig", ".rej", "~")):
        problems.append(f"junk file present: {os.path.relpath(path, root)}")

scripts_dir = os.path.join(root, "scripts")
for name in sorted(os.listdir(scripts_dir)):
    if name.endswith(".sh") and not os.access(os.path.join(scripts_dir, name), os.X_OK):
        problems.append(f"script is not executable: scripts/{name}")

if problems:
    for problem in problems:
        print(f"FAIL: {problem}", file=sys.stderr)
    sys.exit(1)

print(f"docs gate ok: {markdown_files} markdown files, {links_checked} links checked")
print(f"adr gate ok: {adr_files} ADRs with required sections")
print(f"secret scan ok: {scanned_files} files scanned")
print("repository integrity ok: required files, no junk, scripts executable")
PY
