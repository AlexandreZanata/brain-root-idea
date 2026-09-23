# ADR 0013: Human Browser policy

**Status:** Accepted  
**Date:** 2026-09-23

## Context

The [Companion Browser plan](../specs/companion-browser-plan.md) sequences Linux local preview (CB-A, released through `v0.0.8`), then an isolated Human Browser and the Deck (CB-B, post-MVP Phase 8), opt-in data portability (CB-C), and phone-width presentation (CB-D). CB-B requires three prerequisites: the CB-A gate, isolated profile feasibility, and an explicit Human Browser policy. CB-A is complete; the profile isolation probe is B13-S02; this ADR supplies the policy so the implementation batch has fixed boundaries instead of inventing them.

ADR 0006 already separates the four WebView roles and forbids arbitrary remote content in a privileged shell; `docs/08-browser-architecture.md` states that agent access to a Human Browser page needs a specific user-mediated share action; `docs/11-security-and-permissions.md` treats the Human Browser profile as sensitive and unavailable to agents by default. The plan still lists a required decision — arbitrary websites versus a curated destination set — and `docs/17-open-questions.md` keeps profile creation, cookies, downloads, external navigation, sharing, retention, and deletion open.

The roadmap keeps MVP-1 Phases 3, 5, 6, and 7 ahead of the Phase 8 Deck/Human Browser. Approving this policy is prerequisite work; it does not reorder the roadmap.

## Decision

1. **Role and trust boundary.** The Human Browser is a fourth trust role for user-directed remote content, distinct from the shell, the Preview, and the Agent Browser. It has no BrainRoot IPC of any kind, no automatic agent access, and no credential or cookie bridge to the Agent Browser or any installed browser. It never reads an installed Chrome, Firefox, or other browser profile or directory.
2. **Navigation.** The first slice supports arbitrary `http` and `https` after an explicit user direction in the Canvas. `file:`, `javascript:`, `data:`, `blob:`, `about:`, `view-source:`, application-internal schemes, and external protocol handlers are denied. Popups and `target="_blank"` become an explicit user action instead of opening silently. Downloads are denied in the first slice. Every navigation is evaluated by the Rust core; a typed decision (`allow`, `deny`, `open-external`) is the only interface the frontend sees.
3. **Profile.** The Human Browser uses a persistent BrainRoot-owned directory under the application data directory, separate from the Preview profile, the shell's data, the Agent Browser profile, and every installed browser. The profile is created on first use; deletion and full reset are explicit user actions with confirmation that remove the profile data.
4. **Permissions.** Camera, microphone, geolocation, clipboard, notifications, and persistent-storage requests are denied by default with no in-page prompt in the first slice. Site-data deletion and profile reset are available to the user.
5. **Lifecycle.** At most one remote HOT view exists. Inactive content becomes COLD by destroy/recreate following the Deck contract, with an honest message that in-memory state and unsaved forms may be lost; snapshots are placeholders, not a promise of perfect restoration. The Resource Governor owns the record with an owner, state, and cleanup handle.
6. **Sharing with the agent.** There is no ambient access. A future explicit user-mediated share must name the exact page or data allowed and needs its own decision and threat model; it is out of scope for the first slice.
7. **Retention.** Browsing history is not imported, synced, or exported in the first slice. Site data and the profile can be deleted by the user; BrainRoot does not copy Human Browser data into project state, logs, or agent context.
8. **Roadmap honesty.** This policy is approved now as a prerequisite; the CB-B implementation follows the roadmap phases unless the maintainer explicitly reorders. No CB-B batch ID is assigned by this ADR.
9. **Acceptance evidence for the implementation batch.** Negative policy tests for every denied scheme, popup, download, and permission prompt; the profile isolation probe from B13-S02; one-HOT lifecycle and resource-return measurements; keyboard/focus and accessibility checks; and compatibility findings reported rather than concealed.

## Alternatives considered

- **Curated destination set only.** Rejected as the first-slice policy: a Human Browser that cannot open the page the user is working on does not meet the product contract, and the isolation boundary (no IPC, separate profile, denied permissions) is what makes arbitrary pages acceptable. A curated mode can be added later as a restriction, not as the default.
- **Reusing the Preview policy.** Rejected: the Preview is loopback-only and must never load remote content; merging the roles would break the origin policy and the trust boundary.
- **Sharing the Preview or Agent Browser profile.** Rejected: it would leak cookies and authenticated storage between roles.
- **Prompting for permissions in-page.** Rejected for the first slice: remote pages must not be able to negotiate capabilities with the user through the browsing surface; denials stay fail-closed until a reviewed permission design exists.
- **Reading installed browser profiles for convenience.** Rejected: it breaks the profile ownership rule and the privacy boundary; portability is CB-C's explicit, user-selected export path.

## Consequences

The Human Browser becomes a well-bounded feature rather than an ambient one: remote content is explicitly user-directed, isolated, and incapable of reaching BrainRoot capabilities. Some sites will not work because downloads, popups, and permission-gated APIs are denied; that is reported as a compatibility finding rather than hidden. The Deck implementation must live with destroy/recreate semantics instead of native hibernation. The profile probe (B13-S02) must succeed before any CB-B implementation batch is assigned.

## Performance implications

One remote HOT view at most; inactivity destroys it, and the Resource Governor records ownership and cleanup. The implementation batch must measure create/destroy, memory return, and switching latency before claiming any "fast" behavior, and the existing failing process-tree memory TARGET stays visible. No Human Browser resource runs at idle, at launch, or without an explicit user action.

## Security implications

Remote pages are hostile-capable and receive no BrainRoot IPC, so confused-deputy attacks through the shell are structurally impossible. Profile data is user-owned and never read by agents; the profile directory is validated at the privileged boundary and stays outside project state. Denied schemes, popups, downloads, and permission prompts remove the most common capability-escalation paths in the first slice. Any future sharing flow changes this trust boundary and requires a new ADR review before implementation.

## Reversibility

Medium-high. The policy can be revised by a superseding ADR, and the implementation batch is not scheduled by this decision. Tightening the first slice (for example, curated destinations or additional denials) is always possible; loosening a denial requires the same review path. No runtime state, artifact, or public interface depends on this ADR yet.

## References

[Companion Browser plan](../specs/companion-browser-plan.md), [browser architecture](../08-browser-architecture.md), [security and permissions](../11-security-and-permissions.md), [roadmap](../15-roadmap.md), [open questions](../17-open-questions.md), ADR 0006, and ADR 0012.
