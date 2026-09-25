# B-R4 S04 Turn ledger (`/cost`) — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: cada turno agente concluído mostra `1,200 in · 340 out · $0.0042 · 1,000 cached`.
Números do sidecar (`tokens`+`cost` do `GET /session/{id}`), sem estimativa local.

## Probe (sem gasto de token)

`Session` expõe `cost: number` + `tokens{input,output,reasoning,cache{read,write}}`
(confirmado no `/doc` 479KB, destruído após extrair shapes).

## Allowlist respeitada

* `src-tauri/.../agent_host/mod.rs` (`TurnCost`, `validate_session_id`, `parse_turn_cost`, comando + teste diferido)
* `src-tauri/.../main.rs`, `scripts/check-modules.sh`
* `src/agentHost.ts` (tipos, guard, invoke, `formatTurnCost` + testes), `src/agentHost.test.mjs`
* `src/lib/Turn.svelte` (linha de custo), `src/lib/ConversationPanel.svelte` (mapa por aba), `src/App.svelte` (fetch best-effort no `completed`)
* Este spec.

## Comportamento

* Id de sessão validado (prefixo `ses`, charset, 128 bytes) — sem path injection.
* Ausência de campo = 0 (leniente); dinheiro `<$0.01` com 4 casas.
* Fetch best-effort: falha deixa o turno sem rótulo, nunca vira erro.
* Rótulo por `${tabId}:${turnId}`: sobrevive à troca de abas.

## Micro-gate rápido

* `cargo fmt/check` 0 warnings, `pnpm check` 0 erros, build OK, 11/11 testes do arquivo em run local
* `check-modules.sh`, `check-docs.sh`, `check-security.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R4-09 Rust `turn_cost_reads_ledger_and_rejects_bad_ids`.
* T-R4-10 `agentHost.test.mjs` (ledger).
* T-R4-11 E2E opt-in: turno real → rótulo com números do sidecar; sessão inválida → `InvalidInput` sem request.

## Rollback

Reverter este commit único. Turnos voltam sem rótulo; comando sem chamador custa 0.
