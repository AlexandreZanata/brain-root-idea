# ADR 0003: Svelte with TypeScript for the frontend

**Status:** Accepted for prototype  
**Date:** 2026-09-22

## Context

The primary UI is stateful but visually restrained: task progress, chat, layout presets, Canvas chrome, permissions, and optional technical details. It should have a small startup surface, predictable components, strong types, and low ceremony for AI contributors.

## Decision

Use the current stable Svelte toolchain with TypeScript for the Tauri frontend. Prefer Svelte's built-in capabilities, CSS, and small local stores before adding UI/state libraries. Route and render only the desktop product needs; do not introduce a web application backend framework by default. Pin versions at bootstrap after compatibility review.

## Alternatives considered

- **React:** broad ecosystem, rejected for the initial prototype because BrainRoot does not need that ecosystem and dependency pressure is higher.
- **Solid/Vue:** credible; not selected because Svelte's compiler model and concise components fit the small UI hypothesis.
- **Rust-native UI:** could remove JS/WebView, but cross-platform maturity, embedded web content, and contributor ergonomics are less aligned with the current product.

## Consequences

Contributors need Svelte knowledge and disciplined UI/core contracts. No component mega-library is selected. Design tokens and primitives should be built only as used.

## Performance implications

Svelte's compiler approach is promising, not proof. Track initial JS, parsed/evaluated modules, DOM size, update latency, and memory. Lazy optional Code View and technical panels.

## Security implications

The frontend is not trusted with secrets or direct system authority. Avoid unsafe HTML, sanitize rendered untrusted content, and apply a restrictive content security policy.

## Reversibility

Moderate before broad UI implementation. Typed core contracts and framework-neutral product state make replacement possible.

## References

[Svelte official site and documentation](https://svelte.dev/).

