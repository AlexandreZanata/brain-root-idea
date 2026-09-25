# R2-S02 Tabs + host lifecycle — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: tab-strip visual OpenCode em Svelte + Start/Stop/Status do sidecar via IPC real.
Backend segue sessão única legada; sessões por aba chegam com a session API (S03+).

## Allowlist respeitada

* `src/agentHost.ts` (novo: guard `isAgentHostStatus`, `hostLabel` sem segredo, 3 invokes)
* `src/agentHost.test.mjs` (novo: guards + labels; escrito, rodar no gate)
* `src/lib/SessionTabs.svelte` (novo: `tablist`, `aria-selected`, `●` na aba em stream, foco visível)
* `src/App.svelte` (tabs por aba com `prompt`/`turns` próprios, roteamento de stream via `streamTabId`, badge host, título auto do 1º prompt)
* `package.json` (1 linha: teste novo na lista `test:frontend`)
* Este spec.

Forbid respeitado: `src-tauri/`, `provider/*`, `conversation/*` intocados.

## Comportamento

* Abas: criar (`Session N`), trocar sem remount do Canvas, fechar (última protegida; fechar aba em stream cancela antes). Título auto = 28 chars do 1º prompt, só enquanto default.
* Stream continua na aba de origem ao trocar; `●` indica onde está.
* Badge: `Checking… → Stopped/Running vX / Failed detail`. Start idempotente no Rust; Stop sempre volta a `Stopped`. Falha de IPC vira texto, nunca throw na cara.
* `bind:prompt={activeTab.prompt}` compila (svelte-check 0 erros).

## Micro-gate rápido (sem testes — ADR 0014)

* `pnpm run check` (svelte-check) 0 erros, 0 warnings
* `pnpm build` OK: JS 91.64KB (gzip 31.83KB), CSS 19.93KB — +~4KB vs baseline 87KB
* `check-docs.sh`, `check-security.sh` (ver abaixo), `check-modules.sh`, `git diff --check`

## Deferred release tests (rodar no gate)

* T-R2-05 `agentHost.test.mjs` no `test:frontend`.
* T-R2-06 E2E fake: abrir 2ª aba → enviar → trocar → stream roteado certo → fechar aba em stream (cancela, 0 ativo).
* T-R2-07 Start via badge → `Running vX`; Stop → `Stopped`; kill do sidecar externo → próximo Status mostra `Stopped` (stale reaped).
* T-R2-08 teclado só: Tab/navegar/fechar/start sem mouse; leitor anuncia status sem spam de chunk.

## Rollback

Reverter este commit único. `agent_host_*` sem chamador não consome nada (idle 0 por construção).
