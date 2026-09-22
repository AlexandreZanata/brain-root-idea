# B01 — Create the measured Linux shell

> Repository mirror of the [Wiki batch page](https://github.com/AlexandreZanata/brain-root-idea/wiki/Batch-B01-Linux-Shell). The Wiki is the live operations journal; this file is the reviewed, versioned record that is finalized before the batch merges.

## Header

- Status: Ready to merge
- Objective: BrainRoot opens a minimal Linux window and exits cleanly on the frozen reference environment, with a typed core/UI health contract, deterministic repository-owned check scripts, a first measured performance baseline, and reproducible CI — still without project opening, provider, chat, terminal, or second WebView.
- Branch: `batch/b01-linux-shell`
- Draft/final PR: [#9](https://github.com/AlexandreZanata/brain-root-idea/pull/9)
- Baseline commit: `cd35efb8904de5156f7b97b097a65e9e5d4ff55f`
- Target/resulting version: `0.0.1-alpha.2` (annotated tag on the merge commit; pre-release)
- Started/completed: 2026-09-22 / —
- Supported test environment: Pop!_OS 24.04 LTS, kernel `7.1.5-76070105-generic`, Wayland, WebKitGTK 2.52.6, rustc 1.96.0 (pinned), Node.js v26.3.1, pnpm 11.13.0, TypeScript 7.0.2 via `@typescript/native`

## Non-goals

- Project opening, provider calls, credentials, chat, terminal, editor, extra WebView, preview, Playwright, checkpoints.
- Optional UI, state, formatting, or test libraries beyond the approved version record.
- Windows, macOS, updater, cloud sync, plugins, publishing.

## Running microstep log

| Step | Issue | Outcome | Commit | Validation evidence | Deviation/failure | Status |
|---|---|---|---|---|---|---|
| B01-S01 | [#8](https://github.com/AlexandreZanata/brain-root-idea/issues/8) | Reference environment, toolchain, Tauri prerequisites, and approved dependency versions recorded with license, MSRV/engines, maintenance, and OSV evidence (updated for TypeScript 7.0.2 after reopening) | `6ef0f25`, `95f0def` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/8#issuecomment-5777333093), [update](https://github.com/AlexandreZanata/brain-root-idea/issues/8#issuecomment-5777753634) | System prerequisites needed a maintainer `sudo` action; TS 7 requires the `@typescript/native` alias plus the TS 6 backend and `--tsgo` | Closed |
| B01-S02 | [#10](https://github.com/AlexandreZanata/brain-root-idea/issues/10) | Minimal shell with plain Svelte 5 + Vite + TypeScript, Tauri side from the official template; manifests carry the version and Apache-2.0; no IPC capabilities | `8fdf5c2` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/10#issuecomment-5777470365) | Official template generates SvelteKit (outside the approved record); version check validated one manifest — gap deferred to B01-S04 | Closed |
| B01-S03 | [#11](https://github.com/AlexandreZanata/brain-root-idea/issues/11) | Versioned typed `health` command with `deny_unknown_fields` and typed errors; six Rust tests; UI shows `Ready` only after the core result, one request, no polling | `90a08e3` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/11#issuecomment-5777560559) | `svelte-check` misread `$state` when a local variable was named `state`; renamed to `healthState`. Capability expectation corrected: app commands need none | Closed |
| B01-S04 | [#12](https://github.com/AlexandreZanata/brain-root-idea/issues/12) | `check-fast` and `check-full-linux` entry points with a deliberate fixture; version check extended to `src-tauri/Cargo.toml`; CI pinned to Rust 1.96.0 and running the full gate | `01d0993`, `f4eeee8` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/12#issuecomment-5777671123), [correction](https://github.com/AlexandreZanata/brain-root-idea/issues/12#issuecomment-5778139141) | Release build lacked the CLI-managed `custom-protocol` feature (dev semantics); corrected to `pnpm tauri build --no-bundle` after reopening | Closed |
| B01-S05 | [#13](https://github.com/AlexandreZanata/brain-root-idea/issues/13) | Production smoke test: readiness marker, exactly one web and one network process, SIGTERM shutdown, zero leftovers or listeners; readiness line added to `health()` | `f629231` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/13#issuecomment-5778314455) | Baseline guard rewritten to `/proc/<pid>/exe`; cleanup kills the process group; UI automation not approved (limitation recorded) | Closed |
| B01-S06 | [#14](https://github.com/AlexandreZanata/brain-root-idea/issues/14) | First MEASURED baseline: startup p50 0.282 s, idle CPU median 0.172 %, memory 198.5 MB PSS / 418.9 MB VmRSS, bundle sizes, 3 clean cycles | `9625d89` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/14#issuecomment-5778564138) | Memory TARGET fails and is recorded as failing; CPU stays a warn after one 1.310 % window | Closed |
| B01-S07 | [#15](https://github.com/AlexandreZanata/brain-root-idea/issues/15) | Version `0.0.1-alpha.2` synchronized across `VERSION`, `Cargo.toml`, `package.json`, and the lockfile; changelog release section; full CI on the latest head; PR merged with the documented administrator exception; annotated tag and pre-release | `<merge>` | [evidence](https://github.com/AlexandreZanata/brain-root-idea/issues/15) | Ecosystem copies added to the allowlist to satisfy the version policy; final links land in the post-merge record commit | Ready |

## Decisions and changed assumptions

- Decision (B01-S01, maintainer): pnpm 11.13.0 is the frontend package manager; the Tauri 3.0.0-alpha line is not used (ADR 0001).
- Decision (B01-S01, updated): the project is checked with TypeScript 7.0.2 through `@typescript/native@npm:typescript@7.0.2` plus the required `typescript@~6.0.3` backend and `svelte-check --tsgo`.
- Decision (B01-S02): plain Svelte 5 + Vite with the approved versions; the official template is used only for the Rust/Tauri side; no capability file and no `@tauri-apps/api` until the health contract.
- Decision (B01-S04): `check-full-linux` runs in CI; Rust is pinned to 1.96.0 with clippy and rustfmt; the governance template check stays out of CI for the runner PyYAML constraint.
- Decision (B01-S04, corrected): production builds go through `pnpm tauri build --no-bundle`; plain `cargo build --release` leaves development semantics.
- Decision (B01-S05): readiness is proven through the Rust marker and the two WebKit children; `SIGTERM` is the shutdown trigger; UI automation is not approved.
- Decision (B01-S06): the baseline is MEASURED only for the reference environment; the memory target fails and is not adjusted; the measurement script stays out of the gates and CI.

## Failures and recovery

- 2026-09-22, B01-S02: the official `svelte-ts` template generated SvelteKit, outside the approved record and against ADR 0003; recovered with a plain Svelte frontend and the template's Rust side only.
- 2026-09-22, B01-S02: the version check validated one manifest; closed in B01-S04 by covering `src-tauri/Cargo.toml`.
- 2026-09-22, B01-S03: `svelte-check` treated the `$state` rune as a legacy store subscription with a local variable named `state`; renamed to `healthState`.
- 2026-09-22, B01-S04/S05: `cargo build --release` produced a development-semantics binary because the CLI-managed `custom-protocol` feature was missing; the gate and the smoke test now build with the Tauri CLI, and a diagnostic detour (stale embed assumption, `strings` on compressed assets) was discarded rather than reported as fact.
- 2026-09-22, B01-S05: the baseline guard false-positived on the caller's command line and the process-group cleanup leaked a fixture grandchild under `dash`; both fixed and proven by negatives.

## Final gates

- Full CI: `check-full-linux` on the batch head and the post-merge `push` run on `main` — recorded in issue #15
- Review: single-maintainer exception; merged with the documented administrator bypass
- Security/privacy: secret-pattern scans in the gates, manual diff review, zero IPC permissions, no new destination
- Performance: [first baseline](https://github.com/AlexandreZanata/brain-root-idea/blob/main/docs/specs/performance-reports/b01-linux-baseline.md) — startup and idle CPU within target; memory target fails and is recorded
- Cleanup: smoke test and three open/close cycles return to zero processes and no listeners
- Artifact/checksum: none for this batch; the pre-release states the absence explicitly
- Known limitations: memory TARGET fails; idle CPU stability warn; no UI automation; governance template check outside CI

## Result and next batch

In progress. Next microstep: B01-S07 finalization. Batch merges only after every B01 issue closes with evidence, full Linux CI is green on the latest head, and this record matches the Wiki batch page.
