# BrainRoot MVP-0 Linux artifact

**Status:** Documented artifact (test release, not bit-reproducible)
**Date/commit:** 2026-09-22 · `a926c14` (B05-S04 worktree)
**Version:** `0.0.1-alpha.5`
**Owner:** BrainRoot maintainer (B05-S04)

## Claim

BrainRoot ships exactly one packaged Linux artifact: a Debian package built by Tauri's own bundler for the frozen reference environment. It targets Ubuntu/Debian-family x86_64 systems only; no universal Linux compatibility is claimed. The build steps are reproducible, but the checksum is per-build (the deb archives build timestamps), so every release records the checksum of the artifact it actually ships.

## Identity

| Field | Value |
|---|---|
| Artifact | `src-tauri/target/release/bundle/deb/BrainRoot_0.0.1-alpha.5_amd64.deb` (git-ignored) |
| Size | 3,384,838 bytes (latest local build; a previous build was 3,384,842) |
| SHA-256 | `ea4ca51bc24a007cf2b91729e3479774c88508d056f2428a46678ff484f3ccaa` (latest local build) |
| Package | `brain-root` |
| Version | `0.0.1-alpha.5` |
| Architecture | `amd64` |
| Installed size | 7,775 KiB |
| Runtime dependencies | `libwebkit2gtk-4.1-0`, `libgtk-3-0` |
| Contents | `/usr/bin/brainroot`, `/usr/share/applications/BrainRoot.desktop`, hicolor icons (32×32, 128×128, 128×128@2x, 256×256@2x); no resource directory |

## Build

- Command: `sh scripts/package-linux.sh` (wraps `pnpm tauri build --bundles deb` and prints the identity above).
- Tooling: Tauri CLI 2.11.5 with the deb bundler (`dpkg-deb` 1.22.6), Rust 1.96.0, pnpm 11.13.0, frontend `vite build`.
- The bundler embeds the frontend in the binary; the package installs a single executable plus desktop integration files.
- Re-running the build produces the same steps but not the same bytes; record the checksum from the build that is shipped.

## Install, run, remove

```sh
sudo apt install ./BrainRoot_0.0.1-alpha.5_amd64.deb   # resolves the declared dependencies
brainroot                                              # run the shell
sudo apt remove brain-root                             # remove; user state is untouched
```

- `apt` (not bare `dpkg -i`) resolves `libwebkit2gtk-4.1-0` and `libgtk-3-0` from the distribution when they are missing.
- These steps are documented but were not executed here because installation needs root; B05-S07 verifies the published artifact from a clean environment before the release is marked complete.

## Supported reference environment

- Pop!_OS 24.04 LTS (Ubuntu 24.04 family), kernel `7.1.5-76070105-generic`, x86_64, Wayland, WebKitGTK 2.52.6, `libgtk-3-0`.
- No other distribution, architecture, or desktop environment was tested; Windows and macOS are explicitly out of scope for this artifact.

## Known limitations

- Unsigned and unverified by a package repository; there is no auto-update path.
- Experimental prerelease (`0.0.1-alpha.5`); the first release artifact is rebuilt at `0.0.1` in B05-S06/S07 and attached to the pre-release with its own checksum.
- The memory TARGET fails on the reference environment (426.2 MB summed RSS / 241.6 MB PSS against the 150 MB target, see `performance-reports/b05-mvp0-soak.md`).
- The live OpenCode Go path needs a credential in the Linux Secret Service; the deterministic fake is debug-only and the packaged release build does not expose it.
- Debian-family x86_64 only; the `.deb` name and dependencies are distribution-specific.
