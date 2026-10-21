# Quality Center — verificações e gates compreensíveis

> **Classe: PLANO / ALVO.** Inclusão inicial na **0.4.x decidida pelo autor
> em 2026-10-21**; evolução e polimento incrementais depois. Não descreve
> uma funcionalidade já entregue. Ordem e dependências no
> [52 §11.1](../roadmaps/52-arquitetura-executavel-da-0.4.md#111-fluxo-de-execução-reorganizado--2026-10-21).

**Frontend ainda em rascunho (autor, 2026-10-21):** inclusão funcional na
0.4.x não significa desenho visual aprovado. Superfície, hierarquia e
controles propostos devem evoluir a UI atual e receber refinamento/aprovação
visual do autor, conforme a [regra comum](arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21).
Nenhum mockup demonstra suporte ou define um layout final.

## 1. Resultado e limite da primeira entrega

Um iniciante consegue entender e selecionar verificações compatíveis,
executar um gate local e descobrir por que ele passou, falhou ou não pôde
verificar algo. Um usuário experiente consulta ferramenta, argumentos,
configuração efetiva e saída original. O mesmo fluxo valida código humano
ou produzido por agentes externos; a IDE continua independente de IA.

A primeira entrega reúne **build, análise estática e testes já suportados**,
com perfil de rigor existente, explicação contextual, seleção de etapas,
execução explícita, cancelamento e relatório copiável. Inclui a criação
visual de um gate local simples por workspace. Não exige um novo catálogo
de providers, hooks Git, geração de CI ou um editor universal de regras.

## 2. Base conferida em 2026-10-21

Conferência de código e contratos; não é uma nova rodada de prova funcional.

| Capacidade existente | Dono a estender | Limite relevante |
| --- | --- | --- |
| Build e diagnósticos | `handlers/build.rs`, `build/`, [build.*](../arquitetura/03-dominios/build.md) | Depende do contexto e da ferramenta efetivos |
| Clippy, clang-tidy e Ruff | `handlers/build.rs`, `build/tidy.rs`, [quality.*](../arquitetura/03-dominios/quality.md) | `quality.run` aceita Cargo, Python, CMake e Make; C/C++ exige CDB |
| Descoberta e execução de testes | [test.*](../arquitetura/03-dominios/test.md), `test/` | Cargo, CTest e pytest; descoberta pode compilar/importar código |
| Rigor global/workspace | `settings.rs`, `SettingsController.qml`, `SettingsBuildPage.qml` | `strict`, `balanced`, `relaxed`; padrão atual `strict` |
| Resultados, problemas e jobs | `JobsController.qml`, `ProblemsPanel.qml`, `TestsPanel.qml`, `JobsPanel.qml` | Hoje há comandos separados; seleção, gate agregado e relatório são trabalho novo |
| Cobertura | [coverage.*](../arquitetura/03-dominios/coverage.md), `CoverageController.qml` | Rust/Python; C/C++ ainda exige integração de instrumentação, sem promessa de suporte |
| Mudanças de configuração | [configAction.*](../arquitetura/03-dominios/configAction.md) | Reusar prévia e aplicação do dono; não sobrescrever regras do projeto |

Os caminhos Rust da tabela pertencem a `crates/kinein-core/src/`; os QML
ficam nas áreas correspondentes de `ui/qml/`, conforme o
[mapa de módulos](../arquitetura/01-mapa-de-modulos.md).
Os scripts que validam a própria Kinein são referência de engenharia;
**não são um gate portátil para qualquer projeto do usuário**.

## 3. Superfície e jornada

Acesso pelos comandos e pela área de Qualidade/Ferramentas existentes;
a Biblioteca poderá oferecer o mesmo acesso quando sua evolução chegar.
Abrir uma superfície contextual nos hosts atuais, com retorno ao editor.
Resultados continuam em Problemas, Testes, Build e Jobs, ligados por IDs;
não criar uma segunda lista de diagnósticos ou executor.

```text
Abrir Qualidade → entender contexto/perfil → selecionar verificações
→ revisar plano → executar → acompanhar/cancelar → abrir resultado/origem
```

A revisão mostra workspace, alvo, toolchain, perfil efetivo, etapas,
comandos, pré-requisitos e o que pode executar código do projeto. Build e
testes são execução: não iniciam ao abrir a tela, descobrir providers ou
selecionar uma regra. Arquivos ainda não salvos exigem indicação clara do
que será verificado no disco; não apresentar resultado como prova do buffer.

## 4. Perfis e regras sem configuração concorrente

Reutilizar `rigorProfile` e sua precedência atual: workspace sobre global,
com regras explícitas do projeto respeitadas pela ferramenta. O Quality
Center edita o mesmo valor mostrado em Configurações e explica efeitos por
linguagem. A composição de etapas do gate é uma escolha separada de rigor.

Os nomes **Starter / Recommended / Strict / Custom**, vindos do estudo,
ficam como candidatos para presets de seleção. Não substituir nem remapear
silenciosamente `strict/balanced/relaxed`, criar quatro políticas paralelas
ou mudar o padrão existente. Uma primeira versão pode usar os nomes atuais
e uma seleção personalizada de etapas.

Cada regra/verificação deve explicar:

- O problema que ajuda a encontrar e um exemplo curto em linguagem comum.
- Ferramenta, versão/caminho quando medidos, escopo e configuração de origem.
- Efeito real da escolha, custo, pré-requisitos e limitações conhecidas.
- Como conferir os argumentos e voltar à configuração anterior.

Exemplo conceitual: “conversões numéricas que podem perder precisão” pode
explicar `-Wconversion` **se o compilador selecionado a suportar**. Isso não
habilita a flag automaticamente nem promete equivalência entre compiladores.
Código próprio, dependências e código gerado precisam de escopos explícitos.
Regras herdadas de `.clang-tidy`, Ruff ou presets não viram toggles que
prometem contrariar uma configuração que a ferramenta continua usando.

Na 0.4, a edição inicial cobre o rigor e a seleção de verificações que os
donos atuais executam. Quando uma regra individual ainda não tiver aplicação
visual segura, mostrar a origem e orientar; não simular um controle funcional.
Expansão do editor de regras fica para incrementos medidos.

## 5. Gate local mínimo e resultado verificável

O usuário nomeia o gate, escolhe etapas suportadas, marca as obrigatórias e
revê a política de falha. Salvar é explícito, por workspace, com formato
versionado e preservação de configurações desconhecidas/inválidas. O desenho
fino decidirá o formato no dono de configurações antes de escrever o código;
este plano não aprova nomes novos de métodos IPC.

Executar usa os Jobs e runners existentes. O core possui a coordenação e o
veredito; QML apresenta e solicita. Se precisar de um plano agregado, acrescentar
somente a coordenação consumida por esse fluxo, mantendo IDs dos jobs filhos,
cancelamento e saída original. Não lançar shell arbitrário a partir de
campos de regra. Validar dependências das etapas: build antes de testes que
precisam do artefato/CDB. Evitar dois builds ou duas análises iguais na mesma
rodada; não repetir automaticamente por atualização de tela.

| Estado | Significado |
| --- | --- |
| Aprovado | Etapa executada com sucesso, no contexto/configuração registrados |
| Falhou | Critério configurado não foi satisfeito, com evidência |
| Não verificado | Não executado, ferramenta/pré-requisito ausente ou suporte indisponível |
| Cancelado | Interrupção solicitada; preservar os resultados parciais |
| Obsoleto | Código, alvo, toolchain ou configuração relevante mudaram após a rodada |
| Não selecionado | Fora desta rodada; não equivale a aprovação |

Gate só aprova quando todas as etapas obrigatórias selecionadas foram
executadas e aprovadas. Etapa obrigatória não verificada deixa o gate
inconclusivo; nenhuma seleção, zero testes quando testes são obrigatórios,
falha de parser ou execução parcial não produzem “tudo aprovado”. Distinguir
avisos de análise, falhas de infraestrutura e testes falhos. A política de
avisos é mostrada antes de executar e nunca mascara o resultado original.

O relatório inclui escopo, data, perfil, comando/configuração efetivos,
versões confirmadas, IDs de execução, contagens e limitações. Copiar/exportar
omite segredos e permite revisar caminhos locais. Serve para investigação
ou para entregar a um agente externo, sem integração obrigatória com provedor.
Não atribuir nota arbitrária de segurança/qualidade nem afirmar ausência de
defeitos porque as verificações passaram.

## 6. Incrementos com aceite próprio

| Fatia | Entrega | Prova mínima |
| --- | --- | --- |
| Q1 — expor e explicar | Superfície com capacidades reais, rigor existente, ações e acesso aos resultados | Cargo, C/C++ com CDB e Python; ausências explicadas; mesmo valor em Settings/Qualidade |
| Q2 — selecionar e executar | Gate local simples, persistência explícita, sequência, cancelamento e relatório | Reabrir gate; passar/falhar/não verificar/cancelar; correlação por workspace/job; nenhum falso verde |
| Q3 — consolidar | Navegação, resultados obsoletos, configuração com prévia onde suportada e jornada de iniciante | Troca de workspace/perfil/toolchain, erro de configuração e recuperação; mouse/teclado e UI nativa |

Q1 começa junto da frente construir/C na 0.4.2, depois do Setup acoplado.
Q2/Q3 fecham até a consolidação 0.4.6; 0.4.7–0.4.8 auditam o conjunto.
Desenho de contratos antes do produtor, harnesses e testes de despacho onde
necessários, mutações pertinentes e gate estrito de cada fatia continuam
obrigatórios. Não usar a edição especial para adiar aceites desta entrega.

## 7. Evolução posterior

Library por capacidades e providers permanece no
[50](../roadmaps/50-biblioteca-e-providers-0.5.md); ela expõe a base inicial
sem anunciá-la novamente. Editor amplo de regras, sanitizers e cobertura
adicionais, políticas pre-commit, hooks e geração de CI são incrementos
posteriores, com configuração/revisão explícitas e prova de compatibilidade.
Gate local não é imposição para colaboradores nem proteção de branch.
Checks ROS 2 dependem da frente pós-0.4.9, conforme
[integração ROS 2](../integracoes/41-ros2-hud-ux-analise-projeto.md).

O polimento posterior melhora clareza, densidade, acessibilidade, desempenho
e interações sobre esta base. A primeira entrega já deve ser utilizável.
