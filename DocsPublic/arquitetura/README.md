# arquitetura/ — o contrato de engenharia e a forma do código

Comece por [`ARCHITECTURE.md`](ARCHITECTURE.md): é contrato, e o gate o cobra.

```text
ARCHITECTURE.md                  as regras: camadas, corte por responsabilidade,
                                 catraca, gates nascidos de falha silenciosa
01-mapa-de-modulos.md                quem fala com quem, por quê e como (GERADO, com gate)
02-estrutura-do-repositorio.md       o que mora em cada pasta do repositório
03-protocolo-ipc.md               o contrato JSON-RPC entre UI e core: todos os
                                 métodos, eventos e tipos, por domínio, com a
                                 versão em que cada um entrou
04-boot-e-comunicacao.md         como UI e core sobem, falam e se recuperam
06-modo-estrito.md                o rigor de compilação (Rust/C++/QML)
15-divida-de-engenharia-e-refatoracao.md   dívida e refatoração (registro)
16-checklist-de-riscos-ocultos.md     riscos escondidos, em lista
19-compromissos-de-arquitetura.md     trade-offs assumidos
27-modulos-por-dominio.md        módulo por domínio: a catraca do core
32-editor-por-responsabilidade.md  o editor cortado em quatro donos
33-busca-no-projeto.md           os três buscadores
35-crescer-sem-god-object.md     a análise de arquitetura: o que está saudável e
                                 o que recusar
36-casca-da-ide.md               a janela principal: moldura, ilha, trilhos,
                                 barras arrastáveis, layout gravado e foco
```

[37 — Banco de dados](37-banco-de-dados.md): consulta, confirmação, MongoDB,
padrões por motor, limites e provas reproduzíveis, com diagramas.

[38 — Provedores de banco e de linguagem](38-provedores-de-banco-e-linguagem.md):
PLANO de expansão modular, seleção de LSP, atualização de ferramentas,
isolamento por conexão e migração sobre os donos atuais.
