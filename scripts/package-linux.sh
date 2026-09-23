#!/bin/sh
# BrainRoot packaged Linux artifact (.deb).
#
# Builds the documented Debian bundle through Tauri's own bundler and prints
# its identity. The artifact stays in the ignored target/ directory; only its
# checksum is recorded.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
BUNDLE_DIR="$ROOT/src-tauri/target/release/bundle/deb"

if [ -z "${BRAINROOT_PACKAGE_SKIP_BUILD:-}" ]; then
  echo "[package] building the Debian bundle (pnpm tauri build --bundles deb)"
  rm -f "$BUNDLE_DIR"/*.deb
  (cd "$ROOT" && pnpm tauri build --bundles deb)
fi

DEB="$(ls -t "$BUNDLE_DIR"/*.deb 2>/dev/null | head -1)"
[ -n "$DEB" ] || { printf 'FAIL: no .deb found in %s\n' "$BUNDLE_DIR" >&2; exit 1; }

echo "[package] artifact: ${DEB#"$ROOT/"}"
printf '[package] bytes=%s\n' "$(stat -c%s "$DEB")"
printf '[package] sha256=%s\n' "$(sha256sum "$DEB" | cut -d' ' -f1)"
for field in Package Version Architecture Installed-Size Depends; do
  value="$(dpkg-deb -f "$DEB" "$field" 2>/dev/null || true)"
  printf '[package] %s=%s\n' "$field" "${value:-unknown}"
done
echo "[package] contents:"
dpkg-deb -c "$DEB" | awk '{print $NF}' | sed 's|^\./||' | grep -v '/$' | sort -u | head -20
echo "[package] OK"
