# 01 — O que o projeto é, o que ele recusa ser, e por que as regras existem

## A ideia

Uma IDE que **lê o projeto e a máquina e diz o que leu**, e que faz o que o
usuário manda **com o que o usuário escolheu**. Ela orquestra ferramentas
maduras e abertas; não reimplementa compilador, language server, build
system, depurador nem emulador de terminal. Quando existe ferramenta
aberta, madura e gratuita, a resposta certa é integrá-la — e dizer na tela
de onde cada informação veio.

Três consequências que explicam a maior parte do código:

1. **A UI é burra de propósito.** Ela não chama ferramenta externa, não
   toca o filesystem do workspace, não faz parsing de saída de programa,
   não contém regra de negócio. Recebe estado, emite pedidos. Se você se
   pegar escrevendo `if (saida.indexOf("error") …)` em QML, pare: isso é
   do core.
2. **O core decide, mas não impõe.** Ele resolve toolchain, preset,
   interpretador, motor de container, e devolve **a escolha e a prova**
   (de onde veio). Mudanças no projeto do usuário passam por prévia e
   consentimento (as "ações de configuração"). Nada roda como root; nada
   é instalado sem o usuário mandar.
3. **Tudo entre as duas metades é contrato.** Cada método e evento do
   JSON-RPC é um tipo Rust em `kinein-protocol`, documentado em
   `arquitetura/03-ipc-protocol.md`, com `PROTOCOL_VERSION` que sobe a
   cada mudança. A ponte C++ (`ui/src/core_client*.cpp`) é a única que
   fala com o core; os roteadores QML (`ui/qml/ipc/*Router.qml`) levam a
   resposta ao controller dono.

## O que ele recusa ser

- **Não é um editor com plugins.** A integração de uma ferramenta é código
  do core com contrato, teste e documentação — não um pacote de terceiros
  carregado em runtime. (`roadmaps/adaptacao-de-plugins-abertos.md`
  discute o que se aproveita das ferramentas abertas: o *conhecimento*,
  não o *runtime*.)
- **Não é uma IDE com IA dentro.** Agentes rodam no terminal integrado,
  como qualquer programa. O projeto é desenvolvido *com* agentes (ver
  [05](05-com-um-agente-de-ia.md)), mas o produto não os embute.
- **Não é rígida com o usuário.** O rigor (`-Werror`, clippy pedante,
  sanitizers, clang-tidy) é como o **próprio projeto** se compila, nos
  presets `*-strict`/`*-hardened`. Os projetos do usuário compilam com os
  presets e flags **deles**.
- **Não mente.** Um número sem data é uma afirmação sobre agora, e o gate
  de docs confere com o disco. Uma tela que "parece" funcionar sem
  funcionar é a pior falha possível — várias regras (fiação IPC, âncoras,
  bindings tortos, atalhos) nasceram de casos assim, documentados no `40`.

## Por que as regras existem (cada uma tem uma data)

| Regra | Nasceu de | Onde está |
| --- | --- | --- |
| Contrato primeiro (protocolo → core → ponte → router → controller → harness) | domínios inteiros roteados no core que nenhuma tela pedia (`40` §8.2) | `arquitetura/ARCHITECTURE.md`, `arquitetura/03` |
| Catraca de tamanho (Rust 500 sem testes; view QML 300; controller/host 400; ui/src 500) | god objects que ninguém conseguia mudar sem quebrar (`arquitetura/35`) | `scripts/verificar-arquitetura.sh` |
| Fiação IPC de ponta a ponta | oito domínios mudos por seis dias porque um `dispatch*` devolvia `false` (`40` §7.64) | `scripts/verificar-fiacao-ipc.sh` |
| Âncora sem margem / binding torto | dez listas em branco depois de um refactor, sem nenhum erro (`40` §7.66–7.70) | `scripts/verificar_qml_propriedades.py` |
| O binário abre, sem aviso QML | um `Connections` no alvo errado que só aparecia no stderr (`40` §7.72) | `scripts/verificar-binario-abre.sh` |
| Atalhos: a paleta anuncia o que a IDE obedece | `Ctrl+Alt+L` formatava em vez de abrir a biblioteca (`40` 2026-09-04) | `scripts/verificar-atalhos.sh` |
| Medir antes de afirmar; número datado | contagens de métodos erradas por grep de literal (`40` §4.2) | `scripts/verificar-docs.sh` |
| Nunca `git checkout <arquivo>` para desfazer | perdeu-se trabalho não commitado duas vezes (`40` §7.74) | regra de sessão |
| A placa do autor nunca é gravada sem pedido | é a placa de trabalho dele, com o `main.py` dele | `DocsPrivate` e a memória do agente |

Se uma regra atrapalha, o caminho é **medir, propor e registrar** — não
contornar. O padrão histórico do repositório é que o problema era regra
não cumprida, não regra ausente (`../contribuindo.md`, o aviso de
abertura).

## Vocabulário que aparece em toda parte

- **Fatia**: a menor mudança que se pode desenhar, medir, provar e
  commitar sozinha. Uma etapa é uma sequência de fatias (`E3-1`…`E3-7`).
- **Gate**: um script `scripts/verificar-*.sh` que diz *não*. O
  `verificar.sh` roda todos e para no primeiro.
- **Catraca**: o gate de tamanho — um arquivo em débito pode existir, mas
  não pode crescer, e nenhum novo pode entrar em débito.
- **Harness**: um `scripts/qml-harness/tst_*.qml` que carrega um
  controller QML real e afirma o comportamento dele sem tela.
- **Registro**: o diário datado de uma sessão em `DocsPrivate/Codex/`, com
  a pasta `evidencias-<data>-<tema>/` (logs, fotos). O público fica no
  `40` §7.N.
- **Foto**: um screenshot headless (`QT_QPA_PLATFORM=offscreen
  KINEIN_SCREENSHOT=<png>`) — a prova visual que o agente consegue tirar;
  o clique real é do autor.
- **Moldura comum / HUD**: `KvPanelFrame`, `KvPanelHeader`, `KvVerdict`,
  `KvDataGrid` — as peças de UI que todos os painéis de ambiente usam.
