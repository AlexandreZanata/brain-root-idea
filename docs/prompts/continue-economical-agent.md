Leia `AGENTS.md`, `docs/14-mvp-scope.md`, `docs/18-mvp-execution-plan.md`, `docs/19-release-and-versioning.md`, `docs/20-project-history-and-wiki.md` e os ADRs ativos antes de agir.
No GitHub, localize o batch ativo: priorize a primeira microetapa pronta; se não houver, identifique o PR Ready com `CI_PENDING`.
Para uma issue, confirme branch, PR em Draft, SHA inicial, Definition of Ready, arquivos permitidos e proibições; não complete lacunas por suposição.
Implemente apenas o resultado pedido e pare com `BLOCKED` se surgir divergência, nova decisão, dependência, permissão, segredo ou mudança fora do escopo.
Execute os comandos e gates exatos da issue, revise diff, segurança e cleanup, e faça um único commit `type(scope): resumo (refs #N)`.
Envie a branch, publique evidências na issue, atualize checklist do PR e linha da Wiki, vincule e feche manualmente a issue; não inicie outra microetapa.
Se era a última issue, sincronize histórico/Wiki/changelog/versão, marque o PR Ready, registre `CI_PENDING` e SHA no PR/Wiki e encerre sem esperar o CI.
Se o PR já estava Ready, consulte CI/revisão uma vez: pendente → encerre; falhou → issue de correção; verde no último SHA e aprovado → merge, fechamento e tag.
