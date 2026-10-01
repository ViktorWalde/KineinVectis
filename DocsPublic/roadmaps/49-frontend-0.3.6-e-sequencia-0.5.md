# 49 — A casca da IDE na 0.3.6 e a sequência até a 0.5

> **Classe: PLANO / ALVO.** Decisão do autor em 2026-09-29: depois de fechar a
> série 0.3 até a 0.3.5, a prioridade seguinte é a **0.3.6, dedicada ao
> frontend da IDE**. O autor aprovou a direção de reorganização da casca
> descrita nos dois estudos locais em `DocsPrivate/documentacoes/`. Este plano
> traduz essa direção em cortes verificáveis; não declara nenhuma fatia pronta.
>
> Para o estado real, prevalecem código e [roadmap 40](40-estado-e-continuidade.md).
> Para a arquitetura vigente, prevalece a
> [especificação de frontend](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md).
> A [série 0.3](47-estrutura-da-v0.3.md) continua tendo seu próprio fechamento.

## 1. Tese e ordem

```text
0.3.5  fechar, provar e distribuir a série 0.3, incluindo Grafana
   ↓
0.3.6–0.3.9  reorganizar e otimizar a casca da IDE, com o editor no centro
             (trem de versões no roadmap 53 §11)
   ↓
0.4.x  versão dedicada ao ecossistema embarcado; continuar LSP, edição,
       contexto semântico e cache conforme os planos existentes
   ↓
0.5.x  Environment, Library por capacidades/providers, descoberta e Quality
```

A 0.3.6 é uma versão de produto, não apenas uma preparação técnica: o usuário
deve perceber menos ruído, melhor orientação, acesso por teclado e resposta
fluida. A 0.4 mantém o compromisso anterior de dar uma versão própria aos
embarcados; as demais frentes já previstas para 0.4 são ordenadas conforme
dependência e prova, sem diluir esse tema. A 0.5 aproveita a casca estabilizada
para crescer sem acrescentar um ícone ou painel permanente por ferramenta.

## 2. O que os dois estudos acrescentam

O estudo de frontend propõe organizar a UI por responsabilidade: trilho para
áreas, header para contexto e execução, editor dominante, docks para trabalho
temporário, status compacto, Library para descoberta. Propõe ainda classificação
das entradas do trilho, abas inferiores contextuais, foco/teclado, restauração
de layout, HUD de ambiente e configuração visual. O estudo de providers propõe
uma Library orientada por capacidades e resultados normalizados, com detecção,
execução via Jobs e integração às superfícies existentes.

Esses estudos são fontes de desenho, não inventário de entregas. A 0.3.6 paga
primeiro a organização da casca; a 0.5 introduz providers em fatias pequenas.
O [roadmap 50](50-biblioteca-e-providers-0.5.md) detalha a segunda frente.

## 3. Fronteiras que continuam valendo

- `Main.qml`, `ShellHeaderHost`, `ShellWorkspaceHost`, `ShellStatusHost`,
  `ToolWindows`, `SideRail`, `BottomPanelHost` e os controllers atuais são o
  ponto de partida. Não nasce um segundo shell.
- O header de Projeto, Git e Execução conserva seus gestos; sua apresentação
  pode ser compactada e receber contexto efetivo sem duplicar estado.
- `CoreClient` continua a fachada IPC. O Rust Core decide fatos; QML apresenta
  e recebe intenção; operações longas usam Jobs.
- `CommandDispatcher` e `command.list` continuam a rota comum de ações. A
  paleta e Search Everywhere são estendidos, não refeitos.
- O editor segue a área dominante, inclusive no notebook. O primeiro frame e
  a resposta da tecla são orçamentos de produto.
- Não entram API pública de plugins, dock graph arbitrário, stores universais,
  outro terminal, outro renderer de editor ou execução de ferramentas pela UI.
- Métricas são locais e explícitas; não há telemetria de produto/usuário.

## 4. 0.3.6 — cortes de implementação

> A arquitetura executável destes cortes (donos, fluxos, contratos, provas e
> ordem), incluindo abrir pelo terminal sem ruído e zero aviso em execução,
> está no [roadmap 53](53-arquitetura-executavel-da-0.3.6.md).

### F0 — foto, inventário e orçamento

Antes de mudar layout, registrar uma matriz por elemento visível:

| Campo | Pergunta |
| --- | --- |
| Superfície e dono | Em que host está? Quem possui a ação e o estado? |
| Frequência e propósito | É navegação, ação, estado, configuração ou aviso? |
| Duplicação | Há outro gesto que abre o mesmo destino? |
| Decisão | Manter, unir, mover, tornar contextual ou remover? |
| Alcance | Mouse, teclado, paleta e retorno ao editor funcionam? |

Auditar trilho, header, status, abas inferiores, ícones, configurações que
exigem JSON manual e restauração de workspace. Fotografar a IDE em 1024×700,
1366×768 e uma janela ampla. Medir na mesma máquina e binário: primeiro frame,
RSS inicial, tecla→frame (mediana, p95 e pior caso), abertura de painel e custo
de voltar ao editor. O [script existente](../../scripts/medir-performance.sh) e
os [orçamentos existentes](21-roadmap-de-longo-prazo.md) são a referência.

**Saída:** inventário com dono, medida inicial e decisões propostas. Nenhuma
mudança visual é considerada otimização sem antes mostrar o custo afetado.

### F1 — trilho e navegação por áreas

Evoluir as entradas que já são dados em `ToolWindows.qml` para classificar
áreas como principais, contextuais, fixadas ou ocultas. A direção aprovada é
um trilho curto: Projeto, Busca e Ambiente são candidatas a áreas principais;
Git, Embarcados, Banco, Containers, Observabilidade, Remote e Qualidade entram
conforme contexto ou escolha explícita. A detecção de uma ferramenta não fixa
automaticamente um ícone.

Esta direção **substitui a premissa de 2026-09-24 de deixar toda a auditoria do
trilho para a 0.4**. O caso de Busca requer prova concreta: ela havia saído do
trilho para evitar duplicar a busca do header/atalho. Antes de fixá-la outra
vez, testar se uma entrada de *área de busca* melhora a navegação sem repetir
um botão de ação. O widget Git do header continua sendo um gesto válido.

Manter IDs e atalhos existentes durante a migração. Persistir pin/ocultação
somente após definir escopo global ou por workspace, formato versionado e
fallback. Entrada invisível deve continuar alcançável por comando quando a
capacidade estiver disponível.

### F2 — docks e painel inferior contextuais

Reusar as tool windows e os hosts existentes. Projeto/Git/Remote no lado
esquerdo, Símbolos no direito e Terminal/Problems/Jobs/Run/Debug/Busca no
inferior são os casos de paridade. O painel inferior apresenta primeiro o que
é recorrente; Build, Tests, Debug, Jobs e Search aparecem quando há atividade,
seleção ou pin. O usuário consegue recuperar uma aba escondida pelo comando.

Auditar a aba `Tools` e a entrada `Ferramentas` do trilho. Se sua função for
coberta por Ambiente/Library/Setup, migrar todos os gestos, atalhos e estados
antes de removê-las. Não criar `DockHost` universal por simetria: ampliar o
contrato mínimo atual apenas quando left/right/bottom compartilharem uma
necessidade real. Persistência de ordem/tamanho/visibilidade tem schema e
rollback; processo antigo do Terminal nunca é restaurado silenciosamente.

Entregar restauração básica da casca por workspace: área ativa, tamanhos,
visibilidade, ordem e pins, além de arquivos/aba ativa já suportados pelo editor.
Começar com presets **Codificação**, **Depuração**, **Revisão** e **Foco** sobre
os mesmos hosts, sem duplicar painéis. O preset Embarcados será concluído com
os fluxos reais da 0.4. Trocar de preset preserva um caminho para recuperar o
layout anterior; arquivo de layout desconhecido ou antigo abre em padrão seguro.

### F3 — header, status e contexto acionável

Tornar legível o que já está disponível: projeto, perfil de build, target,
toolchain efetiva, configuração de execução, dispositivo, remoto e operação em
curso. Mostrar só os itens pertinentes ao projeto e à largura da janela. O
resumo abre o mesmo dono de configuração ou inspeção; não cria outro valor
editável. Distinguir configurado, detectado, selecionado e efetivo, sempre que
essa diferença existir no core.

O status mostra posição, saúde e trabalho em curso de forma compacta. Detalhe
fica em Environment, Problems, Jobs ou Inspector. Nem o header nem o status
devem virar inventário de todas as ferramentas. Dados sem origem ou idade não
ganham aparência de medida atual.

### F4 — foco, teclado, densidade e carga sob demanda

Definir caminho de foco entre trilho, docks, editor, header e painel inferior;
provar `Esc`, ativação por teclado e retorno ao editor. Garantir foco visível,
texto alternativo e não depender só de cor. Em janela estreita, reduzir
elementos secundários antes de reduzir o editor. Motion deve ser funcional e
respeitar preferência de movimento reduzido.

O modo **Foco** recolhe left/right/bottom e compacta header/status; ao sair,
restaura exatamente o layout que estava ativo antes, sem reconstruir defaults.
Atalho, comando e saída do modo são descobríveis e funcionam por teclado.

Painel fechado não deve manter uma árvore visual ou varredura cara sem uso.
Avaliar carregamento sob demanda e listas virtualizadas somente onde o perfil
local mostrar custo. Preservar o estado de apresentação necessário ao retorno
do painel, inclusive token efêmero do Grafana segundo sua política própria.

### F5 — prova da 0.3.6

Comparar antes/depois no mesmo ambiente e binário equivalente. Uma otimização
de desempenho exige ganho repetível acima da variação das amostras no custo
escolhido; os demais orçamentos não podem regredir. Medir também tarefa real:
abrir projeto, achar arquivo, alternar painel, compilar, entender problema,
voltar ao editor. Contar gestos e registrar onde o usuário perde o contexto.

O corte exige harness QML de visibilidade, ordem, pin/restore, foco e comandos;
qmllint, gate integrado, abertura de binário e screenshots nas três larguras.
Um item cujo comportamento ficou sem prova permanece pendente com nome e
reprodução, não vira entrega por estar desenhado.

## 5. O que a 0.4 herda

A 0.4 usa a casca resultante para o fluxo embarcado de ponta a ponta: alvo,
kit, flash, monitor, debug, dispositivo e a diferença entre bare metal e Linux
remoto. O compromisso de uma versão dedicada a embarcados permanece. O
[modelo semântico do projeto](../especificacoes/modelo-semantico-do-projeto-0.4.md)
prevê grafo de dependências, detecção de deriva e explicação de configuração
efetiva; o [roadmap 45](45-etapa4-backend-lsp-edicao-compiladores.md) prevê LSP,
edição e compiladores. Eles se integram à casca sem transferir lógica de
domínio para QML. Cache de compilação segue a decisão já registrada para a
fase dos embarcados. A 0.3.6 não anuncia esses trabalhos como prontos.

## 6. 0.5 — experiência completa sobre a casca estabilizada

| Fatia proposta | Resultado visível | Dependência |
| --- | --- | --- |
| 0.5-A | Environment Center: contexto efetivo, origem, configuração visual e inspector de compilação | F3 e fatos do core/0.4 |
| 0.5-B | Library por capacidades, estados de provider e dois providers de prova | catálogo atual, ToolDetector, Jobs |
| 0.5-C | Resultados normalizados em Problems/Quality e descoberta em Search Everywhere | 0.5-B e contratos testados |
| 0.5-D | Refinar presets, personalização de widgets e restauração de estados novos; polir ícones e acessibilidade conforme uso real | F1–F4 e uso real |

As letras são **fatias**, não versões publicadas automaticamente. O corte de
release 0.5.x será decidido pela evidência de cada fatia. A lista longa de
ferramentas do estudo é catálogo de candidatos, não obrigação de instalar ou
integrar dezenas de ferramentas na primeira 0.5.

## 7. Decisões fechadas e pontos a medir

**Fechado pelo autor:** 0.3.6 depois da 0.3 inteira; frontend como prioridade;
direção da casca dos novos estudos aprovada; 0.4 reservada ao embarcado;
preservação do core, Jobs e fluxo sem instalação silenciosa.

**Medir antes de fixar comportamento:** Busca como área permanente ou acesso
contextual; quais entradas do trilho ficam visíveis por padrão; escopo da
persistência de pin/layout; destino completo de `Tools`; quantidade de chips
que cabe no header; ganhos reais de lazy loading. O default é o que der melhor
alcance e menor ruído nos cenários medidos, sem romper atalhos existentes.

## 8. Critério de pronto do frontend

Um usuário consegue abrir e navegar no projeto, localizar comandos e áreas,
entender o contexto efetivo, acompanhar uma operação e voltar ao editor sem
procurar qual painel tomou foco. A mesma sequência funciona com mouse e
teclado, em notebook e desktop. Fechar/reabrir o workspace e entrar/sair do
modo Foco preservam o layout esperado. Nenhum provider novo exige um ícone global.
O desempenho fica dentro dos orçamentos locais e ao menos um custo escolhido
na F0 melhora de modo reproduzível. O código continua com um shell, um
dispatcher e os donos de domínio atuais.

## 9. Adendo de 2026-10-01 — Library, Welcome e o que a 0.3.6 precisa preparar

O autor enviou dois estudos novos, guardados em `DocsPrivate/documentacoes/`
(`KINEIN_VECTIS_LIBRARY_HUD_UI_UX_0_5.md` e
`KINEIN_VECTIS_LIBRARY_WELCOME_CAPABILITIES_0_5.md`). Eles descrevem a Library
e a Welcome da 0.5, mas três decisões dependem da casca que a 0.3.6 constrói.
Se a 0.3.6 não as acomodar, a 0.5 refaz o trilho e o host central:

1. **Estados independentes de área.** O trilho da F1 passa a separar
   *disponível*, *detectado*, *habilitado*, *contextual* e *fixado*, além de
   *oculto neste workspace*. As classes "principais/contextuais/fixadas/ocultas"
   da F1 são o subconjunto de apresentação desse modelo. Habilitar nunca fixa,
   e fixar nunca instala. O estado de UI (visível, ordem, dock) fica separado
   do estado de capacidade, que pertence ao core.
2. **Área de superfície completa.** Ao ser aberta, a Library ocupa a área
   principal (navegação, resultados e detalhes), e não um painel lateral
   estreito. A F2 precisa provar que o host central apresenta uma área desse tipo
   e volta ao editor por `Esc`/comando sem perder abas, foco ou layout. O editor
   continua sendo a superfície de trabalho padrão. Isso não cria um segundo shell.
3. **Overflow do trilho.** O default proposto é curto (Projeto, Busca,
   Ambiente, Library e "Mais"), com no máximo 7 itens visíveis. Git, Qualidade,
   Embarcados, ROS 2, Containers, Banco e Observabilidade entram por contexto
   ou pin. Busca continua no grupo "medir antes de fixar" da §7. A Library
   fixada por padrão é proposta nova e entra na mesma medição.

Também convergem com a F2: painel inferior fechado por padrão com abas
contextuais, `Tools` migrando para Ambiente/Library, e nenhuma configuração
suportada exigindo edição manual de JSON. A Welcome (Criar/Abrir/Clonar,
recentes, resumo do ambiente, entrada da Library), o Project Wizard por tipo e
linguagem, com Python no mesmo nível que C/C++/Rust, e a Library completa
ficam na 0.5 ([roadmap 50 §8](50-biblioteca-e-providers-0.5.md)). A F0 apenas
inventaria `StartScreen` e `LibraryPanel` com o resto da casca.
