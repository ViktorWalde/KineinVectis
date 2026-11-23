# Decisões pendentes

Dono: o que ainda é decisão do Viktor, com a recomendação e o documento que
passa a ser dono quando a decisão sai. Decisão tomada sai desta lista e vai
para o dono, com a data.

Estado em 2026-11-22.

## Pendentes

| # | Decisão | Recomendação | Dono depois |
| --- | --- | --- | --- |
| P01 | Licença da biblioteca de algoritmos: LGPL-3.0, MPL-2.0 ou Apache-2.0 | Nenhuma: é do Viktor. Qualquer escolha emenda a ADR-0011 (AGPL-3.0-only, DCO sem cessão de direitos). O histórico tem um só autor humano | ADR nova |
| P04 | Valor do RNF-PERF-01: o crescimento máximo de 20.000 para 40.000 linhas | 2,5 vezes: linear com folga para a variação da máquina (65 §20.6) | [requisitos](requisitos/requisitos-nao-funcionais.md) |
| P07 | `NDEBUG` no perfil `production` | Não definir: o `assert` continua ativo, como o `_GLIBCXX_ASSERTIONS` que o plano já mantém | [perfis](engenharia/perfis-de-compilacao-e-hardening.md) |
| P09 | QML no escopo do dogfooding: o `qmlls` no Neovim | Editar QML como texto no N0, e o `qmlls` no N1: a UI é QML e o dogfooding vai precisar | [requisitos](requisitos/requisitos-funcionais.md) |
| P11 | Fase B: o inventário dos 89 `*Controller.qml` | Fazer antes de mexer na ponte; não bloqueia o core | Arquivo novo, se o Viktor pedir |
| P12 | Fase D: a ordem dos primeiros marcos e os critérios de pronto de cada um | Depois da aprovação dos [módulos](arquitetura/modulos-e-camadas.md), que definem a ordem da base | [definição de pronto](engenharia/definicao-de-pronto.md) |
| P13 | O `clang-analyzer-*` no `core/.clang-tidy`, além das seis categorias pedidas | Manter: pega uso depois de liberar, vazamento e nulo por caminho de execução | [clang-tidy](engenharia/analise-estatica-clang-tidy.md) |
| P14 | Migrar a UI para os módulos de `core/cmake/` no lugar do `cmake/KineinStrictOptions.cmake` | Depois do primeiro marco do core, num passo próprio, com o build da UI verde nos mesmos perfis. Até lá os dois convivem | [perfis](engenharia/perfis-de-compilacao-e-hardening.md) |
| P15 | A arquitetura de módulos e camadas, e o gate de camadas no CMake | Aprovar como está em [módulos e camadas](arquitetura/modulos-e-camadas.md), ou ajustar. O gate só é escrito depois da aprovação | [módulos](arquitetura/modulos-e-camadas.md) |
| P16 | As dependências de terceiros da base: JSON e msgpack | Decidir antes do módulo `codec`. Candidatas de JSON: nlohmann/json (simples, lento), simdjson (só leitura, rápido), glaze (reflexão em compilação). Para o msgpack, reaproveitar o C++ da prova (`origin/prova/eng-60`) | [módulos](arquitetura/modulos-e-camadas.md) |
| P17 | Os gates de texto da 0.3 (idioma dos identificadores, privacidade das docs e os de QML) leem também `core/` e `docs/` | Tirar `core/` do alcance dos gates de texto sobre código, porque o core novo não usa gate de heurística de texto (L15). Por ora, os arquivos novos passam neles: foram renomeados ou entraram na lista de termos técnicos | [gates](engenharia/gates-e-medida-de-eficacia.md) |
| P18 | Exigir a CI verde para integrar na `main` (regra de proteção do ramo no GitHub) | Sim, com os jobs do `core.yml`, depois da primeira rodada verde | [gates](engenharia/gates-e-medida-de-eficacia.md#a-ci) |

## Tomadas em 2026-11-22

| # | Decisão | Onde está agora |
| --- | --- | --- |
| P02 | O contrato tem fonte única gerada a partir do `kinein-protocol`; os dois cores são conferidos contra ela | [ADR-0013](adr/ADR-0013-fonte-unica-do-contrato.md), rascunho para aprovação |
| P03 | RNF-DUR-01: perda máxima de 2 s de digitação numa queda | [requisitos](requisitos/requisitos-nao-funcionais.md) |
| P05 | Versões mínimas: GCC 14 e Clang 20 | [perfis](engenharia/perfis-de-compilacao-e-hardening.md) |
| P06 | Framework de teste: GoogleTest 1.17.0 | [testes](engenharia/estrategia-de-testes.md#framework-de-teste) |
| P10 | O INF-26 corrigido já no QML atual, com prazo máximo de 2 s no rascunho | `ui/qml/editor/EditorPersistenceController.qml` e `ui/tests/tst_editor_draft_deadline.cpp`. O INF-23 segue: abrir só projeto confiável até o lançador em C++ existir |
