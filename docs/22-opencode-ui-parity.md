# 22 — Paridade visual com o OpenCode (pin @34aa427)

**Status:** Adopted as batch **B20** (OpenCode visual parity) on 2026-09-25 — implementation branch `batch/b20-opencode-parity`. Previously a proposed staging plan kept untracked outside the repository.
**Objetivo:** a UI do BrainRoot indistinguível do OpenCode para o olho do
usuário — mesmo layout, tokens, tipografia e comportamentos — reimplementada
em Svelte leve sobre o shell Tauri, sem Electron, sem Tailwind, sem bundle Solid.

## Adoção do batch B20 (2026-09-25)

- **Batch:** B20 · **Branch:** `batch/b20-opencode-parity`, cortada de `f658e40` (a finalização do registro do B19 no `main`). O B19 foi liberado como `v0.0.15` no commit de merge `9e2b51c`.
- **Issues:** [#130](https://github.com/AlexandreZanata/brain-root-idea/issues/130) U1, [#131](https://github.com/AlexandreZanata/brain-root-idea/issues/131) U2, [#132](https://github.com/AlexandreZanata/brain-root-idea/issues/132) U3, [#133](https://github.com/AlexandreZanata/brain-root-idea/issues/133) U4, [#134](https://github.com/AlexandreZanata/brain-root-idea/issues/134) U5, [#135](https://github.com/AlexandreZanata/brain-root-idea/issues/135) U6, [#136](https://github.com/AlexandreZanata/brain-root-idea/issues/136) U7 (`risk:high`) — sete dentro do limite de dez por batch.
- **Decisão do mantenedor:** o B20 é a paridade com o OpenCode. Isso **supersede** o plano anterior que reservava o B20 para "conversation/Canvas polish" em `docs/specs/b19-frontend-system-and-usability-plan.md`.
- **Pin verificado:** `anomalyco/opencode @ 34aa427434b054afcce7184764aa681159b5d769` existe (25/09/2026). Cada etapa cita **tamanhos medidos** da árvore git do pin, não estimativas.
- **v1 contra v2 — o pin traz as duas árvores.** Cada etapa nomeia qual é a autoridade, em vez de misturar: U4 usa `prompt-input-v2.tsx` (21.690 B) e não a v1 de `prompt-input.tsx` (61.170 B); U5/U6 usam a família v2 (`settings-v2/`, e a variante v2 do selecionador de modelos quando ela divergir); U3 e U7 usam a árvore v1 `pages/session.tsx` + `timeline/`, porque é a que a §2 nomeia.
- **Três capacidades não existem hoje, e nenhuma será fingida.** Sem listagem de projeto/workspace, o picker `@` (U4) e as file-tabs/review (U7) entregam a forma real com estado vazio honesto. Sem PTY/Process Manager, o terminal (U7) espelha a saída do turno e se declara espelho. Sem fronteira de permissão do agente, os docks de permissão/pergunta (U6) ficam **fora de escopo** — uma UI que parecesse conceder ou negar autoridade sem fronteira aplicada seria uma capacidade falsa.
- **Os orçamentos da §3 são critério de aceite, não adjetivo:** CSS ≤35 KB gzip, JS ≤60 KB, zero dependência nova de UI, fontes ≤300 KB com subset, input do composer em 1 frame, troca de aba sem remount, 20 ciclos de painel voltando ao baseline em ±5 MB medidos.
- **Ordem:** U1 é pré-requisito de todas as outras (a camada de tokens); U2 depende de U1; U3 e U5 dependem de U1; U4 depende de U1; U6 depende de U1; U7 depende de U1 e idealmente de U3.

## 0. Leitura honesta do "exatamente igual"

Paridade é **visual + comportamental**, não DOM idêntico nem pixel-perfect:

* Motor diferente (Solid vs Svelte) renderiza sub-pixel/antialias de texto de
  forma distinta; screenshots lado a lado valem como `≈`, nunca como prova binária.
* O OpenCode usa Tailwind + `@opencode-ai/ui` (+ v2) + `tw-animate-css` +
  scroll-timeline animations (Chrome-only; WebKitGTK ignora) + Electron chrome.
  Nada disso entra: tokens são portados à mão para as vars BrainRoot.
* Nossos gates continuam valendo e conflitam em pontos com o original
  (ex.: `disabled`, roving `tabindex`, `user-select:none`): onde conflitar,
  visual igual com DOM acessível próprio — registrado por etapa.
* Fontes do original (`Inter` variável + `JetBrainsMono Nerd Font Mono`, ver
  `packages/app/src/index.css`): checar licença/bundle por etapa, subsetar,
  carregar lazy. Sem confirmar licença, não entra no `.deb`.

## 1. Mapa da fonte (pin R1, só leitura)

* Frame: `packages/app/src/app.tsx`, `entry.tsx`, `pages/layout.tsx` (86KB),
  `pages/layout-new.tsx`, `desktop-menu.ts`.
* Abas: `components/titlebar-tab-strip.tsx`, `titlebar-tab-nav.tsx`,
  `titlebar-tab-popover.tsx`, `titlebar-tab-gesture.ts`, `titlebar-tab-order.ts`,
  `titlebar.tsx`, `pages/session/session-sortable-tab*.tsx`,
  `session-header.tsx`, `session-new-view.tsx`.
* Sessão: `pages/session.tsx` (80KB), `pages/session/timeline/`,
  `session-side-panel.tsx` (40KB), `file-tabs.tsx`, `review-tab.tsx`,
  `terminal-panel*.tsx`, `message-gesture.ts`, `helpers.ts`.
* Composer: `components/prompt-input*.tsx`, `prompt-workspace-selector.tsx`,
  `prompt-project-selector.tsx`, `pages/session/composer/`,
  `use-composer-commands.tsx`, `use-session-commands.tsx`.
* Modelos: `dialog-select-model*.tsx`, `dialog-manage-models.tsx`,
  `model-tooltip.tsx`, `settings-models.tsx`, `pages/session/session-model-helpers.ts`.
* Sistema: `packages/ui/src/{components,v2,theme,styles}` (design system),
  `packages/app/src/index.css` (imports Tailwind + fades/scroll-timelines),
  `components/settings-v2/`, `dialog-*.tsx`, `usage-exceeded-dialogs.tsx`,
  `pages/error.tsx`, `command-palette.ts`.

## 2. Etapas (cada uma: 1 branch-issue, 1 commit, screenshots antes/depois)

### U1 — Tokens + tipografia (base de tudo)
Portar para vars BrainRoot: escala `v2-background-bg-*`, texto/ícones
(`--v2-icon-icon-accent` etc.), raios, espaçamentos, `font-size-x-small…`,
pesos. Inter como UI + JB Mono para código/terminal (subset latin, lazy,
licença confirmada). Orçamento: CSS total ≤30KB gzip; sem `@font-face`
bloqueante no primeiro frame.
Aceite: mesma paleta light/dark lado a lado + gates a11y verdes.

### U2 — Frame + titlebar tabs
`titlebar.tsx` + tab-strip/nav/popover/gesture/order: abas com fechar,
indicador de atividade, drag-reorder, popover de overflow, `+` nova sessão,
loader de update (respeitando `prefers-reduced-motion`; sem scroll-timeline).
Nosso `SessionTabs.svelte` vira a base. Aceite: 5 abas + overflow + reorder
por teclado e mouse, foco visível, sem `tabindex` custom.

### U3 — Timeline da sessão
`pages/session.tsx` + `timeline/`: grupos user/assistant, cards de tool
(called/input-delta/progress/success/failed), diff view, reasoning colapsável,
`session-context-tab`, gestos de mensagem, retry/fork. Reusa `Turn.svelte`
estendido; partes não-texto viram placeholder honesto até o backend expor.
Aceite: turno com 3 tools + diff + reasoning colapsa/expande igual.

### U4 — Composer v2
`prompt-input-v2` + `composer/`: textarea auto-grow, `@` file/dir/mcp pickers
(`dialog-select-*.tsx` como referência), botão de modelo, switch de agente,
toggles de tools, submit/stop, slash-commands (`/`) via paleta. Cada picker
abre sob demanda e morre no close (governor). Aceite: paridade de layout em
3 larguras (desktop, 1080px, 640px).

### U5 — Model selector + manage models
`dialog-select-model*.tsx` + `manage-models`: busca com filtro, agrupamento,
linha `nome · provider · ctx · $/M` (dados que já temos do B-R3 S01), tooltip
de modelo, empty/error states. Substitui nosso `<select>` nativo mantendo
a11y (lista custom só com `aria-activedescendant` + mesmo teclado).
Aceite: 460 itens filtram <1 frame de input; leitor anuncia contagem, não itens.

### U6 — Sistema de diálogos + telas
Settings-v2, replies de permissão/pergunta (liga no Permission Broker),
`usage-exceeded`, `error.tsx`, command-palette, `home.tsx` + projetos.
Aceite: cada diálogo abre/fecha por teclado, foco retorna ao invocador,
nenhum vaza segredo para log/screenshot.

### U7 — Painéis file-tabs/terminal/review
`file-tabs.tsx`, `terminal-panel*.tsx`, `review-tab.tsx`, `session-side-panel.tsx`
como views COLD do governor (1 HOT). Terminal real fica para depois do
Process Manager com PTY; até lá, painel espelha saída do turno.
Aceite: abrir/fechar 20x volta ao baseline ±5MB (soak script).

## 3. Orçamentos por etapa (TARGET, medir em cada)

* CSS gzip total ≤35KB; JS gzip total ≤60KB (hoje 34KB).
* Nenhuma dependência nova de UI (sem Tailwind/radix/motion).
* Fontes ≤300KB somadas com subset; fora do caminho crítico.
* Interação: input do composer responde em 1 frame; troca de aba sem remount.

## 4. Riscos assumidos

* Pin desatualiza: re-mapear diffs do OpenCode por etapa (começar sempre pelo pin).
* Scroll-timelines e micro-animações Chrome-only: substituir por estático/respeito
  a `prefers-reduced-motion` — diferença visual documentada, não bug.
* i18n (20 línguas no original): fora; pt-BR/en no nosso vocabulário de produto.

## 5. Definição de pronto por etapa

Skin lado a lado aprovada + gates (`check-fast.sh`) verdes + sem dependência
nova + sem segredo em log/screenshot + spec `docs/specs/U*.md` + commit único.
Testes novos escritos na etapa e executados no portão versionado (ADR 0014).
