# 21 — Pivot Lean YAGNI: visual OpenCode em estrutura leve, desempenho máximo

**Status:** Implementado em `0.0.15` (ver o [registro do batch](history/batches/B19-foundation.md)); ADR 0015 ainda **Proposed**. Substitui a execução pós-MVP-0 e não reescreve a história B00–B05.
**Desvio registrado:** os commits do pivot foram escritos antes de existir um issue por microstep (contraria `docs/18` §4.2); o issue [#128](https://github.com/AlexandreZanata/brain-root-idea/issues/128) é de rastreamento, aberto depois. O gap de workspace/permissões do agente segue **aberto** em `docs/17-open-questions.md`.
**Data:** 2026-09-25
**Objetivo:** IDE de agente IA ultra-leve, personalizável, token-econômica, multi-provider, com visual consolidado do OpenCode portado para estrutura mínima.
**Stack alvo:** Tauri 2 + Rust core + Svelte atual + `opencode serve` sidecar on-demand + regras token `pi-agent-core`.
**Idioma de execução:** pt-BR neste documento; ADRs permanentes continuam em inglês.

## 1. Resultado para o usuário

1. Abrir app leve em ≤1.0s p50, idle ≤150MB RSS total, CPU <1%.
2. Ver interface padrão OpenCode (chat, tabs multi-sessão, model picker com preço/contexto, diff approve/deny), mas sem peso Electron.
3. Usar qualquer provider grande via mesma UX: OpenCode Zen, OpenRouter (300+ modelos), Anthropic, OpenAI, Gemini, DeepSeek via OpenRouter, Ollama local.
4. Agente personalizável em 3 perfis: `fast/cheap`, `balanced`, `max`, com custo/token visível.
5. Múltiplas janelas/sessões com auto-desempenho: 1 HOT, resto COLD/DEAD, auto-suspende e nunca mata trabalho ativo sem aviso.
6. Cancelar/fechar sempre limpa: 0 órfão, volta ao baseline ±5MB.

Não é objetivo: forkar monorepo OpenCode inteiro, Electron, dsh-web Node residente, embeddings/indexador always-on, LSP always-on, marketplace/plugins, cloud sync, mobile, updater auto.

## 2. Decisão final

### 2.1 Interface: MANTER Tauri+Svelte, PORTAR visual OpenCode

Não forkar app inteiro. Fork inteiro = RAM alta + merge hell.

Fonte visual pinada (leitura, não submódulo inicial):

* `anomalyco/opencode#packages/app/src/app.tsx, entry.tsx, components/, pages/` — UI principal Solid.
* `anomalyco/opencode#packages/ui/src/components/, v2/, theme/, styles/` — design system.
* `anomalyco/opencode#packages/opencode/src/server/` — OpenAPI servida em `http://<host>:<port>/doc` por `opencode serve`.
* `anomalyco/opencode#packages/desktop/src/main, preload, renderer/` — referência de comportamento tabs/janelas, **não trazer Electron**.
* `anomalyco/opencode#packages/web/` — site Astro, ignorar.
* `anomalyco/opencode#packages/tui/` — terminal, ignorar agora.

Ordem de port Svelte (só o necessário, resto lazy ou fora):

1. `session list + tabs` (multi-sessão = sua multijanela lógica).
2. `chat input + streaming + stop/cancel` (reusa `src/streamBuffer.ts` com `requestAnimationFrame`).
3. `model picker preço/contexto/privacy` (`GET /config/providers` + OpenRouter + Models.dev).
4. `diff + permission approve/deny` (liga no Permission Broker Rust).
5. Lazy em Developer Mode: settings, MCP manager, LSP status, share link, console.

Corte explícito do OpenCode: `i18n 20 línguas, storybook/, updater.ts, wsl/, share/, slack/, console/, enterprise/, containers/, http-recorder/`, Electron builder, bun+turbo toolchain.

Por que não `dsh web :3080`, Desktop Electron, Cline VSCode ext: residente Node/Electron quebra budget ≤150MB no boot. Prova no spike B-R1.

### 2.2 Agente: `opencode` Go sidecar + bisturi `pi-agent-core`

* **Runtime default:** `opencode serve --port 4096 --hostname 127.0.0.1` gerenciado pelo Rust `Agent Host` (`start/capabilities/new_session/send/cancel/close`). Mata grupo de processo em idle 60s. Frontend nunca fala com LLM direto.
* **Customização sem fork Go:** `opencode.json + @opencode-ai/plugin` (ProviderHook, tool registration). `build` permissivo, `plan` read-only default.
* **Bisturi token YAGNI:** forkar mentalmente só `pi-agent-core` (`ask + tools + skills em loop`). Prototipa regra em ~50 linhas TS, porta como config/plugin opencode. Não forka Go.
* **Opcional atrás de flag:** `dsh --profile acp` ou `sdk-minimal` via ACP stdio. Nunca default. dsh é MIT mas Node 22+ pesado + developer preview com breaking changes.
* Licenças: OpenCode MIT, dsh MIT, pi MIT — compatíveis com Apache-2.0 do BrainRoot. Preservar `LICENSE` + `NOTICE` em artefatos.

Referências:

* OpenCode: `https://opencode.ai`, server `https://opencode.ai/docs/server` / `dev.opencode.ai/docs/server`, repo `https://github.com/anomalyco/opencode`, SDK `createOpencodeServer/createOpencodeClient`, catálogo `https://models.dev`.
* OpenRouter: `https://openrouter.ai/docs`, Agent SDK vs Client SDK, OpenAPI `https://openrouter.ai/openapi.yaml`, modelos `https://openrouter.ai/api/v1/models`.
* pi: `https://pi.dev`, monorepo `https://github.com/agent-harness-labs/pi-mono` (`packages/coding-agent`, `pi-ai`, `pi-agent-core`, `pi-web-ui`, `pi-tui`).
* dsh: `https://github.com/deepseek-ai/deepseek-harness`, docs `https://deepseek-harness.github.io/deepseek-harness/`, `docs/architecture.md`, `packages/bundle/acp-app/README.md`, `SAFETY.md`.
* Evidência token: estudo Scaffold 2026 — Goose ~28k tok/solve `<<` OpenHands `<` OpenCode (~40x). OpenCode default é token-faminto; camada YAGNI corrige sem trocar harness.

## 3. Arquitetura alvo leve

```text
Svelte UI 1 WebView HOT (skin OpenCode portada)
  │ IPC Tauri tipado, validado, sem segredo
Rust Core (dono exclusivo)
  ├─ Workspace / Permissão / Checkpoints / SQLite / Keychain
  ├─ Agent Host → opencode serve filho on-demand
  ├─ Provider Hub → Zen + OpenRouter + BYOK
  └─ Resource Governor 1 timer + Event Bus limitado
        │ on-demand apenas
        ├─ Preview COLD, Human Browser COLD perfil separado
        ├─ Agent Browser Playwright DEAD descartável
        └─ LSP/PTY/dev-server 0 no idle
```

Regras:

* Frontend input é untrusted no Rust. Paths canonicalizados contra roots. IDs/eventos versionados.
* Chave só Rust + OS keychain (Linux Secret Service). Frontend só `configured/not-configured`. Redact em log/screenshot.
* Prompt sistema mínimo e estável (bom p/ KV-cache). Sem preload file-tree. Sequência: goal → arquivos explícitos → `ripgrep` → imports → tree-sitter pontual → LSP só se justificar. Sem embeddings.
* Skill `SKILL.md` lazy: router inlina só top-1. Compaction auto com modelo barato. Sub-agente com budget, sem recursão. Sem fallback silencioso de modelo pago.
* 1 HOT WebView. Troca visual sem remount via `order` CSS (padrão já usado em `src/App.svelte`). Janelas Tauri reais usam `hide/show`, não `close/recreate` em toggle rápido.

Mapa com código atual:

* Manter: `src/App.svelte`, `src/conversation.ts`, `src/streamBuffer.ts`, `src/lib/ConversationPanel.svelte`, `src/lib/CanvasPanel.svelte`, `src-tauri/src/features/`, `src-tauri/src/provider/` como fake determinístico p/ testes.
* Congelar: transporte Go custom MVP-0 vira `legacy_model_loop`, não evolui.
* Tornar lazy: `src/lib/HumanBrowserPanel.svelte`, `src/lib/PreviewPanel.svelte`, `WorkspaceRail` itens browser/files/terminal.
* Não adicionar: CodeMirror, xterm, Playwright permanente, updater, sync.

## 4. Orçamento desempenho máximo

Vocabulário `TARGET/MEASURED/UNKNOWN` de `docs/10-performance-budget.md`. Nada é MEASURED fora do ambiente de referência Ubuntu 24.04 x86_64 release `lto=true strip=true`.

| Cenário | Budget | Status inicial |
|---|---:|---|
| Core+UI idle, sem sidecar, sem preview | RSS total ≤150MB, stretch ≤100MB, CPU <1% após settle 15s | TARGET |
| Warm launch até evento Ready instrumentado | p50 ≤1.0s | TARGET |
| Cancel ack UI | <100ms (terminação filha medida separado) | TARGET |
| Sem tarefa | agent 0, LSP 0, PTY 0, dev-server 0, automation 0, HOT extra 0 | TARGET |
| Preview HOT extra | custo separado `BrainRoot-owned vs project-owned`, nunca somado no budget UI | TARGET |
| Soak 20x open/close + request/cancel | retorno ±5MB baseline, 0 órfão | TARGET |
| Overhead sistema prompt | ≤2k tokens | TARGET |
| Auto-desempenho | >180MB auto-suspende menos recente; >220MB recusa 3ª HOT com explicação leiga + detalhes | TARGET |

Protocolo:

1. Release build, debug off. Registrar commit+lockfile, OS/kernel, CPU/RAM, WebKitGTK, projeto fixture, rede.
2. 10 runs startup, 5 janelas idle 30s. Mediana+p95+range+amostras cruas.
3. Separar core/UI, content WebViews, sidecar `opencode`, projeto. Ferramentas: `ps -o rss,pcpu`, `pgrep -P`, `smem -P brainroot`, `/proc/<pid>/status VmRSS`. Nunca comparar métricas distintas sem label.
4. Scripts: `scripts/measure-rss.sh <pid>`, `scripts/perf-baseline.sh`, `scripts/perf-soak.sh` (criar em B-R1).

## 5. Regras YAGNI preguiçosas (colar no AGENTS.md do batch)

1. 3-strikes: sem código novo se não aparece em 3 tarefas reais.
2. Lazy ou morto: sem owner+timeout+cleanup não entra.
3. Deletar > abstrair: sem marketplace, embeddings, vector DB, multi-protocolo paralelo.
4. 1 timer, 0 polling por recurso. Callbacks OS/WebView.
5. Prompt/tool roster estável por sessão. Mudança quebra KV-cache e dobra custo.
6. Um commit por issue `type(scope): summary (refs #N)`, só allowlist. `Refs #N`, nunca `Closes #N` em microstep de batch.

## 6. Segurança Safe Mode

* Allowlist rede: `api.openrouter.ai`, `opencode.ai`, `models.dev`, endpoints provider configurados. TLS-only prod. Todo resto negado com teste.
* Frontend nunca recebe segredo. `provider_status` só booleano. Logs sem `Authorization`, sem corpo resposta por default.
* Agent output é untrusted. Tool Layer resolve roots/permissão. Sub-agente herda no máximo autoridade do pai.
* Sem `~/.local/share/opencode/auth.json` lido silenciosamente. Credencial só via fluxo próprio + keychain, ou env opt-in p/ smoke local nunca persistido.

## 7. Plano de execução — 5 batches, ~3 semanas

Convenção: 1 branch `batch/bRN-nome`, 1 draft PR, ≤7 issues, 1 commit por issue. A partir de B17: teste escrito mas só roda no release gate; micro-gate é `diff/scope/secrets/format/schema/docs`. Full CI só no Ready do batch. Seguir `docs/18-mvp-execution-plan.md` e `docs/adr/0014-release-only-test-cadence.md`.

### B-R1 Spike medido (2 dias, sem código produto)

Objetivo: decidir com número.

* R1-S01 pin + inventário: pin `opencode@<sha>`, listar `packages/app/src/components`, `packages/ui/src/components`, `packages/opencode/src/server`, `packages/desktop/src/renderer`. Saída tabela o que portar/cortar.
* R1-S02 harness lado a lado mesmo fixture: `opencode serve --port 4096`, `pi -p`, `npx @deepseek-ai/dsh web --no-open`. Medir RSS idle+sessão, startup p50, bundle, tok/solve 5 tarefas, troca OpenRouter.
* R1-S03 scripts medida: criar `scripts/measure-rss.sh`, `scripts/perf-baseline.sh`, `scripts/perf-soak.sh` + `docs/specs/R1-harness-spike.md` MEASURED + proposta ADR-0015.
* Allowlist: `docs/specs/`, `scripts/`, `docs/adr/draft-*.md`. Forbid: `src/`, `src-tauri/src/`.
* Gates: `git diff --check`, secret scan, markdown links. Evidência: tabela crua + SHA pin + comando exato.
* Rollback: deletar scripts/specs, sem impacto produto.

Comandos:

```bash
opencode serve --port 4096 --hostname 127.0.0.1
curl http://127.0.0.1:4096/doc
curl -s https://openrouter.ai/api/v1/models | head -c 2000
npx @deepseek-ai/dsh web --no-open
ps -o pid,rss,pcpu -p <pid>
```

### B-R2 Agent Host + skin OpenCode tabs/chat (5 dias)

Objetivo: prompt→stream→cancel→close via sidecar, visual OpenCode, 0 órfão.

* R2-S01 `features::agent_host`: `start/capabilities/new_session/send/cancel/close`, spawn `opencode serve`, kill grupo em idle 60s/close. Congela legado como `legacy_model_loop`.
* R2-S02 skin tabs multi-sessão portada (Svelte, tokens `packages/ui` theme). 1 WebView HOT, troca sem remount.
* R2-S03 skin chat input/stream/stop portada, reusa `streamBuffer`, bound histórico, sem spam screen-reader.
* R2-S04 cancel/close cleanup: ack <100ms, ignora late events, teste 0 ativo após cancel+close.
* R2-S05 falha compreensível: not configured, offline, invalid key, limit, timeout, malformed. Texto ação + código técnico opcional.
* Allowlist: `src-tauri/src/features/agent_host*`, `src/lib/SessionTabs.svelte`, `src/lib/ConversationPanel.svelte`, `src/conversation.ts`, `docs/specs/R2*`. Forbid: Electron, Node residente, segundo HOT default.
* Release tests (escrever, rodar no gate): fake→sidecar→stream→complete, cancel race, close durante stream, órfão 0, retorno baseline.
* Rollback: `pkill -P` sidecar, revert commit único, legado ainda responde health.

### B-R3 Provider Hub (3 dias)

* R3-S01 descoberta: `GET /config/providers`, `models.dev`, `GET /v1/models` OpenRouter. Cache metadata TTL, stale honesto.
* R3-S02 credencial: keychain + env opt-in local, `provider_status` booleano, redact provado.
* R3-S03 model picker skin OpenCode: preço/contexto/privacy antes do 1º uso, sem fallback pago silencioso. Perfis `fast/balanced/max` iniciais.
* Allowlist: `src-tauri/src/features/provider_hub*`, `src/lib/ModelPicker.svelte`, `docs/specs/provider-hub.md`.
* Rollback: limpar cache, voltar a 1 modelo fixo validado.

### B-R4 Token-saver personalizável (5 dias)

* R4-S01 contexto lazy: goal→explicit→rg→imports→tree-sitter pontual. Sem full scan/embeddings. Teste prova 0 indexação no open.
* R4-S02 skills: `.brainroot/skills/SKILL.md` + router top-1, resto lazy. `plan` read-only default.
* R4-S03 compaction + roteamento barato→caro OpenRouter, `/cost` token/custo por turno/sessão. Prompt/roster estável p/ KV-cache. TARGET overhead ≤2k.
* R4-S04 sub-agente bounded: max turns/tokens, sem recursão, herda permissão mínima.
* Allowlist: `opencode.json`, `.brainroot/skills/`, `src/lib/CostBadge.svelte`, `src-tauri/src/features/context*`. Forbid: vector DB, always-on indexer.
* Rollback: desligar skills/compaction via config, voltar prompt base.

### B-R5 Multijanela auto-desempenho + artefato (3 dias)

* R5-S01 governor 1-timer: `ACTIVE/IDLE/SUSPENDED/TERMINATED`, políticas `preview 30s→SUSPEND→120s→TERMINATE`, sidecar 60s kill, LSP 45s kill. Auto >180MB suspende, >220MB recusa com UX leiga.
* R5-S02 soak + baseline: 20x ciclos, medida separada core/UI/content/sidecar, relatório `docs/specs/performance-reports/R5-soak.md`.
* R5-S03 `.deb` Ubuntu 24.04 x86_64 + checksum + install/run/remove, `CHANGELOG`, Wiki/history. Tag só após merge `main` por `docs/19-release-and-versioning.md`.
* Allowlist: `src-tauri/src/features/governor*`, `scripts/perf-*`, `scripts/package-linux.sh`, `docs/specs/performance-reports/`.
* Rollback: reinstalar versão anterior `.deb`, matar sidecars, limpar `~/.cache/brainroot`.

## 8. Execução imediata (copiar/colar)

```bash
pnpm install
git checkout -b batch/bR1-spike
# pin leitura (não clonar inteiro se disco fraco)
git ls-remote https://github.com/anomalyco/opencode HEAD
opencode serve --port 4096 --hostname 127.0.0.1 &
curl -s http://127.0.0.1:4096/global/health
curl -s http://127.0.0.1:4096/doc | head -c 2000
sh scripts/measure-rss.sh $(pgrep -f brainroot | head -n 1)
```

Definição de pronto por batch: código+testes escritos+docs+evidência fast-gate+PR checklist+Wiki row `IMPLEMENTED_UNVERIFIED` acordados; full CI + review + medida no Ready; merge só green latest-head; falha vira remediation issue, nunca patch silencioso.

## 9. Docs/ADR sync

* Este doc é dono do pivot. `docs/06`, `docs/07`, `docs/09`, `docs/10`, `docs/11` atualizam em B-R2/R3/R5, não agora.
* Propor em B-R1: ADR-0015 `opencode serve default + dsh acp opcional`, ADR-0016 `token-budget YAGNI`, ADR-0017 `provider hub`. ADRs append-only, supersede sem reescrever.
* `docs/17-open-questions.md`: linkar decisão pendente modelo/protocolo exato na data de execução (catálogo mutável).
