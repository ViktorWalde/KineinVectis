# 04 — Os gates que dizem "não"

> **Perfil vigente (2026-10-07):** o Qt local do sistema, por orientação do
> autor — desde a migração para Arch Linux na mesma data, Qt 6.12.0 (antes
> 6.10). G0.2/G0.3 do Qt 6.4 permanecem scripts legados; não são executados
> pelo orquestrador nem exigidos nas próximas fatias. Lint, harnesses e CTest
> locais continuam obrigatórios; provas antigas abaixo conservam suas datas.

`bash scripts/verificar.sh` roda tudo em sequência e **para no primeiro
erro**. `--rapido` pula o build release e os smokes ("o binário abre"), não a
compilação da UI de debug, que roda antes dos lints. `--estrito` combina com
os dois e reprova o que esta máquina não conseguiu provar (abaixo). Cada gate
existe por uma falha real, datada no `40`; quando um deles reprova, a resposta
certa é quase sempre corrigir o código — a exceção com motivo é o último
recurso, e fica escrita no próprio script.

Este arquivo é o catálogo de diagnóstico. O fluxo entre orquestrador,
gates-folha, auxiliares e artefatos, além do dono de cada responsabilidade,
está em
[`07-fluxo-e-responsabilidades-dos-gates.md`](07-fluxo-e-responsabilidades-dos-gates.md).

## Os três desfechos

| Saída do `verificar.sh` | O que significa | O que fazer |
| --- | --- | --- |
| `✓ TUDO VERDE (modo)` | todo gate rodou e passou | nada |
| `✓ VERDE no que esta maquina prova (modo)` + lista `NAO PROVADO` | o código passou em tudo que rodou; os itens da lista **não rodaram** por falta de ferramenta de ambiente (QEMU, debugpy, kit cross, Podman/Docker) | instale o que a lista nomeia, ou rode onde existe; antes de release, `--estrito` |
| `✗ FALHOU em: <etapa>` | um gate reprovou; com `--estrito`, também a lista não vazia | leia a etapa na tabela abaixo |

A regra que decide entre "reprova" e "não provado" está no
[`07` §1.1](07-fluxo-e-responsabilidades-dos-gates.md#11-o-protocolo-não-provado-desde-2026-10-01):
ferramenta de gate do **repositório** ausente reprova; ferramenta de
**ambiente** ausente vira NÃO PROVADO — nunca verde calado.

## O catálogo

| Gate | O que mede | Como ler o "não" |
| --- | --- | --- |
| `cargo fmt --all --check` | formatação | rode `cargo fmt --all` |
| `cmake --build --preset <debug>` | a UI de debug compila | roda **antes** dos lints desde 2026-10-01: o qmllint e os harnesses leem o build, e lintar contra o build de ontem é um gate que mente |
| `cargo test --workspace --all-features --no-fail-fast` | os testes Rust (852 no core em 2026-10-03) | `--no-fail-fast`: todos os crates rodam e o vermelho aparece inteiro. **Em paralelo** de novo desde 2026-10-03 (13 s, contra cerca de 44 s em série). De 2026-09-24 a 2026-10-03 rodou em uma thread por causa da corrida de `ETXTBSY` entre escrever um executável de teste e o `fork` de outro teste. A causa foi resolvida: todo executável de teste é gravado por `crate::write_executable`, que nunca o abre para escrita no processo dos testes. Foram 25 rodadas paralelas sem falha; antes, o teste do LSP `the_rust_server_receives_the_kit_target…` e o da descoberta do pytest falhavam de vez em quando. Um teste novo que grava executável **usa o `write_executable`**. Os testes de execução descontam o que o **perfil de login** imprime (`sh -lc true`), e os de permissão não fazem o pedido quando rodam como root |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pedante, incluindo testes e doc-comments | nomes em doc-comments pedem crase (`` `SQLite` ``); `similar_names`, `too_many_lines` pedem split — não `#[allow]` |
| `verificar-deny.sh` | licenças e advisories das dependências | uma dependência nova precisa de licença compatível (MIT/Apache) |
| `verificar-shell.sh` | shellcheck nos scripts | `source` de outro script pede `# shellcheck disable=SC1091` com o motivo (o shellcheck roda arquivo a arquivo) |
| `verificar-appimage.sh` | invariantes do pacote | |
| `verificar-cpp.sh` | clang-format + clang-tidy com o banco do preset **clang** `linux-clang-debug-strict` | o gate configura esse preset sozinho quando o banco falta ou envelheceu ("preparo: configurando…"); o `dev-local` não serve porque usa o GCC, e o clang-tidy não entende as flags dele. Leva minutos; não compile a UI enquanto roda |
| `verificar-cpp-testes.sh` | teste C++ que falha; **nenhum teste declarado** | desde 2026-09-24. Antes disso o C++ era o único código do projeto sem medida de comportamento |
| `verificar-qml.sh` | qmllint estrito, zero warnings, sobre o build que o orquestrador indica (`KINEIN_QML_BUILD_DIR`); **`.qml` na árvore que não está no módulo** | atualiza sozinho a cópia do QML e os tipos antes de lintar (antes de 2026-09-24 podia **passar lintando QML velho**). O que ele recusa é arquivo que ninguém compila: registre em `ui/CMakeLists.txt` ou remova. **O qmllint que vale é o ≥ 6.5** (decisão do autor, 2026-10-01): o do Qt 6.4 acusa 6 falsos positivos em código correto (`StandardKey.Open`, `push` em array JS, `Qt.callLater`), e por isso, abaixo do 6.5, os achados são **mostrados** e o lint vira **NÃO PROVADO**, limpo ou não (`--estrito` reprova). Versão que não se lê vale como estrita. Relatório ausente ou inválido reprova em qualquer versão |
| `check_identifier_language.py` (G0.1) | identificador novo em português (Rust, C++, QML/JS, Python, shell e Python em heredoc) | traduza pelo [glossário](09-glossario-de-identificadores.md); sigla ou nome de ferramenta vai para `identifier-language-allowlist.txt` com motivo. O legado é catraca (`identifier-language-baseline.txt`): só desce, com `--update-baseline`. Python é lido pela **AST** desde 2026-10-01 — o `tokenize` do 3.11 não via nome dentro de f-string. Um `.py` que não parseia reprova |
| `verificar-qml-qt64.sh` (G0.2) | `header`/`footer`/`highlight`/`section.delegate`/`sourceComponent` num arquivo com `pragma ComponentBehavior: Bound` — também na sintaxe agrupada (`section { delegate: … }`) | o Qt 6.4 do AppImage nunca cria essas partes. Ponha o `Component` num `QtObject` sem o pragma (`*Parts.qml`). Se o erro for "o próprio scanner do G0.2 regrediu", alguém afrouxou `scripts/check_qml_qt64.py`: os casos de mutação dele rodam a cada execução |
| `verificar-qml-logica-qt64.sh` (G0.3) | harness QML que falha no **Qt 6.4.2 do AppImage** (container Debian 12), mesmo verde no Qt local | precisa de Podman ou Docker **com o serviço respondendo**; sem isso é NÃO PROVADO. Reproduza um teste só com `KINEIN_QML_TEST=tst_x` |
| `check_terminal_quiet.py` (G0.5, modo completo) | `kinein <pasta>` chamado de um terminal que demore > 300 ms a devolver o prompt, imprima algo, deixe mensagem no log, ou **saia sem primeiro frame** | "saiu SEM primeiro frame" é crash ou saída precoce do processo desacoplado — o arquivo de `KINEIN_PERF_MARKER_FILE` não nasceu. Saída no terminal depois do prompt quase sempre é stdio herdado pelo filho |
| `run-surface-tour.sh` (G0.4, dentro do `verificar-binario-abre` e do `testar-appimage`) | aviso QML em qualquer passo de `scripts/surface-tour.txt`; id que ninguém trata; passeio que não termina | a linha `[passo]` diz qual comando abriu a superfície que avisou: comece por ela |
| `verificar-qml-fiacao.sh` | binding auto-referente | |
| `verificar-qml-propriedades.sh` | binding para propriedade inexistente; **margem de âncora sem a âncora**; **binding torto** (mais indentado que a propriedade) | os dois últimos são bindings que o Qt aceita e a tela mostra em branco |
| `verificar-qml-duplicacao.sh` | a mesma derivação (`kind === "commit"`) em dois arquivos | dê um dono (`inspector.isCommit`) |
| `verificar-qml-tokens.sh` | raio, tamanho de fonte, duração de animação ou cor escritos à mão no QML, além do congelado em `scripts/qml-tokens-baseline.txt` | use o token do `Theme.qml` (`fontSize*`, `radius*`, `motion*`, `easing*`, cores); a linha de base só desce |
| `verificar-qml-alcance.sh` | componente registrado que nenhuma tela abre | ou ligue, ou remova do CMake e do disco |
| `verificar-fiacao-ipc.sh` | método roteado que nenhum cliente pede; **cliente que pede método que o core não roteia**; evento sem tratador; sinal sem ouvinte; elo de despacho sem chamador | a direção inversa é a perigosa: um botão que não faz nada em tempo de execução |
| `verificar-exercitacao.sh` | o core contra ferramentas reais (git, cmake, cargo…) | o que a máquina não tem vira NÃO PROVADO, com o motivo |
| `verificar-embarcado.sh` | o ciclo de embarcado no QEMU, sem placa | sem `arm-none-eabi-gcc`/`qemu-system-arm`/`gdb`: NÃO PROVADO. **O gdb que vale é o ≥ 16** (decisão do autor, 2026-10-01). Abaixo dele o DAP não tem escopo de globais (medido no 15.1; o 17.2 tem; o anúncio do 16.2 lista a mudança no 16), e o gate prova a **degradação explicada**: o core devolve os registradores com `notice` dizendo por quê (protocolo 0.145.0); a variável global no painel fica NÃO PROVADA |
| `verificar-python-debug.sh` | a porta MicroPython e depurar Python com o debugpy real | sem debugpy (nem `KINEIN_PYTHON_DEBUGPY`): o ciclo de depuração é NÃO PROVADO |
| `verificar-clangd-cross.sh` | o clangd enxerga o cross do kit | sem `clangd`/`arm-none-eabi-g++`: NÃO PROVADO |
| `verificar-atalhos.sh` | dois comandos não declaram o mesmo atalho; todo `default_shortcut` tem `Shortcut` anotado `// comando: <id>` | `Alt+Return` ≠ `Alt+Enter` no Qt |
| `verificar-docs.sh` | contagem de linhas sem data reconhecida nos `.md` que diverge do disco | ponha a data, ou corrija o número |
| `module_map.py --check` | o `arquitetura/01-mapa-de-modulos.md` diverge do código; **ciclo novo** entre módulos do core; aresta Cargo não declarada; evidência de aresta que sumiu; método IPC sem lugar | rode `python3 scripts/module_map.py` e confira o diff. Ciclo novo: desfaça a dependência (ela é o defeito); só um ciclo com motivo entra em `KNOWN_CYCLES`. Contexto novo da IDE é uma entrada em `CONTEXTS`, sem mexer no gerador |
| `verificar-links-docs.sh` | alvo relativo inexistente em Markdown versionado; **citação `DocsPublic/…` inexistente em qualquer arquivo** (código e comentários inclusive, desde 2026-10-01); **documento que não aparece, pelo nome, no `README.md` da sua pasta**; **menção a documento interno do autor** em qualquer arquivo (só o público cita o público); **caminho de código entre crases que não existe**, num documento marcado `<!-- caminhos-conferidos -->` (hoje o roadmap 58); não verifica âncoras/URLs externas | para renomear documento use `scripts/rename_doc.py`, que reescreve link, citação e nome |
| `verificar-arquitetura.sh` | a catraca: Rust 500 (sem testes), view 300, controller/host 400, ui/src 500 | split por responsabilidade; nunca "Part2" |
| `verificar-transicao-workspace.sh` | estado por-workspace com um dono | |
| `verificar-qml-logica.sh` | os harnesses (`tst_*.qml`) num espelho do módulo; **harness verde que imprime aviso de `avisos-qml.txt`** | o bitmask está no `console.error`; aviso na saída quase sempre é um falso incompleto no harness |
| `verificar-binario-abre.sh --preset <p>` | o binário abre, primeiro frame medido, **nenhum aviso QML no stderr** até o frame | um `Connections` no alvo errado só aparece aqui |

## O que o gate assume da máquina

Um gate que só passa na máquina do autor não é gate, é hábito. Esta tabela
diz, para cada premissa de ambiente medida em 2026-10-01 (um Ubuntu 24.04
limpo, Qt 6.4.2, root, sem sessão de login), se o gate já não depende dela ou
se ela ainda é uma pendência aberta.

| Premissa | Antes | Agora |
| --- | --- | --- |
| versão do Python (3.11 vs 3.12+) | G0.1 contava 17.591 no 3.11 e 17.810 no 3.12 | **independente**: AST, mesma contagem no 3.11, 3.12 e 3.13 |
| perfil de login que imprime algo (`/etc/profile.d/*`) | 4 testes de `run` e a porta MicroPython reprovavam | **independente**: o ruído de `sh -lc true` é medido e descontado |
| rodar como root | 2 testes de serial reprovavam | **independente**: o caso de permissão não é pedido como root |
| sessão de login (`XDG_RUNTIME_DIR`) | G0.5 reprovava com aviso do Qt | **independente**: G0.5 e o passeio isolam o diretório |
| plugin SVG do Qt instalado | ícones "Unsupported image format" (o G0.4 pegou) | `instalar-ambiente.sh` instala |
| runtime dos sanitizers do clang | o preset estrito não linkava | `instalar-ambiente.sh` instala |
| qual preset foi configurado (doc manual vs instalador) | `verificar-cpp` reprovava sem o estrito | **independente**: o gate configura o que precisa |
| build de debug compilado | `verificar-qml` reprovava num clone novo | **independente**: o build vem antes, e os tipos são gerados |
| ferramenta de ambiente ausente (QEMU, debugpy, Podman…) | verde calado, ou reprovação | **NÃO PROVADO** listado; `--estrito` reprova |
| versão do qmllint (6.4 vs 6.10) | 6 falsos positivos reprovavam no 6.4 | **decidido (2026-10-01)**: vale o ≥ 6.5; no 6.4, NÃO PROVADO com os achados à vista |
| versão do gdb (15 vs 17) | `verificar-embarcado` reprovava no 15 | **decidido (2026-10-01)**: vale o ≥ 16; no 15, a degradação explicada é provada e a variável global fica NÃO PROVADA |
| headers do Qt 6.4.2 + clang-tidy 18 | um `clang-analyzer-cplusplus.NewDelete` na atribuição de `QPointer` (`window_chrome_controller.cpp`) reprovava o `verificar-cpp` | **provado falso positivo (2026-10-01)**: reproduz só com Qt, some trocando `-isystem` por `-I`, e o padrão roda limpo sob ASan. Exceção estreita no `verificar-cpp.sh` (só esse check, arquivo e ponto do header; qualquer outro achado reprova) que **expira**: quando o clang parar de acusá-lo, o gate pede para apagá-la (40.7 §7.152). **Vale só no clang-tidy em que foi provada (18)**: o 21 não acusa o caso mínimo nem com os headers do Qt 6.4.2, e o "expira" sem essa condição reprovava toda máquina com clang novo; em outra versão o arquivo passa pelo clang-tidy como qualquer outro (40.7 §7.153) |

## O que fazer quando um gate parece "errado"

1. Meça: reproduza o que ele mede à mão (`grep`, o harness, a foto).
2. Se o gate acusou algo real, corrija o código — e registre no `40`.
3. Se o gate acusou algo que não é falha (raro), escreva a exceção **no
   script**, com data e motivo, e a linha no `40`. Uma exceção sem motivo
   é o próximo bug silencioso.
4. Se o gate reprovou por causa da **máquina** e não do código (versão de
   ferramenta, perfil, permissão), isso também é defeito do gate: corrija o
   gate para não depender dela, ou declare a premissa na tabela acima.
5. Se falta um gate (uma falha silenciosa passou), escreva-o: um
   `scripts/verificar_<x>.py` + `.sh`, e **prove por mutação** (quebre de
   propósito, veja o gate reprovar, desfaça). Quando a regra é um parser ou
   uma regex, guarde os casos de mutação **dentro** do gate e rode-os a cada
   execução, como o `check_qml_qt64.py`.
