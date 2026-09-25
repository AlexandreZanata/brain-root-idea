# B-R3 S01 Public catalog merge — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: catálogo público OpenRouter fundido à lista do sidecar, com TTL, stale
honesto e picker com contexto/preço. Preços rotulados pela coleta, nunca fixos.

## Probe (público, sem chave, redigido)

`GET https://openrouter.ai/api/v1/models` → 200, 460 modelos, ~756KB:
`{id, name, context_length, pricing{prompt, completion}, top_provider{…}}`.
Só esses 5 campos são lidos; descrição/links/providers descartados.

## Allowlist respeitada

* `src-tauri/src/features/agent_host/catalog.rs` (novo: parse/match/merge/fetch + testes diferidos)
* `src-tauri/src/features/agent_host/mod.rs` (`mod catalog`, cache 6h, comando)
* `src-tauri/src/main.rs`, `scripts/check-modules.sh` (registro)
* `scripts/check-security.sh` (prefixo `https://openrouter.ai/api/` — só catálogo público)
* `src/agentHost.ts` (tipos, guard, `requestHostCatalog`, formatadores + testes escritos)
* `src/lib/ModelPicker.svelte` (`title` com detalhe, chip `stale`), `src/lib/ConversationPanel.svelte`, `src/App.svelte`
* Este spec.

## Comportamento

* Match por id exato → slug após `/` → contenção; sem match = `None` (UNKNOWN, nunca adivinhado).
* Preço string/número por token ×1e6 = USD/M; `selected` viaja junto (1 chamada).
* TTL 6h; refresh falho com cache = `stale:true`; offline sem cache = `ProviderUnavailable`; sidecar parado = lista vazia (picker `no models`).
* Picker: rótulo curto `nome · provider`, detalhe no `title`, chip `stale` quando aplicável.

## Micro-gate rápido (sem testes — ADR 0014)

* `cargo fmt --check` OK, `cargo check --all-targets` 0 warnings
* `pnpm run check` 0 erros, `pnpm build` OK (JS 97.87KB, +~1KB)
* `check-docs.sh`, `check-security.sh`, `check-modules.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R3-01 Rust: preços string/número, match exato/slug/contains/sem-match, merge preserva ids do sidecar.
* T-R3-02 `agentHost.test.mjs`: guards do catálogo + formatadores.
* T-R3-03 E2E: Start → picker com `ctx` e `$/M` reais → offline simulado → chip `stale` → volta online + refresh → `stale` some.
* T-R3-04 scan: resposta do catálogo nunca contém chave/token/`Authorization`.

## Rollback

Reverter este commit único. Picker volta a ids puros (S03); comando sem chamador custa 0.
