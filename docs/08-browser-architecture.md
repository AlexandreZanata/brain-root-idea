# Browser architecture

## Four separate concerns

1. **Shell WebView:** trusted BrainRoot UI with narrowly scoped Tauri capabilities.
2. **Preview WebView:** project localhost content; untrusted relative to the core.
3. **Human Browser:** user-controlled authenticated browsing; private by default.
4. **Agent Browser:** automation context for QA; isolated profile, ephemeral by default.

Origin, profile, cookies, cache, storage, IPC capability, navigation, downloads, popups, clipboard, and agent inspection policies are explicit for each type. Never load arbitrary remote content into a WebView holding privileged shell capabilities.

## MVP strategy

Use one primary shell WebView plus one on-demand preview child WebView if the platform implementation proves reliable. The preview receives no privileged core API by default. Agent QA runs Playwright as an on-demand child process against localhost rather than reusing a human browsing session.

On Linux the platform evidence is now in: the managed child-WebView path fails geometry (B08-S01), and the preview is a `wry` WebView hosted in a `GtkFixed` overlay inside the same window, outside the Tauri webview manager and with no Tauri IPC by construction (B09-S01). See [ADR 0012](adr/0012-linux-preview-hosting.md) and the [hosting probe](specs/linux-preview-hosting-probe.md).

## Platform facts and limits

- Windows uses Edge WebView2, macOS uses WKWebView, and Linux uses WebKitGTK through WRY/Tauri.
- WRY is a cross-platform abstraction, not identical browser behavior.
- Tauri's background-throttling control is supported on recent Apple platforms but documented as unsupported on Windows and Linux. It is not a portable suspend/resume API.
- Windows WRY exposes a platform extension for lower WebView2 memory targets; equivalent behavior is not promised elsewhere.
- Web storage, process lifetime, cookie behavior, media, downloads, focus, and snapshot fidelity need platform tests.

Sources: [Tauri process model](https://v2.tauri.app/concept/process-model/), [Tauri Webview API](https://github.com/tauri-apps/tauri/blob/dev/packages/api/src/webview.ts), and [WRY repository/changelog](https://github.com/tauri-apps/wry).

## Lifecycle strategy

The portable Deck contract is:

1. record safe reconstructable metadata;
2. capture a visual placeholder where permitted;
3. destroy the old heavy content WebView;
4. create the selected WebView with the correct isolated profile/policy;
5. navigate and restore only supported nonsecret state;
6. swap the placeholder after readiness.

HOT/WARM/COLD/DEAD are product lifecycle states. WARM maps to native throttling only on platforms where behavior is supported and benchmarked. There is no claim that arbitrary authenticated pages can be perfectly restored.

## Human versus agent access

The agent may use its own browser context for localhost, screenshots, DOM, console, network, and interaction. Access to a Human Browser page requires a specific user-mediated share action describing exactly what becomes visible. Cookies and authenticated storage are never copied to the agent profile automatically.

## Navigation and content policy

- Allow only expected localhost origins in MVP preview, exactly as defined by the [localhost preview origin policy](specs/preview-origin-policy.md).
- Deny privileged IPC from remote and preview origins.
- Open unapproved schemes and external domains through an explicit decision.
- Restrict downloads, file URLs, new windows, permission prompts, and cross-origin navigation.
- Redact tokens and sensitive form values from logs and artifacts.
- Treat page content as prompt-injection-capable untrusted input.

## Verification spike before implementation commitment

Build a minimal multi-platform probe measuring create/destroy latency, memory release, focus, resize, localhost navigation, crash recovery, cookie isolation, storage directory behavior, screenshots, background throttling, and repeated lifecycle leaks. Results become MEASURED entries in the performance contract and may revise ADR 0006.

The [Companion Browser plan](specs/companion-browser-plan.md) sequences Linux local preview, an isolated Human Browser, opt-in data portability, Deck switching, and phone-width presentation. Importing portable bookmarks is not equivalent to reusing Chrome/Firefox profiles, cookies, passwords, or extensions.

