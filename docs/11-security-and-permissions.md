# Security and permissions

## Security objective

Limit the harm that compromised pages, malicious repository content, prompt injection, dependencies, agent mistakes, and stolen provider credentials can cause. BrainRoot combines AI, filesystem, shell, and browser capabilities; no single prompt or frontend approval is a sufficient boundary.

## Trust boundaries

- Rust Core and permission policy: trusted computing base, kept small.
- Shell UI: trusted presentation but untrusted input at core commands.
- Project files, instructions, dependencies, scripts, and build output: untrusted.
- Agent/provider messages and tool arguments: untrusted proposals.
- Preview and remote WebViews: hostile-capable web content.
- Human Browser profile: sensitive and unavailable to agents by default.
- Agent Browser: isolated, disposable, least privileged.
- Child processes: potentially hostile even when requested by a trusted agent.

## Initial threat model

| Threat | Example | Required controls |
|---|---|---|
| Prompt injection | README or webpage asks agent to exfiltrate `.env` | data/source labels, tool enforcement, secret detection, network permission, no ambient human browser access |
| Path escape | traversal or symlink reaches SSH keys | canonical path validation at use time, approved roots, OS sandbox where possible |
| Dangerous command | recursive deletion or disk formatting | command classification, narrow approval, sandbox, checkpoint, denylist as defense-in-depth only |
| Secret leakage | token enters prompt, log, screenshot, diff | OS credential store, redaction, explicit secret access, bounded retention |
| Malicious dependency | install script executes arbitrary code | dependency review, lockfiles, network/process scope, sandboxed execution roadmap |
| Browser identity theft | agent reads personal cookies | separate profiles/processes, no automatic sharing, scoped user-mediated bridge |
| Git destruction | reset/clean/rebase removes work | forbid destructive defaults, preserve user changes, preview restore, checkpoint metadata |
| UI/core confused deputy | remote WebView invokes privileged command | origin-specific Tauri capabilities, no privileged IPC for content views, core authorization |
| Resource abuse | fork bomb, runaway server, huge output | process groups, quotas, timeouts, bounded streams, cancellation, Governor |
| Supply chain/update compromise | tampered release | reproducible/signable builds goal, signed updates, SBOM, provenance and release policy |

## Modes

### Safe Mode — default

- Workspace-scoped read/write only after the user selects a root.
- No reading `.env`, credentials, SSH material, browser profiles, or paths outside scope by default.
- Commands run through policy with visible purpose, working directory, timeout, and resource limits where available.
- Network, package installation, downloads, publishing, and external browser access are separate permissions.
- Destructive actions are denied or require a precise approval and recoverable plan.
- Human Browser and Agent Browser remain isolated.

### Power Mode — explicit

Allows advanced grants such as broader paths, commands, or network access. Each grant remains scoped, reviewable, revocable, and auditable. Power Mode is not “trust everything,” cannot silently persist broad authority, and does not disable redaction or ownership cleanup.

## Permission record

Record principal, capability, action, resource scope, purpose, session/task, decision source, time, expiry, persistence, and revocation. The effective policy is deny by default. The user can inspect and revoke grants in plain language.

## Platform sandbox reality

- **macOS:** App Sandbox is kernel-enforced and user-selected folders can be persisted with security-scoped bookmarks. Running arbitrary project executables and App Store constraints require a feasibility spike. [Apple App Sandbox](https://developer.apple.com/documentation/security/protecting-user-data-with-app-sandbox) and [file access](https://developer.apple.com/documentation/security/accessing-files-from-the-macos-app-sandbox).
- **Windows:** AppContainer/Win32 app isolation can restrict files, registry, processes, windows, and network, but integrating ordinary toolchains needs a prototype. [AppContainer](https://learn.microsoft.com/en-us/windows/win32/secauthz/implementing-an-appcontainer).
- **Linux:** Flatpak provides an application sandbox and portals; Landlock can add unprivileged restrictions inherited by child processes where supported. Distribution/kernel variation prevents assuming either universally. [Flatpak concepts](https://docs.flatpak.org/en/latest/basic-concepts.html) and [Landlock](https://cdn.kernel.org/doc/html/latest/userspace-api/landlock.html).

Tauri capabilities authorize UI-to-core IPC but do not sandbox arbitrary commands. The initial product must report its actual enforcement level per platform and never market UI approval as containment.

## Secrets

Secrets never enter `.brainroot/`, SQLite logs, task history, screenshots, or source control. Store small provider credentials through macOS Keychain, Windows Credential Locker, and a compatible Linux Secret Service when available; define a secure failure UX when no store exists. [Apple Keychain](https://developer.apple.com/documentation/security/keychain-services) and [Windows Credential Locker](https://learn.microsoft.com/en-us/windows/apps/develop/security/credential-locker). Tauri Stronghold is an encrypted vault option, not automatically equivalent to OS credential storage and requires a key-unlock design.

## Security verification before MVP release

Threat-model review, path traversal/symlink tests, WebView origin/IPC tests, secret redaction fixtures, permission persistence/revocation tests, hostile repository prompt-injection tests, process-tree cleanup tests, dependency audit/SBOM, and platform sandbox capability matrix.


## MVP-0 implemented posture (2026-09-22)

Implemented and verified in the current release line: credentials live only in the Rust core (Linux Secret Service with an honest session-only fallback); no Tauri capability file grants filesystem, shell, HTTP, dialog, updater, or process permissions; outbound requests target only the documented `https://opencode.ai/zen/go/v1/…` endpoints with TLS; the frontend receives only the versioned neutral contract and can never read the key; provider text is never surfaced; `scripts/check-security.sh` enforces destinations, telemetry absence, capabilities, log hygiene, and the direct-dependency allowlists; `scripts/check-docs.sh` scans for secret patterns; and the built bundle was scanned with no credential or header material.

Known MVP-0 limitations, stated rather than hidden: the shipped Debian artifact is unsigned and experimental; there is no platform sandbox beyond Tauri's capability model yet; the live path needs a credential in the Secret Service and its cancellation latency is bounded by the transport read rather than measured; and the deterministic fake is available only in debug builds.
