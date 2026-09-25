# R2-S05 Agent path live — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: composer envia pela sessão do sidecar quando ele está vivo; legado vira fallback.
Loop legado aposentado no happy path, não deletado.

## Allowlist respeitada

* `src/agentHost.ts` (envelope `agent_event`, `chooseSendPath`, send/cancel invokes)
* `src/agentHost.test.mjs` (path + guards do envelope; escrito, rodar no gate)
* `src/App.svelte` (2º listener, `sendPath`/`agentSession`, roteamento por sessão)
* Este spec. Rust e legado intocados.

## Comportamento

* Submit decide por aba-envio: sidecar `running` → `agent_host_send`, senão legado.
* Eventos cruzados são ignorados: legado ignora quando `sendPath==="agent"`;
  agente ignora sessão diferente e transição ilegal (`acceptsEvent` reutilizado).
* Terminal sempre volta `sendPath` a `legacy`; próxima submissão re-decide.
* Cancel agente: flag + abort; worker emite o único terminal — `cancelling` resolve sempre.
* Stop do sidecar com turno voando: stream EOF → `Failed` compreensível, UI recupera.
* Fechar aba em stream cancela antes (vale pros dois paths).

## Micro-gate rápido (sem testes — ADR 0014)

* `pnpm run check` 0 erros, `pnpm build` OK (JS 97.11KB, +~2KB)
* `check-docs.sh`, `check-security.sh`, `check-modules.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R2-16 `agentHost.test.mjs` completo.
* T-R2-17 E2E fake do worker (mock `/event` com frames v1+v2): started→chunks→completed numa aba; sessão estrangeira ignorada; cancel→um `Cancelled`.
* T-R2-18 E2E alternância: envio legado (sidecar parado) → Start → envio agente → Stop no meio → `Failed` + recuperação + novo envio legado OK.
* T-R2-19 teclado/leitor: identical ao legado (mesmo pipeline de turns).

## Rollback

Reverter este commit único. Sem sidecar rodando, o app é byte-a-byte o comportamento anterior.
