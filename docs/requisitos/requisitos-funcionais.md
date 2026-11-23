# Requisitos funcionais do core em C++

Dono: o que o core faz, em que nível de prioridade e qual teste aceita cada
capacidade. Os limites de tempo, memória e segurança são do
[requisitos-nao-funcionais.md](requisitos-nao-funcionais.md).

Estado: rascunho de 2026-11-22. Nenhum requisito está implementado. Os testes
de aceite citados ainda não existem.

## Escopo

- **Linguagens:** C e C++. Rust e Python voltam depois que o núcleo estiver
  sólido (ADR-0012).
- **Fora do core em C++:** banco de dados, Grafana, remoto por SSH, contêineres,
  Cargo e Python. Os métodos `lsp.*`, `syntaxTree.*` e `draft.*` da 0.3 saem
  com o Neovim ([motor de edição](../arquitetura/motor-de-edicao-neovim.md)).
- **Origem:** cada capacidade aponta os métodos da 0.3 que a cobrem hoje
  (protocolo 0.223.0, tabela de rotas de `crates/kinein-core/src/lib.rs`). O
  aceite herdado é o teste diferencial contra o core em Rust, salvo as
  divergências intencionais ([estratégia de testes](../engenharia/estrategia-de-testes.md#diferencial)).

## Níveis

| Nível | Critério de entrada |
| --- | --- |
| **N0** | Sem ele, não dá para desenvolver a Kinein na própria Kinein (dogfooding) |
| **N1** | Paridade com a 0.3 para C e C++ |
| **N2** | O diferencial: ambiente C/C++, embarcados e análise semântica própria |

A jornada de dogfooding que define o N0 é a do 65 §13 (65:484-497), restrita a
C e C++: abrir o repositório, editar, usar o clangd, compilar e testar com o
CMake, usar o terminal, reiniciar sem perder alteração e lidar com ferramenta
ausente ou travada.

## N0: dogfooding

| ID | Capacidade | Métodos 0.3 | Aceite |
| --- | --- | --- | --- |
| RF-CORE-01 | Ciclo de vida: responder ao ping, informar a versão do protocolo, encerrar limpo | `core.ping`, `core.shutdown` | Diferencial; encerrar sem órfão (R5 do 65) |
| RF-CORE-02 | Todo pedido pode ser cancelado e tem prazo; operação longa é job com progresso | `job.list`, `job.cancel` | Teste com ferramenta falsa travada (RNF-EXE-02) |
| RF-WS-01 | Abrir, fechar e consultar o workspace; lista de recentes | `workspace.open`, `workspace.close`, `workspace.status`, `workspace.recent.*` | Diferencial; abrir pasta sem confiança não grava nela (RNF-DADOS-05) |
| RF-WS-02 | Confiança no projeto: estado, concessão e o modo restrito | `workspace.trust` | Fixture do projeto hostil com zero marcas (RNF-SEG-01) |
| RF-FS-01 | Ler, gravar (com conteúdo esperado e gravação atômica), criar, renomear, copiar, mandar para a lixeira | `fs.read`, `fs.write`, `fs.createFile`, `fs.createDirectory`, `fs.rename`, `fs.copy`, `fs.trash`, `fs.delete`, `fs.list` | Diferencial; queda no meio da gravação não deixa arquivo parcial (RNF-DUR-02) |
| RF-FS-02 | Busca no projeto e substituição | `fs.search`, `fs.findFiles`, `fs.replace` | Diferencial; teste de complexidade (RNF-PERF-01) |
| RF-FS-03 | Mudança externa no disco vira evento | eventos do watcher | Teste com arquivo mudado por fora |
| RF-TC-01 | Toolchain C/C++ no modelo Detectado → Selecionado → Efetivo | `tools.detect`, `tools.status`, `toolchain.get`, `toolchain.set`, `toolchain.setKit` | Diferencial; compilador do projeto nunca é executado sem confiança |
| RF-CMK-01 | CMake: estado, presets, alvos, configurar, cache e reparo da pasta de build; `compile_commands.json` do preset ativo | `cmake.status`, `cmake.presets.list`, `cmake.targets.list`, `cmake.configure`, `cmake.inspectCache`, `cmake.repairBuildDir` | Diferencial; configurar sem confiança é recusado |
| RF-BLD-01 | Compilar, executar e testar o alvo, com saída e diagnóstico | `build.run`, `run.start`, `run.stop`, `test.discover`, `test.run`, `runConfig.*` | Diferencial; cancelar mata o grupo de processos |
| RF-TERM-01 | Terminal: abrir, escrever, redimensionar, rolar, selecionar, fechar | `terminal.*` | Diferencial; falha de `exec` vira erro tipado (L18) |
| RF-ED-01 | Criar e supervisionar o `nvim` com contenção, entregar o endereço do socket à UI e manter o espelho do buffer | novo | Os 15 itens da prova (65 §20.2) no produto integrado |
| RF-ED-02 | Gravar o arquivo do editor pelo `BufWriteCmd` e recuperar o texto depois de queda pelo espelho | `draft.*` sai | Queda com zero tecla perdida acima do limite (RNF-DUR-01) |
| RF-LSP-01 | Uma sessão clangd por raiz, criada pelo lançador, com o contexto do core (`compile_commands.json`, `--query-driver` só de compilador detectado sem confiança), ligada ao Neovim pelo relé | `lsp.*` sai | R7 e R11 do 65 no produto integrado |
| RF-SET-01 | Configuração da IDE e do projeto | `settings.get`, `settings.set` | Diferencial |

## N1: paridade C/C++

| ID | Capacidade | Métodos 0.3 | Aceite |
| --- | --- | --- | --- |
| RF-GIT-01 | Git: status, diff, log, blame, stage, commit, branches; desligado sem confiança | `git.*` | Diferencial; o `fsmonitor` do projeto nunca roda (64:119) |
| RF-DBG-01 | Depuração por DAP: iniciar, parar, passos, pilha, variáveis, avaliar, memória, desmontagem | `debug.*` | Fluxo vertical do 64:914 |
| RF-CMK-02 | Edição assistida do CMake: alvos, bibliotecas, padrão da linguagem, presets, sanitizers | `cmake.add*`, `cmake.set*`, `cmake.create*Preset`, `cmake.enable*`, `cmake.fetchContent`, `cmake.findPackage` | Diferencial; prévia antes de gravar |
| RF-CA-01 | Receitas de configuração com prévia e aplicação | `configAction.list`, `configAction.preview`, `configAction.apply` | Diferencial |
| RF-PRJ-01 | Criar projeto C/C++ a partir de modelo | `workspace.createProject`, `workspace.createFolder` | O modelo novo ignora o estado da IDE no git (L13) |
| RF-SETUP-01 | Guia de instalação das ferramentas | `setup.list`, `toolchain.installable`, `toolchain.install` | Diferencial; nenhum `--version` roda no laço (L08) |
| RF-FMT-01 | Formatar pelo `clang-format` do projeto, com confiança | `format.text`, `format.capabilities` | Diferencial |

## N2: o diferencial

A ordem e o desenho do N2 são do Viktor, na biblioteca sem Qt. A lista abaixo
só registra o que a 0.3 já cobre, para nada se perder.

| ID | Capacidade | Métodos 0.3 |
| --- | --- | --- |
| RF-EMB-01 | Gravação e sondas | `runConfig.flashProposal`, `probe.*`, `dfu.*`, `picotool.*` |
| RF-EMB-02 | Serial | `serial.*` |
| RF-EMB-03 | SDKs: ESP-IDF, Zephyr (`west`), PlatformIO | `idf.*`, `espidf.*`, `west.*`, `platformio.*` |
| RF-EMB-04 | Memória e registradores | `memory.x`, `debug.readMemory`, `debug.disassemble` |
| RF-SEM-01 | Análise semântica própria: índice, definição, referência e escopo locais | `index.*`, `local.*` |
| RF-SEM-02 | Qualidade e cobertura | `quality.run`, `coverage.*` |
