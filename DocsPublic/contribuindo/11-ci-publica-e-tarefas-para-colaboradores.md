# CI pública e tarefas para colaboradores

> **Classe: PLANO.** Recomendação e ideia registradas em 2026-10-19 a pedido
> do autor. O autor segue sozinho até o fechamento da 0.4, agora
> **0.4.9** (limite atualizado em 2026-10-20, 57 §1.1); a porta e as tarefas
> entram depois dele (40 §5). Cards no site são
> uma proposta para outra sessão, não requisito novo de release. Este registro
> não cria issues, labels, configurações no GitHub ou alterações no site.

## 1. O lugar da colaboração

**Recomendação:** código, documentação de desenvolvimento, issues, PRs e CI
no mesmo repositório público que recebe as contribuições. Assim, quem faz
fork encontra o código e os comandos que serão usados para revisar seu PR.
Um espelho reduzido exigiria manter outro fluxo de contribuição e adaptar
gates que dependem dos documentos ausentes.

**Conferido em 2026-10-19:** a [API pública do repositório](https://api.github.com/repos/ViktorWalde/KineinVectis) informa
`ViktorWalde/KineinVectis`, `private: false`, `visibility: public`, branch
`main` e issues habilitadas. O estado em [DocsPublic/README](../README.md#repositório-público--cópia-de-distribuição)
já distingue repositório público de cópia de distribuição. A frase antiga
do 21 M7.3 sobre fonte privada não descrevia mais o projeto; não é preciso
alterar visibilidade para instalar esta CI.

O exportador da cópia limpa continua destinado à distribuição para quem usa
a IDE. Seu resultado não precisa virar outro repositório de colaboração.
O contrato técnico e a cobertura da CI estão no
[07 §2.1](07-fluxo-e-responsabilidades-dos-gates.md#21-a-camada-rápida-da-ci-2026-10-19).

## 2. Uma CI que ajuda quem chega

- Resultado da camada rápida em cada PR, com o mesmo comando reproduzível
  localmente e diagnóstico claro da etapa que falhou.
- Camada UI e testes pesados nas camadas planejadas, sem exigir hardware de
  todo contribuidor para uma correção que não depende dele. O que só uma
  placa prova continua identificado, não declarado verde por omissão.
- Guia curto de ambiente, contribuição e entrega; exemplos de tarefas
  pequenas e um mantenedor indicado para tirar dúvidas e revisar.
- Checks obrigatórios antes de merge, depois de executados e estabilizados
  no GitHub. Badge só representa a cobertura que efetivamente roda.

A CI reduz a incerteza sobre a entrega; tarefas bem delimitadas e revisão
acessível completam essa porta. Não é necessário construir um portal de
gestão próprio para começar.

## 3. A tarefa nasce no GitHub

Publicar poucas tarefas revisadas e prontas para receber contribuição, em
inglês na abertura da colaboração, após a 0.4.9. Não transformar automaticamente
cada pendência do roadmap em issue: várias ainda dependem de desenho ou
decisão do autor.

Cada issue apta a receber colaboração deve conter:

| Campo | Conteúdo necessário |
| --- | --- |
| Problema e resultado | Situação atual, reprodução ou exemplo e comportamento esperado |
| Escopo | O que entra, o que fica fora e dependências que precisam estar resolvidas |
| Ponto de entrada | Módulos/arquivos relevantes e documentos que são donos das regras |
| Requisitos | Linguagem, ferramentas, ambiente e hardware quando realmente necessário |
| Dificuldade e apoio | Nível estimado, motivo da estimativa e mantenedor para dúvidas; prazo não presumido |
| Aceite | Checklist observável do que precisa funcionar, incluindo casos de erro relevantes |
| Validação | Comandos, testes e evidências exigidos; foto antes/depois quando houver mudança visual |
| Entrega | Fork/branch, PR vinculado à issue, descrição, documentação e checklist da revisão |

Usar `help wanted` nas oportunidades revisadas e `good first issue` apenas
nas tarefas de entrada que de fato tenham escopo pequeno. Área e dificuldade
podem ser labels adicionais. O GitHub usa `good first issue` para ajudar a
expor oportunidades ([documentação](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/encouraging-helpful-contributions-to-your-project-with-labels),
consulta 2026-10-19); isso não garante recrutamento.

Formulários de issue podem tornar o briefing consistente, e um modelo de PR
pode pedir o vínculo e as evidências. As configurações futuras pertencem a
`.github/`, mas nenhuma é criada nesta fatia. A sintaxe de formulários está
na [documentação oficial](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms)
(consulta 2026-10-19).

## 4. Ideia para o site: uma apresentação das mesmas issues

O site segue a direção em inglês (40 §5); os nomes em português abaixo
explicam o fluxo. Textos finais e visual pertencem à sessão do site.

**Proposta, sem implementação nesta sessão:** em "Colaborar", cards com
resumo, área, dificuldade, requisitos essenciais e estado; filtros por área,
dificuldade e disponibilidade; expansão para objetivo, escopo, critérios de
aceite e forma de entrega. Um botão explícito **"Ver tarefa no GitHub"**
abre a issue canônica, onde a pessoa tira dúvidas, combina a execução e
vincula o PR. As regras completas continuam nos documentos do projeto.

O site não mantém outra fila, outra atribuição ou outra conversa de revisão.
O identificador é a issue do GitHub; título, briefing, labels e estados vêm
dela. A apresentação pode ser simples mesmo quando os requisitos são
técnicos: resumo primeiro e detalhes quando a pessoa pedir.

| Grupo proposto | Critério no GitHub |
| --- | --- |
| Disponíveis | Issue aberta, revisada para colaboração e ainda sem responsável confirmado |
| Em andamento | Responsável confirmado pelo mantenedor; trabalho pode estar em branch/fork |
| Em revisão | PR vinculado e pronto para revisão; um PR em rascunho não significa entrega pronta |
| Tasks realizadas | Entrega aceita pelo mantenedor, issue encerrada como concluída e PR integrado quando a tarefa exigir PR |

Em **"Tasks realizadas"**, mostrar resultado, issue, PR quando houver,
créditos dos participantes e data de conclusão. Uma issue fechada por
cancelamento, duplicidade ou descarte não conta como trabalho realizado.
Para estudo/triagem sem PR, a aceitação precisa estar registrada pelo
mantenedor na própria issue. A ligação de PR à issue e o fechamento por
merge são recursos do GitHub
([documentação](https://docs.github.com/en/issues/tracking-your-work-with-issues/using-issues/linking-a-pull-request-to-an-issue),
consulta 2026-10-19); as regras de apresentação acima são propostas nossas.

## 5. Atualização com pouco custo de manutenção

Uma primeira implementação pode buscar apenas as issues selecionadas para
colaboração e gerar um snapshot público durante o build do site ou em uma
rotina agendada. O site serve esses dados com cache; não precisa consultar o
GitHub a cada visitante. Respeitar paginação, atualização condicional/ETag e
limites da API; conservar o último snapshot válido, mostrando quando foi
atualizado e mantendo o link direto ao GitHub quando a sincronização falhar.

A REST API permite ler issues públicas sem autenticação e pode retornar PRs
no mesmo endpoint: filtrar entradas com `pull_request` e distinguir o
motivo de fechamento. Se um token for necessário ao sincronizador, ele fica
no ambiente de geração, com leitura limitada, e não no navegador.
Fontes consultadas em 2026-10-19:
[API de issues](https://docs.github.com/en/rest/issues/issues) e
[práticas da API](https://docs.github.com/en/rest/using-the-rest-api/best-practices-for-using-the-rest-api).

O mecanismo exato de sincronização e o desenho dos cards ficam para a
sessão do site. Esta ideia não acrescenta serviço, banco, telemetria ou
cadastro de colaboradores à IDE.

## 6. Preparação e critério de pronto

1. Executar a CI no repositório público e conferir um PR de fork, incluindo
   falha real, correção e logs legíveis; manter permissões adequadas a PRs.
2. Preparar modelos, labels e poucas issues de entrada, com os campos da §3
   preenchidos e a primeira pessoa conseguindo reproduzir a validação.
3. Publicar a porta de contribuição em inglês depois da 0.4.9, conforme a fronteira vigente do 57 §1.1.
4. Em outra sessão, avaliar e implementar a apresentação no site. Provar
   navegação issue/PR, filtros, atualização, falha de sincronização e os
   estados concluído/cancelado/reaberto antes de anunciar automação.

CI e portal não tornam a revisão dispensável. O mantenedor continua sendo
quem aceita escopo e entrega. Push, configuração e publicação no GitHub
continuam com o autor (40 §5).
