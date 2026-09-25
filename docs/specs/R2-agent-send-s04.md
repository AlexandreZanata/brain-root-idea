# R2-S04 Sidecar send via sessions — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: `agent_host_send` (sessão fresca + `prompt_async` + worker SSE) e
`agent_host_cancel_send` (flag + abort best-effort). Frontend wiring no S05.

## Protocolo real (probe `/doc` 478.968 bytes, pin R1, redigido, `/doc` destruído)

* `POST /session {}` → `{id: "ses_…"}`; `POST /session/{id}/prompt_async {parts:[{type:"text",text}], model?}` → 204; `POST /session/{id}/abort` → bool; `GET /event` SSE `data: {json}` com duas gerações: v1 `{id,type,properties.sessionID}` e v2 `{id,type,data:{sessionID,field,delta}}`.
* Sem chamada LLM no probe: só `/doc`, `/global/health`, `server.connected`. Nenhum token gasto.

## Allowlist respeitada

* `src-tauri/src/features/agent_host/mod.rs` (send/cancel/worker/SSE/envelope + testes diferidos)
* `src-tauri/src/main.rs` (2 comandos), `scripts/check-modules.sh` (registro)
* Este spec. Frontend e legado intocados.

## Comportamento

* Sessão fresca por envio (isolamento total entre turns; multi-aba backend depois).
* Modelo da seleção S03 incluído só quando presente; vazio = default do sidecar.
* Worker: 1 `std::thread` por envio (mesmo modelo do runtime MVP-0), `Started` imediato, deltas `field=="text"` da nossa sessão, terminal exatamente-um: `Completed` (`session.idle`), `Failed` (`session.error`/cap/EOF), `Cancelled` (flag).
* Bounds: prompt 16 KiB bytes, delta 64K chars, turno 2M chars, stream 10 min; fila nunca cresce (frontend coalesce já existe).
* Cancel idempotente: flag + abort best-effort + `status()`; worker emite o terminal.
* Segredo: senha só em memória do worker; erros nunca a interpolam.

## Micro-gate rápido (sem testes — ADR 0014)

* `cargo fmt --check` OK, `cargo check --all-targets` 0 warnings
* `check-modules.sh`, `check-docs.sh`, `check-security.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R2-13 unitários novos: `classify_frame` v1+v2/foreign/non-text/error, `validate_prompt`, `session_id_of`.
* T-R2-14 integração opt-in (chave local, nunca CI): send "reply with OK" → `Started` + ≥1 chunk + `Completed`; segundo send concorrente → `InvalidState`; cancel mid-turn → exatamente-um `Cancelled`, 0 worker ativo.
* T-R2-15 scan: `agent_event` nunca contém senha/`sk-`/`Authorization`.

## Rollback

Reverter este commit único. Comandos sem chamador custam 0; sessões órfãs no sidecar morrem com ele (kill no close/idle).
