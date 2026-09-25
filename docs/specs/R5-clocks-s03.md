# B-R5 S03 Preview/browser clocks — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: relógios idle de preview e browser no registro do governor, report-only.
Auto-destroy segue exigindo decisão UX (registrada abaixo, não implementada).

## Allowlist respeitada

* `src-tauri/src/features/preview/mod.rs` (`activity`, `touch`, `idle_secs`, 5 chamadas)
* `src-tauri/src/features/human_browser/mod.rs` (idem, 8 chamadas)
* `src-tauri/src/features/governor/mod.rs` (2 campos no status)
* `src/agentHost.ts` + `src/agentHost.test.mjs` (tipos, guard, teste escrito)
* Este spec. Comandos existentes intocados em semântica (só `touch()` somado).

## Comportamento

* Comandos mutantes movem o relógio; leituras de status, nunca.
* `governor_status` agrega `preview_idle_secs`/`browser_idle_secs` (`None` = nunca usado).
* Sem auto-destroy: matar preview/browser sozinho surpreenderia trabalho ativo
  (governor nunca mata sem aviso — invariante). Política explícita no S04 ou ADR.

## Micro-gate rápido (sem testes — ADR 0014)

* `cargo fmt --check` OK, `cargo check --all-targets` 0 warnings (2 chaves `}` restauradas no caminho, sem mudança de lógica)
* `pnpm run check` 0 erros, `pnpm build` OK
* `check-modules.sh`, `check-docs.sh`, `check-security.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R5-06 `agentHost.test.mjs` (governor com os 2 campos).
* T-R5-07 Rust (futuro): `touch` move o relógio; status nunca move.
* T-R5-08 E2E: preview show → `preview_idle_secs` 0 → `governor_status` reflete.

## Rollback

Reverter este commit único. Comandos voltam sem relógio; governor ignora os campos.
