# 07 — Fluxo e responsabilidades dos gates

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
   valor diferente de zero reprova a execução. Alguns gates permitem
   dependências opcionais ausentes ou cenários inconclusivos com retorno `0`
   e mensagem explícita. É preciso ler o diagnóstico: “pulou/não provado”
   nunca equivale a evidência de funcionamento.
3. **Diagnóstico:** `stdout` informa progresso e evidência; `stderr` explica a
   falha. O `trap` do orquestrador repete o nome exato da etapa que falhou.

Não existe barramento, arquivo de estado ou variável global pelos quais um gate
irmão passe um veredito a outro. O repositório é a fonte comum lida por todos.
Arquivos temporários pertencem ao gate que os criou e devem ser removidos pelo
seu `trap`.

## 2. O que o orquestrador possui

`scripts/verificar.sh` é o único dono de:

- ordem e política *fail-fast*;
- seleção entre `--rapido` e `--completo`;
- nome e descrição curta de cada etapa;
- presets de build escolhidos por `KINEIN_PRESET_DEBUG` e
  `KINEIN_PRESET_RELEASE`;
- mensagem final “tudo verde” e identificação da etapa que falhou.

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
| `verificar-cpp.sh` | `scripts/verificar-cpp.sh` + `.clang-tidy`/formatação | Formatação e análise estática da ponte C++/Qt. |
| `verificar-qml.sh` | `scripts/verificar-qml.sh` e `scripts/verificar_qml.py` | `qmllint` estrito no contexto tipado produzido pelo build; atualiza a cópia do QML antes de lintar e recusa `.qml` fora do módulo. |
| `verificar-qml-fiacao.sh` | `scripts/verificar-qml-fiacao.sh` | Binding QML auto-referente. |
| `verificar-qml-propriedades.sh` | wrapper `.sh` + `scripts/verificar_qml_propriedades.py` | Propriedade/sinal inexistente e bindings estruturalmente tortos. |
| `verificar-qml-duplicacao.sh` | wrapper `.sh` + `scripts/verificar_qml_duplicacao.py` | Catraca de regra derivada duplicada entre arquivos QML. |
| `verificar-qml-alcance.sh` | wrapper `.sh` + `scripts/verificar_qml_alcance.py` | Componente registrado no módulo mas inalcançável por qualquer tela. |
| `verificar-fiacao-ipc.sh` | wrapper `.sh` + `scripts/verificar_fiacao_ipc.py` | Cadeia método/evento/sinal/dispatcher/consumidor de ponta a ponta. |
| `verificar-exercitacao.sh` | `scripts/verificar-exercitacao.sh` e suas fixtures | Core real contra ferramentas externas disponíveis. |
| `verificar-embarcado.sh` | wrapper `.sh` + `scripts/verificar_embarcado.py` + fixture embarcada | Ciclo DAP de embarcado no QEMU, sem placa física. |
| `verificar-python-debug.sh` | wrapper `.sh` + `verificar_micropython_porta.py` + `verificar_python_debug.py`/`verificar_python_attach.py` | Propagação da porta MicroPython e ciclos launch/attach DAP com `debugpy` real. |
| `verificar-clangd-cross.sh` | wrapper `.sh` + `scripts/verificar_clangd_cross.py` | Resolução dos cabeçalhos C++ do compilador cross pelo `clangd`. |
| `verificar-atalhos.sh` | wrapper `.sh` + `scripts/verificar_atalhos.py` | Coerência entre atalhos declarados, paleta, menus e tratamento no host. |
| `verificar-docs.sh` | `scripts/verificar-docs.sh` | Confere contagens de linhas reconhecidas por padrões textuais; não prova todos os números nem a semântica dos documentos. |
| `verificar-links-docs.sh` | `scripts/verificar-links-docs.sh` | Existência dos alvos relativos reconhecidos em Markdown versionado e novo não ignorado; não verifica URLs externas nem âncoras. |
| `verificar-arquitetura.sh` | `scripts/verificar-arquitetura.sh` + `arquitetura-baseline.txt` | Catraca de tamanho por categoria de arquivo; responsabilidade semântica ainda exige revisão. |
| `verificar-transicao-workspace.sh` | `scripts/verificar-transicao-workspace.sh` | Dono único da mutação de estado por workspace no core. |
| `verificar-qml-logica.sh` | script `.sh` + `scripts/qml-harness/tst_*.qml` | Lógica QML real em espelho temporário do módulo e modo headless. |
| builds debug/release | chamadas `cmake`/`cargo` no orquestrador e presets do projeto | Produzir os binários dos presets selecionados; confirmar separadamente que o launcher usa esses caminhos. |
| `verificar-binario-abre.sh` | wrapper `.sh` + `scripts/verificar_binario_abre.py` | Abrir o binário recém-produzido, atingir o primeiro frame e rejeitar avisos QML. |

O wrapper `.sh` é a interface pública de um gate-folha: prepara ambiente,
verifica pré-condições e apresenta o resultado. Um auxiliar `.py` concentra a
análise quando fazê-la em shell seria frágil. O auxiliar não vira um segundo
gate e não deve ser chamado diretamente pelo orquestrador.

## 4. Artefatos compartilhados não são mensagens escondidas

Algumas etapas consomem artefatos reais do build, mas isso é uma pré-condição
declarada, não comunicação entre gates:

- `verificar-cpp.sh` lê `compile_commands.json` do build debug estrito;
- `scripts/testar-remote-ssh.sh` **não** é gate: é a prova pesada do Remote
  contra um sshd de verdade (podman; na primeira vez, rede). Rode-a de propósito
  ao mexer em `remote.*` — foi ela que achou o primeiro deploy quebrado em alvo
  novo e a divergência de config entre descoberta e resolução, duas coisas que o
  `ssh` falso do gate não tinha como mostrar. Mesma separação do AppImage:
  `verificar-*` é a catraca hermética, `testar-*` é a prova cara.
- `verificar-fiacao-ipc.sh` acha método roteado pelo formato do braço de
  `match` (`"dominio.metodo" => ...`) nos roteadores do core. Um roteador escrito
  de outro jeito continua funcionando e **some** da contagem e da lista canônica
  do `03-ipc-protocol` sem nada reprovar — foi o que aconteceu em 2026-09-24 ao
  extrair o `remote.command` para arquivo próprio (167 → 166, percebido só porque
  alguém olhou o número). Desde então ele confere também a direção inversa: o
  cliente pedir um método que o core não roteia, que é um botão que não faz nada
  em tempo de execução. Ao mover um roteador, mantenha o braço de `match`.
- `verificar-qml.sh` lê as **cópias** do QML no diretório de build, não a
  árvore. Como o `verificar.sh` o roda antes do build que atualiza essas cópias,
  até 2026-09-24 ele podia reprovar à toa numa propriedade nova — e, pior,
  **passar lintando QML velho**, que é o gate afirmando verde sobre código que
  não é o do commit. Agora ele refaz a cópia (`kinein-vectis_copy_qml`), diz que
  refez, e compara conteúdo para provar que leu a árvore de agora. Cópia é
  cache, então refazê-la é preparo; o que ele **recusa** é o defeito de verdade:
  um `.qml` na árvore fora do `ui/CMakeLists.txt`, que ninguém compila nem linta.
  Há **um débito declarado** no script (`EditorUnsavedChangesDialog.qml`, sem
  referência em todo o repo): entrar no módulo ou sair da árvore é decisão do
  autor, não do gate — arquivo novo nessa situação reprova.
- `verificar-qml.sh` lê `.qmltypes`, `qmldir` e a configuração de imports do
  módulo compilado;
- no modo completo, o build debug é seguido pelo smoke do **mesmo** preset;
- o core release é produzido antes da UI release, que é seguida pelo smoke do
  **mesmo** preset release.

O modo rápido exige ambiente preparado e omite os builds **finais** e os dois
smokes finais, mas não é “sem compilação”: testes Rust compilam seus alvos,
exercitações podem construir o core e o gate QML pode atualizar artefatos via
CMake. O modo completo é o gate de entrega dos presets escolhidos. Core e UI
são processos separados; não há link da UI contra o executável Rust.

Mesmo um gate completo verde não substitui testar o launcher do desktop,
interação visual, SSH real ou hardware. A saída deve registrar o que foi
exercitado, o que foi pulado e qual binário foi aberto.

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
