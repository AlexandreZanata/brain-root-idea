#!/bin/sh
# BrainRoot governance template check.
#
# Validates the GitHub issue form and pull request template under .github/.
# Offline and deterministic; the same check is intended for the batch CI gate.
set -eu

ROOT="${1:-}"
if [ -z "$ROOT" ]; then
  ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
fi

form="$ROOT/.github/ISSUE_TEMPLATE/microstep.yml"
pr="$ROOT/.github/PULL_REQUEST_TEMPLATE.md"

[ -f "$form" ] || { printf 'FAIL: missing %s\n' "$form" >&2; exit 1; }
[ -f "$pr" ] || { printf 'FAIL: missing %s\n' "$pr" >&2; exit 1; }

python3 - "$form" "$pr" <<'PY'
import sys

try:
    import yaml
except ImportError:
    sys.exit("FAIL: PyYAML is required for governance template validation")

form_path, pr_path = sys.argv[1], sys.argv[2]

with open(form_path, encoding="utf-8") as handle:
    form = yaml.safe_load(handle)

if not isinstance(form, dict):
    sys.exit("FAIL: issue form is not a YAML mapping")

allowed_top = {"name", "description", "title", "labels", "assignees", "body"}
unknown_top = sorted(set(form) - allowed_top)
if unknown_top:
    sys.exit(f"FAIL: unknown issue form keys: {unknown_top}")
for key in ("name", "description", "body"):
    if not form.get(key):
        sys.exit(f"FAIL: issue form is missing '{key}'")

body = form["body"]
if not isinstance(body, list) or not body:
    sys.exit("FAIL: issue form 'body' must be a non-empty list")

allowed_types = {"markdown", "input", "textarea", "dropdown", "checkboxes"}
expected_fields = {
    "batch-step-id", "batch-branch", "batch-pr", "starting-commit",
    "milestone", "risk", "agent-profile", "outcome", "required-reading",
    "facts", "allowed-changes", "forbidden-scope", "implementation-sequence",
    "dependency-authorized", "validation", "rollback", "stop-conditions",
}
expected_checklists = {
    "security-cleanup": 4,
    "definition-of-ready": 5,
    "definition-of-done": 6,
}

seen_ids = set()
for element in body:
    if not isinstance(element, dict):
        sys.exit("FAIL: issue form body elements must be mappings")
    kind = element.get("type")
    if kind not in allowed_types:
        sys.exit(f"FAIL: unsupported body element type: {kind!r}")
    attributes = element.get("attributes")
    if not isinstance(attributes, dict):
        sys.exit(f"FAIL: body element '{kind}' is missing attributes")
    if kind == "markdown":
        if not attributes.get("value"):
            sys.exit("FAIL: markdown element requires a non-empty value")
        continue
    element_id = element.get("id")
    if not element_id:
        sys.exit(f"FAIL: {kind} element is missing an id")
    if element_id in seen_ids:
        sys.exit(f"FAIL: duplicate element id: {element_id}")
    seen_ids.add(element_id)
    if not attributes.get("label"):
        sys.exit(f"FAIL: element '{element_id}' is missing a label")
    if element_id in expected_fields and not element.get("validations", {}).get("required"):
        sys.exit(f"FAIL: element '{element_id}' must be required")
    if element_id in expected_checklists:
        options = attributes.get("options")
        if kind != "checkboxes" or not isinstance(options, list) or len(options) != expected_checklists[element_id]:
            sys.exit(f"FAIL: checklist '{element_id}' must contain {expected_checklists[element_id]} options")

missing_fields = sorted(expected_fields - seen_ids)
if missing_fields:
    sys.exit(f"FAIL: issue form is missing required elements: {missing_fields}")
missing_checklists = sorted(set(expected_checklists) - seen_ids)
if missing_checklists:
    sys.exit(f"FAIL: issue form is missing checklists: {missing_checklists}")

with open(pr_path, encoding="utf-8") as handle:
    pr = handle.read()

required_sections = [
    "# [Bxx]",
    "## Outcome",
    "## Branch and release",
    "## Explicitly out of scope",
    "## Ordered microsteps",
    "## Risk summary",
    "## Dependency changes",
    "## Final evidence",
    "## Merge gate",
]
for section in required_sections:
    if section not in pr:
        sys.exit(f"FAIL: PR template is missing section: {section}")
if "Refs #N" not in pr:
    sys.exit("FAIL: PR template must keep the 'Refs #N' guidance")
if pr.count("- [ ]") < 8:
    sys.exit("FAIL: PR template must keep at least 8 checklist items")

print(f"issue form ok: {len(body)} body elements, {len(seen_ids)} fields")
print(f"pr template ok: {len(required_sections)} required sections")
print("OK: governance templates validated")
PY
