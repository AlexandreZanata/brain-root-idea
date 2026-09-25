# R2-S01 Agent Host — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: `AgentHostState` gerencia `opencode serve --pure` como filho on-demand.
Legado `features::conversation` congelado: zero toque neste microstep.

## Allowlist respeitada

* `src-tauri/src/features/agent_host/mod.rs` (novo, ~400 linhas: spawn + health + kill + 3 comandos + testes diferidos)
* `src-tauri/src/features/mod.rs` (1 linha: `pub mod agent_host;`)
* `src-tauri/src/main.rs` (manage + 3 comandos + shutdown no close)
* `scripts/check-modules.sh` (registro público `agent_host::...`)
* `scripts/check-security.sh` (prefixo `http://127.0.0.1:` documentado: só loopback do sidecar com auth efêmera; nenhum outro plain-http permitido)
* Este spec.

Forbid respeitado: `src/` frontend, `provider/*`, `conversation/*` intocados.

## Comportamento (spike R1 confirmado)

* `agent_host_start` gera senha efêmera `uuid v4`, spawna `opencode --pure --port 4099 --hostname 127.0.0.1`, poll `/global/health` com Basic até 10s. Idempotente se saudável.
* `agent_host_status` retorna `{running, port, version, pid}` — sem campo secreto por construção. Inalcançável = `running:false` + kill do stale.
* `agent_host_stop` / `shutdown` (no `CloseRequested`): `kill` + grace 5s + `wait`. Sem `unwrap` em lock envenenado.
* Erros normalizados sem vazar senha: 401 → `AuthenticationFailed`, timeout → `TimedOut`, spawn fail → `ProviderUnavailable` com hint de instalação.
* Porta 0 rejeitada (`InvalidInput`). Single-flight via mutex único. `stop_if_idle` presente para o governor B-R5.

## Micro-gate rápido (sem testes — ADR 0014, release gate decide)

* `cargo fmt --all --check` OK
* `cargo check --all-targets` OK, 0 warnings
* `sh scripts/check-modules.sh` OK (ver abaixo)
* `sh scripts/check-docs.sh` OK
* `sh scripts/check-security.sh` OK
* `git diff --check` OK

## Deferred release tests (escritos em `agent_host/mod.rs deferred`, rodar no gate)

* T-R2-01 start→health→status version presente, segundo start idempotente mesmo pid.
* T-R2-02 stop→status `running:false`, restart gera pid novo.
* T-R2-03 scan: `serde_json` do status + `Debug` do interno nunca contêm `password`/`sk-`/`Authorization`.
* T-R2-04 soak `perf-soak.sh` 5/5 + `measure-rss.sh` sem órfão após stop/close.
* Unitários já escritos: base64 vectors, port zero, 401 mapping, health trim, secret absence, uuid uniqueness.

## Rollback

Reverter este commit único; remover `agent_host` do `mod.rs`/`main.rs`/registry. Legado `conversation_send/cancel` continua respondendo health. Sidecar órfão: `pkill -f "opencode serve"`.
