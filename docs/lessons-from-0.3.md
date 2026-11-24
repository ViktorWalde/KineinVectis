# Lições da 0.3

Dono: o que a série 0.3 errou e acertou, com a evidência de origem e o
mecanismo que, no core em C++, impede a repetição. As citações seguem a forma
do [README](README.md#como-citar-a-03).

O padrão dos erros fundamentais é um só: **a regra existia como intenção, e
nada a impunha onde o efeito acontece.** A confiança era consultada em alguns
lugares, e não onde o processo nasce. A durabilidade era um `Timer` de QML. O
prazo era escolha de cada handler. O gate lia texto, e não a estrutura. Por
isso cada lição abaixo termina numa coluna "quem impõe", e não numa recomendação.

## Erros que não podem se repetir

| ID | O que aconteceu | Origem | O que o core em C++ faz | Quem impõe |
| --- | --- | --- | --- | --- |
| L01 | A UI era dona do texto: cada pedido LSP e cada `syntaxTree.update` levava o arquivo inteiro, e a volta trazia de 1 a 8 MiB por tecla | ENG-55; 65:67-75; 64:211-232 | O Neovim é o dono do texto. O core recebe um espelho versionado pelo `changedtick`, e nenhum pedido depois da abertura leva o documento inteiro | RNF-EDI-01 ([requisitos](requirements/non-functional-requirements.md)) |
| L02 | Abrir um arquivo de 19.970 linhas levava de 46,2 a 47,9 s: 8,1 vezes mais texto, 48,6 vezes mais tempo | INF-25; 62:1371-1394 | Todo caminho que depende do tamanho do arquivo tem teste de complexidade | RNF-PERF-01 |
| L03 | A CPU ao digitar 40 teclas ia de 2,31 s (85 linhas) a 24,75 s (19.970 linhas); a causa não foi atribuída | INF-27; 62:1426-1447 | O trabalho do core por evento de edição não depende do tamanho do arquivo | RNF-EDI-01 |
| L04 | O rascunho tinha debounce sem prazo máximo: digitando sem pausa de 1,5 s, nada era gravado, e a queda perdia tudo | INF-26; 62:1395-1425; `ui/qml/editor/EditorPersistenceController.qml:119-158` | A durabilidade é regra do core, com prazo máximo, e não um temporizador da UI | RNF-DUR-01 |
| L05 | Um projeto aberto como não confiável executava código dele por cinco caminhos | INF-23; 64:107-130 | A confiança é decidida no lançador, onde o processo nasce, inclusive para o argumento que nomeia programa (`--query-driver`) | RNF-SEG-01 ([processos](architecture/processes-and-trust.md)) |
| L06 | Processo criado em cerca de 122 lugares, sem política única | ENG-51; 64:297 | Só o lançador cria processo | clang-tidy, a medir ([gates](engineering/gates-and-effectiveness.md)) |
| L07 | O core não registrava duração, processo nem fila | ENG-52; 62:1994-1998 | Um registro estruturado por pedido, por processo e por job, aberto num ponto só | RNF-OBS-01 |
| L08 | Uma chamada lenta ao `git` prendeu o laço por 18,0 s, e o `core.ping` e o `job.cancel` esperaram 17,8 s | ENG-50; 64:193-209 | O laço não bloqueia; toda chamada externa roda fora dele, com prazo e cancelamento | RNF-EXE-01, RNF-EXE-02 |
| L09 | A resposta não tinha correlação com o pedido, e o pedido não tinha prazo nem cancelamento | ENG-53; 62:1999-2002 | Todo pedido tem id, prazo e cancelamento; resposta velha é descartada pela geração | RNF-EXE-02 ([execução](architecture/execution-and-concurrency.md)) |
| L10 | Uma carga antiga do contexto de compilação caía por cima da nova | INF-22; 66:56-60 | Geração por workspace e por documento: o resultado de uma geração velha não se aplica | Teste de ordem ([testes](engineering/test-strategy.md)) |
| L11 | No Rust, um `Parser::new()` a cada análise incremental e o realce percorrendo a árvore inteira a partir da raiz | `crates/kinein-core/src/lang/service.rs:230`, `:292` e `:340` (lido; nenhum doc registra) | Estado incremental guardado entre chamadas; trabalho proporcional à mudança. Com o Neovim, a análise do editor sai do core; a lição vale para os algoritmos da biblioteca | Teste de complexidade (RNF-PERF-01) |
| L12 | O contrato parou: o schema ficou no protocolo 0.62.0 enquanto o código ia a 0.223.0, e nada o lia | ENG-54; 64:304; `schemas/ipc.schema.json:4` | O contrato tem uma fonte, e os dois lados são conferidos contra ela | RNF-CON-01 ([protocolo](architecture/protocol-and-contract.md)) |
| L13 | Estado da pessoa (rascunhos, sessão) gravado na árvore do projeto, que dois dos três modelos não ignoravam no git | ENG-56; 64:152-156 e 64:315-318 | Estado e cache fora da árvore do projeto, pelas classes do XDG | RNF-DADOS-05 ([persistência](architecture/persistence-and-durability.md)) |
| L14 | QML de área falando direto com a ponte (8 arquivos, 99 acessos) e uma fachada de 791 linhas no `EditorController.qml` | ENG-22, ENG-24; 63:620-622 | A QML decide apresentação e intenção; fato, regra, caminho efetivo e confiança são do core | Regra de fronteira ([fronteiras](architecture/boundaries-and-responsibilities.md)) |
| L15 | Gates de heurística de texto: 36 de 120 defeitos plantados detectados, e 14 de 15 formas corretas rejeitadas | ENG-41; 66:46-49 e 66:96-99 | Só entra gate determinístico. Gate próprio só entra com eficácia medida | [gates](engineering/gates-and-effectiveness.md) |
| L16 | A suíte reprovava sob a própria carga, sem defeito plantado | INF-24; 62:1302-1370 | Teste não sincroniza por tempo; prazo é parâmetro do teste; a intermitência se mede em lote | [testes](engineering/test-strategy.md) |
| L17 | O gate deu verde exercitando um binário velho da árvore, e não o que a rodada compilou | INF-31; 62:1544-1568 | O teste confere o artefato da própria rodada (o `build.elf_hardening` lê o ELF compilado) | `core/tests/build/` |
| L18 | A falha de `exec` numa sessão de terminal abortava o filho e gravava um core dump | INF-29; 62:1480-1509 | O lançador devolve a falha de `exec` como erro tipado, sem aborto | Teste do lançador |
| L19 | Prazo estourado era reportado como "obsoleto", mandando a pessoa repetir um pedido que dá o mesmo resultado | INF-30; 62:1510-1543 | Cada falha tem a própria causa: prazo vencido é `DeadlineExceeded`, cancelado é `RequestCancelled` | Teste de prazo |
| L20 | Documentação e cerimônia crescendo mais que o produto: 126 mil linhas de Markdown contra 166 mil de Rust e 74 mil de QML e C++; dos últimos 400 commits, 99 de docs e 5 de refatoração | 64:319-320; medido em 2026-11-22 | Documento novo só com pedido; um fato, um dono; regra imposta por ferramenta não é repetida em texto | [README](README.md#as-regras-desta-árvore) |

## Acertos que ficam

| ID | Acerto | Origem | Como continua |
| --- | --- | --- | --- |
| A01 | Dois processos, com o core dono dos fatos e a QML só enviando intenção | 64:48-53 | Igual, agora com o Neovim como terceiro processo ([fronteiras](architecture/boundaries-and-responsibilities.md)) |
| A02 | Contrato de protocolo explícito, com versão | 64:50; `crates/kinein-protocol/src/lib.rs:121` | O core em C++ fala o mesmo protocolo ([protocolo](architecture/protocol-and-contract.md)) |
| A03 | A UI já escolhe o binário do core por variável de ambiente | `ui/src/core_client_process.cpp:240` (`KINEIN_CORE_BIN`) | A troca entre os dois cores não exige mudança na UI |
| A04 | Jobs com cancelamento | 64:52 | Toda operação longa é job cancelável |
| A05 | Lançador único como intenção (`owned_child`, SIGTERM e depois SIGKILL) | 64:297-298, 64:435-436 | Vira regra imposta ([processos](architecture/processes-and-trust.md)) |
| A06 | Toolchain no modelo Detectado → Selecionado → Efetivo | `DocsPublic/especificacoes/instalar-ferramentas.md:117` | RF-TC-01 ([requisitos](requirements/functional-requirements.md)) |
| A07 | Zero telemetria, tudo local | 19:66; `README.md:232` | RNF-PRIV-01 |
| A08 | Medir a eficácia de um gate com defeitos plantados e formas corretas | 63 §D.5; 66:96-118 | Método obrigatório para gate próprio ([gates](engineering/gates-and-effectiveness.md)) |
| A09 | Critérios de refutação escritos antes da prova | 65:38-61 | Os critérios de pronto de cada marco são escritos antes do código |
| A10 | Flags estritas, sanitizers e `-Werror` já compilavam a UI | `cmake/KineinStrictOptions.cmake` | São a base dos perfis do core ([perfis](engineering/build-profiles-and-hardening.md)) |
| A11 | O CMake é a verdade do projeto C/C++ | 64:923 | O core lê o modelo do CMake, não o reconstrói |
| A12 | Ferramenta ausente é NÃO PROVADO, nunca aprovado; refatoração não edita os testes que a provam | R-TEST-01 e R-TEST-07 do 63 | Regras da [estratégia de testes](engineering/test-strategy.md) |
| A13 | Não seguir link, não abrir o que não é arquivo regular, não gravar por cima do que não se entende | R-DADOS-02 e R-DADOS-03 do 63 | Regras da [persistência](architecture/persistence-and-durability.md) |
