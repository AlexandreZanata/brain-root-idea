# Companion Canvas

## Role

The Companion Canvas is the largest visual region and the primary evidence surface. It displays and accepts interaction with the thing BrainRoot is building or the resource needed to direct that work. It is a first-class subsystem with navigation, lifecycle, permission, focus, and recovery contracts—not an arbitrary iframe.

## MVP Canvas

The first Canvas supports one local preview surface:

- start or attach to a project dev server;
- readiness and failure state;
- safe navigation to approved local origins;
- reload and open externally;
- viewport presets for desktop and common responsive sizes;
- handoff to on-demand agent testing;
- visible boundary between project content and BrainRoot chrome.

Remote browsing, social destinations, multi-card Deck navigation, and phone swipe interaction are post-MVP.

The staged interaction, isolation, and import plan for a future integrated side browser is in [the Companion Browser plan](specs/companion-browser-plan.md). It preserves the local-preview-first MVP boundary.

## Surfaces

- **Preview:** localhost/project output; first-class MVP surface.
- **Agent Browser evidence:** screenshots, inspected UI, console, and test results generated in an isolated automation context.
- **Human Browser:** user-operated web content with its own profile and explicit sharing.
- **Document:** trusted local or remote reference rendered with restricted capabilities.
- **Code View:** optional inspection/editing surface, not the default Canvas.

These surface types do not share ambient permissions merely because they occupy the same rectangle.

## Canvas state

`EMPTY | STARTING | READY | STALE | FAILED | CLOSED`

READY requires successful navigation/readiness evidence. STALE means the last visual can remain visible but its backing process or content is not current. A failed preview must keep the last useful snapshot when possible and expose Retry or Technical details.

## Companion Deck, later

After prompt → build → preview → test → fix is excellent, a Deck may switch among Preview, Browser, Docs, and explicitly added destinations. Cards use lifecycle tiers:

- **HOT:** active native WebView; by default only one heavy content card.
- **WARM:** platform-specific reduced-resource state only when measured and supported.
- **COLD:** URL, safe navigation metadata, viewport/scroll hints where permitted, and a visual snapshot.
- **DEAD:** reconstruction metadata only.

Portable behavior is COLD reconstruction. WARM is an optimization, not a cross-platform promise.

## Companion Phone, later

Phone is a Deck presentation mode, not a TikTok feature. Presets include phone/9:16, square, landscape, desktop, and full canvas. Horizontal swipe or drag changes cards; vertical scrolling remains within a card. Buttons and keyboard commands must duplicate every gesture.

## Privacy and focus

Human browsing data is never automatically copied to agent context. Screenshots, DOM, console, network information, and cookies each require an applicable permission. BrainRoot must provide a reliable keyboard command to move focus out of embedded content.

