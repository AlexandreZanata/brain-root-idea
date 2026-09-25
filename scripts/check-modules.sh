#!/bin/sh
# BrainRoot feature-module structure gate.
#
# Enforces the feature layout under src-tauri/src/features/: every module file
# and directory is declared by its parent, and code outside the feature tree
# references only the registered public interface. Offline and deterministic.
set -eu

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
fi

FEATURES="$ROOT/src-tauri/src/features"
[ -d "$FEATURES" ] || { printf 'FAIL: missing %s\n' "$FEATURES" >&2; exit 1; }

python3 - "$ROOT" <<'PY'
import os
import re
import sys

root = sys.argv[1]
src = os.path.join(root, "src-tauri", "src")
features = os.path.join(src, "features")

mod_decl = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?mod\s+([A-Za-z_][A-Za-z0-9_]*)\s*;", re.MULTILINE)
feature_ref = re.compile(r"\bfeatures::[A-Za-z0-9_:]+")
# The explicit interface registry: only these paths may be used outside the
# feature tree. Anything else is private or undeclared.
allowed = re.compile(
    r"features::(?:"
     r"conversation::(?:ConversationSession|conversation_send|conversation_cancel)"
     r"|agent_host::(?:AgentHostState|AgentHostStatus|AgentModelList|AgentModelSelection|AgentSendAccepted|AgentStreamEvent|AgentEventEnvelope|AgentConfig|agent_host_start|agent_host_status|agent_host_stop|agent_host_models|agent_host_select_model|agent_host_send|agent_host_cancel_send|agent_host_catalog|agent_host_set_agent)"
    r"|health::health"
    r"|preview::(?:PreviewState|debug_fixture|preview_start|preview_stop|preview_status|preview_show|preview_set_bounds|preview_view_status|preview_hide)"
    r"|human_browser::(?:HumanBrowserState|HumanStatus|debug_fixture|human_browser_show|human_browser_navigate|human_browser_back|human_browser_forward|human_browser_reload|human_browser_set_bounds|human_browser_hide|human_browser_status|human_browser_clear_data)"
     r"|deck::debug_fixture"
     r"|governor::(?:GovernorState|GovernorStatus|governor_status)"
     r")\b"
)

problems = []

for dirpath, dirnames, filenames in os.walk(features):
    mod_rs = os.path.join(dirpath, "mod.rs")
    if not os.path.isfile(mod_rs):
        problems.append(f"missing mod.rs: {os.path.relpath(dirpath, root)}")
        continue
    declared = set(mod_decl.findall(open(mod_rs, encoding="utf-8").read()))
    for name in filenames:
        if name.endswith(".rs") and name != "mod.rs" and name[:-3] not in declared:
            problems.append(
                f"undeclared module file: {os.path.relpath(os.path.join(dirpath, name), root)}"
            )
    for name in dirnames:
        if not name.startswith(".") and name not in declared:
            problems.append(
                f"undeclared module directory: {os.path.relpath(os.path.join(dirpath, name), root)}"
            )

for dirpath, _dirnames, filenames in os.walk(src):
    if os.path.abspath(dirpath).startswith(os.path.abspath(features) + os.sep):
        continue
    for name in filenames:
        if not name.endswith(".rs"):
            continue
        path = os.path.join(dirpath, name)
        text = open(path, encoding="utf-8").read()
        for reference in feature_ref.findall(text):
            if not allowed.match(reference):
                problems.append(
                    f"private feature path {reference!r} in {os.path.relpath(path, root)}"
                )

if problems:
    for problem in problems:
        print(f"FAIL: {problem}", file=sys.stderr)
    sys.exit(1)

print("module gate ok: feature tree declared, public interface only")
PY
