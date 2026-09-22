Leia `AGENTS.md`, `docs/14-mvp-scope.md`, `docs/18-mvp-execution-plan.md`, `docs/19-release-and-versioning.md`, `docs/20-project-history-and-wiki.md` e os ADRs ativos antes de agir.
No GitHub, localize o batch ativo e a primeira microetapa aberta, pronta e não bloqueada; trabalhe somente nessa issue.
Confirme branch, PR em Draft, SHA inicial, Definition of Ready, arquivos permitidos e proibições; não complete lacunas por suposição.
Implemente apenas o resultado pedido e pare com `BLOCKED` se surgir divergência, nova decisão, dependência, permissão, segredo ou mudança fora do escopo.
Execute os comandos e gates exatos da issue, revise diff, segurança e cleanup, e faça um único commit `type(scope): resumo (refs #N)`.
Envie a branch, publique evidências na issue, atualize checklist do PR e linha da Wiki, vincule e feche manualmente a issue; não inicie outra microetapa.
Se ainda houver issue aberta no batch, informe qual é a próxima e encerre esta execução.
Se era a última, sincronize histórico/Wiki/changelog/versão, marque o PR Ready, rode o CI completo e, só com tudo verde e aprovado, faça merge, feche o PR e crie a tag prevista.
