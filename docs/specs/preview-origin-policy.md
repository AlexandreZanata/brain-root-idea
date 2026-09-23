# Localhost preview origin policy

**Status:** approved policy for the future Preview role — **not yet enforced** (no preview WebView exists in the product)

**Reviewed:** 2026-09-23

**Scope:** the top-level navigation and new-window behavior of the Preview WebView (the user's local app shown in the Companion Canvas). It does not govern the shell, Human Browser, Agent Browser, subresource loads inside an approved document, or browser-data import.

**Related:** [Companion Browser plan](companion-browser-plan.md) · [Browser architecture](../08-browser-architecture.md) · [Security and permissions](../11-security-and-permissions.md) · [Linux child-WebView probe](linux-webview-probe.md) · ADR 0006

## Decision

The Preview role loads only the user's own project dev server on loopback, and only while BrainRoot owns that dev server. The policy is fail-closed: anything not explicitly allowed is denied, no prompt appears inside the preview, and the decision is made in the Rust core, never in the frontend.

## Allowed origins

A navigation is allowed only when every condition holds:

1. **Scheme:** `http` or `https`, exactly, after URL-parser normalization.
2. **Host:** the canonical loopback literal `127.0.0.1`, `::1`, or the name `localhost` (ASCII, case-insensitive). No userinfo component, no trailing dot, no other DNS name or IP form.
3. **Port:** explicitly present and registered as owned by a BrainRoot-managed dev server for the active project. Default ports (`80`, `443`) are not allowed, and an absent port is denied.
4. **Origin registry entry:** the active project has an owned dev-server registration; with no registration the preview cannot load anything.

Path, query, and fragment are unrestricted within an allowed origin. Every redirect and every new navigation event is evaluated again against these rules; an allowed origin never grants a redirect target in advance.

## Denied (fail-closed)

- **Schemes:** `file:`, `javascript:`, `data:`, `blob:`, `about:`, `view-source:`, `chrome:`, and any custom or external protocol handler.
- **Hosts:** any non-loopback address; DNS suffixes that merely contain `localhost` or `127.0.0.1` (`localhost.evil.example`, `127.0.0.1.evil.example`); trailing-dot forms (`localhost.`); alternate IP encodings (`2130706433`, `0x7f000001`, `017700000001`); unspecified or mapped forms (`0.0.0.0`, `[::ffff:127.0.0.1]`); any userinfo trick (`http://127.0.0.1@evil.example/`).
- **Ports:** unregistered or missing ports, and default ports.
- **Surfaces:** `window.open`, `target="_blank"`, popups, downloads, and permission prompts (camera, microphone, geolocation, clipboard, notifications, persistent storage). All are denied by default with no in-preview prompt in the first slice.
- **Privileged access:** any Tauri IPC from preview content. The preview capability scope exposes no core command; a content view invoking one is a defect, not a permission decision.
- **Parse failures:** a URL that cannot be parsed is denied.

## Denial behavior

- The navigation is cancelled; the previously approved document stays loaded, or a neutral blocked state is shown when the preview has no approved document.
- The user sees a plain-language reason and a retry action that re-checks the policy; the product language never exposes the raw policy internals outside technical details.
- Denials are logged without URLs or secrets beyond the scheme/host needed for diagnostics, and never retried automatically.

## Enforcement contract for CB-A A4

This policy ships as a gate; B08 deliberately adds no code, because no preview exists and dead policy code would not be verifiable. CB-A A4 must implement and prove, at minimum:

1. **Rust-owned decision:** the core owns the origin registry and the navigation decision; the frontend only expresses intent.
2. **Creation and navigation coverage:** the check runs when the preview is created (initial URL) and on every subsequent navigation event, including redirects and in-page pushes that cross origins.
3. **Owned-port registry:** the port comes from the active project's dev-server registration defined by the dev-server lifecycle contract; the registry is cleared when the project closes or the dev server stops.
4. **Fail-closed:** missing registry, parse errors, and unknown values deny.
5. **Lifecycle:** the preview WebView is destroyed on close with no listener, process, or profile left behind, following the measured cleanup rules.
6. **Product behavior:** a denied navigation produces the visible blocked state and never a silent blank page or an automatic retry loop.
7. **Tests:** the positive case (owned origin loads and reloads) and every negative case below pass deterministically, with a local fixture dev server and no remote destination.

### Negative test corpus (A4)

| Case | Expected |
|---|---|
| `http://127.0.0.1:<owned>/` | allowed |
| `http://127.0.0.1@evil.example/` | denied (userinfo) |
| `http://127.0.0.1.evil.example/` | denied (host suffix) |
| `http://localhost.evil.example/` | denied (host suffix) |
| `http://2130706433/`, `http://0x7f000001/` | denied (alternate IP encoding) |
| `http://0.0.0.0/`, `http://[::ffff:127.0.0.1]/` | denied (non-canonical loopback) |
| `http://localhost/`, `https://127.0.0.1:443/` | denied (missing/default port) |
| `http://127.0.0.1:<unowned>/` | denied (unregistered port) |
| `file:///etc/passwd`, `javascript:alert(1)`, `data:text/html,<p>` | denied (scheme) |
| owned origin that redirects to another origin | redirect denied, previous document kept |
| `window.open` or `target="_blank"` to any URL | denied, no new window |
| download response from the owned origin | denied by default |
| camera/microphone/geolocation/clipboard request | denied, no prompt |

## Non-goals

- Human Browser navigation, which is user-directed remote browsing with its own later policy.
- Browser-data import policy.
- Subresource requests inside an approved document: a local app may legitimately load fonts, scripts, and images from its own dev server or the public web; the trust boundary here is that preview content has no privileged IPC and cannot navigate the preview away from the owned loopback origin.
- HTTPS/dev-certificate handling and remote-device testing, which stay with the dev-server lifecycle contract and the ADR 0006 review.

## Open questions

- Origin discovery when a framework chooses a random port: the dev-server lifecycle contract must define how the owned port is learned and re-registered on restart.
- IPv4/IPv6 dual-stack: whether `localhost` resolution differences need a separate decision once a preview exists.
- Subresource policy: whether a later slice needs a documented subresource allowlist beyond the no-privileged-IPC boundary.
