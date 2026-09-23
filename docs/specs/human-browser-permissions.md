# Human Browser permissions and browser-data clear (B16-S01)

**Status:** measured permission denial and clear-data behavior

**Reviewed:** 2026-09-23

**Related:** [ADR 0013](../adr/0013-human-browser-policy.md) · [Human Browser navigation record](human-browser-navigation.md) · [Human profile isolation probe](human-profile-isolation-probe.md)

## Permission denial

The Human Browser connects the WebKit `permission_request` signal and denies every request with no in-page prompt, recording `human_permission_denied:<kind>` in the browser status. The kind is read by downcasting the request:

| Request kind | Status code | Behavior |
|---|---|---|
| Geolocation | `human_permission_denied:geolocation` | denied |
| Camera/microphone (`getUserMedia`) | `human_permission_denied:media` | denied |
| Notifications | `human_permission_denied:notifications` | denied |
| Pointer lock | `human_permission_denied:pointer_lock` | denied |
| Device info | `human_permission_denied:device_info` | denied |
| Encrypted media | `human_permission_denied:media_key_system` | denied |
| Website data access | `human_permission_denied:website_data_access` | denied |
| Any future kind | `human_permission_denied:unknown` | denied |

Denial is fail-closed: the handler denies first and reports the code, so a new WebKit permission type is denied without a code change.

## Browser-data clear

`human_browser_clear_data` destroys the view, drops the cached `WebContext` when it matches the profile root, removes the BrainRoot-owned `human-profile` directory, and recreates it empty. In the current single-profile design this is both **clear site data** and **profile reset**: BrainRoot owns the whole profile, there is no separate settings store yet, and the next load starts with empty cookies, storage, and cache. The typed result is the reset `HumanStatus`.

The command honors the debug-only profile override so the harness exercises the real command path against a probe directory instead of wiping a user profile.

## Harness result

The human harness (`BRAINROOT_HUMAN_FIXTURE=1`, debug builds only) now triggers `navigator.geolocation.getCurrentPosition` and `navigator.mediaDevices.getUserMedia({ audio: true })` on its local fixture, then writes a `localStorage` marker, clears the browser data, reloads, and reads the marker again:

```json
{"decision":"go","page_a_reached":true,"page_b_reached":true,"back_ok":true,"forward_ok":true,
 "denial_code":"human_scheme_denied","permission_denied":true,"data_cleared":true,
 "hidden":true,"reasons":[]}
```

All `MEASURED` on the frozen Pop!_OS 24.04 reference environment (Wayland, WebKitGTK 2.52.6); the app exited `0` and left no process, listener, or probe profile behind. The permission check observed the geolocation/media denial path; the other kinds share the same handler.

## Canvas control (B16-S02)

The Browser surface shows the permission denial as a plain-language line ("This page asked for a permission. BrainRoot blocks camera, microphone, location, notifications, and clipboard in this browser.") and offers a **Clear browser data** action that reveals "Clear everything" / "Cancel" before running `human_browser_clear_data`; the action never runs without the explicit confirmation.

## Remaining

- Downloads and popups were already denied in B14; unsafe schemes are denied by the navigation policy.
- Site compatibility findings and the permission UX copy for denied requests land with the surface work; no permission is ever granted in the first slice.
