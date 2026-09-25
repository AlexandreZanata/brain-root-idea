# B-R5 S02 Governor visible — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: badge do sidecar expõe a política idle + contador de auto-stops via
`governor_status`. Registro de preview/browser fica para o S03 com política UX.

## Allowlist respeitada

* `src/agentHost.ts` (tipos, guard, invoke, `governorTitle` + teste escrito)
* `src/agentHost.test.mjs` (shape + título)
* `src/App.svelte` (fetch no refresh/start/stop, `title` no badge)
* Este spec. Rust intocado.

## Comportamento

* `title` do badge: `Resource governor: sidecar stops after 60s idle (N auto-stops)`;
  antes do primeiro fetch: `starting`. Falha de IPC = `null` (sem mentir número).
* 10/10 testes do arquivo passam em run local rápido (verificação do autor;
  o gate versionado continua dono da certificação).

## Micro-gate rápido

* `pnpm run check` 0 erros, `pnpm build` OK
* `check-docs.sh`, `check-security.sh`, `check-modules.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R5-04 `agentHost.test.mjs` (governor).
* T-R5-05 E2E: Start → badge title com `60s idle` → 65s idle → auto-stop → title com `1 auto-stop`.

## Rollback

Reverter este commit único. Badge volta ao texto sem `title`.
