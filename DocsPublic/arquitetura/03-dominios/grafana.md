# `grafana.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `grafana.*` — a observabilidade pela HTTP API, e só

Quatro métodos, um evento.

```text
grafana.get    {}          -> { profile: GrafanaProfile | null }
grafana.save   { profile }
grafana.forget {}
grafana.probe  { token? }  -> GrafanaProbeAccepted   (job)
```

```text
event.grafana.probed  { jobId, reachable, authenticated, authRefused,
                        version, database, message,
                        dataSources[], dashboards[], matches[] }
```

**Os quatro exigem workspace aberto.** Desde 2026-10-16 (16.4c do
`roadmaps/59` §7.1.4, sem mudança de forma), o `grafana.save` recusa URL com
credencial (`https://usuário:senha@…`): o perfil é gravado no `.kinein/` do
projeto, e o token vai pela variável de ambiente. A aba Web só manda ao
navegador do sistema `http`/`https`; outro esquema (`file://`, `smb://`…)
fica.

**`authenticated: false` sozinho não distingue dois estados opostos:** ninguém
ofereceu token, e o servidor recusou o que foi oferecido. O primeiro é um
convite; o segundo pede outra credencial. Por isso `authRefused` existe desde a
`0.136.0` — e a UI deriva o que mostra dele, nunca do texto de `message`.

**O Grafana nunca é embutido** — licença AGPL, decisão registrada em
`../roadmaps/40` §5. A integração é HTTP e o cliente é o `ureq`. Desde a `0.154.0`, os dashboards
**também abrem na aba Web opcional da IDE**, desligada por padrão
(40.7 §7.218–§7.219). Desligada, a IDE não carrega o motor web; o navegador
do sistema continua disponível. O que justifica o domínio existir é o
`GrafanaMatch`: o cruzamento entre o datasource do Grafana e o perfil de banco
do projeto, que é a pergunta que nenhuma das duas ferramentas responde sozinha.

O token segue a mesma regra da senha: `GrafanaTokenSource` diz de onde ele vem,
e o valor viaja por chamada.
