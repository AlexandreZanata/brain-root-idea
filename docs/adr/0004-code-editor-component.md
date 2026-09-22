# ADR 0004: CodeMirror 6 for deferred optional Code View

**Status:** Accepted, implementation deferred to Phase 7  
**Date:** 2026-09-22

## Context

Some users need to inspect, search, lightly edit, and navigate agent changes. A full IDE editor and extension ecosystem would pull the product away from its agent-first interface and add idle cost.

## Decision

When Code View enters scope, use modular CodeMirror 6 packages for basic file viewing/editing, search, selection, syntax highlighting, diff navigation, and changed-file links. Load the component and language packages only when Code View opens. Do not start an LSP merely by opening Code View.

## Alternatives considered

- **Monaco:** mature and familiar, rejected initially because it encourages full-IDE scope and a heavier integration surface.
- **Plain textarea/contenteditable:** too weak for accessible code selection, large documents, and highlighting.
- **Custom editor:** rejected as high-risk, low user-value infrastructure.
- **No code view:** too restrictive for trust, inspection, and advanced users.

## Consequences

CodeMirror's modular setup requires deliberate package selection. The first release omits debugger, minimap, extension marketplace, refactoring suite, and complete language intelligence.

## Performance implications

Zero CodeMirror JavaScript, language package, document model, and LSP cost before the view opens. Measure lazy chunk size, open latency, large-file behavior, and disposal.

## Security implications

File access still flows through scoped core APIs. Rendered untrusted content cannot acquire privileged IPC. Very large/binary/sensitive files require guardrails.

## Reversibility

High because Code View is an isolated optional surface behind file/diff contracts.

## References

[CodeMirror system guide](https://codemirror.net/docs/guide/) and [reference](https://codemirror.net/docs/ref/).

