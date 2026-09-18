# 37 — Banco de dados e observabilidade: o levantamento

> **Classe: ESTADO.** Licenças **verificadas na fonte em 2026-09-03**, no
> arquivo do projeto quando o detector automático falhou — o que aconteceu em
> 1 das 2.
>
> **Frente H do [`roadmaps/35`](../roadmaps/35-ambiente-cpp-e-embarcados.md).**
> Escrito quando eram candidatas; **ADOTADAS em 2026-09-04** (`roadmaps/40` §4
> item 27, fechado): PostgreSQL/TimescaleDB, SQLite e MongoDB no domínio
> `datasource` (perfil sem senha em disco, teste de conexão, leitura de
> esquemas/tabelas/colunas ou coleções), o Grafana pela HTTP API no domínio
> `grafana`. **Em 2026-09-18 (0.121.0, `roadmaps/35` §7.4):** executar
> consulta e escrever (`datasource.query`) e o TLS do `postgres`
> (`tokio-postgres-rustls`) entraram. A experiência de banco "à JetBrains (DataGrip), adaptada" é
> a etapa de UX/UI/HUD que o autor separou em 2026-09-13 (`40` §5) — depois do
> backend impecável.

## 1. As duas, medidas

| ferramenta | licença | versão | data | como a IDE falaria |
| --- | --- | --- | --- | --- |
| **Grafana** | **AGPL-3.0** | v13.2.1 | 2026-09-02 | HTTP API |
| **TimescaleDB** | **Apache-2.0 + TSL** (misto) | 2.29.2 | 2026-08-18 | protocolo Postgres |

As duas ativas, push no mesmo dia da medição.

## 2. A licença do Grafana muda o significado de "nativo"

Grafana é **AGPL-3.0** — copyleft forte, com cláusula de rede. Isso não impede
a integração, mas **decide a forma dela**, e a distinção é a diferença entre
legal e ilegal:

```text
PODE   a IDE CONVERSA com uma instancia de Grafana pela HTTP API dele:
       cria dashboard, consulta painel, abre no navegador.
       A Kinein nao redistribui nem modifica o Grafana.

NAO    embutir o Grafana dentro da Kinein, ou distribui-lo no AppImage.
       Ai a AGPL alcanca a Kinein inteira.
```

A regra que sustenta isso **já existe** e não foi inventada para esta frente —
`LEITURA_TECNICA` §4 fato 5: *"ferramenta externa com licença copyleft (o GDB é
GPL-3) é **executada como processo**, nunca linkada."* A AGPL é a versão mais
exigente dessa família, e a mesma fronteira serve.

**Consequência prática para "plug and play":** a IDE pode *detectar* um Grafana
rodando, *oferecer* subir um via Docker (que é domínio nativo, já decidido) e
*gerar* dashboards — mas o Grafana continua sendo processo do usuário, não peça
da Kinein.

## 3. TimescaleDB não é uma licença só, e a parte fechada tem nome

Verificado no `LICENSE` do projeto:

> *"Outside of the 'tsl' directory, source code in a given file is licensed
> under the Apache License Version 2.0"*
>
> *"Within the 'tsl' folder, source code in a given file is licensed under the
> Timescale License"*

```text
fora de tsl/    Apache-2.0        open source
dentro de tsl/  Timescale License SOURCE-AVAILABLE, nao OSI
```

A **TSL não é open source** no sentido da OSI: ela restringe oferecer o produto
como serviço. O autor definiu a frente como *"open source, sem ser
proprietário"*, e essa ressalva é registrada aqui em vez de escondida atrás do
nome "TimescaleDB".

**Por que isso não bloqueia a integração:** a Kinein fala **protocolo Postgres**
com um servidor que o *usuário* instalou. A IDE não linka, não redistribui e não
escolhe qual edição ele roda. A licença do TimescaleDB é decisão de quem opera o
banco — a IDE é agnóstica, e continuar agnóstica é o desenho certo.

## 4. O que "nativo" significa aqui, para não virar promessa vaga

O contrato do projeto já diz que **Docker e banco são domínios NATIVOS, não
plugins** (`LEITURA_TECNICA` §6). "Nativo" tem significado preciso:

```text
E' NATIVO      dominio do core em Rust, com testes deste repositorio,
               passando pelos mesmos gates — como git, lsp e terminal
NAO E' NATIVO  embutir Grafana, ou reimplementar Postgres
```

A referência funcional de cliente de banco continua sendo o **DBeaver**, não o
IntelliJ Community — que não tem Database Tools (é Ultimate), verificado em
2026-07-17.

## 5. O que ainda NÃO foi medido

```text
driver Postgres   AUDITADO em 2026-09-04, ver §5.1 — `postgres` 0.19.14
Grafana API       quais endpoints bastam para criar/consultar dashboard
series temporais  o que a IDE MOSTRA de uma tabela hypertable, e onde
segredo            DECIDIDO em 2026-09-04, ver `../seguranca/40`
```

## RESPONDIDO em 2026-09-04: os endpoints, e o cliente

A pergunta acima foi respondida implementando (roadmaps/35 §9.6). **Três
endpoints bastam para LER**, e a IDE só lê:

| endpoint | o que dá | precisa de token |
|---|---|---|
| `GET /api/health` | `version`, `database`, `commit` | **não** |
| `GET /api/datasources` | `uid`, `name`, `type`, `typeName`, `url`, `database`, `isDefault` | sim (`datasources:read`) |
| `GET /api/search?type=dash-db` | `uid`, `title`, `url`, `folderTitle` | sim |

Fonte: documentação oficial do Grafana, consultada em 2026-09-04; **verificado
contra um servidor real 13.0.2** subido em contêiner. Duas divergências entre a
documentação e o servidor, registradas porque importam:

- a doc de `/api/datasources` lista um campo **`password`**; o 13.0.2 **não o
  envia**. A struct da IDE não tem o campo de qualquer forma, então o `serde` o
  descartaria — a garantia é estrutural, e há teste;
- o 13.0.2 envia **`typeName`** (`"PostgreSQL"`), que a doc não lista. A IDE o
  usa quando existe e cai para o `type` quando não.

**Cliente:** `ureq` 3.4 — +5 crates sem TLS, todas `MIT OR Apache-2.0`.

## O TLS entrou em 2026-09-04, e a decisão vale para todos os clientes

Decisão do autor, ao escolher que a IDE fale com um MongoDB remoto. Duas
licenças entraram no `deny.toml` com justificativa datada:

| licença | crate | o que é |
|---|---|---|
| BSD-3-Clause | `subtle` | criptografia de tempo constante; permissiva, OSI-approved, mesma família da ISC já aceita |
| CDLA-Permissive-2.0 | `webpki-roots` | **não é código**: é a lista de certificados raiz da Mozilla, empacotada como crate |

Com elas, o `ureq` ligou o `rustls` e o `mongodb` entrou com as features
padrão. O `postgres` ficou sem TLS até 0.120.0 — a licença estava
resolvida, mas o conector muda a chamada de conexão. **Em 2026-09-18
(0.121.0) entrou o `tokio-postgres-rustls` 0.14** (MIT): usa o mesmo `rustls`
0.23 e o `webpki-roots` que já estavam na árvore; +11 crates medidos
(`x509-cert`, `der`, `spki`, `tls_codec`, `const-oid`, `base64ct`, `flagset`,
`der_derive`, `tls_codec_derive`, `zeroize_derive` e ele próprio — todos
MIT/Apache), `cargo deny` verde sem nova licença. Política do perfil:
`tls: require` = `verify-full` (cadeia e host), com `caFile` para o
autoassinado; sem "cifra sem conferir".

Custo de dependência medido em 2026-09-04, para comparação futura:

```text
ureq sem TLS       +5 crates
postgres           +52 crates
mongodb 3.9        +104 crates   (o maior que o projeto aceitou)
```


O item de **segredo** era o mais perigoso e o mais fácil de esquecer, e foi o
primeiro a ser resolvido: **a IDE guarda o perfil e nunca a senha**
([`../seguranca/40-cofre-de-credencial.md`](../seguranca/40-cofre-de-credencial.md),
decisão do autor em 2026-09-04). O domínio `datasource` do core nasceu com essa
garantia testada — um campo `password` enviado pela UI é **recusado**, não
ignorado em silêncio.

### 5.1 O driver, auditado em 2026-09-04

Medido com a *toolchain* pinada do repositório e o `deny.toml` **deste**
projeto, contra o workspace de 84 crates que ele tinha na data:

```text
crate            versao     licenca              novas crates   deny.toml
postgres         0.19.14    MIT OR Apache-2.0        +52         OK
sqlx             0.9.0      MIT OR Apache-2.0       +105         REPROVA
```

**O `sqlx` reprova por um motivo concreto e verificável:** ele traz
`foldhash 0.2.0`, licenciada **Zlib**, que não está na lista do `deny.toml`.
Zlib é permissiva e OSI-approved — dá para adicioná-la —, mas acrescentar
licença à lista é ato deliberado neste projeto, não efeito colateral de escolher
uma dependência.

**O `postgres` 0.19.14** é o cliente *síncrono* dos mesmos mantenedores do
`tokio-postgres` (repositório `rust-postgres`), atualizado em 2026-06-12. Ele
embute um `tokio` 1.53.1 como detalhe de implementação — a árvore tem tokio —,
mas **o código deste repositório continua síncrono**, que é o desenho do core
(`Core::new()` puro roda sem GUI e sem runtime).

(Escrito em 2026-09-04, quando a árvore não tinha `rustls` nem `openssl`; o
`mongodb` trouxe o `rustls` no mesmo dia, e o `postgres` o usa desde 0.121.0
— ver acima.)

## 6. Descobrir e criar (2026-09-18, `0.124.0`)

O primeiro teste do autor na IDE polida disse que o painel de banco "parecia
só visual": perfil, testar, consultar — e nada para achar um banco nem para
criar um. O que entrou (`roadmaps/40` §7.66; contrato em `arquitetura/03`):

- **Descobrir** (`datasource.discover`, ao abrir o painel e no ↻): o socket
  do PostgreSQL de distro, as portas 5432/27017 no loopback, os containers
  com imagem `postgres`/`timescale`/`mongo` (pelo mesmo motor do painel de
  Containers; parados aparecem como parados), os `.sqlite/.db` do projeto
  com o cabeçalho `SQLite format 3`. Cada achado traz o perfil pronto;
  clicar põe no formulário — salvar continua sendo do autor.
- **Novo banco** (`datasource.create`): um arquivo SQLite em
  `data/<nome>.sqlite`; um PostgreSQL ou MongoDB **em container no
  loopback** (o comando `podman|docker run …` exato aparece antes do clique
  e na saída do job; imagens pinadas `postgres:16`/`mongo:7`; autenticação
  `trust` só em `127.0.0.1`, porque a IDE não guarda senha); um banco
  dentro do PostgreSQL do perfil em edição (`CREATE DATABASE`, pela escrita
  confirmada — o perfil clonado com o banco novo é salvo).
- **O que se prova sem servidor:** SQLite real e um `podman` falso no gate.
  O `run` real baixa ~150 MB e é um clique do autor.
