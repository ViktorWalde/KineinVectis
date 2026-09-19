# 07 — Sua primeira contribuição, passo a passo (um exemplo real, camada por camada)

O [03](03-o-ritual-de-uma-fatia.md) é o ritual; este capítulo é o **exemplo
concreto**: a fatia E3-1 da Etapa 3 (`datasource.destroy`, commit
`4a79c28`, 2026-09-19), que atravessa todas as camadas. Abra os arquivos
citados ao lado deste texto — os trechos abaixo são os do repositório.

## O pedido

> "Quero que seja possível remover/excluir o banco de dados criado pela
> parte visual." — o autor, no teste da Etapa 2.

Antes existia `datasource.remove` (só o perfil). A fatia: **remover o
perfil e, se o usuário mandar, os dados** (arquivo SQLite, container,
banco dentro de um PostgreSQL) — com o plano dito antes do clique.

## 1. O desenho (roadmap 44 §4, E3-1) — escrito antes do código

```text
E3-1  datasource.destroy { name, data? }                     contrato 0.129.0
      - sem `data`: só o perfil (= remove). Com `data`, por motor:
        SQLite → apaga o arquivo SÓ se estiver dentro do workspace (fora fica, com note)
        container kinein-<name> → `rm -f` como job High, com o comando visível
        PostgreSQL → DROP DATABASE pelo banco `postgres` do mesmo servidor (job)
        MongoDB / banco de manutenção → nunca; só o perfil, com note
      - resposta: { profiles } na hora, ou { jobId, command, note? } + evento
        event.datasource.destroyed { jobId, success, message, profiles? }
      - tela: "Remover…" abre uma caixa (só o perfil / com os dados) com o texto
        do que vai sumir; veredito depois.
      Medida: teste de despacho com podman falso; harness; foto.
```

O que fica **fora** (dito no mesmo lugar): o `DROP` real e o `rm` real só
o autor mede (Podman de verdade); a foto da caixa offscreen não seleciona
perfil.

## 2. O contrato — `crates/kinein-protocol/src/datasource_discover.rs`

```rust
/// `datasource.destroy` — the saved profile and, with `data`, what it points to.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct DataSourceDestroyParams {
    /// The saved profile.
    pub name: String,
    /// `true` also destroys the data: the `SQLite` file (inside the
    /// workspace only), the `kinein-<name>` container, or the database
    /// inside a `PostgreSQL` server (`DROP DATABASE` via the maintenance
    /// database). `MongoDB` data is never dropped from here.
    #[serde(default)]
    pub data: bool,
}
```

Mais o `DataSourceDestroyResult` e o `DataSourceDestroyedEvent`,
reexportados em `lib.rs`; `PROTOCOL_VERSION` de `0.128.0` para `0.129.0`;
a entrada no changelog do topo de `arquitetura/03-ipc-protocol.md` **e** a
seção `datasource.*` com a assinatura. Regra de ouro dos tipos: doc-comment
em inglês (é a linguagem dos tipos), `camelCase` no fio, `deny_unknown_fields`
quando não há `#[serde(flatten)]` (os dois não convivem — `40` §7.66).

## 3. O core — a lógica pura e o handler fino

`crates/kinein-core/src/datasource/destroy.rs` — **puro**: recebe o
perfil e o workspace, devolve um plano; nada de processo aqui:

```rust
pub enum DestroyPlan {
    ProfileOnly { note: Option<String> },
    SqliteFile { path: PathBuf },
    Container { name: String },
    PostgresDrop { database: String, maintenance: String },
}
pub fn plan(profile: &DataSourceProfile, root: &Path, data: bool) -> DestroyPlan { /* … */ }
```

`crates/kinein-core/src/handlers/datasource_destroy.rs` — **fino**: valida
os params, chama `plan`, e para cada variante faz o efeito: apaga o
arquivo e responde `{ profiles }`; ou sobe um **job** (`jobs.spawn(…,
JobRisk::High, …)`) que roda o motor linha a linha e emite
`event.datasource.destroyed` ao fim. O roteamento do método fica em
`handlers/datasource.rs`:

```rust
"datasource.destroy" => Some(self.datasource_destroy_response(request_id, params)),
```

O teste, em `crates/kinein-core/src/tests/datasource_discover.rs`, monta um
**cenário** com um `podman` falso no `PATH` (um shell script que ecoa um
`ps` fixo e grava o `argv` do `rm`), abre um workspace temporário, chama
`handle_request` com JSON e lê o evento pelo canal:

```rust
let podman = dir.join("bin/podman");
std::fs::write(&podman, format!("#!/bin/sh\n{PODMAN_FALSO}")).unwrap();
let mut core = crate::Core::with_detector(ToolDetector::with_search_path(dir.join("bin")));
core.enable_lsp(sender);                       // o canal por onde os eventos saem
let r = c.rpc("datasource.destroy", json!({ "name": "pg", "data": true }));
assert!(r.error.is_none());
let ev = c.evento("event.datasource.destroyed");
assert_eq!(std::fs::read_to_string(dir.join("bin/rm.argv")).unwrap().trim(), "rm -f kinein-pg");
```

É assim que se prova o core sem tocar em ferramenta real: **a ferramenta
falsa registra o que recebeu**, e o teste afirma o comando exato.

## 4. A ponte C++ — `ui/src/core_client_datasource.cpp` e `core_client.h`

```cpp
// o pedido
void CoreClient::dataSourceDestroy(const QString& name, bool data)
{
    QJsonObject params{{QStringLiteral("name"), name}};
    if (data) {
        params.insert(QStringLiteral("data"), true);
    }
    sendRequest(QStringLiteral("datasource.destroy"), params);
}
// a resposta, dentro de dispatchDataSourceResult(method, result)
if (method == QStringLiteral("datasource.destroy")) {
    const bool immediate = result.contains(QStringLiteral("profiles"));
    emit dataSourceDestroyResolved(result.value("profiles").toArray().toVariantList(), immediate,
                                   result.value("jobId").toString(), result.value("command").toString(),
                                   result.value("note").toString());
    return true;
}
```

E, em `core_client_notifications.cpp`, o evento
`event.datasource.destroyed` vira `emit dataSourceDestroyed(success,
message, profiles)`. Os dois sinais são declarados em `core_client.h`.
Cada `dispatch*Result` **tem de estar na cadeia** de quem recebe respostas
— o gate de fiação confere (foi um `return false` esquecido que silenciou
oito domínios em 2026-09-18).

## 5. Os roteadores QML — `ui/qml/ipc/DataSource{Request,Event}Router.qml`

```qml
// RequestRouter: do controller para a ponte
Connections {
    target: root.dataSourceController.discovery      // o FILHO que é dono do pedido
    function onDestroyRequested(name, data) { root.coreClient.dataSourceDestroy(name, data); }
}
// EventRouter: da ponte para o controller
function onDataSourceDestroyResolved(profiles, immediate, jobId, command, note) {
    root.dataSourceController.discovery.handleDestroyResolved(profiles, immediate, jobId, command, note);
}
function onDataSourceDestroyed(success, message, profiles) {
    root.dataSourceController.discovery.handleDestroyed(success, message, profiles);
}
```

Repare no `target`: o dono é `discovery`, um controller **filho**. Um
`Connections` no pai teria compilado, lintado e ficado mudo (`40` §7.72).

## 6. O controller e a view

`ui/qml/datasource/DataSourceDiscoveryController.qml` guarda o estado
(`destroying`, `destroyMessage`, `destroyOk`, `destroyNote`), pede por
`signal destroyRequested(string name, bool data)` e recebe por
`handleDestroyResolved`/`handleDestroyed`/`handleFailed`. **Achado do
caminho:** a função se chamava `destroy` e era engolida pelo `destroy()`
que todo objeto QML tem — virou `destroyProfile`. O harness pegou.

`DataSourceDestroyBox.qml` é a view: dois chips ("só o perfil" / "com os
dados"), o texto do que vai sumir (derivado das propriedades do
controller, nunca recalculado na view), o `KvVerdict` com o desfecho.
Registrada em `ui/CMakeLists.txt` (nas duas listas). `DataSourcePanel.qml`
troca o botão "Remover…" pela caixa.

O harness `scripts/qml-harness/tst_datasource_discovery.qml` afirma os
quatro caminhos: imediato, job, nota, falha — com bitmask.

## 7. Provar e documentar

Gates: testes (843), clippy `--all-targets`, qmllint, propriedades,
alcance, duplicação, **fiação (162 métodos / 56 eventos, todos com dono)**,
catraca, binário abre. O C++ (clang-tidy, ~1 h) em segundo plano; o
commit só depois do `exit=0` dele. Docs: `arquitetura/03` (contrato),
`manual.md` §banco, `40` §7.80, `44` §7.1, o registro privado com o log
do clang-tidy e a foto.

## 8. O commit

```text
feat(banco): os chips cabem; datasource.destroy — remover o que foi criado (E3-1, 0.129.0)

- o quê e por quê (o pedido do autor);
- o contrato e o plano por motor;
- o achado (destroy → destroyProfile);
- Provado: … (testes, harness, gates, números);
- o que NÃO fez (o DROP real; a foto da caixa).
```

Uma fatia = um commit, e a mensagem sobrevive sem o chat.

## O que este exemplo ensina que o ritual não diz

- O **plano puro** separado do **efeito** é o que torna a fatia testável
  e o handler fino.
- A **ferramenta falsa que grava o argv** é a prova mais barata e mais
  honesta do que um mock.
- **Nomes têm armadilhas** (`destroy`); o harness existe para isso.
- O que só o autor consegue medir **é dito**, não inventado.
