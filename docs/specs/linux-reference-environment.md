# Linux reference environment and dependency decisions

**Status:** Frozen for batch B01 (2026-09-22)
**Scope:** MVP-0 Linux model loop — the platform reference for B01 measurements and scaffolding.

## Purpose

Records the measured Linux reference environment, the installed Tauri build prerequisites, and the approved direct dependency versions for the first Linux shell. B01-S02 scaffolds with these versions; a later batch changes them only through an updated record with fresh evidence.

## Reference environment

Measured on 2026-09-22 with the commands shown.

| Fact | Value | Command |
|---|---|---|
| Distribution | Pop!_OS 24.04 LTS (`ID=pop`, `ID_LIKE="ubuntu debian"`) | `cat /etc/os-release` |
| Kernel | `7.1.5-76070105-generic` | `uname -srmo` |
| Architecture | x86_64 | `uname -m` |
| Display protocol | wayland session with XWayland (`DISPLAY=:1`, `WAYLAND_DISPLAY=wayland-1`) | `echo "$XDG_SESSION_TYPE"` |
| CPU | 16 logical CPUs | `nproc` |
| Memory | 31.0 GiB | `awk '/MemTotal/{printf "%.1f GiB\n", $2/1048576}' /proc/meminfo` |
| Storage | 460 GB total, 57 GB free on `/` | `df -h /` |

## Toolchain

| Tool | Version | Command |
|---|---|---|
| rustc / cargo | 1.96.0 | `rustc --version`, `cargo --version` |
| Node.js | v26.3.1 | `node --version` |
| npm | 12.0.2 | `npm --version` |
| pnpm (chosen package manager) | 11.13.0 | `pnpm --version` |
| gcc | 13.3.0 | `gcc --version` |
| pkg-config | 1.8.1 | `pkg-config --version` |

## System build prerequisites

Source: official Tauri v2 Debian/Ubuntu list at <https://v2.tauri.app/start/prerequisites/> (read 2026-09-22). Installed by the maintainer on 2026-09-22 with:

```sh
sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev build-essential curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev
```

| Package | Installed version | `pkg-config` version |
|---|---|---|
| libwebkit2gtk-4.1-dev | 2.52.6-0ubuntu0.24.04.1 | webkit2gtk-4.1 2.52.6, javascriptcoregtk-4.1 2.52.6 |
| libgtk-3-dev | 3.24.41-4ubuntu1.3 | gtk+-3.0 3.24.41 |
| libsoup-3.0-dev | 3.4.4-5ubuntu0.8 | libsoup-3.0 3.4.4 |
| librsvg2-dev | 2.58.0+dfsg-1build1 | librsvg-2.0 2.58.0 |
| libxdo-dev | 1:3.20160805.1-5build1 | no pkg-config file; the apt package is installed |
| libssl-dev | 3.0.13-0ubuntu3.15 | openssl 3.0.13 |
| libayatana-appindicator3-dev | 0.5.93-1build3 | ayatana-appindicator3-0.1 0.5.90 |
| build-essential | 12.10ubuntu1 | gcc 13.3.0 |

The agent environment cannot use `sudo` ("no new privileges"), so reinstalling or adding system packages remains a maintainer action.

## Dependency decision record

Decision basis: Tauri 2 is fixed by [ADR 0001](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/adr/0001-desktop-runtime.md) and Svelte + TypeScript by [ADR 0003](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/adr/0003-frontend-stack.md). Versions were verified on 2026-09-22; the Tauri 3.0.0-alpha line is not used.

Rust:

- `tauri` 2.11.6 — license Apache-2.0 OR MIT, MSRV 1.77.2 (satisfied by rustc 1.96.0), published 2026-09-19.
- `tauri-build` 2.x (max stable 2.6.3) — build-time code paired with tauri.
- `wry` 0.57.0 — transitive through tauri, recorded for advisory tracking.

Frontend:

- `@tauri-apps/cli` 2.11.5, `@tauri-apps/api` 2.11.1 — license Apache-2.0 OR MIT.
- `svelte` 5.57.1, `@sveltejs/vite-plugin-svelte` 7.3.0, `vite` 8.3.0, `typescript` 6.0.3 plus `@typescript/native` 7.0.2, `svelte-check` 4.7.6.
- TypeScript decision (updated 2026-09-22 by maintainer decision): the project is checked with **TypeScript 7.0.2** through the officially supported `svelte-check` path — `typescript@~6` remains installed as the required compatibility backend and the native TS 7 compiler is aliased as `@typescript/native@npm:typescript@7.0.2`; the `check`/`check:watch` scripts use `svelte-check --tsgo`. Installing `typescript@7.0.2` alone breaks `svelte-check` 4.7.6 with "TypeScript 7 support currently requires both TypeScript 7 and TypeScript 6 installed in your project". Verified in an isolated copy: `svelte-check found 0 errors and 0 warnings` and `vite build` exit 0.
- Package manager: pnpm 11.13.0 (maintainer decision). Node engines: `vite` 8.3.0 requires `^20.19.0 || >=22.12.0`, satisfied by Node.js v26.3.1.

| Package | Version | License | Basis |
|---|---|---|---|
| tauri | 2.11.6 | Apache-2.0 OR MIT | ADR 0001; stable at verification date |
| tauri-build | 2.x (2.6.3 max stable) | Apache-2.0 OR MIT | Build dependency of the Tauri app |
| wry | 0.57.0 | Apache-2.0 OR MIT | Transitive; tracked for advisories |
| @tauri-apps/cli | 2.11.5 | Apache-2.0 OR MIT | Scaffold and build CLI |
| @tauri-apps/api | 2.11.1 | Apache-2.0 OR MIT | Frontend-to-core IPC |
| svelte | 5.57.1 | MIT | ADR 0003 |
| @sveltejs/vite-plugin-svelte | 7.3.0 | MIT | Peers `vite ^8.0.0 || ^8.0.0-beta.7`, `svelte ^5.46.4` |
| vite | 8.3.0 | MIT | Frontend build tool |
| typescript | 6.0.3 | Apache-2.0 | Compatibility backend required by `svelte-check` 4.7.6 alongside the TS 7 native compiler |
| @typescript/native | 7.0.2 (`npm:typescript@7.0.2`) | Apache-2.0 | TypeScript 7.0.2 native compiler used for checking via `svelte-check --tsgo` |
| svelte-check | 4.7.6 | MIT | Type and accessibility checking |

## Advisory check

- Source: OSV query API <https://api.osv.dev/v1/query> (queried 2026-09-22; `typescript` 6.0.3 and 7.0.2 re-queried on the same date during the B01-S01 update).
- Queried versions: `tauri` 2.11.6, `wry` 0.57.0, `tauri-build` 2.6.3, `svelte` 5.57.1, `vite` 8.3.0, `@tauri-apps/api` 2.11.1, `@tauri-apps/cli` 2.11.5, `svelte-check` 4.7.6, `@sveltejs/vite-plugin-svelte` 7.3.0, `typescript` 6.0.3, `typescript` 7.0.2.
- Result: none for every queried version.
- The resolved lockfile receives a fresh advisory check in B01-S02; this table is not a substitute for it.

## Installation and maintenance

- The Tauri prerequisites were installed by the maintainer on 2026-09-22; this agent environment cannot run `sudo`, so package installation is always a maintainer action.
- Release recency at verification: `tauri` 2.11.6 was published 2026-09-19, three days before this record.
- Tauri, Svelte, Vite, and TypeScript are actively maintained; review and refresh this record whenever a batch changes a pinned version.

## Deferred measurements

Resolved transitive dependency counts require the lockfile that B01-S02 creates. Measure with `cargo tree --prefix none | wc -l` and `pnpm list --depth Infinity`, then record the numbers in the batch history instead of estimating them here.

## Smoke test coverage and limitations

`scripts/smoke-linux.sh` runs on the reference session (not in CI, which has no display) and asserts:

- the release binary launches, stays alive, and creates exactly one `WebKitWebProcess` and one `WebKitNetworkProcess` as direct children (matched through `/proc/<pid>/cmdline`, never through global WebKit patterns);
- the readiness marker `brainroot: health contract v… served` proves the frontend loaded and completed the typed health round-trip;
- `SIGTERM` shuts the main process and both children down, and no listener remains afterward.

Limitations, recorded instead of worked around: no UI automation is approved, so the window-close path and the rendered `Ready` label are not asserted automatically; `SIGTERM` is the shutdown trigger and the readiness marker is the Rust-side proof of the frontend contract call. The smoke test is deliberately excluded from `check-fast`, `check-full-linux`, and CI.

Production builds must go through `pnpm tauri build --no-bundle` (or enable `tauri/custom-protocol` explicitly): a plain `cargo build --release` does not enable the CLI-managed `custom-protocol` feature, so the binary keeps development semantics and does not serve the embedded frontend. This was measured on 2026-09-22 while validating the smoke test.

## Sources

- Tauri Linux prerequisites: <https://v2.tauri.app/start/prerequisites/> (read 2026-09-22).
- crates.io API: <https://crates.io/api/v1/crates/tauri> and version metadata (2026-09-22).
- npm registry: <https://registry.npmjs.org> package metadata (2026-09-22).
- OSV advisory API: <https://api.osv.dev/v1/query> (2026-09-22).
