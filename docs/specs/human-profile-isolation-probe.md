# Human Browser profile isolation probe (B13-S02)

**Status:** measured go for the profile isolation prerequisite — the Human Browser implementation remains post-MVP

**Reviewed:** 2026-09-23

**Related:** [ADR 0013](../adr/0013-human-browser-policy.md) · [Companion Browser plan](companion-browser-plan.md) · [Linux preview hosting probe](linux-preview-hosting-probe.md)

## Question

ADR 0013 and the Companion Browser plan require the Human Browser profile to be a BrainRoot-owned directory under the application data directory, separate from the Preview profile, the shell, the Agent Browser, and installed browsers, and to persist across close/reopen. The CB-B prerequisite asks to verify isolated profile feasibility and to fail closed where isolation cannot be demonstrated. This probe answers that through real `wry` views with separate `WebContext` data directories.

## Method

A disposable probe (`src-tauri/src/bin/human_profile_probe.rs`) built only with `--features probe` starts a local fixture server, prepares `preview-profile-probe` and `human-profile-probe` directories under the application data directory, and drives real views through a main-thread state machine:

1. human-profile view writes `localStorage["hb"] = "human-marker"` and reads the preview key (expect `empty`);
2. preview-profile view writes `localStorage["pv"] = "preview-marker"` and reads the human key (expect `empty`);
3. both views are destroyed, then a fresh human-profile view reads `hb` (expect `human-marker`) and `pv` (expect `empty`);
4. the probe removes its namespace, verifies removal, and prints one bounded JSON report.

Exact reproduction at the batch head:

```sh
pnpm build
cargo build --release --manifest-path src-tauri/Cargo.toml --features probe --bin human_profile_probe
timeout 300 ./src-tauri/target/release/human_profile_probe
```

## Environment

- Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6 (`MEASURED`).
- Tauri `2.11.6`, `wry 0.55.1`, `gtk 0.18.2`.

## Results

One release run returned `"decision":"go"` with empty stderr and exit `0`; no app process, listener, or probe directory remained.

| Check | Expected | Measured |
|---|---|---|
| human profile write | `stored:human-marker` with preview key empty | `stored:human-marker\|empty` |
| preview profile write | `stored:preview-marker` with human key empty | `stored:preview-marker\|empty` |
| human profile after destroy/recreate | human marker present, preview key empty | `read:human-marker\|empty` |
| isolation | both cross-profile reads empty | `true` |
| persistence | marker survives view destroy/recreate | `true` |
| profile paths | inside the application data directory and distinct | `within_app_data: true`, distinct |
| probe namespace cleanup | removed after the run | `probe_dir_removed: true` |
| probe namespace size | recorded | 50 323 bytes |

The full report:

```json
{"decision":"go","human_read_after_reopen":"read:human-marker|empty","human_write_title":"stored:human-marker|empty",
 "isolation":true,"persistence":true,"preview_write_title":"stored:preview-marker|empty","probe_dir_removed":true,
 "probe_dir_size_bytes":50323,"reasons":[],"within_app_data":true}
```

## Interpretation

- Two role profiles with separate `WebContext` data directories are isolated in both directions: each profile's `localStorage` is invisible to the other.
- A role profile persists across view destroy/recreate, which is the close/reopen and COLD-reconstruction case the Deck contract needs.
- The profile directories are BrainRoot-owned paths under the application data directory and never point at an installed browser; the probe only uses its own namespace and removes it.
- The disk footprint of a fresh profile after this minimal run is about 50 KB, which is a `MEASURED` starting point, not a budget.

## Decision

- **Go for the profile isolation prerequisite.** The isolation and persistence behavior required by CB-B is demonstrated with the same `wry`/WebKitGTK stack the product uses.
- The CB-B implementation still owns the remaining evidence: cookie and cache isolation across roles, deletion/reset behavior with confirmation, crash recovery, keyboard/focus and accessibility checks inside remote content, and compatibility findings reported rather than concealed.
