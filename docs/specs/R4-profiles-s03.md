# B-R4 S03 Cost profiles — IMPLEMENTED_UNVERIFIED 2026-09-25

Slice: perfis Fast/Balanced/Max roteando por preço vivo do catálogo + agente.
Sem modelo hardcoded em lugar nenhum; picker manual continua soberano.

## Allowlist respeitada

* `src/agentHost.ts` (`CostProfile`, `pickProfileModel` puro + testes escritos)
* `src/agentHost.test.mjs` (roteamento sobre fixture)
* `src/lib/ConversationPanel.svelte` (segmentado de perfis + wrap do composer)
* `src/App.svelte` (estado `profile`, `selectProfile`, manual volta a custom)
* Este spec. Rust intocado.

## Comportamento

* Fast = plan + menor $/M; Balanced = build + menor $/M com ≥128k ctx (fallback: menor);
  Max = build + maior $/M (proxy de frontier, documentado no `title`).
* Sem preços (offline/stale sem match) = primeiro da lista; sem modelos = no-op.
* Escolha manual de modelo/agente limpa o perfil (custom honesto, sem mentir ativo).
* Falha no meio do perfil: agente reverte sozinho, modelo marca `failed` (comportamento herdado).

## Micro-gate rápido (sem testes — ADR 0014)

* `pnpm run check` 0 erros (1 reparo de edição no caminho, verificado por diff)
* `pnpm build` OK (JS 100.38KB, +~1.5KB)
* `check-docs.sh`, `check-security.sh`, `check-modules.sh`, `git diff --check` (abaixo)

## Deferred release tests (rodar no gate)

* T-R4-06 `agentHost.test.mjs` (perfis sobre fixture).
* T-R4-07 E2E: com catálogo vivo, Fast seleciona plan+barato; Balanced build+roomy;
  Max build+caro; manual após perfil limpa o ativo.
* T-R4-08 sem catálogo: perfil é no-op, picker manual funciona.

## Rollback

Reverter este commit único. Toggle Plan/Build e picker seguem intactos.
