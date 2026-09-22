#!/bin/sh
# BrainRoot distribution artifact manifest check.
#
# Verifies that the artifacts required in every distributed BrainRoot build are
# present and unmodified. Offline and dependency-free; safe to run in CI.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"

EXPECTED_LICENSE_SHA256="cfc7749b96f63bd31c3c42b5c471bf756814053e847c10f3eb003417bc523d30"
CANONICAL_PROJECT_REFERENCE="https://github.com/AlexandreZanata/brain-root-idea"
REQUIRED_ARTIFACTS="LICENSE NOTICE"

tmp_scan=""

cleanup() {
  if [ -n "$tmp_scan" ] && [ -f "$tmp_scan" ]; then
    rm -f "$tmp_scan"
  fi
}
trap cleanup EXIT INT TERM

fail() {
  printf 'FAIL: %s\n' "$1" >&2
  exit 1
}

sha256_of() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | cut -d' ' -f1
  else
    shasum -a 256 "$1" | cut -d' ' -f1
  fi
}

printf 'Required distribution artifacts: %s\n' "$REQUIRED_ARTIFACTS"

for artifact in $REQUIRED_ARTIFACTS; do
  [ -f "$ROOT/$artifact" ] || fail "required distribution artifact '$artifact' is missing"
  [ -s "$ROOT/$artifact" ] || fail "required distribution artifact '$artifact' is empty"
done

actual_license_sha="$(sha256_of "$ROOT/LICENSE")"
if [ "$actual_license_sha" != "$EXPECTED_LICENSE_SHA256" ]; then
  fail "LICENSE is not the pinned unmodified Apache License 2.0 text (expected sha256 $EXPECTED_LICENSE_SHA256, found $actual_license_sha)"
fi
printf 'LICENSE: unmodified Apache License 2.0 (sha256 %s)\n' "$actual_license_sha"

grep -q 'Apache License' "$ROOT/LICENSE" || fail "LICENSE is missing the 'Apache License' marker"
grep -q 'Version 2.0, January 2004' "$ROOT/LICENSE" || fail "LICENSE is missing the 'Version 2.0, January 2004' marker"

grep -q 'BrainRoot' "$ROOT/NOTICE" || fail "NOTICE is missing the BrainRoot project name"
grep -Fq -- "$CANONICAL_PROJECT_REFERENCE" "$ROOT/NOTICE" ||
  fail "NOTICE is missing the canonical project reference $CANONICAL_PROJECT_REFERENCE"
printf 'NOTICE: BrainRoot attribution present (canonical reference %s)\n' "$CANONICAL_PROJECT_REFERENCE"

tmp_scan="$(mktemp)"
find "$ROOT" \
  \( -name .git -o -name node_modules -o -name target \) -prune -o \
  \( -name Cargo.toml -o -name package.json \) -type f -print >"$tmp_scan"

metadata_count=0
while IFS= read -r metadata; do
  metadata_count=$((metadata_count + 1))
  if grep -Eq '^[[:space:]]*("license"|license)[[:space:]]*[:=][[:space:]]*"Apache-2\.0"' "$metadata"; then
    printf 'metadata: %s declares SPDX Apache-2.0\n' "$metadata"
  else
    fail "packaging metadata '$metadata' does not declare SPDX Apache-2.0"
  fi
done <"$tmp_scan"

if [ "$metadata_count" -eq 0 ]; then
  printf 'no packaging metadata yet (dormant check)\n'
fi

manifest_summary=""
for artifact in $REQUIRED_ARTIFACTS; do
  manifest_summary="$manifest_summary $artifact=verified"
done
printf 'manifest:%s\n' "$manifest_summary"
printf 'OK: distribution artifact manifest verified\n'
