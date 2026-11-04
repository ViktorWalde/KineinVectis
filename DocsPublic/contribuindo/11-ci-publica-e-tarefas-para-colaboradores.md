# CI pública e tarefas para colaboradores

> **Classe: PLANO.** Fluxo público de contribuição, CI e requisitos de tarefas.
> A abertura de novas frentes de colaboração segue o mapa de versões
> (57 §1.1). Cards no site são uma proposta independente, sem alterar os
> critérios de release. Este documento define o plano técnico, não configura
> automaticamente serviços externos.

As [quatro tarefas iniciais de Embarcados](15-tarefas-iniciais-embarcados-0.4.md)
estão delimitadas no checkout: duas bare metal e duas Linux embarcado.
A trilha 0.4.1–0.4.9 permanece aberta, independente do fechamento da base
pública 0.4; os mínimos obrigatórios e a porta de contribuição seguem a
fronteira vigente do [62](../roadmaps/62-base-publica-0.4.md). Preparar essa
porta integra o fechamento da 0.4, antes da divulgação.

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

Preparar poucas tarefas revisadas e prontas para receber contribuição, em
inglês para a divulgação da base 0.4. Não transformar automaticamente
cada pendência do roadmap em issue: várias ainda dependem de desenho ou
decisão do projeto.

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

Os modelos locais de [bug](../../.github/ISSUE_TEMPLATE/bug_report.md),
[tarefa](../../.github/ISSUE_TEMPLATE/contribution_task.md) e
[PR](../../.github/pull_request_template.md) pedem reprodução, escopo e evidências.
Eles não criam issues, labels nem checks obrigatórios no serviço. Sua presença
no checkout não comprova ativação no GitHub. O formato segue a
[documentação oficial](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/syntax-for-issue-forms)
(consulta 2026-11-03).

### 3.1 Briefings iniciais para revisão e publicação

Os recortes abaixo partem dos mecanismos existentes e dispensam hardware.
São briefings locais; antes de publicar, conferir duplicatas, completar a
reprodução/comandos, indicar revisor e confirmar o rótulo. As jornadas do
[15](15-tarefas-iniciais-embarcados-0.4.md) são contribuições mais amplas.

| Tarefa | Entrada e escopo | Aceite e validação | Rótulo candidato |
| --- | --- | --- | --- |
| Documentar um projeto mínimo Eigen/CMake pela Biblioteca | `crates/kinein-core/src/library/catalog.rs`, `library/` e seção Bibliotecas do manual; usar a entrada Eigen já existente | Projeto host que configura/compila, mostra o diff de ativação/desativação e explica dependência ausente; comandos e resultado numérico reproduzíveis; gate documental | `good first issue`, documentação |
| Documentar seleção e troca de presets sem edição manual | `crates/kinein-core/src/tests/cmake.rs`, dono CMake e manual; usar os casos existentes de vários presets e escolha persistida | Tutorial exercitado na IDE, incluindo preset removido e recuperação; distinguir projeto/local/kit; gate documental | `good first issue`, documentação |
| Preparar fixture SVD pequena e redistribuível | F5 da infraestrutura; descrição genérica com periférico, registrador e campos, origem/licença explícitas | XML conferido contra schema CMSIS-SVD, endereços e bits esperados descritos; não implementar parser nem alegar suporte a um chip real | `good first issue`, fixtures |
| Ampliar a matriz real CMake/presets | `crates/kinein-core/src/tests/cmake.rs` e donos CMake/configaction; selecionar lacuna após conferir a cobertura atual | Caso com CMake real, caminho com espaços/Unicode e configuração herdada preservada; reproduzir falha, corrigir a lacuna e executar regressões pertinentes | `help wanted`, build |

Implementar DAP/probe-rs completo, VFS ou um parser SVD inteiro exige desenho
e conhecimento de domínio. Essas frentes devem ser divididas antes de receber
`good first issue`; o suporte DAP/probe-rs já presente precisa ser estendido,
sem criar outro adaptador da IDE.

## 4. Ideia para o site: uma apresentação das mesmas issues

O site segue a direção em inglês (40 §5); os nomes em português abaixo
explicam o fluxo. Textos finais e visual pertencem à sessão do site.

**Proposta, sem implementação nesta verificação:** em "Colaborar", cards com
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
3. Validar a porta de contribuição em inglês no fechamento da base 0.4,
   conforme o 62 B10; divulgar somente com o estado técnico conferido.
4. Em frente própria, avaliar e implementar a apresentação no site. Provar
   navegação issue/PR, filtros, atualização, falha de sincronização e os
   estados concluído/cancelado/reaberto antes de anunciar automação.

## 7. Demonstração e divulgação da base 0.4

O material de divulgação depende dos aceites B1–B10 do 62. Destacar integração
de ferramentas, configuração assistida e o ciclo sem placa, distinguindo as
plataformas e perfis comprovados. Evitar afirmação de ser a primeira IDE a
combinar essas áreas e comparações de mercado sem evidência.

Preparar uma demonstração curta, com alvo de até um minuto: abrir a fixture,
mostrar toolchain/perfil, compilar, iniciar QEMU, atingir breakpoint via DAP e
inspecionar um valor. Identificar versão, máquina e ferramentas; declarar cache
e cortes quando usados. A duração do vídeo não é um benchmark do ciclo inteiro.
O roteiro só fica aceito com gravação do produto funcional, sem simular estados.

README, changelog e texto para fóruns devem apontar release, demonstração,
limitações e tarefas revisadas. A mensagem central é: **Kinein Vectis 0.4
integra firmware, ferramentas científicas e emulação em uma IDE open source,
com ambiente e escolhas visíveis.** Usar essa mensagem como anúncio de entrega
somente após a validação. Até lá, comunicar os itens como desenvolvimento.

CI e portal não tornam a revisão dispensável. O mantenedor continua sendo
quem aceita escopo e entrega. Push, configuração e publicação no GitHub
continuam com o autor (40 §5).
