# B-R4 S02 Plan/Build toggle — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: segmented Plan/Build no composer ligado ao `agent_host_set_agent`.
Perfis fast/balanced/max no S03; aqui só o modo.

## Allowlist respeitada

* `src/agentHost.ts` (`AgentMode`, guard, invoke — snake_case como o Tauri exige)
* `src/agentHost.test.mjs` (guard; escrito, rodar no gate)
* `src/lib/ConversationPanel.svelte` (grupo Plan/Build com `aria-current` + títulos de custo)
* `src/App.svelte` (estado `agentMode`, otimismo com revert; Rust default `build` casa com o inicial)
* Este spec. Rust intocado (S01 já expôs o comando).

## Comportamento

* Plan = exploração read-only barata; Build = edita. Modo viaja em cada `prompt_async` (S01).
* Toggle otimista com revert em falha; falha de núcleo vira `failed` honesto (só ocorre com botões válidos se o core estiver quebrado).
* Teclado/leitor: botões nativos, `aria-current`, sem estado só-visual.

## Micro-gate rápido (sem testes — ADR 0014)

* `pnpm run check` 0 erros (1 prop restaurada no caminho: `onselectmodel`)
* `pnpm build` OK (JS 98.93KB, +~1KB)
* `check-docs.sh`, `check-security.sh`, `check-modules.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R4-04 `agentHost.test.mjs` (modos).
* T-R4-05 E2E: alternar Plan→enviar→Rust recebe `agent:"plan"`; Build→`agent:"build"`; falha injetada reverte o toggle.

## Rollback

Reverter este commit único. Envio volta ao default `build` do Rust.
