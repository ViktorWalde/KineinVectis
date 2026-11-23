# Decisões pendentes

Dono: o que ainda é decisão do Viktor, com a recomendação e o documento que
passa a ser dono quando a decisão sai. Decisão tomada sai desta lista e vai
para o dono, com a data.

Estado em 2026-11-23. As decisões já tomadas estão no dono de cada uma, com a
data, e no histórico do git.

## Pendentes

| # | Decisão | Recomendação | Dono depois |
| --- | --- | --- | --- |
| P01 | Licença da biblioteca de algoritmos: LGPL-3.0, MPL-2.0 ou Apache-2.0 | Nenhuma: é do Viktor. Qualquer escolha emenda a ADR-0011 (AGPL-3.0-only, DCO sem cessão de direitos). O histórico tem um só autor humano | ADR nova |
| P04 | Valor do RNF-PERF-01: o crescimento máximo de 20.000 para 40.000 linhas | 2,5 vezes: linear com folga para a variação da máquina (65 §20.6) | [requisitos](requisitos/requisitos-nao-funcionais.md) |
| P07 | `NDEBUG` no perfil `production` | Não definir: o `assert` continua ativo, como o `_GLIBCXX_ASSERTIONS` que o plano já mantém | [perfis](engenharia/perfis-de-compilacao-e-hardening.md) |
| P09 | QML no escopo do dogfooding: o `qmlls` no Neovim | Editar QML como texto no N0, e o `qmlls` no N1: a UI é QML e o dogfooding vai precisar | [requisitos](requisitos/requisitos-funcionais.md) |
| P11 | Fase B: o inventário dos 89 `*Controller.qml` | Fazer antes de mexer na ponte; não bloqueia o core | Arquivo novo, se o Viktor pedir |
| P12 | Fase D: a ordem dos primeiros marcos e os critérios de pronto de cada um | Depois da aprovação dos [módulos](arquitetura/modulos-e-camadas.md); a proposta da ordem da base está nos [requisitos funcionais](requisitos/requisitos-funcionais.md#a-ordem-da-base) | [definição de pronto](engenharia/definicao-de-pronto.md) |
| P13 | O `clang-analyzer-*` no `core/.clang-tidy`, além das seis categorias pedidas | Manter: pega uso depois de liberar, vazamento e nulo por caminho de execução | [clang-tidy](engenharia/analise-estatica-clang-tidy.md) |
| P14 | Migrar a UI para os módulos de `core/cmake/` no lugar do `cmake/KineinStrictOptions.cmake` | Depois do primeiro marco do core, num passo próprio, com o build da UI verde nos mesmos perfis. Até lá os dois convivem | [perfis](engenharia/perfis-de-compilacao-e-hardening.md) |
| P15 | A arquitetura de módulos e camadas, e o gate de camadas no CMake | Aprovar como está em [módulos e camadas](arquitetura/modulos-e-camadas.md), ou ajustar. O gate só é escrito depois da aprovação | [módulos](arquitetura/modulos-e-camadas.md) |
| P16 | As dependências de terceiros da base: JSON e msgpack | Decidir antes do módulo `codec`. Candidatas de JSON: nlohmann/json (simples, lento), simdjson (só leitura, rápido), glaze (reflexão em compilação). Para o msgpack, reaproveitar o C++ da prova (`origin/prova/eng-60`) | [módulos](arquitetura/modulos-e-camadas.md) |
| P17 | Os gates de texto da 0.3 (idioma dos identificadores, privacidade das docs e os de QML) leem também `core/` e `docs/` | Tirar `core/` do alcance dos gates de texto sobre código, porque o core novo não usa gate de heurística de texto (L15). Por ora, os arquivos novos passam neles: foram renomeados ou entraram na lista de termos técnicos | [gates](engenharia/gates-e-medida-de-eficacia.md) |
| P18 | As regras do GitHub para o ramo `reescrita/core-cpp23`, o único em foco (a `main` fica de fora) | Já: ações só do GitHub e por SHA, token padrão só de leitura, aprovação de contribuidor externo, ramo sem apagar nem forçar push. Exigir os checks verdes recusa o push direto ("commits must first be pushed to another ref where the checks pass", doc da API de rulesets): só com pull request | [gates](engenharia/gates-e-medida-de-eficacia.md#a-ci) |
| P19 | Impor por mecanismo as regras 6 a 9 do [README](README.md#as-regras-desta-árvore): tamanho, cabeçalho e citação da 0.3 | Sim: um gate com corpus e eficácia medida, como os outros, no CI. Até lá, são convenção | [gates](engenharia/gates-e-medida-de-eficacia.md) |
| P20 | O T4: a mesma lógica escrita de outro jeito, que nenhuma ferramenta decide (teorema de Rice). O gate de T1 a T3 foi aprovado e entrou em 2026-11-23 | Discutir com o Viktor. Defesa proposta: um dono por responsabilidade, o aberto/fechado (a segunda ocorrência cria a porta) e a revisão | [gates](engenharia/gates-e-medida-de-eficacia.md) |
