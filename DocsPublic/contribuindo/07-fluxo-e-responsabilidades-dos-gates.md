# 07 — Fluxo e responsabilidades dos gates

> **Perfil vigente (autor, 2026-10-07):** desenvolvimento/validação usam o
> Qt local do sistema — Arch Linux desde a mesma data, Qt 6.12.0 (antes 6.10).
> O orquestrador executa qmllint, harnesses, CTest e abertura locais;
> `verificar-qml-qt64.sh` e `verificar-qml-logica-qt64.sh` saem da sequência
> obrigatória. Seus scripts/provas ficam históricos para o pacote anterior.
> Check barato de invariantes AppImage continua sem gerar pacote/rodar Qt 6.4.

Este documento descreve **como** os gates se relacionam e qual código é dono
de cada responsabilidade. Para saber o que cada reprovação significa e como
corrigi-la, consulte o catálogo
[`04-os-gates-que-dizem-nao.md`](04-os-gates-que-dizem-nao.md).

## 1. O contrato de comunicação

Os gates-folha **não chamam uns aos outros**. O único orquestrador é
`scripts/verificar.sh`, que os executa em sequência e para na primeira falha.
Isso evita dependências circulares, execuções escondidas e duas ordens de gate
que possam divergir.

```text
pessoa ou CI
      |
      v
scripts/verificar.sh                 dono da ordem, modo e descrição
      |
      +--> comando direto            cargo/cmake
      |
      +--> verificar-*.sh            fronteira executável do gate-folha
                 |
                 +--> verificar_*.py análise auxiliar, quando necessária
                 +--> ferramenta     qmllint, clang-tidy, QEMU, debugpy...
                 +--> fixture        projeto/harness mínimo do próprio gate
      |
      v
exit code + stdout/stderr            único retorno ao orquestrador
```

A comunicação entre o orquestrador e uma etapa tem somente três partes:

1. **Entrada:** argumentos explícitos e, quando documentadas pelo próprio
   script, variáveis `KINEIN_*`.
2. **Resultado:** `0` significa que nenhuma condição obrigatória reprovou;
   valor diferente de zero reprova a execução. Um gate de **ambiente** que
   não tem a ferramenta nesta máquina sai `0` **e registra** o que não provou
   pelo protocolo NÃO PROVADO (§1.1); ele nunca sai `0` calado.
3. **Diagnóstico:** `stdout` informa progresso e evidência; `stderr` explica a
   falha. O `trap` do orquestrador repete o nome exato da etapa que falhou.

Não existe barramento, arquivo de estado ou variável global pelos quais um gate
irmão passe um veredito a outro. O repositório é a fonte comum lida por todos.
Arquivos temporários pertencem ao gate que os criou e devem ser removidos pelo
seu `trap`. A única exceção é declarada e tem um sentido só — §1.1.

### 1.1 O protocolo NÃO PROVADO (desde 2026-10-01)

**O problema que ele resolve.** Até 2026-10-01, gate de ambiente sem a
ferramenta imprimia “não reprova” e saía `0`; o orquestrador só vê códigos de
saída e terminava em “✓ TUDO VERDE”. Medido num Ubuntu 24.04 limpo: embarcado,
depuração Python e clangd-cross passaram em 0 s sem rodar. O G0.3 fazia o
oposto — sem Podman/Docker reprovava —, falso positivo para quem clona numa
máquina sem container.

**A regra que decide qual comportamento um gate tem:**

| O gate verifica… | Ferramenta ausente | Exemplos |
| --- | --- | --- |
| o **repositório** (o código como texto ou compilado) | **reprova**, com o comando que instala | clang-tidy, clang-format, qmllint, shellcheck, cargo-deny |
| a **integração com um ambiente** externo | **NÃO PROVADO**: sai `0` e registra | QEMU/gdb (embarcado), debugpy, kit cross (clangd-cross), Podman/Docker (G0.3), os itens opcionais da exercitação |

**O mecanismo.** O orquestrador cria um arquivo temporário e exporta
`KINEIN_UNPROVEN_FILE`. O gate chama `record()` de `scripts/unproven.py` (ou
`record_unproven` de `scripts/unproven.sh`): a linha sai no stdout com o
prefixo fixo `NAO PROVADO` e é acrescentada ao arquivo como
`gate<TAB>o que<TAB>por que`. Nenhum gate **lê** o arquivo; só o orquestrador,
no fim:

```text
lista vazia            ✓ TUDO VERDE (modo)
lista não vazia        == NAO PROVADO nesta maquina (N) ==  + cada item
                       ✓ VERDE no que esta maquina prova (modo)  — sai 0
  com --estrito        ✗ FALHOU em: --estrito: N item(ns) NAO PROVADO(S) — sai 1
```

Rode `--estrito` antes de release e em CI: ali, “não provado” é uma pendência,
não um detalhe da máquina.

## 2. O que o orquestrador possui

`scripts/verificar.sh` é o único dono de:

- ordem e política *fail-fast*;
- seleção entre `--rapido` e `--completo`, e o `--estrito` (§1.1), que
  combina com os dois;
- nome e descrição curta de cada etapa;
- presets de build escolhidos por `KINEIN_PRESET_DEBUG` e
  `KINEIN_PRESET_RELEASE`;
- a lista NÃO PROVADO, a mensagem final (“tudo verde” só com a lista vazia)
  e a identificação da etapa que falhou;
- o build de debug **antes** dos gates que leem o build, e a variável
  `KINEIN_QML_BUILD_DIR` que diz a eles qual build é esse.

Cada chamada a `passo` exige **nome e função**. A expansão `${2:?...}` rejeita
descrição ausente ou vazia, em vez de deixar a etapa muda no log:

```bash
passo "scripts/verificar-exemplo.sh" \
    "Explica em uma frase qual propriedade este gate prova."
bash scripts/verificar-exemplo.sh
```

O orquestrador não deve conter a implementação de uma análise extensa. Se a
regra tem lógica própria, ela ganha um gate-folha; o `verificar.sh` conserva
somente a chamada e a descrição.

## 3. Responsabilidade do código de cada etapa

“Dono” abaixo significa o lugar que deve mudar quando a **regra de verificação**
muda. O código do produto que um gate inspeciona continua em seu domínio normal.

| Etapa | Dono da verificação | Responsabilidade exclusiva |
| --- | --- | --- |
| `cargo fmt --all --check` | configuração `rustfmt` + chamada no orquestrador | Formatação Rust, sem reescrever arquivos. |
| `cargo test --workspace --all-features` | testes nos crates + chamada no orquestrador | Comportamento Rust isolado e integrações codificadas nos testes. |
| `cargo clippy ... -D warnings` | lints dos crates + chamada no orquestrador | Diagnósticos estáticos Rust em todos os alvos. |
| `verificar-deny.sh` | `scripts/verificar-deny.sh` + `deny.toml` | Licenças, advisories, origens, bans e duplicações de dependências Rust. |
| `verificar-shell.sh` | `scripts/verificar-shell.sh` | Portabilidade e defeitos estáticos dos scripts shell, inclusive dos gates. |
| `verificar-appimage.sh` | `scripts/verificar-appimage.sh` | Invariantes baratas da distribuição; não gera o AppImage. |
| `verificar-cpp.sh` | `scripts/verificar-cpp.sh` + `.clang-tidy`/formatação | Formatação e análise estática da ponte C++/Qt, com o banco de compilação do preset clang `linux-clang-debug-strict` — que ele configura sozinho quando falta ou é mais velho que uma entrada do CMake. |
| `verificar-cpp-testes.sh` | `scripts/verificar-cpp-testes.sh`, `ui/tests/` e a biblioteca `kinein-ui-puro` | Testes C++ (Qt Test + CTest) das unidades que nao dependem de janela. |
| `verificar-qml.sh` | `scripts/verificar-qml.sh` e `scripts/verificar_qml.py` | `qmllint` estrito no contexto tipado do build que o orquestrador indicar (`KINEIN_QML_BUILD_DIR`); atualiza a cópia do QML e os tipos antes de lintar e recusa `.qml` fora do módulo. |
| `check_identifier_language.py` (G0.1) | o script + `identifier-language-allowlist.txt` + `identifier-language-baseline.txt` | Identificador novo em português; nomes Python pela AST, com o mesmo resultado em qualquer Python ≥ 3.8. |
| `verificar-qml-qt64.sh` (G0.2) | wrapper `.sh` + `scripts/check_qml_qt64.py` | Parte que o Qt 6.4 não cria num arquivo Bound, inclusive na sintaxe agrupada; roda os próprios casos de mutação antes da árvore. |
| `verificar-qml-fiacao.sh` | `scripts/verificar-qml-fiacao.sh` | Binding QML auto-referente. |
| `verificar-qml-propriedades.sh` | wrapper `.sh` + `scripts/verificar_qml_propriedades.py` | Propriedade/sinal inexistente e bindings estruturalmente tortos. |
| `verificar-qml-duplicacao.sh` | wrapper `.sh` + `scripts/verificar_qml_duplicacao.py` | Catraca de regra derivada duplicada entre arquivos QML. |
| `verificar-qml-tokens.sh` | wrapper `.sh` + `scripts/verificar_qml_tokens.py` | Catraca de valor literal (raio, fonte, duração, cor) no QML, por arquivo e categoria. |
| `verificar-qml-alcance.sh` | wrapper `.sh` + `scripts/verificar_qml_alcance.py` | Componente registrado no módulo mas inalcançável por qualquer tela. |
| `verificar-fiacao-ipc.sh` | wrapper `.sh` + `scripts/verificar_fiacao_ipc.py` | Cadeia método/evento/sinal/dispatcher/consumidor de ponta a ponta. |
| `verificar-exercitacao.sh` | `scripts/verificar-exercitacao.sh` e suas fixtures | Core real contra ferramentas externas disponíveis. |
| `verificar-embarcado.sh` | wrapper `.sh` + `scripts/verificar_embarcado.py` + fixture embarcada | Ciclo DAP de embarcado no QEMU, sem placa física. |
| `verificar-python-debug.sh` | wrapper `.sh` + `verificar_micropython_porta.py` + `verificar_python_debug.py`/`verificar_python_attach.py` | Propagação da porta MicroPython e ciclos launch/attach DAP com `debugpy` real. |
| `verificar-clangd-cross.sh` | wrapper `.sh` + `scripts/verificar_clangd_cross.py` | Resolução dos cabeçalhos C++ do compilador cross pelo `clangd`. |
| `verificar-atalhos.sh` | wrapper `.sh` + `scripts/verificar_atalhos.py` | Coerência entre atalhos declarados, paleta, menus e tratamento no host. |
| `verificar-docs.sh` | `scripts/verificar-docs.sh` | Confere contagens de linhas reconhecidas por padrões textuais; não prova todos os números nem a semântica dos documentos. |
| `module_map.py --check` | `scripts/module_map.py` (`EDGES`, `CONTEXTS`, `KNOWN_CYCLES`) | O mapa de módulos gerado confere com o código; nenhum ciclo novo no core; todo método IPC classificado. Reusa o extrator do `verificar_fiacao_ipc.py` (dono único). |
| `verificar-links-docs.sh` | `scripts/verificar-links-docs.sh` | Existência dos alvos relativos reconhecidos em Markdown versionado e novo não ignorado; citações `DocsPublic/…` em qualquer arquivo; todo documento no índice da pasta; nenhuma menção a documento interno; caminhos de código nos documentos marcados `caminhos-conferidos`. Não verifica URLs externas nem âncoras. |
| `verificar-arquitetura.sh` | `scripts/verificar-arquitetura.sh` + `arquitetura-baseline.txt` | Catraca de tamanho por categoria de arquivo; responsabilidade semântica ainda exige revisão. |
| `verificar-transicao-workspace.sh` | `scripts/verificar-transicao-workspace.sh` | Dono único da mutação de estado por workspace no core. |
| `verificar-qml-logica.sh` | script `.sh` + `scripts/qml-harness/tst_*.qml` | Lógica QML real em espelho temporário do módulo e modo headless; reprova também harness verde cuja saída tem aviso de `avisos-qml.txt`. |
| `verificar-qml-logica-qt64.sh` | script `.sh` + `packaging/appimage/Containerfile.qml64` | Os mesmos harnesses com o runner do Qt 6.4.2 do AppImage; sem engine (ou engine sem serviço), NÃO PROVADO. |
| `check_terminal_quiet.py` | script `.py` (pty, XDG isolados, inclusive `XDG_RUNTIME_DIR`) | `kinein <pasta>` desacopla como `code .`: < 300 ms, mudo, e o processo desacoplado **prova** que chegou ao primeiro frame (`KINEIN_PERF_MARKER_FILE`). |
| `unproven.py` / `unproven.sh` | os dois arquivos | O protocolo NÃO PROVADO (§1.1); não é gate, é a única forma de um gate de ambiente dizer o que não provou. |
| `run-surface-tour.sh` | script `.sh` + `scripts/surface-tour.txt` | Passeio por superfícies com o dono de cada aviso; chamado pelo `verificar-binario-abre` e pelo `testar-appimage`. |
| builds debug/release | chamadas `cmake`/`cargo` no orquestrador e presets do projeto | Produzir os binários dos presets selecionados; confirmar separadamente que o launcher usa esses caminhos. |
| `verificar-binario-abre.sh` | wrapper `.sh` + `scripts/verificar_binario_abre.py` | Abrir o binário recém-produzido, atingir o primeiro frame e rejeitar avisos QML. |

O wrapper `.sh` é a interface pública de um gate-folha: prepara ambiente,
verifica pré-condições e apresenta o resultado. Um auxiliar `.py` concentra a
análise quando fazê-la em shell seria frágil. O auxiliar não vira um segundo
gate e não deve ser chamado diretamente pelo orquestrador.

## 4. Artefatos compartilhados não são mensagens escondidas

Algumas etapas consomem artefatos reais do build, mas isso é uma pré-condição
declarada, não comunicação entre gates:

- `verificar-cpp.sh` lê `compile_commands.json` do preset
  `linux-clang-debug-strict`. Tem de ser um banco de **clang**: o clang-tidy
  interpreta as flags como clang, e o `dev-local` usa `c++` (o GCC no Arch e
  no Debian) — com ele o clang-tidy reprova em `-Wlogical-op`,
  `-Wuseless-cast`, `-Wtrampolines` e perderia as diagnósticas que só o clang
  tem (`-Wcomma`, `-Wheader-hygiene`). Desde 2026-10-01 o gate configura o
  preset sozinho quando o banco falta ou é mais velho que qualquer
  `CMakeLists.txt`/`*.cmake`/preset, e diz que configurou; não precisa
  compilar;
- `scripts/testar-remote-ssh.sh` **não** é gate: é a prova pesada do Remote
  contra um sshd de verdade (podman; na primeira vez, rede). Rode-a de propósito
  ao mexer em `remote.*` — foi ela que achou o primeiro deploy quebrado em alvo
  novo e a divergência de config entre descoberta e resolução, duas coisas que o
  `ssh` falso do gate não tinha como mostrar. Mesma separação do AppImage:
  `verificar-*` é a catraca hermética, `testar-*` é a prova cara.
- `verificar-cpp-testes.sh` é o **primeiro** gate que mede o que o C++ decide.
  Até 2026-09-24 o projeto media Rust e QML e não media C++ — sem decisão
  registrada justificando, e com o `verificar-exercitacao.sh` anotando "zero
  testes declarados" ao rodar o `ctest` real contra este próprio repositório. Ele
  recusa se nenhum teste for declarado, porque `ctest` com zero testes sai 0 e o
  gate passaria vazio. Só entra na biblioteca `kinein-ui-puro` o que é testável
  sem janela: as 34 fontes do módulo QML ficam de fora porque seis cabeçalhos
  registram tipo QML (`QML_ELEMENT`, inclusive o `CoreClient`), e tirá-los do
  `qt_add_qml_module` quebraria o registro. A fronteira cresce por unidade.
- `verificar-fiacao-ipc.sh` acha método roteado pelo formato do braço de
  `match` (`"dominio.metodo" => ...`) nos roteadores do core. Um roteador escrito
  de outro jeito continua funcionando e **some** da contagem e da lista canônica
  do `03-protocolo-ipc` sem nada reprovar — foi o que aconteceu em 2026-09-24 ao
  extrair o `remote.command` para arquivo próprio (167 → 166, percebido só porque
  alguém olhou o número). Desde então ele confere também a direção inversa: o
  cliente pedir um método que o core não roteia, que é um botão que não faz nada
  em tempo de execução. Ao mover um roteador, mantenha o braço de `match`.
- `verificar-qml.sh` lê as **cópias** do QML no diretório de build, não a
  árvore. Até 2026-09-24 ele podia reprovar à toa numa propriedade nova — e,
  pior, **passar lintando QML velho**, que é o gate afirmando verde sobre código
  que não é o do commit. Agora ele refaz a cópia (`kinein-vectis_copy_qml`), diz
  que refez, e compara conteúdo para provar que leu a árvore de agora. Desde
  2026-10-01 o orquestrador compila o debug **antes** dele e indica o build por
  `KINEIN_QML_BUILD_DIR`; um build configurado e nunca compilado (o clone que
  seguiu a documentação) ganha os tipos gerados, em vez de reprovar. Cópia é
  cache, então refazê-la é preparo; o que ele **recusa** é o defeito de verdade:
  um `.qml` na árvore fora do `ui/CMakeLists.txt`, que ninguém compila nem linta.
  A lista de débito declarado está **vazia** desde 2026-09-25: o único morador
  (`EditorUnsavedChangesDialog.qml`, na árvore desde a fundação e sem uma única
  referência) foi descartado pelo autor. Entrar no módulo ou sair da árvore é
  decisão dele, não do gate — arquivo novo nessa situação reprova.
- o **primeiro** passo confere que `build/<preset-debug>` e
  `build/<preset-release>` existem. Custa milissegundos e existe porque em
  2026-09-25 o gate reprovou com "build/dev-local-release is not a directory"
  **depois de 25 minutos** de clang-tidy, testes e harnesses: o preset estava no
  `CMakeUserPresets.json`, o diretório é que nunca tinha sido configurado
  naquela máquina. A mensagem diz o `cmake --preset` que resolve;
- `verificar-qml.sh` lê `.qmltypes`, `qmldir` e a configuração de imports do
  módulo compilado;
- o build debug roda logo depois da checagem de diretórios, **nos dois modos**
  (incremental: segundos quando nada mudou); no modo completo, o smoke usa o
  **mesmo** preset;
- o core release é produzido antes da UI release, que é seguida pelo smoke do
  **mesmo** preset release.

O modo rápido exige ambiente preparado e omite o build **release** e os
smokes, mas não é “sem compilação”: a UI de debug compila antes dos lints,
testes Rust compilam seus alvos, exercitações podem construir o core e o gate
QML pode atualizar artefatos via CMake. O modo completo é o gate de entrega dos presets escolhidos. Core e UI
são processos separados; não há link da UI contra o executável Rust.

Mesmo um gate completo verde não substitui testar o launcher do desktop,
interação visual, SSH real ou hardware. O que a máquina não provou aparece na
lista NÃO PROVADO do fim (§1.1); qual binário foi aberto aparece no smoke.

## 5. Como estender sem duplicar

Ao surgir uma classe de falha que nenhum gate atual detecta:

1. confirme primeiro se a responsabilidade cabe num gate existente;
2. estenda esse gate quando a propriedade verificada for a mesma;
3. crie outro gate somente para uma propriedade realmente diferente;
4. mantenha uma única fronteira `.sh`, com auxiliar interno se necessário;
5. registre nome e função no orquestrador, e diagnóstico no catálogo `04`;
6. prove por mutação que o gate reprova o defeito e aceita o código correto.

Um gate não deve chamar um irmão para “reaproveitar o fluxo”. Reuso de análise
vai para uma função ou auxiliar comum; composição de execução continua sendo
responsabilidade exclusiva do `scripts/verificar.sh`. É a aplicação do
princípio aberto/fechado ao próprio sistema de qualidade: acrescentar uma
prova sem criar um segundo orquestrador nem alterar o significado das provas
existentes.
