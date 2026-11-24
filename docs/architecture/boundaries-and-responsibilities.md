# Fronteiras e responsabilidades

Dono: os processos, os componentes, o que cada um decide e a direção das
dependências. O desenho interno de cada componente fica com o dono dele
(execução, processos, persistência, protocolo, motor de edição).

## Processos

```text
UI (Qt Quick + QML)  --JSON-RPC por linha, stdio-->  core (C++23)  --lançador-->  ferramentas
   |                                                   |                          (cmake, compiladores,
   |                                                   |                           git, gdb, sondas)
   |                                                   +--lançador + Landlock-->  nvim --embed
   |                                                   +--relé LSP-->  clangd (criado pelo lançador)
   +--socket privado: grade, teclado, mouse-------------------------------------> nvim
```

- **A UI e o core** são processos separados, como na 0.3 (A01 das
  [lições](../lessons-from-0.3.md)). A UI escolhe o binário do core por
  `KINEIN_CORE_BIN` (`ui/src/core_client_process.cpp:240`): é assim que o core
  em C++ entra no lugar do core em Rust, sem mudar a UI.
- **O `nvim`** nasce pelo lançador do core e morre quando o core fecha o canal
  dele (65 §6.1). A tecla vai da UI direto ao `nvim`; o core fica fora do
  caminho da digitação.
- **Toda ferramenta externa** nasce pelo lançador
  ([processos e confiança](processes-and-trust.md)).

## O que cada componente decide

Regra herdada do 64 §5.2 (64:384-393), adaptada ao core em C++.

| Componente | Decide | Nunca decide |
| --- | --- | --- |
| QML | apresentação e intenção | fato, regra, caminho efetivo, confiança |
| Ponte da UI (C++) | transporte, correlação, prazo do lado da UI, vida da resposta | regra de domínio |
| `core/qt` (adaptador) | laço de eventos, transporte do fio, tradução entre o fio e a biblioteca | regra de domínio |
| `core/lib`, domínio | regra e estado do domínio, algoritmos | como um processo nasce, onde um arquivo próprio mora |
| `core/lib`, lançador | origem, confiança, ambiente, prazo, teto de saída, grupo de processos | regra de domínio |
| `core/lib`, persistência | caminho, classe e esquema de cada arquivo próprio, durabilidade | conteúdo de domínio |
| Neovim | texto, `changedtick`, cursor, protocolo LSP do documento | processo, confiança, configuração do servidor |

**O teste de cada função** de controller QML e da ponte: *se um cliente de
linha de comando precisaria da mesma lógica, ela pertence ao core.* O
inventário dos 89 `*Controller.qml` por esse teste é a Fase B, pendente
([decisões](../pending-decisions.md)).

## Direção das dependências

```text
core/lib   ->  biblioteca padrão C++23 (e dependência aprovada, uma a uma)
core/qt    ->  core/lib, Qt Core
executável ->  core/qt
testes     ->  core/lib (sem laço de eventos), core/qt só nos testes do adaptador
```

| Regra | Quem impõe |
| --- | --- |
| `core/lib` não inclui nem liga Qt | O preset `dev` nem procura o Qt: um `#include <QObject>` em `core/lib` não compila (`core/CMakeLists.txt`) |
| O moc nunca vê o código de `core/lib` | O AUTOMOC só é ligado nos alvos de `core/qt`; nenhum `CMAKE_AUTOMOC` global |
| `core/lib` é testável sem laço de eventos | Os testes de `core/lib` não ligam `core/qt` |
| Nada de `core/` depende de `crates/` nem de `ui/` | O projeto CMake de `core/` não referencia essas pastas |

Os módulos dentro de `core/lib`, as camadas e o gate que recusa ciclo estão
em [modules-and-layers.md](modules-and-layers.md). O grafo do CMake não recusa
ciclo sozinho: ele aceita ciclo entre bibliotecas `STATIC` (medido em
2026-11-22).

## Estrangulamento

O core em Rust fica em `crates/` como produto em uso e como oráculo do teste
diferencial até o core em C++ passar nos critérios de troca da ADR-0012. Nada
em `crates/` é apagado nem movido antes disso.
