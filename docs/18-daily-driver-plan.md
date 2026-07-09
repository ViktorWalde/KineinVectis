# 18 — Plano de daily driver e escada de rigor

> **Status:** ativo
> **Prioridade:** P0 (direção de produto/engenharia)
> **Fonte de verdade:** direção e ordem de execução; UX-alvo continua em
> `docs/specs/`, contratos em `docs/03-ipc-protocol.md`, ponte backend→UI em
> `docs/BACKEND_TO_UI_UX_ROADMAP.md`
> **Ultima revisao:** 2026-07-08

## Objetivo

Tornar o Kinein Vectis a IDE principal do autor **o quanto antes**, sem perder
qualidade, e sustentar isso com rigor crescente de engenharia. Réguas
explícitas:

```text
- Navegação/refactor/analise: régua JetBrains (não "um VS Code").
- Velocidade e composição de ferramentas: régua Neovim super configurado
  (LSP + treesitter + telescope + dap + gitsigns + conform + toggleterm).
- Filosofia inalterada (AGENTS.md): orquestrar ferramentas maduras, nunca
  reimplementar compilador, LSP, debugger ou build system.
```

## Critério de pronto: dogfooding

Cada marco é medido por uma pergunta concreta no repositório do próprio
Kinein Vectis (Rust + C++/CMake, os dois kinds já suportados):

```text
M0  "Consigo abrir, editar, buildar e testar?"                 [feito]
M1  "Consigo passar um dia inteiro só no Kinein?"              [curto prazo]
M2  "Consigo depurar e rodar qualquer alvo sem terminal?"      [médio prazo]
M3  "Consigo commitar, revisar diff e refatorar sem sair?"     [médio/longo]
M4  "Está rápido, estável e agradável o dia inteiro?"          [contínuo]
```

Quando a resposta de um marco vira "sim" no uso real (não em demo), o marco
está pronto. Regressão em marco anterior bloqueia o seguinte.

## M0 — Base funcional (feito, 2026-07-08)

Editor com abas/salvar/highlight, project tree confinada ao workspace,
build/test/quality como jobs assíncronos canceláveis, aba Jobs genérica,
Problems com build/quality/LSP, LSP básico (diagnostics, hover, completion,
definition, references, rename), run + terminal PTY, scan de ambiente,
Project Health banner mínimo. Gate único `scripts/verificar.sh` verde
(Rust + C++ + QML estrito).

## M1 — Edição diária confortável (curto prazo)

Meta: um dia inteiro de edição sem abrir outra IDE. Fatias, em ordem
recomendada (cada uma pequena, com UI visível e gate verde):

```text
1. Formatação orquestrada: rustfmt/clang-format (e qmlformat quando o tree
   estiver formatado) via core como job; format-on-save opt-in.
   Régua Neovim: conform.nvim.
2. Semantic tokens na UI: o core já decodifica (lsp/parse.rs); falta o
   EditorHighlighter consumir spans semânticos por completo e pedir refresh
   no didChange. Régua Neovim: treesitter/semantic tokens.
3. Code actions / quick fixes LSP (pendência declarada da Fase 5), começando
   por diagnósticos com fix associado (clippy/clangd sugerem muitos).
4. Go-to-symbol de arquivo e workspace (documentSymbol/workspaceSymbol)
   dentro do Search Everywhere. Régua Neovim: telescope lsp_*_symbols.
5. Sessão por workspace: reabrir abas/aba ativa ao reabrir o mesmo root
   (persistir em .kinein/, schema versionado).
6. Ergonomia de editor incremental: duplicar linha, mover linha, comentar
   seleção, ir para linha — atalhos JetBrains-like no CommandDispatcher.
```

Fora de escopo do M1 (explicitamente pós-V1, custo alto no TextEdit atual):
multi-cursor real, minimap, split de editor. Não fingir paridade aqui.

## M2 — Build/Run/Debug de verdade (médio prazo)

```text
- CMake service: presets.list/configure/targets (roadmap P1) + toolbar de
  profile/target; compile_commands.json alimentando clangd.
- Cargo service: metadata/check/features + toolbar equivalente.
- Run configurations persistidas (roadmap P2) no lugar da heurística atual.
- Debugger via DAP: orquestrar lldb-dap (e/ou gdb DAP) como serviço do core,
  com breakpoints na gutter, stack/variáveis em painel inferior.
  Régua Neovim: nvim-dap + nvim-dap-ui. É a maior peça nova; nasce como
  domínio próprio no core (debug/), operações longas como jobs, contrato
  tipado em kinein-protocol antes de UI.
```

## M3 — Git e refactor (médio/longo prazo)

```text
- Git MVP (roadmap P2): status/branch na status bar, mudanças na tree,
  painel de diff/stage/commit; ações destrutivas atrás de risk/confirmação.
  Régua Neovim: gitsigns + fugitive.
- Blame/annotations no editor.
- Refactorings por cima do LSP além de rename (organize imports, extract
  quando os servers expuserem); régua JetBrains como norte da Fase 5+.
```

## M4 — Polimento contínuo (longo prazo)

```text
- Orçamento de performance medido localmente (startup, latência de digitação,
  memória com N abas) — sem telemetria, medição manual/scripts locais.
- Settings UI com schema/migração (roadmap P1) e seletor Strict/Balanced/
  Relaxed explícito.
- First Run/onboarding dos specs quando houver mais de um usuário real.
```

## Escada de rigor

Degrau novo só entra **verde**: primeiro zerar os achados, depois ligar o
gate (como foi feito com o qmllint em 2026-07-08: 117 avisos zerados e só
então `verificar-qml.sh` entrou no `verificar.sh`).

Hoje (2026-07-08):

```text
Rust : unsafe forbid; -D warnings; clippy all/pedantic/nursery deny;
       missing_docs deny; unwrap/expect/panic proibidos fora de teste;
       fmt --check; 170+ testes de comportamento.
C++  : clang-format --Werror; clang-tidy amplo com WarningsAsErrors;
       sanitizers em Debug; hardening+LTO em Release.
QML  : qmllint estrito (zero warnings) com contexto de módulo, via
       scripts/verificar-qml.sh; pragma ComponentBehavior: Bound e
       required properties em delegates são o padrão do repositório.
Gate : scripts/verificar.sh único, para no primeiro erro.
```

Próximos degraus, em ordem de custo/benefício:

```text
1. cargo-deny check no gate (deny.toml já existe; instalar cargo-deny).
2. shellcheck nos scripts/ (instalar shellcheck; são 5 scripts pequenos).
3. qmlformat --check: exige reformatar o tree QML inteiro uma vez — fazer
   numa fatia dedicada, sem misturar com feature.
4. Testes de lógica QML (Qt Quick Test) para controllers não visuais
   (ProjectHealthController, ShellController, etc.) — hoje só o Rust tem
   teste automatizado; a lógica QML depende de smoke manual.
5. CI real (GitHub Actions rodando verificar.sh) quando o repo ganhar
   remote; até lá o gate local é o contrato.
6. Cobertura no core (cargo-llvm-cov) com piso acordado, sem virar métrica
   de vaidade.
```

## Princípios de lógica (valem para toda fatia nova)

```text
- Handler IPC fino; regra de negócio em serviço de domínio no core, com teste.
- QML visual burro: dados por property, ação por signal; estado/timer em
  controller; evento IPC em roteador (guardrails de docs/17).
- Uma fonte de verdade por estado; nada de estado derivado duplicado na UI.
- Operação longa = job cancelável com eventos (nunca bloquear UI/stdio).
- Erro estruturado no protocolo; a UI nunca parseia stdout de ferramenta.
- Contrato novo: tipos em kinein-protocol + docs/03 + schema quando persistir.
```

## Como usar este documento

```text
1. Pegar a próxima fatia do marco corrente (ordem acima).
2. Cruzar com docs/BACKEND_TO_UI_UX_ROADMAP.md (contrato/backend necessário)
   e docs/specs/ (UX alvo inegociável).
3. Entregar com gate verde + smoke, atualizar ContextoIA.md e este doc
   (marcar fatia feita com data).
4. Ao fechar um marco, revalidar os anteriores no uso real antes de seguir.
```
