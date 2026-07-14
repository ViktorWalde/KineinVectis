# PONTO ATUAL — guia de execução (2026-07-14)

> **O que este doc é:** a lista **explícita do que falta fazer**, em ordem, com
> detalhe suficiente pra executar sem reler o projeto inteiro.
> **O que ele NÃO é:** o contexto do produto. Isso vive no `ContextoIA.md`
> (norte, mandatos, decisões) e nos `docs/` numerados. Aqui só entra **tarefa**.
>
> **Regra que vale pra tudo abaixo:** NÃO commitar — o Viktor faz o commit; o
> working tree acumula.
>
> **Ambiente:** o Viktor voltou pro **CLion** e desenvolve o Kinein lá até o
> projeto ficar pronto. O dogfooding está **pausado** — então não chega mais bug
> de uso diário por uso real, e a régua deixou de ser *"dá pra usar hoje?"* e
> virou **"está pronto pra virar o daily driver?"**. É essa régua que ordena a
> lista.

## Checkpoint de execução — 2026-07-14 (protocolo 0.50.0)

O lote recomendado foi implementado sem criar subsistemas paralelos:

- D3/F1/F2: Tree-sitter incremental C/C++/Rust, highlight/folding/outline/
  locals, registry e cache limitado; design em `docs/25`.
- Workspace edits LSP: preview, aplicar/cancelar, snapshot versionado,
  gravação multi-arquivo atômica e rollback.
- T3–T6: branches/pull/push/stash; substituir no projeto; salvar tudo;
  arquivos recentes.
- D4 material: ícones/componentes vetoriais, App Bar, Main Toolbar, Start
  Screen, preview de criação, scaffold C++ target-based e KV Context.
- Q1: harness QML está no gate oficial (7 cenários atuais).
- Correção funcional pós-teste visual: menus Arquivo–Ajuda agora usam overlay
  global e ações auditadas; tooltips não ficam sob o editor; Arquivo e clique
  direito criam arquivo/pasta pelo fluxo existente; KV Context inicia
  Claude/Codex instalados pelo usuário sobre o `TerminalManager`; Estrutura e
  painéis são redimensionáveis, recolhíveis/responsivos e persistidos.

**O que ainda depende de pessoa/tela:** executar a validação ao vivo da seção
1 e o aceite visual R7/C6 contra as specs. Os próximos passos de engenharia,
depois desse aceite, são as fases maiores do KSWE (grafo de targets/contextos,
CMake File API completa e scheduler semântico), não correções das lacunas
T3–T6 listadas historicamente abaixo.

---

## 0. Como trabalhar (o mínimo)

**Ritual por fatia:** design em `docs/24` (fases D) ou `docs/18` (fases M/E/T)
→ protocolo (`crates/kinein-protocol`) → core (`crates/kinein-core`) → handler →
testes → UI (`ui/qml`, `ui/src`) → sincronizar docs (`docs/03` se mudar
protocolo, `MANUAL.md` se mudar UX).

**Gate (tem que ficar verde antes de dar por pronto):**

```bash
bash scripts/verificar.sh              # fmt/clippy -D warnings/testes/clang-tidy/qmllint/builds
bash scripts/verificar-qml-logica.sh   # lógica QML headless — também executada pelo gate oficial
cargo build -p kinein-core             # OBRIGATÓRIO antes de qualquer sonda e2e
cmake --build build/dev-local --target kinein-vectis
```

**Três armadilhas que já morderam:**

1. `cargo test` e o gate **não recompilam** `target/debug/kinein-core`. Sem
   `cargo build -p kinein-core` antes, a sonda e2e roda contra binário velho.
2. Controller QML é `Item { visible: false }`. **Nunca** use o `visible` dele
   como estado de UI, nem faça `alias` pra `.visible` — o Qt lê a visibilidade
   *efetiva* (do pai), não o valor gravado. Use `property bool` própria. Foi
   essa armadilha que segurou o D1 por dois ciclos.
3. Todo `.qml` novo precisa de `QT_RESOURCE_ALIAS` no `ui/CMakeLists.txt`; toda
   classe C++ exposta ao QML precisa entrar em `SOURCES` do `qt_add_qml_module`.

---

## 1. Pendente de você: validação ao vivo (3 minutos)

As quatro frentes estão **verdes no código e nas verificações automatizadas —
mas ainda falta confirmar os gestos na tela.** Enquanto isso, trate a validação
visual como pendente.

```bash
./build/dev-local/ui/kinein-vectis
```

- **D1 (autocomplete):** abrir projeto **Rust**, abrir um `.rs`, esperar ~10s
  (rust-analyzer indexando), digitar `let x = St`. **O popup tem que aparecer.**
  Em C/C++, rode "CMake: Configure" antes — o clangd precisa do
  `compile_commands.json`.
- **D1b (Find/Replace):** `Ctrl+F` abre a barra pré-preenchida com a palavra sob
  o cursor, realça todas as ocorrências, mostra "3 de 17". `Enter`/`Shift+Enter`
  navegam (circular), `Esc` fecha. `Ctrl+H` mostra o campo de substituir.
  Toggles `Aa` / `W` / `.*`. `F3`/`Shift+F3` navegam com a barra fechada.
- **B1/B2 (scroll + barras):** abrir o terminal (`Alt+F12`), rodar algo longo
  (`seq 1 300`), e **rolar com a roda**. A tela tem que subir no histórico e a
  **barra de rolagem** tem que aparecer à direita (dá pra arrastar). No editor,
  abrir um arquivo grande: a barra aparece à direita e arrasta. Conferir também
  uma lista longa em Build/Problemas/Busca.
- **T1/D2.3 (múltiplos terminais):** no Terminal, clicar `+`, deixar um comando
  contínuo na primeira aba e usar a segunda; alternar os chips preserva cada
  tela, e `×` fecha só a aba escolhida.

---

## 2. BUGS — B1 e B2 RESOLVIDOS (2026-07-12, protocolo 0.43.0)

### ✅ B1 — a roda do mouse não rolava o terminal

**A causa era grave e não estava onde eu suspeitei.** Não era a fiação (estava
correta) nem o MouseArea. Era um **bug do `vt100` 0.15.2 que DERRUBAVA O CORE**:
`visible_rows()` monta a tela como *(últimas `offset` linhas de histórico) +
(`rows - offset` linhas vivas)*, e um `offset` maior que a altura da tela fazia
`24 - 278` em `usize` → **overflow**.

E a UI alimentava exatamente esse caso: o `onWheel` somava **sem teto**. Com
passo de 3 linhas e tela de ~24, **~9 cliques de roda** estouravam o limite. O
core morria, a recuperação de crash da M4.3 o **relançava calada**, a sessão de
terminal ia junto — e você via "não acontece nada".

**Fix (3 camadas):** (1) `vt100` **0.15 → 0.16.2**, que corrige o overflow
upstream — e de quebra libera o scrollback inteiro (no 0.15 ele era inalcançável
além de UMA tela); (2) o core passou a mandar `scrollback` (offset real, já
clampado) e `scrollbackMax` (quanto histórico existe) no `event.terminal.render`;
(3) a UI trocou o `MouseArea.onWheel` por um `WheelHandler`, **clampa** em
`scrollbackMax` e reconcilia o offset local pelo que o core devolve.

**Provado:** `scripts/sonda_scrollback.py` (e2e, core real) — antes, offset
absurdo matava o core com `attempt to subtract with overflow`; agora clampa em
278 e a tela rola de verdade.

### ✅ B2 — não havia barra de rolagem em lugar NENHUM da IDE

**Confirmado por varredura:** `ScrollBar`/`ScrollView` não apareciam **uma única
vez** em todo o `ui/qml`. Sem barra você não sabe que há conteúdo fora da tela,
onde está no arquivo, nem **se uma rolagem aconteceu** — foi o que manteve o B1
invisível.

**Feito:** `ui/qml/shell/VerticalScrollBar.qml`, barra própria (o projeto não usa
`QtQuick.Controls`) e genérica na unidade — serve em PIXELS (editor) e em LINHAS
(terminal, onde é sintética e de eixo invertido). Aplicada no **editor** e no
**terminal** e reaproveitada nos `ListView` dos painéis inferiores. Esta sobra
barata está fechada; `qmllint` estrito verde.

---

## 3. Inventário: o que falta pro Kinein virar o daily driver

Levantado lendo o código em 2026-07-12 (não é chute; cada item foi verificado).
**A infra difícil já existe** — build, debug (DAP), git, LSP (navegação,
diagnósticos em tempo real, code actions, rename), terminal com PTY real, rede de
segurança contra perda de dado. O que falta é o que segue.

### 🔴 Bloqueia o daily driver

**Concluído desde o levantamento:** **T1 / D2.3 — múltiplas abas de terminal**
(protocolo 0.44.0) e **T2 — mudanças externas protegidas** (protocolo 0.45.0).
T2 usa `notify` lazy/debounced, atualiza editor/árvore/Git e adiciona
compare-before-save: uma aba velha nunca sobrescreve o disco silenciosamente.

| # | Falta | Por que bloqueia | Onde |
|---|---|---|---|
| ~~**T2**~~ | ~~**Mudança externa de arquivo não é detectada**~~ | ✅ **FEITO (0.45.0):** watcher `notify` lazy + debounce; `event.fs.changed`; auto-reload de aba limpa; conflito preserva o buffer; `fs.write` exige `expectedContent` e recusa snapshot velho com `FILE_CHANGED` | core/protocolo/UI + auditoria em `docs/tooling/` e ADR |
| ~~**T3**~~ | **Git: branches/remotos/stash** | ✅ **FEITO (0.49.0):** branches/checkout/create, pull/push como jobs, stash push/pop com guarda de buffers sujos e exclusão de `.kinein` | core/protocolo/UI/testes Git reais |
| ~~**T4**~~ | **Substituir no projeto (`Ctrl+Shift+H`)** | ✅ **FEITO (0.48.0):** literal, confirmação, ignores/limites, transação multi-arquivo e rollback | `fsops/replace.rs` + transação compartilhada + Busca |
| ~~**T5**~~ | **Salvar tudo (`Ctrl+Shift+S`)** | ✅ **FEITO:** fila determinística e format-on-save, com stale-drop | EditorController + harness QML |
| ~~**T6**~~ | **Arquivos recentes (`Ctrl+E`)** | ✅ **FEITO:** MRU reutiliza Search Everywhere, sem modelo paralelo | Editor/Search controllers + harness QML |

### 🟡 Fundação (mandato do produto)

| # | Falta | Detalhe |
|---|---|---|
| ~~**F1**~~ | **Tree-sitter** (D3) | ✅ **FEITO (0.46.0):** design `docs/25`, pins/licenças auditados, `lang/`, incremental C/C++/Rust, highlight/fold/outline/locals |
| ~~**F2**~~ | **Views em árvore + ícones** (D3) | ✅ **FEITO:** Outline estrutural recolhível + `KvIcon` vetorial central e migração das ações QML |
| **F3** | **Convergência UI/UX** (D4) | ✅ Implementação material C2/C3/C5 feita; **resta validação visual R7/C6 pelo usuário** |

### 🟢 Conforto (não bloqueia — puxar quando incomodar)

Split editor (lado a lado) · multi-cursor · zoom `Ctrl+±` · EditorConfig ·
primitiva de toast (aviso discreto) · links clicáveis no terminal · busca no
scrollback · debug: watch/expressões e breakpoints condicionais · terminal:
duplo-clique = palavra.

---

## 4. Pendência técnica atravessada em tudo

### Q1 — Teste que EXECUTA lógica QML no gate — FEITO

`scripts/verificar.sh` cobre Rust (fmt/clippy/testes), C++ (clang-format/tidy) e
`qmllint` — mas **nada executa a lógica QML**, que é justamente onde a IDE guarda
o estado da UI. Foi por esse buraco que o bug do D1 sobreviveu a **dois ciclos**
de "correção" (sonda verde no backend, GUI quebrada). **O B1 é o mesmo padrão
acontecendo de novo.**

Já existe o começo, **fora do gate**:

```bash
bash scripts/verificar-qml-logica.sh   # hoje: 4 testes, todos verdes
```

`tst_completion.qml`, `tst_find.qml`, `tst_multi_terminal.qml` e
`tst_external_change.qml` carregam os
controllers **REAIS** (não cópias) com bridges falsos, em `qml6` offscreen, e codificam as
falhas no código de saída. O `tst_completion` é a regressão do D1; o `tst_find`
cobre os casos que matam um find/replace (regex de largura zero → o TIMEOUT pega
o laço infinito; `Substituir tudo` com substituto maior → offsets); o terceiro
cobre troca/fechamento/isolamento e limpeza após crash dos terminais; o quarto
cobre conflito externo, auto-reload e a base usada pelo save seguro.

O harness foi integrado ao `scripts/verificar.sh` em 2026-07-14. Os controllers
reais agora são executados tanto no gate rápido quanto no completo, antes dos
builds da UI.

---

## 5. Ordem recomendada

1. ~~**B1 + B2**~~ — ✅ **FEITOS** (2026-07-12). Falta só o seu OK ao vivo.
2. ~~**Barra de rolagem nos painéis inferiores**~~ — ✅ **FEITA**.
3. ~~**T1 / D2.3** — múltiplas abas de terminal~~ — ✅ **FEITA** (0.44.0;
   falta OK ao vivo).
4. ~~**T2** — watcher + save protegido~~ — ✅ **FEITO** (0.45.0).
5. ~~**Q1** — plugar o teste de QML no gate~~ — ✅ **FEITO**.
6. ~~**F1/F2 (D3)** — tree-sitter + views em árvore~~ — ✅ **FEITOS**.
7. ~~**T3–T6**~~ — ✅ **FEITOS**.
8. **F3/D4** — correção funcional automatizada feita; validar ao vivo menus,
   tooltips, criação, KV Context e o recolhimento/redimensionamento da
   Estrutura antes de aceitar R7/C6.
9. Depois do aceite: iniciar KSWE em fatias (Project Graph/Context Matrix,
   CMake File API completa, scheduler e Diagnostic/Symbol Brokers), com design
   e orçamento antes de ampliar o core.
10. Norte de longo prazo: fazer a IDE entrar em “simbiose com os compiladores”
    (`docs/21`, seção própria) — contexto incremental profundo de projeto sem
    reimplementar compilador/build system/LSP. O primeiro patamar desejado é
    “um nível abaixo do CLion”; depois, evoluir por profundidade de integração.
    Inclui targets/toolchains, debug e variáveis, flash/serial/QEMU/OpenOCD,
    split editor/multi-cursor/EditorConfig e testes prolongados em projetos
    reais.
11. Depois da validação funcional: acrescentar a opção guiada de **outra IA**
    (abrir terminal + instruir o comando manual, sem provider embutido) e
    retomar a **biblioteca de funções / Configuration Actions** para facilitar
    configuração de ambiente, seguindo a spec própria e sem criar engine
    genérica prematuramente.
12. **Workspaces recentes:** promover o item já desenhado em `docs/21` M4.4
    para a próxima fatia de Start Screen. Persistir uma lista global limitada,
    ordenar por último acesso, remover caminhos inexistentes e permitir
    fixar/remover entradas; reusar `workspace.open` e o seletor atual.
13. **Distribuição:** hoje o produto é Linux-first (Arch/CachyOS validado;
    bootstrap Debian/Ubuntu/Fedora). Windows não está suportado ainda. AUR e
    AppImage devem empacotar UI + core + runtime Qt; compiladores/LSP/debuggers
    continuam dependências externas escolhidas por linguagem.
14. **Espelho público futuro:** o privado permanece fonte completa. O export
    público leva código, ativos e avisos de licença, mas entre arquivos
    Markdown publica somente `README.md` e `MANUAL.md`. Excluir `ContextoIA.md`,
    `PONTO_ATUAL.md`, `AGENTS.md`, `docs/**/*.md`, specs e roadmaps internos.
    Criar exportador allowlist com `--dry-run` e auditoria de segredos antes de
    tornar qualquer remoto público; nunca apenas trocar a visibilidade deste
    repositório privado.

---

## 6. Onde ler mais (só se precisar)

| Doc | Pra quê |
|---|---|
| `ContextoIA.md` | Norte do produto, mandatos, decisões. **Não precisa ler pra executar a lista acima.** |
| `docs/24-paridade-e-fundacao.md` | A fase D1–D4 (status vivo) |
| `docs/03-ipc-protocol.md` | Contrato IPC implementado (**0.50.0**) |
| `docs/18-daily-driver-plan.md` | Design das fatias M/E/T |
| `docs/20-ui-spec-convergence-plan.md` | Regras vinculantes de convergência visual (C0–C6) |
| `KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` | **Ler antes do D3** — como adotar ferramenta open-source |
