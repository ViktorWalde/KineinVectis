# 08 — Convenções: código, testes, commits e documentação

O que o gate cobra está no [04](04-os-gates-que-dizem-nao.md). Este
capítulo é o que o gate **não** cobra e o revisor cobra.

## Idioma

- **Documentação e comentários**: português do Brasil. Nos `.md`, com
  acentos. Nos **comentários de código** (Rust, QML, C++, shell), o
  repositório usa ASCII na maior parte (`nao`, `e'`, `ja'`) por hábito
  histórico; siga o arquivo em que está — não "corrija" um arquivo
  inteiro para o outro estilo numa fatia que não é sobre isso.
- **Identificadores**: inglês nos tipos do protocolo e nas APIs
  (`DataSourceDestroyParams`, `handleDestroyed`); português é aceito em
  variáveis locais e funções internas quando o domínio é em português
  (`fn cenario`, `let pedido`, `PODMAN_FALSO`). O arquivo em que você está
  manda.
- **Doc-comments do protocolo** (`kinein-protocol`): inglês, porque são a
  documentação do fio para qualquer cliente.
- **Textos da tela**: português, sempre em `qsTr("…")`, com acentos.

## Comentários: o "por quê", com data

O padrão do repositório é o comentário que diz **por que o código existe
e o que ele recusa fazer**, com a data e a referência quando nasceu de
incidente:

```qml
// `destroyProfile`, nao `destroy`: todo objeto QML ja' tem um `destroy()`.
```
```rust
//! Fino como o `datasource.rs`: valida, delega a `crate::datasource::
//! {discover, create}` e formata. O `discover` roda ADIADO (`defer_work`):
//! ele bate em portas com 200 ms de tolerancia — nada disso pode segurar o laco.
```

Arquivos novos abrem com um bloco `//` (QML/C++) ou `//!` (Rust) que diz
o que o arquivo é, por que existe separado e o que **não** faz. Não
descreva o óbvio (`// incrementa i`); descreva a decisão.

## Rust

- Edição e versão pela `rust-toolchain.toml`; `cargo fmt` sempre; clippy
  **pedante** com `-D warnings` em `--all-targets`.
- Proibidos fora de teste: `unsafe`, `unwrap`, `expect`, `panic!`.
  Erros são tipos e viram `JsonRpcError` com código (`InvalidParams`,
  `ToolNotFound`, `InvalidRequest`…) no handler.
- **Serviço puro + efeito separado**; handler fino; nada demorado dentro
  do laço (`defer_work`/`defer_then`/job — `arquitetura/04` §6).
- Processos externos passam por `crate::process` (streaming de linhas,
  cancelamento, `stderr_tail`), nunca `Command::output()` solto no
  handler.
- Nomes de arquivo por domínio; ≤ 500 linhas fora de `#[cfg(test)]`.
  Quando um `impl Core` cresce, o split é por **método**
  (`handlers/datasource_destroy.rs` ao lado de `handlers/datasource.rs`).
- Testes de despacho em `src/tests/<dominio>.rs`, com cenário temporário
  em `std::env::temp_dir()/kinein-core-tests/<pid>-<nome>`, ferramenta
  falsa no `PATH` via `ToolDetector::with_search_path`, eventos pelo canal
  de `enable_lsp(sender)`; helpers compartilhados em `tests/mod.rs`.

### Modelo de teste de despacho

```rust
#[test]
fn o_metodo_faz_x_e_emite_y() {
    let mut c = cenario("x-e-y", true);                 // workspace temporário + ferramenta falsa
    let r = c.rpc("dominio.metodo", json!({ "campo": "valor" }));
    assert!(r.error.is_none(), "{:?}", r.error);
    assert_eq!(r.result.unwrap()["campo"], json!("esperado"));
    let ev = c.evento("event.dominio.aconteceu");       // espera até 20 s pelo evento
    assert_eq!(ev["success"], json!(true));
    // a ferramenta falsa gravou o que recebeu: afirme o comando EXATO
    assert_eq!(std::fs::read_to_string(c.dir.join("bin/x.argv")).unwrap().trim(), "x --flag alvo");
}
```

Recusas também são testadas: sem workspace (`InvalidRequest`), params
inválidos (`InvalidParams`), ferramenta ausente (`ToolNotFound`).

## QML

- `pragma ComponentBehavior: Bound` em todo arquivo com delegates;
  `required property` nos delegates; `id: root` na raiz.
- **Controller** (`Item { visible: false }`): estado + `signal
  xRequested(...)` + `function handleX(...)`; sem `Theme`, sem geometria,
  para o harness carregar sem tela. Derivações em `readonly property`.
- **View**: lê do controller, emite gestos; usa as peças `Kv*`; toda
  âncora com margem tem a âncora; um binding nunca fica mais indentado que
  a propriedade (o gate acusa); não misture `Layout` com `anchors` no
  mesmo item.
- Derivação que dois arquivos precisam **tem um dono** (o gate de
  duplicação congela a baseline).
- `ListModel` não guarda arrays/objetos aninhados: serialize (a HUD do Git
  usa `refsText` com o separador US, `String.fromCharCode(31)`).
- Nomes: `<Dominio><Papel>.qml` (`GitWindow`, `GitViewerPane`,
  `SymbolsController`, `SymbolResultsList`). Registro nas **duas** listas
  do `ui/CMakeLists.txt`.

### Modelo de harness

```qml
import QtQuick
import "../../ui/qml/<dominio>"

Item {
    id: root
    property var pedidos: []

    XController {
        id: ctl
        onXRequested: function(arg) { root.pedidos.push(arg); }
    }

    Component.onCompleted: {
        let failures = 0;
        // Uma regra por asserção, dita em uma linha.
        ctl.doSomething("a");
        if (root.pedidos.length !== 1 || root.pedidos[0] !== "a") failures += 1;
        // Resposta sem pedido é ignorada (quem pediu recebe; ninguém mais).
        ctl.handleX([1, 2, 3]);
        if (ctl.items.length !== 3) failures += 2;
        // O exit code tem 8 bits: o bitmask vai para a saída, o exit só diz passou/falhou.
        if (failures !== 0) console.error("FALHAS bitmask=" + failures);
        Qt.exit(failures === 0 ? 0 : 1);
    }
}
```

Potências de dois distintas por asserção; a mensagem `FALHAS bitmask=N`
diz qual falhou.

## C++ (a ponte)

- Só `ui/src/`; clang-format do repositório; clang-tidy limpo; ≤ 500
  linhas por arquivo (`core_client_<dominio>.cpp`).
- Um `Q_INVOKABLE` por método do protocolo; um `dispatch<Dominio>Result`
  por domínio, **encadeado** no despacho geral; um sinal Qt por resposta e
  por evento, com parâmetros tipados (`QVariantList` para arrays, nunca o
  `QJsonObject` cru para o QML).
- A ponte **não interpreta**: extrai campos e emite. Regra de negócio é do
  core; formatação é do controller.

## Shell e Python (gates e scripts)

- `set -Eeuo pipefail`; shellcheck limpo; mensagens em português; saída
  `✓`/`✗` no fim; o script diz **o que mede e por quê** no cabeçalho.
- Gates em Python (`scripts/verificar_<x>.py`) com um `.sh` fino que os
  chama; sem dependências fora da stdlib.
- Um gate novo é provado por **mutação**: quebre de propósito, veja-o
  reprovar, desfaça, registre no `40`.

## Commits

Formato (o `git log` é a referência):

```text
<tipo>(<área>): <o quê, em português, na voz do que mudou> (<fatia>, <versão se contrato>)

<parágrafos: o porquê (o pedido/incidente); o que mudou por camada;
achados no caminho; "Provado:" com os números; o que NÃO fez e por quê.>
```

Tipos usados: `feat`, `fix`, `perf`, `docs`, `test`, `refactor`. Área:
`ui`, `core`, `git`, `banco`, `embarcados`, `containers`, `lsp`,
`etapaN`. Um commit por fatia; nunca "wip"; nunca commit que deixa o gate
rápido vermelho. Sem push/tag/release sem o mantenedor.

## Documentação — onde cada coisa vai

| O que mudou | Onde escrever |
| --- | --- |
| Contrato IPC | `arquitetura/03` (changelog no topo **e** a seção do domínio) |
| O que o usuário vê/faz | `manual.md` (seção da área; atalhos na tabela) |
| O estado do projeto | `roadmaps/40` (números do cabeçalho; §7.N o diário da fatia) |
| A etapa | `roadmaps/<NN>` (desenho fino antes; "feito" depois) |
| Decisão de arquitetura durável | `arquitetura/*.md` ou um ADR em `decisoes-adr/` |
| Como usar uma ferramenta externa | `integracoes/` |
| Instalar/distribuir | `tutorial.md`; a versão em `CHANGELOG.md` |
| A sessão (privado) | `DocsPrivate/Codex/<data>-<tema>.md` + `evidencias-<data>-<tema>/` |

Todo número é **datado** ("843 testes em 2026-09-19"); todo "não feito" é
escrito com o porquê; o gate `verificar-docs.sh` confere números sem data
contra o disco e o `verificar-links-docs.sh` os links.
