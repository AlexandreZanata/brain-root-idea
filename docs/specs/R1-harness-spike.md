# R1 harness spike — MEASURED 2026-09-25 (spike, não gate de release)

Pin: `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769` via `git ls-remote`.
`opencode --version`: `1.18.31`. Host: i7-13620H, 16 cores, 31GB RAM, BrainRoot binary 8.085.672 bytes.

## 1. Sidecar `opencode serve` — MEASURED

Comandos (senhas efêmeras, nada persistido):

```bash
opencode serve --port 4099 --hostname 127.0.0.1
OPENCODE_SERVER_PASSWORD=<ephemeral> opencode serve --port 4098 --hostname 127.0.0.1
curl -u opencode:<ephemeral> http://127.0.0.1:4098/global/health
curl -u opencode:<ephemeral> http://127.0.0.1:4098/doc
curl -u opencode:<ephemeral> http://127.0.0.1:4097/config/providers
sh scripts/measure-rss.sh <pid>
BRAINROOT_BASELINE_RAW=/tmp/r1-baseline.log sh scripts/perf-baseline.sh
sh scripts/perf-soak.sh 5 4099
```

Resultados (ambiente acima, builds release, settle ~3s):

| Probe | Resultado |
|---|---|
| Sem `OPENCODE_SERVER_PASSWORD` | `GET /global/health` vazio, `GET /config` e `GET /doc` HTTP 401. Serve exige basic auth (`opencode:<pass>`) nesta versão. |
| Com auth, full (com plugins) | `{"healthy":true,"version":"1.18.31"}`, `/doc` 478.968 bytes HTTP 200, RSS ~434.940 KB aos 3s, CPU ~57% durante warmup. |
| Com auth, `--pure` | health OK, RSS 311–444 MB aos ~3s conforme run (311.412 KB spike manual, 444.628 KB RSS / 442.141 KB PSS no `perf-baseline.sh`), `/config/providers` 33.463 bytes HTTP 200, `/doc` 478.968 bytes. Full sem `--pure` ~434.940 KB no mesmo ponto (~120MB acima do melhor run pure, mas com sobreposição de variância — tratar como faixa, não número único). |
| Cleanup | `kill <pid>` + 1s → `terminated-ok`, 0 órfão em todos os ciclos manuais. Soak script valida 5/5 no gate. |

Status: MEASURED apenas para este host/versão. Não generalizar para Windows/macOS.

## 2. Achado de segurança (não logar de novo)

`GET /config/providers` autenticado retorna metadados + **chave do provider inline** (exposição confirmada, valor redigido aqui e arquivos `/tmp` destruídos com `shred -u`).

Consequência para B-R2/R3: Rust nunca encaminha payload bruto ao frontend nem a log. `provider_status` expõe só booleano + lista de IDs sem segredo. Teste de negação obrigatório.

## 3. Inventário UI para port leve (pin acima)

Verificado via API GitHub em `packages/app/src/components`, `packages/app/src/components/session`, `packages/ui/src`, `packages/desktop/src`, `packages/opencode/src`:

Portar (Svelte, tokens visuais, sem Solid residente):

* tabs multi-sessão: `packages/app/src/components/titlebar-tab-strip.tsx`, `titlebar-tab-nav.tsx`, `titlebar-tab-popover.tsx`, `session-sortable-tab*.tsx`, `session-header.tsx`, `session-new-view.tsx`.
* chat input/stream: `prompt-input*.tsx`, `prompt-workspace-selector.tsx`, `session-context-tab.tsx`, `session-context-breakdown/metrics/format.ts`.
* model picker: `dialog-select-model*.tsx`, `dialog-manage-models.tsx`, `model-tooltip.tsx`, `settings-models.tsx`.
* design tokens: `packages/ui/src/theme/`, `packages/ui/src/styles/`, botões/inputs base em `packages/ui/src/components/`.

Cortar (não entra no shell leve): `packages/desktop/` Electron (`main/preload/renderer`), `packages/web/` Astro marketing, `packages/tui/`, `i18n/` 20 línguas, `storybook/`, `updater.ts`, `wsl/`, `share/`, `slack/`, `console/`, `enterprise/`, `containers/`, `http-recorder/`, toolchain bun+turbo.

Motor: usar `packages/opencode/src/server/` via HTTP localhost autenticado como sidecar. Não forkar Go; customizar via `opencode.json + @opencode-ai/plugin`.

## 4. Decisão proposta (ADR-0015 draft)

`opencode serve --pure` atrás de `Agent Host` Rust como default; full só com plugin allowlist explícito; `dsh --profile acp` opcional atrás de flag; `pi-agent-core` como referência das regras token (prompt estável, skill top-1, sem preload, compaction barata). Sidecar sempre medido separado do budget UI ≤150MB — ele sozinho usa ~300MB+, então nunca pode ser idle-residente.

## 5. Deferred release tests (escrever em B-R2, rodar no gate)

* T-R1-01 fake→sidecar→stream→complete via `Agent Host`, asserts estados + bound.
* T-R1-02 cancel race + close durante stream, 0 ativo após.
* T-R1-03 providers filtrado sem segredo (falha se `sk-`/`Authorization` aparecer em IPC/log).
* T-R1-04 soak 5/5 `perf-soak.sh` + `measure-rss.sh` sem órfão.
* T-R1-05 `401 sem auth` documentado como comportamento esperado desta versão pinada.

Evidência deste spike: SHA pin + tabela acima + scripts novos. Sem mudança em `src/` ou `src-tauri/src/`. Rollback: `rm scripts/measure-rss.sh scripts/perf-baseline.sh scripts/perf-soak.sh docs/specs/R1-harness-spike.md docs/adr/draft-0015-opencode-serve-default.md`.
