# Requisitos não funcionais do core em C++

Dono: o limite medido de cada qualidade do core, o método de medida e quem
impõe. Requisito sem quem impõe não conta como cumprido.

Estado: rascunho de 2026-11-22. Nenhum requisito está cumprido ainda. Os IDs
herdados do 19 (19:114-123) mantêm o nome, para a linhagem ficar visível; os
limites foram aceitos pelo Viktor na DP-03 do 64 (64:938). Os requisitos novos
estão marcados assim, e o valor deles é **proposta** até a decisão.

## Referência de medida

- Máquina: a de medição do projeto (16 núcleos, 21 GiB, Linux), a mesma do 64 e do 65.
- Tamanhos: arquivos C/C++ de **85, 2.500, 20.000 e 40.000 linhas**. As linhas
  de base da 0.3 usaram arquivos Rust de 85, 2.463 e 19.970 linhas (65:670-673);
  as fixtures C/C++ ainda precisam ser geradas e medidas no produto atual
  antes da primeira comparação ([testes](../engineering/test-strategy.md#desempenho)).
- Toda medida diz data, binário, perfil e método.

## Execução e protocolo

| ID | Requisito | Limite | Método | Quem impõe |
| --- | --- | --- | --- | --- |
| RNF-EXE-01 | O laço do core só faz trabalho de estado; processo, disco do projeto e rede nunca rodam nele | Pedido de estado: p99 ≤ 10 ms, máximo ≤ 50 ms | Registro por pedido | Guarda que reprova E/S no laço nos testes e no build de depuração |
| RNF-EXE-02 | Todo pedido tem prazo e é cancelável | `core.ping` e o cancelamento respondem em ≤ 50 ms com uma ferramenta presa | O experimento do 64 §2.3 como teste | Teste com ferramenta falsa travada por trava, sem `sleep` |
| RNF-CON-01 | O contrato tem fonte única | Mudar um método ou campo só de um lado reprova um teste | Mutante do contrato | Teste de contrato ([protocolo](../architecture/protocol-and-contract.md)) |
| RNF-OBS-01 | Todo pedido, processo e job deixa registro estruturado, sem segredo | 100% dos pedidos com registro; zero segredo | Teste com coletor em memória; teste de redação | O ponto único de entrada do pedido |

## Edição e desempenho

| ID | Requisito | Limite | Método | Quem impõe |
| --- | --- | --- | --- | --- |
| RNF-EDI-01 | Editar não custa o arquivo inteiro | Por evento de edição de 1 caractere no arquivo de 20.000 linhas: trabalho do core p95 ≤ 16 ms; nenhuma mensagem depois da abertura carrega o documento inteiro | Registro; tamanho das mensagens | Teste nos tamanhos de referência. Adaptado do 19:121, que media o `syntaxTree.update`, que sai |
| RNF-PERF-01 *(novo)* | Nenhum caminho do core cresce mais que linearmente com o tamanho do arquivo ou do projeto | De 20.000 para 40.000 linhas, o tempo cresce no máximo 2,5 vezes (**proposta**) | Teste de complexidade nos quatro tamanhos | Teste de complexidade por caminho ([testes](../engineering/test-strategy.md#complexidade)) |
| RNF-MEM-01 | Orçamento de memória do core com edição | RSS do core ≤ 60 MB depois de abrir, indexar e editar o arquivo de 20.000 linhas | `/proc/<pid>/status` | Teste de desempenho com veredito |
| RNF-ED-01 *(novo)* | As condições da DP-10 no produto integrado | R1 e R12 pelo método da prova, R10 com o documento inteiro, IME real e Windows | 65 §20.3 e 65:874-879 | Os testes da fase D ([motor de edição](../architecture/neovim-editing-engine.md)) |

## Segurança e privacidade

| ID | Requisito | Limite | Método | Quem impõe |
| --- | --- | --- | --- | --- |
| RNF-SEG-01 | Sem confiança, nada que o projeto define é executado | Zero marcas no projeto hostil para o `workspace.open`, para cada pedido automático da UI e para o primeiro arquivo de cada linguagem | Os vetores do 64 §2.1 e os do Neovim (65 §20.2, item 14) como fixture | O lançador e o teste do projeto hostil, com mutante da guarda |
| RNF-SEG-02 | Todo processo diz de onde veio | Todo lançamento registrado com programa, origem e decisão de confiança | Registro do lançador | A origem é campo obrigatório do pedido de lançamento |
| RNF-SEG-03 *(novo)* | O `nvim` não executa programa nenhum | Zero `execve` a partir do `nvim`; sem Landlock, o projeto não confiável abre sem o editor novo | Os 14 vetores do 65 §20.2, item 14 | Teste hostil com mutante que tira a regra Landlock |
| RNF-PRIV-01 | Zero telemetria; nada sai da máquina sem um gesto da pessoa | Zero conexão de rede iniciada pelo core sem pedido explícito | Teste com rede negada | RNF1 do 19 (19:66) |

## Dados e durabilidade

| ID | Requisito | Limite | Método | Quem impõe |
| --- | --- | --- | --- | --- |
| RNF-DUR-01 *(novo)* | O texto digitado sobrevive à queda de qualquer processo | Perda máxima de **2 s** de digitação contínua, qualquer que seja a duração da digitação (decisão do Viktor de 2026-11-22; no produto atual, desde a correção do INF-26 no QML) | `kill -9` na UI, no core e no `nvim` depois de digitação longa sem pausa (INF-26) | Teste de queda com mutante que tira o prazo máximo |
| RNF-DUR-02 *(novo)* | Gravação do projeto é atômica | Nenhuma queda deixa arquivo do projeto truncado ou misturado | Queda injetada entre a escrita e a troca de nome | Teste de queda na gravação |
| RNF-DADOS-05 | Estado da pessoa e cache não moram na árvore do projeto | Zero arquivo por pessoa sob o `.kinein`; abrir sem confiança não grava no projeto | Listagem da árvore antes e depois | Teste de abertura ([persistência](../architecture/persistence-and-durability.md)) |

## Qualidade de engenharia

| ID | Requisito | Limite | Quem impõe |
| --- | --- | --- | --- |
| RNF-QUA-01 *(novo)* | Código sem aviso | Zero aviso no perfil `dev` com GCC e no `lint` com Clang | `-Werror` nos presets de desenvolvimento ([perfis](../engineering/build-profiles-and-hardening.md)) |
| RNF-QUA-02 *(novo)* | Análise estática limpa | Zero achado do clang-tidy | Preset `lint` ([clang-tidy](../engineering/static-analysis-clang-tidy.md)) |
| RNF-QUA-03 *(novo)* | Testes verdes sob os sanitizers | Toda a suíte passa nos perfis `dev`, `asan-ubsan` e `tsan` | `cmake --workflow` de cada preset |
| RNF-QUA-04 *(novo)* | C++23 sem extensões, e o código compila em C++26 | Sondagem `cxx26-probe` verde | Teste `build.contract` |
| RNF-ESC-01 | A biblioteca do core não depende de Qt nem de periferia | `core/lib` compila sem o Qt instalado | O preset `dev` não procura o Qt ([fronteiras](../architecture/boundaries-and-responsibilities.md)) |
| RNF-POR-01 *(novo)* | Linux x86_64 primeiro; Windows é condição da DP-10 | Os testes da etapa 10 do 62 §3 no Windows | NÃO VERIFICADO: nada do core novo rodou no Windows |
