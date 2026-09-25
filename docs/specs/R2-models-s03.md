# R2-S03 Live model picker — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: catálogo vivo do sidecar (só identificadores) + picker nativo no composer.
Preço/contexto/privacidade ficam UNKNOWN até o catálogo OpenRouter (B-R3).

## Allowlist respeitada

* `src-tauri/src/features/agent_host/mod.rs` (`extract_models` + `models`/`select_model` + 2 comandos; `authed_get` extraído do health)
* `src-tauri/src/main.rs` (2 comandos no handler)
* `scripts/check-modules.sh` (2 comandos + 2 tipos no registro)
* `src/agentHost.ts` (tipos, guards, 2 invokes — `provider_id`/`model_id` snake_case como o Tauri exige)
* `src/agentHost.test.mjs` (guards do catálogo; escrito, rodar no gate)
* `src/lib/ModelPicker.svelte` (novo: `<select>` nativo estilizado — a11y grátis, 0 JS de dropdown)
* `src/lib/ConversationPanel.svelte` (chip hardcoded `glm-5.3-flash` → picker)
* `src/App.svelte` (carga após status/start, seleção, `modelDisabled` no stream)
* Este spec.

## Comportamento

* `agent_host_models` exige sidecar rodando; lê `/config/providers` com auth e devolve só `{provider_id, provider_name, model_id, model_name}` (cap 500, trim 200). Chaves/URLs/preços nunca copiados — provado por teste que serializa a saída e procura `sk-`/`apiKey`.
* `agent_host_select_model` valida não-vazio + tamanho, guarda em memória (SQLite só se virar necessidade 3-strikes).
* Picker vazio (`sidecar parado`) mostra chip `no models`; com lista, `<select>` desabilitado durante stream.
* Sem preço/contexto ainda: payload do sidecar não traz; documentado como UNKNOWN, não inventado.

## Micro-gate rápido (sem testes — ADR 0014)

* `cargo fmt --check` OK, `cargo check --all-targets` 0 warnings
* `pnpm run check` 0 erros, `pnpm build` OK (JS 95.33KB, +~4KB)
* `check-docs.sh`, `check-security.sh`, `check-modules.sh`, `git diff --check`

## Deferred release tests (rodar no gate)

* T-R2-09 `agentHost.test.mjs` (inclui guards do catálogo).
* T-R2-10 Rust `models_keep_ids_and_drop_secrets` + `models_reject_missing_catalog`.
* T-R2-11 E2E: Start → picker lista `provider/model` reais → selecionar persiste após `status` → Stop → picker volta a `no models`.
* T-R2-12 scan de segredo no IPC: nenhuma resposta de `agent_host_*` contém `sk-`, `Authorization`, `apiKey`.

## Rollback

Reverter este commit único. Picker some, chip volta; comandos sem chamador custam 0.
