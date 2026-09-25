# B-R4 S01 Stable system prompt + agent selector — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: todo envio carrega `system` mínimo/estável + `agent` (`plan`/`build`).
UI toggle no S02; aqui só o backend e o contrato.

## Allowlist respeitada

* `src-tauri/src/features/agent_host/mod.rs` (`SYSTEM_PROMPT`, `set_agent`, envio inclui `system`+`agent`, testes diferidos)
* `src-tauri/src/main.rs`, `scripts/check-modules.sh` (registro)
* Este spec. Frontend intocado.

## Comportamento

* Prompt do sistema (~450 bytes, inglês, 5 regras YAGNI) idêntico em todo envio:
  KV-cache do sidecar fica quente. Teste trava o texto — edição casual quebra
  o teste de propósito e exige justificativa de tokens.
* `agent_host_set_agent` aceita só `plan` (exploração read-only, barata) e
  `build` (default, edita). Valor viaja em cada `prompt_async`; inválido nunca
  chega ao sidecar (`InvalidInput` no Rust).
* Sem dependência nova, sem rede nova, sem estado persistido (memória; SQLite só sob 3-strikes).

## Micro-gate rápido (sem testes — ADR 0014)

* `cargo fmt --check` OK, `cargo check --all-targets` 0 warnings
* `check-modules.sh`, `check-docs.sh`, `check-security.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R4-01 `validate_agent` + pin do `SYSTEM_PROMPT` + default `build`.
* T-R4-02 E2E opt-in: send com `plan` → lado read-only observado (sem escrita);
  send com `build` após `select_model` barato → resposta normal.
* T-R4-03 overhead: corpo `prompt_async` menos `parts` ≤600 bytes (2ª parte do TARGET ≤2k tokens de overhead de sistema).

## Rollback

Reverter este commit único. Envio volta a sem `system`/`agent` (default do sidecar).
