# Caminho para o MVP: as etapas, em ordem linear

> **Classe: PLANO** (`docs/README.md`). Descreve o alvo e pode divergir da
> implementação — reconciliar ao retomar. O que é **medição** aqui está datado.
>
> **Decidido pelo autor em 2026-08-30.** Esta lista substitui "o que fazer
> depois?" espalhado por seis roadmaps. Ela é linear de propósito: cada etapa
> só entra depois que a anterior fecha, e a ordem é por **dependência**, não
> por preferência.

## 1. O que "inicialmente completo" quer dizer, medido

O alvo é o **MVP essencial** da §11.1 de
`docs/specs/KINEIN_VECTIS_FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST.md`. Ele
lista 21 itens. Medição de **2026-08-30**, conferindo item por item contra o
código:

```text
19 EXISTEM     workspace, deteccao de projeto, scan de toolchain, cmake
               configure/build, cargo metadata/check/build, editor, Tree-sitter,
               status de clangd/rust-analyzer, diagnosticos, terminal, wizard
               de projeto C++/Rust, settings, project health, jobs, eventos,
               storage, UI QML e core por JSON-RPC

 1 NULO        "AI Terminal externo simples" — a linha de IA na IDE foi
               CANCELADA em 2026-07-17 e o `aiBridge` saiu do codigo no 0.59.0.
               E' item morto na spec, nao pendencia. A spec §11.1 nao foi
               reescrita porque e' PLANO datado; esta linha e' a reconciliacao.

 1 FALTA       Configuration Actions — ZERO ocorrencias em `crates/` e `ui/`.
```

**Ou seja: o MVP não está longe; falta uma feature e sobra dívida de
honestidade.** É isso que a ordem abaixo reflete.

Duas correções à fila que circulava antes desta medição, ambas verificadas:

- **"AppImage: frente Distribuição intocada" era falso.** A frente tem quatro
  commits reais (`c75537d` estabiliza distribuição e self-hosting híbrido,
  `ebec631`, `948535d`, `e40b200`) e cinco scripts. O que falta é outra coisa:
  `appimage` aparece **0 vezes** no `verificar.sh`. É frente construída e **não
  verificada** — a mesma forma do `deny.toml` antes de 2026-08-30.
- **"reabrir documentos após configure toca o `EditorController`" era falso.**
  Ver `roadmaps/29` §5b: é fatia de core, e a razão está medida lá.

## 2. As etapas

### Etapa 1 — Verdade: nenhum `[x]` apontando para artefato ausente ✅ FEITA (2026-08-30)

Dois itens marcados `[x] validado e2e` citavam sondas que **nunca foram
commitadas**: `sonda_drafts.py` (`seguranca/23` §P2.6) e `sonda_terminal.py`
(`roadmaps/24`, D2.1/D2.2).

**Decisão do autor: reescrever, não rebaixar.** Ambas existem agora, rodam e
foram provadas por mutação. O que a etapa ensinou, e que vale mais que ela:
a **primeira** versão da `sonda_drafts.py` ficou **verde com o produto
quebrado** — ela olhava a resposta do `workspace.open`, e o `recover_drafts`
descarta sozinho o rascunho idêntico ao disco, escondendo a linha órfã. A
checagem certa é na store SQLite, no mesmo processo, antes de qualquer
reabertura.

*Veio primeiro porque era a mais barata e porque tudo abaixo lê esses
documentos como estado.*

### Etapa 2 — Configuration Actions

O único item de MVP que não existe, e o diferencial do produto. Escopo fechado
pela §12 da spec de MVP: **10 ações CMake** (habilitar `compile_commands.json`,
criar preset Debug, criar preset Release, adicionar executável, adicionar
biblioteca estática, adicionar fonte a um target, adicionar include dir,
adicionar `target_link_libraries`, inspecionar o cache, reparar build dir
obsoleto) e **6 Cargo** (add dependency, add dev-dependency, add feature, set
edition, cargo check, criar run config).

Nasce como **domínio novo** — protocolo, core, handler, UI —, portanto nasce
dentro do limite da catraca e **não paga dívida de ninguém**. É a razão de ela
vir antes das etapas de dívida.

*Aceite:* domínio completo nas quatro camadas + `src/tests/configaction.rs` com
teste de integração, e cada ação com efeito verificado contra arquivo real.

### Etapa 3 — Servidor LSP falso para os testes

Hoje **15 métodos IPC de `lsp.*` não têm um único teste de comportamento**: as
286 linhas de `src/tests/lsp.rs` são todas "requires open workspace" ou
"reports unavailable". Nenhum teste deste repositório sobe um language server.

É a maior lacuna de evidência do projeto e o motivo pelo qual a etapa 4 está
barrada. Forma provável: um `ServerSpec` injetável apontando para um script que
fala JSON-RPC de mentira, para que `didOpen`/`didClose`/`didChange` passem a
ser observáveis.

*Aceite:* `did_open`, `did_close` e `did_change` provados por mutação.

### Etapa 4 — Reabrir documentos após `cmake.configure` (§5b do `roadmaps/29`)

O terreno já está pronto: `lsp/sync.rs` nasceu em 2026-08-30 e o `manager.rs`
caiu de 732 para 556 linhas, abrindo espaço. Falta a peça: um objeto
compartilhado entre o job do configure e o manager (ver `arquitetura/04` §3,
sobre a fronteira de thread), para que a próxima sincronização de um documento
C/C++ faça `didClose` + `didOpen` de verdade.

*Estava barrado por prova, não por espaço — por isso vem depois da etapa 3.*

### Etapa 5 — Toolchain como entidade (B2 do TR2)

Compilador, gerador, sysroot e flags são implícitos hoje: o que estiver no
`PATH`. Não existe o "kit" que CLion e Qt Creator expõem, então trocar de
compilador ou cruzar-compilar é editar arquivo à mão. É o item caro da §5 do
`roadmaps/29`, e **etapa própria por decisão do autor**.

### Etapa 6 — Pagar o `EditorController.qml` (1070/400)

`LEITURA_TECNICA` §4: *"o maior débito bloqueia por área"*. Enquanto ele
estiver assim, qualquer feature ou polimento de editor entra travado, porque a
catraca reprova o arquivo que crescer. Ele já tem subcontrollers; é continuar
movendo por responsabilidade, com o teste de vocabulário da §4 regra 9 como
critério.

### Etapa 7 — soak / estresse

Não existem, e são exigidos para afirmar "substituição diária". Sem eles não há
base para dizer que a IDE aguenta um dia de uso — só que ela passa em 378
testes curtos.

### Etapa 8 — Pôr o AppImage no gate

Os cinco scripts existem e funcionam; **nada os executa**. Mesma forma exata do
`deny.toml` antes de 2026-08-30 e do `shellcheck` antes de 2026-08-29: regra que
mora num arquivo que ninguém roda. Aqui o gate **não** nasce de falha
silenciosa nova — nasce de uma frente inteira sem verificação, que é o mesmo
critério da §4 regra 11 aplicado a build em vez de código.

### Etapa 9 — Busca e substituição multi-linha

`handlers/fs.rs:76` recusa `\n` em `query` e `replacement`. O autor confirmou
que vai precisar. Fatia própria.

### Etapa 10 — `WorkspaceUiResetter`: remover `shellController`

Fiação morta no composition root (`WorkspaceUiResetter.qml:15`: a propriedade é
recebida e nunca usada). Limpeza de um gesto.

## 3. Onde cortar, se a pergunta for "o mínimo para ser distribuível"

```text
FECHA O MVP        1 (feita), 2, 7, 8, 10
SEPARA MVP DE      3, 4, 5, 6, 9
DAILY DRIVER
```

As etapas 1, 2, 7, 8 e 10 entregam o MVP com honestidade e algo que se
distribui. As demais são o que transforma MVP em ferramenta de uso diário — que
é o alvo **seguinte**, não este.

## 4. O que esta lista deliberadamente NÃO inclui

Nada aqui reabre decisão registrada. Em particular, seguem fora:

```text
IA na IDE        FORA DE ESCOPO desde 2026-07-17 (docs-legada/)
Python           adiado; foco em C/C++ e Rust (2026-08-30). A excecao do
                 Tree-sitter de Python NAO foi tomada
Pylance          PROIBIDO (licenca so para produtos Microsoft). Se Python
                 voltar: basedpyright (MIT)
Docker / banco   NATIVOS, nao plugins. L5/L5.5, depois de L2-L4
host de plugins  nunca houve; nao ha runtime de plugin de terceiro
```
