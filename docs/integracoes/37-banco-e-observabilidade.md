# 37 — Banco de dados e observabilidade: o levantamento

> **Classe: ESTADO.** Licenças **verificadas na fonte em 2026-09-03**, no
> arquivo do projeto quando o detector automático falhou — o que aconteceu em
> 1 das 2.
>
> **Frente H do [`roadmaps/35`](../roadmaps/35-ambiente-cpp-embarcados-simulacao.md).**
> Candidatas, **nenhuma adotada**: a adoção passa pelo checklist de
> [`README.md`](README.md).

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
driver Postgres   qual crate Rust, com licenca e manutencao auditadas
                  (candidatas obvias: tokio-postgres, sqlx — NAO auditadas)
Grafana API       quais endpoints bastam para criar/consultar dashboard
series temporais  o que a IDE MOSTRA de uma tabela hypertable, e onde
segredo            senha de banco NAO pode ir para .kinein/ em texto puro;
                   o projeto tem `docs/seguranca/23` e nao tem cofre
```

O item de **segredo** é o mais perigoso e o mais fácil de esquecer: hoje o
`.kinein/` guarda rascunho e toolchain em texto. Credencial de banco ali seria
uma regressão de segurança, não uma feature.
