# arquitetura/ — o contrato de engenharia e a forma do código

Comece por [`ARCHITECTURE.md`](ARCHITECTURE.md): é contrato, e o gate o cobra.

```text
ARCHITECTURE.md                  as regras: camadas, corte por responsabilidade,
                                 catraca, gates nascidos de falha silenciosa
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
```
