# 04 — Os gates que dizem "não"

`bash scripts/verificar.sh` roda tudo em sequência e **para no primeiro
erro**. `--rapido` pula os builds e o "binário abre". Cada gate existe por
uma falha real, datada no `40`; quando um deles reprova, a resposta certa é
quase sempre corrigir o código — a exceção com motivo é o último recurso,
e fica escrita no próprio script.

| Gate | O que mede | Como ler o "não" |
| --- | --- | --- |
| `cargo fmt --all --check` | formatação | rode `cargo fmt --all` |
| `cargo test --workspace --all-features` | os testes Rust (843 em 2026-09-19) | um teste do LSP (`the_rust_server_receives_the_kit_target…`) é sensível a carga da máquina: se falhou sozinho durante um clang-tidy, rode-o isolado antes de investigar |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pedante, incluindo testes e doc-comments | nomes em doc-comments pedem crase (`` `SQLite` ``); `similar_names`, `too_many_lines` pedem split — não `#[allow]` |
| `verificar-deny.sh` | licenças e advisories das dependências | uma dependência nova precisa de licença compatível (MIT/Apache) |
| `verificar-shell.sh` | shellcheck nos scripts | |
| `verificar-appimage.sh` | invariantes do pacote | |
| `verificar-cpp.sh` | clang-format + clang-tidy (~1 h) | rode em segundo plano; não compile a UI enquanto roda |
| `verificar-qml.sh` | qmllint estrito, zero warnings, sobre o módulo do build `debug-strict` | "was not found" para um tipo novo = o build não foi refeito; `cmake --build --preset debug-strict` primeiro |
| `verificar-qml-fiacao.sh` | binding auto-referente | |
| `verificar-qml-propriedades.sh` | binding para propriedade inexistente; **margem de âncora sem a âncora**; **binding torto** (mais indentado que a propriedade) | os dois últimos são bindings que o Qt aceita e a tela mostra em branco |
| `verificar-qml-duplicacao.sh` | a mesma derivação (`kind === "commit"`) em dois arquivos | dê um dono (`inspector.isCommit`); a baseline só se atualiza para fatos diferentes que compartilham a string |
| `verificar-qml-alcance.sh` | componente registrado que nenhuma tela abre | ou ligue, ou remova do CMake e do disco |
| `verificar-fiacao-ipc.sh` | método sem handler, evento sem emissor, sinal C++ sem consumidor QML, sinal QML sem tratador, `dispatch*` fora da cadeia | 8 exceções ditas no script, com motivo; a sua precisa de um |
| `verificar-exercitacao.sh` | o core contra ferramentas reais (git, cmake, cargo…) | pula com motivo o que a máquina não tem |
| `verificar-embarcado.sh` | o ciclo de embarcado no QEMU, sem placa | |
| `verificar-python-debug.sh` | depurar Python com o debugpy real | |
| `verificar-clangd-cross.sh` | o clangd enxerga o cross do kit | |
| `verificar-atalhos.sh` | dois comandos não declaram o mesmo atalho; todo `default_shortcut` tem `Shortcut` anotado `// comando: <id>` com a mesma sequência; todo item de menu tem `case` no host | `Alt+Return` ≠ `Alt+Enter` no Qt |
| `verificar-docs.sh` | número sem data nos `.md` que diverge do disco | ponha a data, ou corrija o número |
| `verificar-links-docs.sh` | link relativo morto | |
| `verificar-arquitetura.sh` | a catraca: Rust 500 (sem testes), view 300, controller/host 400, ui/src 500; 1 arquivo em débito (`EditorController.qml`, 790) que não pode crescer | split por responsabilidade; nunca "Part2" |
| `verificar-transicao-workspace.sh` | estado por-workspace com um dono | |
| `verificar-qml-logica.sh` | os harnesses (`tst_*.qml`, 55 em 2026-09-19) num espelho do módulo | o bitmask está no `console.error`; a asserção correspondente está no arquivo |
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
