# B-R5 S01 Governor idle-enforce — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: 1 thread global faz cumprir o budget idle do sidecar (60s) + status.
Fecha o loop de RAM: sidecar nunca fica residente esquecido.

## Allowlist respeitada

* `src-tauri/src/features/governor/mod.rs` (novo: tick 5s, auto-stop, contadores, status, teste diferido)
* `src-tauri/src/features/agent_host/mod.rs` (`has_sidecar` sem probe; `stop_if_idle` agora usado, `allow(dead_code)` removido)
* `src-tauri/src/features/mod.rs`, `src-tauri/src/main.rs` (manage, start no setup, shutdown no close, comando), `scripts/check-modules.sh`
* Este spec. Frontend e preview/browser intocados (registro deles no S02).

## Comportamento

* Thread única: a cada 5s, `stop_if_idle(60s)` — só leitura de timestamps, zero rede no tick (não cria o problema que resolve).
* Start idempotente; shutdown sinaliza saída em ≤1 tick; close da janela para tudo (sidecar + governor).
* `governor_status`: `{tick_secs, sidecar_idle_secs, sidecar_running, auto_stops}` — sem probe de saúde (barato e sem efeito colateral).
* Contador `saturating_add`; lock envenenado nunca mata o enforcement.

## Micro-gate rápido (sem testes — ADR 0014)

* `cargo fmt --check` OK, `cargo check --all-targets` 0 warnings (1 import `Manager` no caminho)
* `check-modules.sh`, `check-docs.sh`, `check-security.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R5-01 unit `budgets_are_sane`.
* T-R5-02 manual opt-in: Start sidecar → 65s idle → `governor_status.auto_stops` 1 e `sidecar_running` false; turno ativo recente nunca é morto (last_used atualiza em send/status).
* T-R5-03 close com sidecar vivo: 0 órfão, thread encerra.

## Correção no gate 0.0.15 (2026-09-25)

A garantia "turno ativo recente nunca é morto" **não se sustentava**: `last_used`
só é atualizado em `start`, `status` e no início do `send`, então uma geração mais
longa que os 60s do budget era derrubada no meio da resposta pelo tick do
governor. Correção escopada em `stop_if_idle`: devolve `false` enquanto existir
um `ActiveSend` não concluído, e também quando o lock de estado estiver
envenenado (não matar às cegas é preferível a arriscar trabalho ativo).
Coberto pelo teste `idle_stop_never_kills_an_active_turn`. O T-R5-02 passa a
exigir o caso de turno longo, não apenas de sidecar ocioso.

## Rollback

Reverter este commit único. Sidecar volta a exigir Stop/close manual.
