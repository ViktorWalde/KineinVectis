# 41 — ROS 2: HUD, configuração visual e análise de projeto

> **Classe: PLANO / ALVO de experiência.** Consolidado em 2026-10-21 a partir
> do estudo e das referências fornecidos pelo autor. Não implementa ROS 2,
> define novo IPC ou fixa sua versão. Frente e ensaio **após a 0.4.9**;
> pesquisa técnica e matriz de suporte no [40](40-ros2-ecossistema-e-integracao.md).

**UX em rascunho, esclarecimento do autor em 2026-10-21:** imagens e
reformulações do estudo não são fiéis à UI atual nem representam design
aprovado. Incrementar/polir a interface real, mantendo a aprovação visual
do autor como referência de direção. Aplicar a
[regra comum de frontend](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21),
sem tratar escolhas de organização abaixo como contratos já aprovados.

## 1. Princípio e fronteiras

A IDE compreende e explica o projeto, mantendo código e terminal como áreas
principais. RViz2, Gazebo e ferramentas 3D continuam programas externos.
Configuração visual oferece orientação a iniciantes e deixa ambiente,
comandos e arquivos padrão inspecionáveis por usuários experientes.

Este documento é o dono da experiência ROS 2; não duplica a investigação do
40, o sequenciamento do [57](../roadmaps/57-mapa-de-versoes-ate-a-1.0.md),
o [Quality Center](../especificacoes/quality-center-e-gates.md) ou a
[reformulação de Instalar Ferramentas](../especificacoes/instalar-ferramentas.md).
As duas referências visuais do instalador estão nessa especificação.
Preparar componentes reutilizáveis na 0.4 não significa implementar ROS 2.

## 2. Interface preservada

| Região atual | Integração proposta |
| --- | --- |
| Barra superior | Indicação compacta do ambiente selecionado e acesso ao seu dono de configuração |
| Explorador esquerdo | Arquivos como hoje; identificação discreta de pacotes; visão lógica opcional posterior |
| Editor central | Edição normal de URDF, Xacro, launch, mensagens e demais arquivos; continua prioritário |
| Painel inferior | Análise ROS 2 contextual em Problemas ou subvisão/aba, reutilizando resultados e Jobs |
| Status | Resumo pertinente com detalhes acessíveis; sem inventário permanente de ferramentas |

Escolher entre filtro/subvisão de Problemas e aba contextual após ensaio de
usabilidade, sem duplicar resultados. Não criar painel direito permanente,
dashboard no lugar do editor ou visualizador obrigatório ao abrir URDF.
Configuração, análise e instalação são acessos contextuais, não formulários
sempre visíveis. Fechar o painel não deve fazê-lo reabrir a cada aviso comum.

## 3. Configurar ambiente

```text
Indícios de workspace → detectar opções → explicar ambiguidade
→ usuário seleciona → resolver contexto → conferir ambiente efetivo
```

Reusar seleção válida por workspace; diante de várias opções, mostrar
distribuição/prefixo, compilador, Python, middleware RMW e cadeia de
underlays/overlays. Separar **detectado, selecionado e efetivo**. O mesmo
contexto resolvido deve alimentar LSP, CDB/build, execução, testes e debug.
Comandos, variáveis e caminhos detalhados ficam sob demanda, sem exigir JSON
manual. Perfil salvo não prova que a instalação continua disponível.

Abrir uma pasta permite reconhecer manifests/metadados; não autoriza build,
instalação, `rosdep install`, launch Python, expansão Xacro, consultas ao
grafo de execução ou mudanças em arquivos de shell. Resolução que execute
scripts de ambiente precisa de ação explícita e contexto de confiança.
Ausência/ambiguidade mantém arquivos e terminal utilizáveis com limitações
explicadas; não converter ausência de informação em incompatibilidade.

## 4. Analisar Projeto ROS 2

| Nível | Trabalho | Gatilho |
| --- | --- | --- |
| Contínuo | Diagnósticos do arquivo e leitura estática de metadados reconhecíveis | Feedback de baixo custo; sem executar o projeto |
| Workspace | Ambiente, pacotes, dependências, underlays/overlays, toolchains e código integrado | Pedido explícito, com escopo arquivo/pacote/workspace/ambiente |
| Execução | Nós, tópicos, QoS, parâmetros e comunicação observada | Inspeção opcional com timeout, cancelamento e contexto informado |

Avaliar `colcon list`, `colcon info` e `colcon graph` no ensaio técnico do 40;
versão, plugins, comportamento e riscos precisam de prova. Um comando de
descoberta pode carregar extensões: não classificá-lo como leitura puramente
estática apenas por não compilar. Runners e resultados reutilizam os donos
atuais; novos contratos só após o desenho fino, antes do produtor.

O painel informa escopo, contexto, progresso real e cancelamento. Filtros
por gravidade, origem, pacote e verificação; seleção estável, evidência,
navegação ao arquivo/linha ou configuração e cópia do diagnóstico. Resultados
parciais e antigos ficam identificados; mudança de ambiente invalida a prova.

## 5. Diagnósticos com evidência

Separar gravidade (erro/aviso/informação), verificação
(confirmado/pendente/não verificado/obsoleto) e origem
(código/ambiente/dependência/build/execução). Ícone e texto acompanham cor.

Cada item apresenta: problema, origem/localização, evidência, consequência
prática e ação possível. Dados de caminho/versão semelhantes ou diferentes
não provam sozinhos compatibilidade/incompatibilidade.

| Exemplo ilustrativo | Tratamento esperado |
| --- | --- |
| Python do editor diverge do ambiente ROS | Comparar contexto efetivo e evidências de resolução/import; abrir configuração |
| Dependência `robot_interfaces` não resolvida | Origem em `package.xml`, descoberta e ambiente usados; navegar e orientar |
| Interfaces geradas ainda não verificadas | Pendente de build/validação; não afirmar erro de código |
| Launch dinâmico sem resultado estático suficiente | Explicar limite e oferecer inspeção explícita; não avaliar Python ao abrir |
| Falha de ferramenta/parser | Preservar saída e comando; não mostrar “nenhum problema” |

São exemplos, não resultados reais do workspace do autor. Reanálise deve
ser possível por escopo, sem build completo desnecessário. Jobs demorados
ficam fora do laço de UI; a view não executa ferramenta externa.

## 6. Corrigir e editar

Fluxo: diagnóstico → explicação → prévia/diff → confirmação → aplicar pelo
dono atual → revalidar. Selecionar diagnóstico não aplica alterações.
Configuração de ambiente e quick-fix seguro podem entrar primeiro;
refatoração estrutural exige fixtures, conflitos, rollback e prova próprios.

URDF, Xacro, launch, `.msg`, `.srv`, `.action`, YAML e `package.xml` continuam
arquivos textuais normais. Inicialmente priorizar XML/manifests, referências
literais, CDB real multipacote e Python com interfaces geradas. Expansão
Xacro e execução launch dependem de autorização; não prometer compreensão
universal de código dinâmico. Grafos de pacotes, inspetores de modelos,
refatoração e diagnóstico profundo de runtime ficam em incrementos posteriores.

## 7. Implementação e aceites por dependência

1. Ensaiar workspace real no terminal e em ferramenta de referência;
   registrar atritos de Python, CDB, dependências e overlays no 40.
2. Configuração visual/contexto efetivo, reutilizando os mecanismos da IDE.
3. Análise de ambiente/pacotes com navegação e evidência no painel inferior.
4. Integração C++/Python, interfaces geradas, build/teste/debug verificáveis.
5. Runtime/modelos avançados conforme os casos medidos.

Microinterações, teclado, contraste e movimento reduzido acompanham cada
fatia, sobre Theme/Kv*; não esperar a última fase para ter UI utilizável.
Movimentos curtos são hipóteses, não metas de desempenho: aproximadamente
80–120 ms em hover, 100–150 ms em seleção e 140–200 ms em expansão, sujeitos
aos tokens existentes e medição com QML Profiler. Não animar continuamente
código, árvore ou diagnósticos. A especificação comum de acabamento permanece
no [52 §14.17](../roadmaps/52-arquitetura-executavel-da-0.4.md#1417-auditoria-geral-de-frontend-antes-do-appimage--decisão-de-2026-10-20).

Matriz mínima: projeto sem ROS; ambiente válido/ausente/ambíguo; distribuição
trocada; dependência ausente; interface pendente de build; análise cancelada;
resultado obsoleto; ferramenta falha; workspace sem confiança; navegação por
teclado; listas grandes; escalonamento e carga concorrente. Provar C++ e Python
consumindo interfaces geradas, e distinguir testes de fixture de execução real.
Não fixar distribuição ROS, pacote, versão Qt ou contrato a partir do mockup.
