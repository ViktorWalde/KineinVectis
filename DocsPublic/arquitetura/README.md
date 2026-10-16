# arquitetura/ — o contrato de engenharia e a forma do código

Comece por [`ARCHITECTURE.md`](ARCHITECTURE.md): é contrato, e o gate o cobra.

```text
ARCHITECTURE.md                  as regras: camadas, corte por responsabilidade,
                                 catraca, gates nascidos de falha silenciosa
01-mapa-de-modulos.md                quem fala com quem, por quê e como (GERADO, com gate)
02-estrutura-do-repositorio.md       o que mora em cada pasta do repositório
03-protocolo-ipc.md               o contrato JSON-RPC entre UI e core: o formato
                                 base, as listas canônicas de métodos e eventos,
                                 as regras e a tabela dos domínios
03-dominios/                      um arquivo por domínio do protocolo (desde
                                 2026-10-16): métodos, tipos e a versão em que
                                 cada um entrou; leia só o da mudança
03.1-notas-por-versao-do-protocolo.md  o que cada versão do protocolo acrescentou,
                                 da mais nova para a mais antiga (a nota nova
                                 entra no topo)
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

[39 — Drivers externos e compatibilidade de versões](39-drivers-externos-e-compatibilidade.md):
PLANO revisto antes do código: adaptadores em processos escolhíveis, API
negociada, perfis preservados, falhas de escrita, transação e migração gradual.

[40 — Contrato operacional de drivers](40-contrato-operacional-de-drivers.md):
D1a.3, mensagens de operações, chunks e terminal; contexto e budgets
validados por serviço puro, antes do runtime de processo externo.
