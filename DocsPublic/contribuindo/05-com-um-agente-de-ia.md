# 05 — Trabalhar com um agente de IA neste repositório

O projeto é desenvolvido **com** agentes (Claude Code foi o principal em
2026; Codex como auxiliar) e **sem** IA no produto. O que segue é o que
funcionou e o que deu errado — para quem vai colaborar assim, e para quem
vai revisar o que um agente entregou.

## O que o agente precisa receber antes de tocar em código

1. **O prompt de entrada** com o estado real: versão do protocolo, os
   números do `40` (cabeçalho), o roadmap da etapa e a fatia; as regras
   inegociáveis; o que ele pode e não pode fazer na máquina. O modelo
   deste prompt é `DocsPrivate/Codex/PROMPT-proxima-sessao.md`; para quem
   não tem acesso ao privado, o conteúdo público equivalente é o
   [03](03-o-ritual-de-uma-fatia.md) + o roadmap da etapa + o `40`.
2. **A ordem de leitura**: `40` §cabeçalho e §7 recentes → roadmap da
   etapa → `arquitetura/ARCHITECTURE.md` → os arquivos da área. Um agente
   que começa pelo código repete o que já foi descartado.
3. **As regras que ele não pode "otimizar"** (todas nasceram de incidente):

```text
- contrato primeiro; PROTOCOL_VERSION sobe; arquitetura/03 antes do core
- catracas: Rust 500 / view 300 / controller-host 400 / ui/src 500
- medir antes de afirmar; número com data; foto antes e depois
- um commit por fatia; a mensagem diz o que provou e o que NÃO fez
- NUNCA `git checkout <arquivo>` para desfazer
- a IDE não roda sudo nem instala em silêncio — e o agente também não
  (ele pode instalar no PRÓPRIO ambiente só se o mantenedor autorizar)
- nunca gravar firmware/arquivos na placa do mantenedor sem pedido explícito
- nada de push, release, AppImage, tag sem o mantenedor pedir
- não compilar a UI enquanto o verificar-cpp.sh roda
- todo contexto salvo na documentação ANTES de prosseguir com código
  (uma sessão pode acabar no meio; o próximo lê o roadmap, não a memória)
```

## O ciclo que funciona

- **Uma fatia por vez**, o desenho fino escrito no roadmap antes do
  código (o `44` §4.1 é um exemplo). O agente que "resolve tudo de uma
  vez" produz uma árvore que ninguém consegue revisar nem commitar por
  partes.
- **Foto antes/depois** para qualquer tela; **harness** para qualquer
  controller; **teste de despacho** para qualquer método. O agente não
  clica — diga isso a ele e cobre que ele diga o que não mediu.
- **O gate é o revisor que não cansa.** Peça o `--rapido` verde antes de
  cada commit e o completo (com o C++ em segundo plano) antes de fechar a
  fatia que mexeu em `ui/src`.
- **Registro datado ao fim** (`DocsPrivate/Codex/<data>-<tema>.md`) com a
  tabela fatia × commit, as provas, as notas de método (o que deu errado
  e como se pegou), o que ficou. É o que faz a próxima sessão começar do
  lugar certo.
- **"Save point" quando a tarefa cresce**: pare, documente, commite o que
  está provado, e só então continue. O mantenedor pediu isso mais de uma
  vez e sempre valeu a pena.

## O que deu errado (e virou regra)

| O que aconteceu | O que se aprendeu |
| --- | --- |
| Um agente usou `git checkout -- arquivo` para limpar "lixo" de um harness; o arquivo tinha trabalho | proibido; `git stash`/`git diff` e edição |
| Refactor de anchors deixou dez listas em branco sem erro nenhum | gate de propriedades (âncora sem margem, binding torto); foto depois de todo refactor de layout |
| Um `dispatch` devolvendo `false` silenciou oito domínios por seis dias | gate de fiação de ponta a ponta; toda ponta nova tem consumidor |
| Contagens "de cabeça" de métodos/eventos erradas na doc | número com data; `verificar-docs.sh` |
| Uma função QML chamada `destroy` era engolida pelo `destroy()` de todo objeto | nomes: `destroyProfile`; o harness pegou |
| O `indexController` não descia até a tela: o campo aceitava texto e nada acontecia | binding para `null` é falha silenciosa que gate nenhum vê — foto com o gesto |
| Clippy da lib verde, `--all-targets` vermelho | o gate roda `--all-targets`; rode o gate, não o atalho |

## Como revisar o que um agente entregou

1. A mensagem do commit diz o que provou? Reproduza **uma** prova
   (o harness, a foto, o teste).
2. O `40` §7.N e o roadmap §feito foram escritos? Dizem o que **não**
   foi feito?
3. O contrato (se houve) está no `03` com versão nova?
4. O gate `--rapido` passa no seu checkout?
5. Há alguma linha que "contorna" um gate (exceção nova, `#[allow]`,
   baseline atualizada)? Ela tem motivo escrito?

## Codex, ou um segundo agente

Só um agente commita. Um segundo pode ser **informado** do que está sendo
feito (`DocsPrivate/Codex/TAREFAS-PARA-O-CODEX.md`) e pode revisar, medir,
propor — mas duas mãos na mesma árvore sem coordenação produz exatamente o
tipo de merge que este repositório evita. Se dois agentes precisam
trabalhar, é em fatias diferentes, em worktrees diferentes, e o
mantenedor integra.

## Sem IA

Tudo acima vale igual para uma pessoa: o ritual, os gates, o registro. A
diferença é só quem digita. Um contribuidor humano tem uma vantagem que o
agente não tem — **ele clica** — e é por isso que a medida que o agente
não alcança (o gesto real, a placa, o Grafana real) fica escrita como
"não medido: o autor testa". Se você é essa pessoa, o teste real é a sua
contribuição mais valiosa, e o lugar de registrá-la é o `40` §7.N.
