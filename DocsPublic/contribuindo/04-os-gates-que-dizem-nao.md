# 04 — Os gates que dizem "não"

`bash scripts/verificar.sh` roda tudo em sequência e **para no primeiro
erro**. `--rapido` pula os builds finais e o "binário abre", não toda
compilação que os testes precisem. Cada gate existe por
uma falha real, datada no `40`; quando um deles reprova, a resposta certa é
quase sempre corrigir o código — a exceção com motivo é o último recurso,
e fica escrita no próprio script.

Este arquivo é o catálogo de diagnóstico. O fluxo entre orquestrador,
gates-folha, auxiliares e artefatos, além do dono de cada responsabilidade,
está em
[`07-fluxo-e-responsabilidades-dos-gates.md`](07-fluxo-e-responsabilidades-dos-gates.md).

| Gate | O que mede | Como ler o "não" |
| --- | --- | --- |
| `cargo fmt --all --check` | formatação | rode `cargo fmt --all` |
| `cargo test --workspace --all-features` | os testes Rust (844 em 2026-09-23) | um teste do LSP (`the_rust_server_receives_the_kit_target…`) é sensível a carga da máquina: se falhou sozinho durante um clang-tidy, rode-o isolado antes de investigar |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pedante, incluindo testes e doc-comments | nomes em doc-comments pedem crase (`` `SQLite` ``); `similar_names`, `too_many_lines` pedem split — não `#[allow]` |
| `verificar-deny.sh` | licenças e advisories das dependências | uma dependência nova precisa de licença compatível (MIT/Apache) |
| `verificar-shell.sh` | shellcheck nos scripts | |
| `verificar-appimage.sh` | invariantes do pacote | |
| `verificar-cpp.sh` | clang-format + clang-tidy (~1 h) | rode em segundo plano; não compile a UI enquanto roda |
| `verificar-cpp-testes.sh` | teste C++ que falha; **nenhum teste declarado** | desde 2026-09-24. Antes disso o C++ era o unico codigo do projeto sem medida de comportamento: clang-tidy acha padrao, a fiacao acha elo sem dono, e "o binario abre" nao passa da primeira tela |
| `verificar-qml.sh` | qmllint estrito, zero warnings, sobre o módulo do build `debug-strict`; **`.qml` na árvore que não está no módulo** | desde 2026-09-24 ele atualiza sozinho a cópia do QML no build antes de lintar — antes disso podia **passar lintando QML velho**. O que ele recusa é arquivo que ninguém compila: registre em `ui/CMakeLists.txt` ou remova |
| `verificar-qml-fiacao.sh` | binding auto-referente | |
| `verificar-qml-propriedades.sh` | binding para propriedade inexistente; **margem de âncora sem a âncora**; **binding torto** (mais indentado que a propriedade) | os dois últimos são bindings que o Qt aceita e a tela mostra em branco |
| `verificar-qml-duplicacao.sh` | a mesma derivação (`kind === "commit"`) em dois arquivos | dê um dono (`inspector.isCommit`); a baseline só se atualiza para fatos diferentes que compartilham a string |
| `verificar-qml-alcance.sh` | componente registrado que nenhuma tela abre | ou ligue, ou remova do CMake e do disco |
| `verificar-fiacao-ipc.sh` | método roteado que nenhum cliente pede; **cliente que pede método que o core não roteia** (desde 2026-09-24); evento do core que o C++ não trata; sinal sem ouvinte; elo de despacho sem chamador | a direção inversa é a perigosa: é um botão que não faz nada em tempo de execução, e nada compila errado |
| `verificar-exercitacao.sh` | o core contra ferramentas reais (git, cmake, cargo…) | pula com motivo o que a máquina não tem |
| `verificar-embarcado.sh` | o ciclo de embarcado no QEMU, sem placa | |
| `verificar-python-debug.sh` | depurar Python com o debugpy real | |
| `verificar-clangd-cross.sh` | o clangd enxerga o cross do kit | |
| `verificar-atalhos.sh` | dois comandos não declaram o mesmo atalho; todo `default_shortcut` tem `Shortcut` anotado `// comando: <id>` com a mesma sequência; todo item de menu tem `case` no host | `Alt+Return` ≠ `Alt+Enter` no Qt |
| `verificar-docs.sh` | contagem de linhas sem data reconhecida nos `.md` que diverge do disco; não prova toda afirmação numérica/semântica | ponha a data, ou corrija o número |
| `verificar-links-docs.sh` | alvo relativo inexistente em Markdown versionado ou novo não ignorado; não verifica âncoras/URLs externas | |
| `verificar-arquitetura.sh` | a catraca: Rust 500 (sem testes), view 300, controller/host 400, ui/src 500; 1 arquivo em débito (`EditorController.qml`, 790) que não pode crescer | split por responsabilidade; nunca "Part2" |
| `verificar-transicao-workspace.sh` | estado por-workspace com um dono | |
| `verificar-qml-logica.sh` | os harnesses (`tst_*.qml`, 61 em 2026-09-23) num espelho do módulo | o bitmask está no `console.error`; a asserção correspondente está no arquivo |
| `verificar-binario-abre.sh --preset <p>` | o binário abre, primeiro frame medido, **nenhum aviso QML no stderr** até o frame | um `Connections` no alvo errado só aparece aqui |

## O que fazer quando um gate parece "errado"

1. Meça: reproduza o que ele mede à mão (`grep`, o harness, a foto).
2. Se o gate acusou algo real, corrija o código — e registre no `40`.
3. Se o gate acusou algo que não é falha (raro), escreva a exceção **no
   script**, com data e motivo, e a linha no `40`. Uma exceção sem motivo
   é o próximo bug silencioso.
4. Se falta um gate (uma falha silenciosa passou), escreva-o: um
   `scripts/verificar_<x>.py` + `.sh`, e **prove por mutação** (quebre de
   propósito, veja o gate reprovar, desfaça).
