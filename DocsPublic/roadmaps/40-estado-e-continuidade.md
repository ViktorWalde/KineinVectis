# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`DocsPublic/README.md`). Tem de ser verdade hoje; se
> divergir do código, o código vence e este arquivo se corrige no mesmo gesto.
> Até 2026-10-15 acumulava cabeçalhos de cada retomada; o texto daquela data está em
> [`registro/40-o-estado-ate-2026-10-15.md`](registro/40-o-estado-ate-2026-10-15.md),
> e uma citação "40 §1–§4, §6 ou §8" anterior a ela se lê lá
> ([61](61-organizacao-da-documentacao.md) §3.3). Este cabeçalho é só o
> resumo técnico: cabe em 60 linhas (gate), e o que é decisão ou plano mora no dono.

> **RESUMO TÉCNICO (2026-11-09; provas funcionais datadas abaixo).**
>
> **Estado funcional:** binário **0.3.9**, protocolo **0.209.0**; etapa interna
> **0.4.1 aberta**, sem declaração de conclusão pela renumeração da entrega.
> Entrega pública decidida: **Kinein Vectis 0.4**, SemVer **0.4.0**.
> **Foundation — Engineering Consolidation Release**; visão em **62 §1.1**,
> fronteira/bloqueadores em **62 §§1–3** e inventário de dívida em **62 §3.4**.
>
> **Ordem (autor, 2026-11-08; 62 §3, com estado por etapa):** C com C23 → C++
> → Rust/C/C++ → Python → DAP/QEMU → F5–F9 → algoritmo semântico de ambiente →
> algoritmo especializado → camadas finais de teste (62 §3.5) → Windows básico
> em VM. Fundação transversal (INF-02/INF-04 no Linux, INF-03 e INF-05; §§7.390–7.392, §7.395); dívida na 0.4.
> Recorte atual: **C1** (consolidação C §4.1), com C-04 do 62 §3.4; uso confiável
> em projetos pequenos/médios e na própria IDE. Polimento UI/HUD/UX acompanha
> as jornadas; Setup exige polimento cromático/de interação (62, §7.373).
> **Fundamentos obrigatórios:** ambiente, DAP, QEMU, SVD, VFS, plotagem, notebooks,
> engenharia e Windows básico (porte completo por colaboração); B6–B10 no 62.
>
> **Trilha aberta:** os marcos antigos **0.4.1–0.4.9** permanecem no 52 para
> colaboradores; os mínimos da infraestrutura bloqueiam esta base, sem exigir
> a conclusão indistinta de toda a trilha.
> Quatro tarefas iniciais (duas bare metal, duas Linux) em contribuindo/15.
> West nativo tem desenho e produtor ausente (52 §14.56), agora tarefa BM1.
> Capacidades oferecidas precisam de regressão; hardware e emulação são provas
> distintas. Assistente completo continua aberto; recorte inicial em 62 §4.
>
> **Fechamento:** idiomas/traduções preservados (57 §3), gate completo,
> metadados, documentação e pacote Linux; release ainda não pronta.
> Contexto externo Codex `/ide`: aceite da jornada ainda a provar (62 §5).
>
> **Jornadas C anteriores:** C11 de host (§7.376), padrão C por alvo
> (§7.378) e projeto C próprio/dialetos abertos (§7.379), com regressões.
> **Identidade documental C/C++ (2026-11-04, §7.380):** gramática e estrutura
> por contexto, `languageId` por documento e reabertura com texto igual;
> quatro regressões e três mutantes; gramática `typeof` permanece aberta.
> **Referência de headers (§7.381):** alvo/fonte únicos orientam clangd;
> ambiguidade/base manual explícitas, invalidação e buffers não salvos.
> Sete regressões de identidade/headers e três mutantes; C1–C8 abertos.
> **C1 — criação (§7.383):** dialeto livre e driver C validados. **C1 —
> existentes (§7.384):** histórico separado do efetivo, recuperação do preset
> e guardas de raiz. **C1 — seletores/buffers (§7.385):** buffer preservado.
> **C1 — instalação em existentes (§7.386):** caminho no kit com prévia, sem
> troca silenciosa. **C-05 (§7.393):** CDB de presets e outras ferramentas,
> escolha sem reconfigurar e clangd na mesma pasta. **C1-18 (§7.394):** `include`
> de presets como o CMake. **C1-19–C1-21 (§7.396):** cada configuração na pasta dela.
> **Provas anteriores:** completo estrito de 2026-10-27 (§7.365): 1395 testes
> Rust, 8 CTest, 175 harnesses QML e análise C++ limpa. Matriz Linux medida em
> 2026-11-01 (§7.368), com cobertura parcial. Não validam mudanças posteriores.
> Requisitos em [build/14](../build/14-ambiente-de-desenvolvimento.md); EditorController congelado (§2).

## 1. O estado, em números

```text
protocolo   0.209.0 (crates/kinein-protocol, medido em 2026-11-09; o
            historico das versoes e' o arquitetura/03)
metodos     195 roteados e 64 eventos (os titulos do arquitetura/03, que o
            verificar-fiacao-ipc.sh confere contra o core)
harnesses   182 tst_*.qml em scripts/qml-harness (medido em 2026-11-09)
testes      1539 Rust no gate rápido de 2026-11-09 (40.7 §7.396);
            1530 Rust no gate rápido de 2026-11-09 (40.7 §7.395);
            1526 Rust no gate rápido de 2026-11-09 (40.7 §7.394);
            1521 Rust no gate rápido de 2026-11-09 (40.7 §7.393);
            1492 Rust no gate rápido de 2026-11-08 (40.7 §7.390);
            1469 Rust no gate rápido de 2026-11-08 (40.7 §7.387);
            1468 Rust no gate rápido de 2026-11-08 (40.7 §7.386);
            1462 Rust no gate rápido de 2026-11-05 (40.7 §7.384);
            1455 Rust no gate rápido de 2026-11-04 (40.7 §7.383);
            1450 Rust no gate rápido de 2026-11-04 (40.7 §7.382);
            1444 Rust no gate rápido de 2026-11-04 (40.7 §7.380);
            1440 Rust no gate rápido de 2026-11-04 (40.7 §7.379);
            1437 Rust no gate rápido de 2026-11-04 (40.7 §7.378);
            1430 Rust no gate rápido de 2026-11-03 (40.7 §7.376);
            1426 Rust no gate rápido de 2026-11-02 (40.7 §7.372);
            1395 no gate completo estrito de 2026-10-27 (40.7 §7.365);
            920 no Windows com 73 adiados e 1006 no Fedora do WSL (2026-10-14,
            §7.277); 1191 no gate rápido estrito local de 2026-10-20 (§7.321).
            1194 Rust no gate rápido estrito integral da 0.4.1e em
            2026-10-20 (§7.325), sem NÃO PROVADO.
            1200 Rust na 0.4.1f em 2026-10-20 (§7.326), gate estrito integral verde.
            1200 Rust na 0.4.1g em 2026-10-21 (§7.327), gate completo estrito verde.
            1227 Rust na 0.4.1h3 em 2026-10-21 (§7.330), gate rápido estrito verde.
            1241 Rust na 0.4.1h4a em 2026-10-21 (§7.332), gate rápido estrito verde.
            1251 Rust na 0.4.1h4b1 em 2026-10-21 (§7.333), gate completo estrito verde.
            1256 Rust na 0.4.1h4b2a em 2026-10-21 (§7.337), gate completo estrito verde em cópia isolada.
            1259 Rust na 0.4.1h4b2b1 em 2026-10-21 (§7.338), gate rápido estrito verde em cópia isolada.
            1278 Rust na 0.4.1h4b2b2 em 2026-10-22 (§7.339), gate rápido estrito verde em cópia isolada.
            1278 Rust na 0.4.1h4b2b3 em 2026-10-22 (§7.340), gate completo estrito verde com target independente.
            1297 Rust na 0.4.1h4b2b4b1 em 2026-10-22 (§7.343), gate rápido estrito verde em cópia isolada.
            1297 Rust na 0.4.1h4b2b4b2 em 2026-10-22 (§7.344), gate completo estrito verde em cópia isolada.
            1315 Rust na 0.4.1h4b2b4f/g em 2026-10-23 (§7.351–§7.352); completo anterior no §7.345.
```

Os comandos que medem estão no
[`build/comandos-de-build-e-verificacao.md`](../build/comandos-de-build-e-verificacao.md);
número sem data aqui é afirmação sobre hoje e o `verificar-docs.sh` o cobra.

## 2. A dívida da catraca: 1 arquivo, e ele tem decisão do projeto

```text
ui/qml/editor/EditorController.qml   791/400
```

**Não corte sem falar com o autor.** A decisão de mantê-lo congelado está em
[`../arquitetura/32`](../arquitetura/32-editor-por-responsabilidade.md) §8.4, e
as duas saídas estão medidas em [`39`](39-divida-tecnica-paga.md) §6: dissolver
a fachada (~190 pontos de chamada em 15 arquivos) ou corrigir a categoria.
Alterar a exceção exige decisão arquitetural registrada.

## 3. O que este arquivo já guardou

As seções §3 (as entregas de 2026-09-03/04), §4.1 e §4.2 (as ordens e panoramas
de setembro), §6 (a lacuna do dogfooding) e §8 (a varredura de 2026-09-10),
e os resumos históricos, estão no
[registro](registro/40-o-estado-ate-2026-10-15.md). São registro do dia:
antes de aceitar um item de lá como pendente, meça.

## 4. O que está aberto

Este arquivo diz qual é a próxima fatia; a ordem mora no dono.

**Fronteira vigente:** a base pública 0.4 tem critérios próprios no
[62](62-base-publica-0.4.md). A trilha completa de Embarcados 0.4.1–0.4.9
continua aberta; a tabela abaixo conserva os compromissos e referências de
escopo. Os mínimos da [infraestrutura fundamental](../especificacoes/infraestrutura-fundamental-0.4.md)
integram o fechamento; a ordem antiga não impõe o backlog inteiro. Novas tarefas no
[contribuindo/15](../contribuindo/15-tarefas-iniciais-embarcados-0.4.md).
Provas de software, emulação e hardware permanecem distintas.

A próxima frente é a consolidação das linguagens pelo [62 §3.1](62-base-publica-0.4.md#31-matriz-de-consolidação-e-prontidão)
e pela [especificação comum](../especificacoes/orquestracao-e-verificacao-de-linguagens.md).
O trabalho atual é a [etapa profunda C](../especificacoes/consolidacao-nativa-de-c.md):
C23, criação/configuração, editor/headers/LSP, ferramentas de qualidade,
ciclo diário, embarcados e plataformas. C1–C8 condicionam a consolidação C++.
O próximo recorte é **C1 — Projeto e ambiente**, com critérios no
[§4.1 da consolidação C](../especificacoes/consolidacao-nativa-de-c.md#41-c1--projeto-e-ambiente-recorte-prioritário):
consolidar criação/abertura, escolha livre de instalação e padrão, configuração,
prévia, persistência e recuperação nos donos existentes. A criação já oferece
seleção de ambiente (§7.382); essa entrega não encerrou C1. A validade do
contexto e a retomada após configure incompleto avançam na consolidação C §13;
seletores e preservação de buffer ao retornar pela árvore estão no §14;
instalação por caminho no kit de projetos existentes está no §15.
Projetos manuais arbitrários, divergências e plataformas conservam seus aceites.
`typeof(expr)`
permanece pendente em C3, e a prova QEMU pertence a C6/F3.
A primeira jornada C11 de host tem prova em software no
[registro §7.376](40.7-registro-das-entregas.md#7376--jornada-c-de-host-e-sincronização-do-contexto-após-build-2026-11-03).
A consolidação C/C++ continua pela matriz; ferramentas e verificador avançam
por jornada, conforme a
[orquestração §12.1](../especificacoes/orquestracao-e-verificacao-de-linguagens.md#121-dependências-entre-ferramentas-e-verificador).
A expansão do corpus de SDK segue C/C++, Rust/C/C++ e Python. O polimento
cromático/de interação de Instalar Ferramentas permanece pendente (§0.1 de sua
especificação), mesmo com a primeira migração funcional concluída.

| O quê | Dono da ordem e do detalhe |
| --- | --- |
| Fundamentos obrigatórios da 0.4: perfis de toolchain, DAP, QEMU, SVD, VFS, plotagem, notebooks e engenharia; contribuição e demonstração. Windows exige só a validação básica em VM desde 2026-11-08. Planejamento atualizado, sem novas provas de execução | [Infraestrutura F1–F9](../especificacoes/infraestrutura-fundamental-0.4.md); [62 §3.2](62-base-publica-0.4.md#32-fundamentos-obrigatórios-e-porta-de-contribuição) |
| Embarcados: acesso, ciclo de vida, reservas e armazenamento fechados nas fatias anteriores; intenção/UI esptool h4b2b3, consulta de sondas A1/A2 e probe-rs nativo b4b1 (core), b4b2 (UI) e b4b3 (depuração na mesma sonda) fechados; gravação esptool nativa provada no ESP32 CP2102 (b4c) e sem ambiente/configuração fora da prévia (d1); build que queima eFuses recusado (d2); clangd/FreeRTOS (e); compilador do chip no kit (f); monitor sugerido pelo projeto (g); `idf.py flash` nativo gravado no ESP32 (d3); PlatformIO nativo para ESP32 e prévia sem código do projeto (d4). Descoberta USB DFU (n1), DfuSe explícito (n2) e BIN conferido com ELF do alvo CMake (n3) fechados; consulta picotool RP2040 (o1) e UF2/ELF CMake com gravação nativa (o2) fechados em software; evoluções dessa trilha: west, matriz integrada e recuperação (52 §14.55) | [52](52-arquitetura-executavel-da-0.4.md) §14.19–14.55 e [40.7](40.7-registro-das-entregas.md) §7.326–§7.365 |
| C: etapa profunda dedicada C1–C8, incluindo C23 e preservação de C17/C11, projetos, headers/LSP, análise/memória, build/test/debug, contexto embarcado e plataformas. Jornada C11 tem prova parcial; etapa C aberta antes da consolidação C++ | [Consolidação nativa de C](../especificacoes/consolidacao-nativa-de-c.md); [orquestração](../especificacoes/orquestracao-e-verificacao-de-linguagens.md); [52 §14.14](52-arquitetura-executavel-da-0.4.md#1414-auditoria-de-c-puro-antes-do-fechamento--2026-10-20) |
| Instalar Ferramentas: dock, busca/filtro e recuperação implementados; polimento cromático, hierarquia e estados de interação pendentes. S2/revisão em lote continua aberta; Quality Center Q1–Q3 permanece na evolução | [Setup §0.1](../especificacoes/instalar-ferramentas.md#01-polimento-visual-e-de-interação-da-base-04); [62](62-base-publica-0.4.md); [Quality Center](../especificacoes/quality-center-e-gates.md) |
| Acabamento de HUD/UX/UI incorporado ao polimento da base pública 0.4. Inglês/traduções e divulgação conservam os compromissos do 57 §3; ROS 2 segue posterior | [62](62-base-publica-0.4.md); [57](57-mapa-de-versoes-ate-a-1.0.md) §3 |
| HUD por contexto 0.4.1e fechada: PlatformIO/Linux e aplicação CMake com kit Linux, gate integral estrito verde. Schema 3 dos kits segue após os bloqueadores de orquestração e Setup. CPython revalidado, MicroPython corrigido, CircuitPython ainda por separar/implementar (2026-10-20) | [52 §14.13](52-arquitetura-executavel-da-0.4.md#1413-hud-por-contexto-e-destaque-platformio--desenho-em-2026-10-20) e [41 §3.4.1](41-ecossistema-embarcados-e-python.md#341-ampliação-confirmada-e-revisão-medida-em-2026-10-20) |
| Prévia de navegação compacta continua sem escolha visual aprovada. Pesquisa de Embarcados recebida em 2026-10-21: cinco imagens conceituais, ainda sujeitas a refinamento e aprovação. Estudos de frontend são rascunhos; evoluir a UI real, sem copiar mockups. Trabalho funcional independente pode seguir | [52](52-arquitetura-executavel-da-0.4.md) §11.1/§14.16–14.17; [rascunho de Embarcados](../especificacoes/embarcados-hud-ux-0.4.md) e [regra de aprovação visual](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21) |
| CI pública: camada rápida preparada e validada localmente; execução no GitHub pela manutenção, camadas UI/pesada e matriz Arch pendentes; porta para colaboradores em preparação, com ideia de cards no site em frente própria | [contribuindo/07 §2.1](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md#21-a-camada-rápida-da-ci-2026-10-19) e [contribuindo/11](../contribuindo/11-ci-publica-e-tarefas-para-colaboradores.md); decisão no §5 |
| ROS 2: frente posterior à base pública 0.4; escolher versão exata pela pesquisa medida e provar jornadas próprias | [integracoes/40 §1.2](../integracoes/40-ros2-ecossistema-e-integracao.md#12-decisão-recuperada-escopos-por-ferramentas-e-melhorias-próprias-2026-10-19) |
| Algoritmos complementares: primeiro verificador host integrado; consolidar C/C++, introduzir Rust e integração nativa, depois Python, extraindo núcleo comum com consumidores reais. Depois de DAP/QEMU e F5–F9, o algoritmo semântico de ambiente e, por último, a fase dedicada do algoritmo especializado (2026-11-08). SDKs e S2–S4 seguem abertos | [Orquestração](../especificacoes/orquestracao-e-verificacao-de-linguagens.md); [Verificador](../especificacoes/verificador-de-correcoes-lsp.md); [52 §11.2](52-arquitetura-executavel-da-0.4.md#112-escopo-dedicado--assistente-semântico-decisão-do-projeto-2026-10-23-posição-de-2026-10-24) |
| Alocações históricas de Banco 0.4.5, consolidação 0.4.6 e estabilização 0.4.7–0.4.9 permanecem na trilha pública aberta. Não são requisito de conclusão integral antes do pacote da base | [52](52-arquitetura-executavel-da-0.4.md) §11; [62](62-base-publica-0.4.md) |
| Entrega acumulada: etapas não publicadas 0.3.6–0.3.9 e trabalho interno 0.4 integram a base pública 0.4 / SemVer 0.4.0; pacote após os aceites próprios, sem declarar concluído o backlog | [62](62-base-publica-0.4.md); [57](57-mapa-de-versoes-ate-a-1.0.md) §1.1 |
| Os adiados do Banco que seguem sem versão: MySQL/MariaDB nativo, a edição de célula, os drivers em processo externo (9b/9c); o LSP por conexão (10) e o InfluxDB 3 (11) voltaram na 0.4.5 | [59](59-fechamento-da-0.3.9.md) §2.3 (registro); a arquitetura no [arquitetura/38](../arquitetura/38-provedores-de-banco-e-linguagem.md) e [39](../arquitetura/39-drivers-externos-e-compatibilidade.md) |
| O porte para o Windows (pausado na W6b4): W6c, depois W7–W12 | [60](60-windows-e-edicao-especial.md) §3.3 e §3.4; os testes adiados na catraca `scripts/verificar-adiados-windows.sh` |
| Windows nativo e instalador: desde 2026-11-08 a 0.4 exige só a validação básica numa VM, perto do fim, e o porte completo é colaboração (porte completo obrigatório de 2026-11-03 a 2026-11-08); WSL2 conserva contexto e prova adicionais. O porte ainda tem os aceites acima pendentes | [60](60-windows-e-edicao-especial.md) §1.1 e §4; [62](62-base-publica-0.4.md) B7 |

As dívidas antigas, de antes da 0.3.9 (o TLS do `postgres`, o DAP com
probe-rs, as fatias de embarcados da trilha), estão na lista datada do §4 do
[registro](registro/40-o-estado-ate-2026-10-15.md) e nos donos delas
([35](35-ambiente-cpp-e-embarcados.md), [42](42-trilha-profunda-embarcados.md),
[52](52-arquitetura-executavel-da-0.4.md)). Quem abrir a 0.4 confere ali o
que ainda vale, medindo antes.

## 5. Decisões técnicas vigentes

Esta tabela resume as decisões de produto e aponta seus documentos canônicos.
Mudanças de escopo devem ser registradas no documento responsável.

| Tema | Decisão | Referência |
| --- | --- | --- |
| Licenciamento | Código próprio AGPL-3.0-only; documentação própria CC-BY-SA-4.0. Preservar exceções, avisos e permissões anteriores; contribuições com DCO. | [ADR-0011](../decisoes-adr/ADR-0011-copyleft-e-procedencia.md), [guia 12](../contribuindo/12-licenciamento-e-procedencia.md) |
| Pilha | Core Rust e frontend Qt/QML com ponte C++; Qt do projeto na série 6.12, inclusive testes e empacotamento. | [arquitetura](../arquitetura/ARCHITECTURE.md), [ambiente](../build/14-ambiente-de-desenvolvimento.md) |
| Escopo | IDE para software e sistemas embarcados, com C, C++, Rust e Python; sem simulação física/matemática própria. | [mapa de versões](57-mapa-de-versoes-ate-a-1.0.md), [escopo da 0.4](52-arquitetura-executavel-da-0.4.md) §10.1 |
| IA | Sem painel de IA, telemetria ou dependência de provedor de IA no produto. Ferramentas externas podem rodar no terminal. | [leitura técnica](../leitura-tecnica.md), [ADR-0005](../decisoes-adr/ADR-0005-tres-arvores-de-documentacao.md) |
| Componentes | Reusar ferramentas maduras com origem e licença verificadas; Pylance e Symbolica permanecem fora do conjunto adotado. | [integrações](../integracoes/README.md), [trilha Python](41-ecossistema-embarcados-e-python.md) |
| Integrações nativas | Docker/Podman, banco e demais integrações são domínios nativos, não um runtime de plugins. Respeitar ativação e contexto do projeto. | [arquitetura](../arquitetura/ARCHITECTURE.md), [escopo da 0.4](52-arquitetura-executavel-da-0.4.md) §11 |
| Projeto e toolchain | Ler o projeto completo, preservar configurações herdadas e distinguir detectado, selecionado e efetivo. Seleção automática segue evidências do projeto. | [toolchains](../integracoes/39-toolchains-por-alvo.md), [compilação](../build/22-compilacao-c-cpp-rust.md) |
| Instalação | Sem sudo ou instalação silenciosa pela IDE; origem oficial e verificação de integridade para toolchains gerenciadas. | [toolchains](../integracoes/39-toolchains-por-alvo.md), [Instalar Ferramentas](../especificacoes/instalar-ferramentas.md) |
| UI | Componentes reutilizáveis; layout pelo conteúdo e limites da tela; feedback claro, contexto explícito e editor dominante. Estudos visuais não são design aprovado. | [frontend](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md), [componentes](../especificacoes/sistema-de-componentes-de-ui.md) |
| Validação visual | Testar jornadas integradas com mouse/teclado; automação em instância isolada complementa o aceite visual. | [casca da IDE](../arquitetura/36-casca-da-ide.md), [escopo da 0.4](52-arquitetura-executavel-da-0.4.md) §11 |
| EditorController | Fachada congelada até revisão arquitetural; não ampliar a exceção de tamanho. | [editor](../arquitetura/32-editor-por-responsabilidade.md) §8.4, [dívida](39-divida-tecnica-paga.md) §6 |
| Ferramentas de linguagem | Usar provedores existentes; não duplicar parser, LSP ou motor semântico. GDB DAP exige versão compatível; degradações devem ser explícitas. | [semântica](56-motor-semantico-profundo-cpp-rust.md), [contrato debug](../arquitetura/03-dominios/debug.md) |
| Banco | Provedores externos versionados, perfil separado de credencial, contexto explícito de produção e custo de leitura visível. | [banco](../arquitetura/37-banco-de-dados.md), [drivers](../arquitetura/40-contrato-operacional-de-drivers.md), [cofre](../seguranca/40-cofre-de-credencial.md) |
| Grafana | Integração por HTTP API e serviço externo; navegador embutido opcional. | [integrações](../integracoes/README.md), [especificação Grafana](../especificacoes/grafana-ui-ux-0.3.5.md) |
| TLS | Dependências e licenças de TLS seguem o registro de componentes e o gate cargo-deny. | [integrações](../integracoes/README.md) |
| Embarcados | Bare metal e Linux embarcado; PlatformIO em destaque. QEMU/Renode são alvos de execução, não simuladores próprios da IDE. | [0.4](52-arquitetura-executavel-da-0.4.md) §10.1–§11, [trilha embarcados](42-trilha-profunda-embarcados.md) |
| Placas e sondas | Identificação sugere; a pessoa confirma o alvo e cada operação de gravação. Validar capacidades reais; USB/serial não comprova interface JTAG. | [conectividade](../integracoes/38-conectividade-bare-metal.md), [protocolo probe](../arquitetura/03-dominios/probe.md) |
| Templates | Catálogo curado com origem, licença e parâmetros explícitos; sem geração arbitrária de configuração. | [0.4](52-arquitetura-executavel-da-0.4.md), [biblioteca](../arquitetura/03-dominios/library.md) |
| Linguagens e integração | C e C++ primeiro; Rust como terceira linguagem do verificador, com integração Rust/C/C++; Python depois. CPython, MicroPython e CircuitPython têm contextos distintos. | [orquestração](../especificacoes/orquestracao-e-verificacao-de-linguagens.md), [Python](41-ecossistema-embarcados-e-python.md) |
| Build e SVD | C1 ao salvar para C/C++ com CDB; Rust mediante opção. SVD na área Embarcados; delta de tamanho relativo ao último build. | [0.4](52-arquitetura-executavel-da-0.4.md) §13 |
| Terminal | Comando pedido pela IDE espera o primeiro prompt (OSC 133;A); integração de bash, zsh e fish só na sessão com comando, preservando a configuração do usuário; outro shell recebe na abertura. | [03 terminal](../arquitetura/03-dominios/terminal.md), [62 §3.4](62-base-publica-0.4.md#34-inventário-sistemático-de-dívida-técnica) INF-02 |
| Setup e qualidade | Setup no dock; polimento cromático/de interação pendente na base. Quality Center mantém Q1–Q3 na trilha aberta. | [Setup §0.1](../especificacoes/instalar-ferramentas.md#01-polimento-visual-e-de-interação-da-base-04), [Quality Center](../especificacoes/quality-center-e-gates.md) |
| Ordem de implementação | Autor, 2026-11-08: C em profundidade, depois C++, Rust/C/C++ e Python; em seguida DAP e QEMU sólidos, F5–F9, o algoritmo semântico de ambiente por ecossistema e o algoritmo especializado; depois, as camadas finais de teste (62 §3.5) e a validação básica do Windows em VM. Orquestrar os motores maduros já integrados, sem reimplementá-los; fundação geral transversal, com INF-02 primeiro. Integridade, otimização e polimento por jornada; dívida que compromete esses aceites é paga na 0.4. Prontidão de projetos pequenos/médios e autodesenvolvimento exige prova; `/ide` tem aceite próprio. | [base 0.4](62-base-publica-0.4.md), [orquestração](../especificacoes/orquestracao-e-verificacao-de-linguagens.md) |
| Fronteira de versão | 0.4 / SemVer 0.4.0: Foundation — Engineering Consolidation Release. Capacidades declaradas integradas, documentadas e validadas, com limites explícitos; eliminar a dívida que compromete seus aceites. F1–F9 obrigatórios, com Windows nativo só no recorte básico em VM desde 2026-11-08; melhorias futuras não exigem esgotamento na 0.4. | [62 §1.1](62-base-publica-0.4.md#11-visão-da-04--foundation) e [§3.4](62-base-publica-0.4.md#34-inventário-sistemático-de-dívida-técnica), [infraestrutura](../especificacoes/infraestrutura-fundamental-0.4.md), [versões](57-mapa-de-versoes-ate-a-1.0.md) §1.1 |
| Internacionalização | UI em inglês no polimento; documentação integral em inglês e divulgação com a base finalizada, preservando português e strings traduzíveis. | [versões](57-mapa-de-versoes-ate-a-1.0.md) §3 |
| Frentes posteriores | ROS 2, colaboração remota, motor semântico profundo e estudo de CLI conservam seus planos. A trilha pública de contribuições já tem quatro tarefas; primeiro verificador na base 0.4. | [versões](57-mapa-de-versoes-ate-a-1.0.md), [ROS 2](../integracoes/41-ros2-hud-ux-analise-projeto.md), [pesquisa](../pesquisa/02-analise-e-refatoracao-complementares.md) |
| CI e recursos | CI pública compartilha os gates; limites de RAM, swap zero por escopo e limpeza de temporários devem ser verificáveis. | [gates](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md), [comandos](../build/comandos-de-build-e-verificacao.md) |
| Documentação | Material público é técnico; contexto pessoal e instruções de agentes são locais e não publicáveis. | [política documental](../README.md#repositório-público--cópia-de-distribuição) |

## 7. O registro das entregas

O registro datado (§7.5 em diante) é o
[`40.7-registro-das-entregas.md`](40.7-registro-das-entregas.md): o índice das
partes fechadas em [`registro/`](registro/README.md) e a fase aberta. A
numeração é a mesma desde que saiu daqui em 2026-10-01: "40 §7.148" e "40.7
§7.148" são a entrada `### 7.148`. Ele é LOG (nunca se reescreve) e este
arquivo é ESTADO.
