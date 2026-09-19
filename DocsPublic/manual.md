# Manual do Kinein Vectis

Guia completo para usar e testar a IDE depois que ela estiver aberta. Este
documento cobre os fluxos, recursos, atalhos e problemas encontrados durante o
uso.

> **O que é:** Kinein Vectis é uma IDE para C, C++, Rust, Python e sistemas
> embarcados. Ela orquestra ferramentas maduras (clangd, rust-analyzer,
> basedpyright, cargo, CMake, pytest, debugpy, clang-format, ruff...) em vez
> de reimplementá-las. Os fluxos da IDE
> são locais e não têm telemetria; somente uma CLI externa iniciada
> explicitamente por você no terminal pode usar rede conforme a política dela.
>
> **Estado atual:** em desenvolvimento ativo. A interface já segue o sistema
> visual Kinein (App Bar, toolbar principal, ícones vetoriais e Start Screen);
> a validação visual final em hardware real continua pendente.

---

## 1. Primeiros passos

1. A IDE abre na **Start Screen**, que mostra o estado das ferramentas sem
   instalar ou alterar nada automaticamente. Clique em **Abrir workspace**
   (ou `Ctrl+O`).
2. Navegue até a pasta do seu projeto (qualquer projeto com `Cargo.toml` ou
   `CMakeLists.txt` é detectado automaticamente) e confirme.
3. A árvore de arquivos aparece à esquerda. Clique num arquivo para editar.

Se a raiz tiver `Cargo.toml` **e** `CMakeLists.txt`, a Kinein reconhece o
workspace híbrido: menus e barra superior oferecem Build/Teste Cargo e CMake
separadamente, e o Project Health verifica as duas toolchains.
4. Ao reabrir o mesmo projeto depois, **suas abas voltam como estavam**
   (sessão automática).

Depois da primeira abertura bem-sucedida, o projeto aparece em **Workspaces
recentes** na Start Screen e em **Arquivo → Abrir recente**. Um clique reabre o
workspace e restaura suas abas. Na Start Screen também é possível **fixar** os
projetos mais importantes no topo, remover uma entrada ou limpar toda a lista.
Se uma pasta foi movida ou apagada, ela aparece como **caminho ausente**, fica
desabilitada e pode ser removida com segurança.

Também dá para **criar** um projeto novo (Rust/Cargo ou C++/CMake) pela Start
Screen. O seletor mostra antes os arquivos e comandos que serão usados. O
template C++ é C++23 target-based, com Ninja, presets Debug/Release,
`src/`, `include/`, `tests/` e `.gitignore`; o Rust delega a criação ao Cargo.

Um banner discreto de **Project Health** aparece acima do editor quando
algo do ambiente merece atenção (ex.: ferramentas faltando para o tipo do
projeto, ou projeto CMake ainda sem configure), com botão para
verificar/resolver. Quando está tudo bem, ele some.

**Projetos CMake:** ao abrir um projeto CMake ainda não configurado, a
IDE roda o configure **sozinha** (aviso discreto acima do editor;
progresso na aba **Jobs**) — é o passo que gera o build system e o
`compile_commands.json` que deixa a análise do clangd precisa. Salvar o
`CMakeLists.txt`/`CMakePresets.json` pela IDE reconfigura
automaticamente. O botão **[Configurar CMake]** da barra superior segue
existindo para reconfigurar na mão (ex.: mudou algo por fora). Depois do
primeiro configure, reabra o arquivo (ou o projeto) para o clangd usar
os flags reais. Projetos Rust não têm esse passo (o cargo se vira).

---

**Abrir direto num projeto** (desde 2026-09-18): `kinein-vectis /caminho/do/projeto`
abre a pasta sem passar pela tela inicial — serve para um lançador, um
atalho de área de trabalho ou um `alias`. Com `KINEIN_STARTUP_COMMANDS=build.run`
(ids da paleta, separados por vírgula) a IDE executa os comandos logo depois
de abrir — é o que os testes headless usam para fotografar um estado.

## 2. O layout

```text
┌──────────────────────────────────────────────────────────────┐
│ App Bar: Arquivo · Editar · Exibir · Navegar · Código ...    │
│ Barra: [Projeto ▾] [⎇ git]           [config ▾] ▶ 🐞 [⋯]   │
├───┬───────────────┬─────────────────────────────┬────────────┤
│ R │ Projeto OU    │ Editor (abas + código;      │ Símbolos   │
│ a │ Git (Commit/  │  o diff/commit do Git abre  │ (estrutura │
│ i │ Log, em pé)   │  aqui como visualização)    │  + busca)  │
│ l │               │                             │            │
├───┴───────────────┴─────────────────────────────┴────────────┤
│ Painel inferior: Build | Jobs | Problemas | Testes |         │
│                  Terminal | Debug | Busca | IDE |            │
│                  Ferramentas                                 │
├──────────────────────────────────────────────────────────────┤
│ Status: projeto · toolchain │ job em curso ▬▬ ✕ │ LSP · IDE · core│
└──────────────────────────────────────────────────────────────┘
```

- **Barra principal** (desde 2026-09-18, Etapa 2 F1): **três widgets**, como
  nas IDEs JetBrains. *Projeto* — o nome do workspace, o que ele é (Cargo +
  CMake) e o ponto do core; o clique abre recentes, abrir e fechar. *Git* —
  a branch, ↑↓ e o contador de alterações; o clique abre a janela do Git.
  *Executar* — a configuração ativa (▾ troca), **▶ Rodar**, **🐞 Depurar** e
  o menu **⋯** com Compilar/Testar/Análise/Cobertura/Configurar de cada
  sistema que o projeto tem, com rótulo (antes eram dois pares de botões
  iguais sem rótulo). Um ponto pulsa no ⋯ enquanto um build, teste ou
  análise roda.
- **Barra de status** (desde 2026-09-18, Etapa 2 F2): diz **o que está
  acontecendo**. À esquerda o projeto e a toolchain; no centro o **job em
  curso** — build, testes, índice, configure, deploy — com o título, uma
  barra de progresso (que anda sozinha quando o job não mede), a última
  linha da saída e o ✕ para cancelar (clicar no título abre a aba Jobs);
  sem job, os resumos do projeto. À direita, os **servidores de linguagem**
  (`LSP ● 2` todos rodando, `LSP … cpp` subindo, `LSP ✗ python` caiu — o
  motivo ao pairar; antes isso só existia no log da aba IDE), o botão IDE e
  o core.
- **Editor e explorer** (desde 2026-09-18, Etapa 2 F3): a **aba ativa** tem
  o fundo do editor e uma borda de acento em cima; o arquivo modificado
  mostra **●** no lugar do ✕ (o ✕ volta ao pairar); o número da **linha
  atual** fica em destaque na calha; o **explorer segue o arquivo ativo**
  (abre as pastas até ele e o seleciona). O botão "Salvar" saiu: **Ctrl+S**
  ou o **salvar automático** — ligado por padrão: 2 s depois de você parar
  de digitar, ao trocar de aba e ao sair do editor; um arquivo mudado por
  fora nunca é sobrescrito (a IDE compara antes de salvar e avisa); o
  rascunho de segurança continua gravando entre um salvar e outro. Desliga
  em Configurações → "Salvar automaticamente".
- **Explorer** (desde 2026-09-18, Etapa 2 F4): as pastas do projeto vêm
  primeiro; as da máquina (`.git`, `.idea`, `.kinein`, `build`, `target`,
  `dist`, `.venv`, `node_modules`…) ficam em cinza e depois delas; os
  arquivos por último. O que o projeto é (Cargo + CMake) está no widget de
  projeto da barra principal, não mais num chip no explorer.
- **Problemas com o próximo passo** (desde 2026-09-18, Etapa 2 F5): cada
  problema mostra, à direita, o que fazer com ele — **Ações** (o Alt+Enter,
  quando o servidor de linguagem oferece correção), **Configurar CMake**
  (quando a mensagem diz que falta a `compile_commands.json`) ou
  **Ferramentas** (quando falta um programa). O que não tem passo conhecido
  fica só com o clique que abre a linha. A aba mostra a contagem.
- **Rail** (coluna fininha à esquerda; o `›` do pé expande com os
  rótulos): **Projeto · Git · Embarcados · Banco · Containers · Grafana ·
  Ferramentas**. Projeto e Git alternam no mesmo slot à esquerda; os
  outros abrem o painel de cada um. Busca, Build e Debug **não estão** no
  rail (desde a Etapa 3): a busca no projeto é `Ctrl+Shift+F` / a aba
  Busca; Build e Debug são o widget Executar do cabeçalho (▶ 🐞 ⋯), o
  menu Build e o painel inferior.
- Clicar numa aba do painel inferior que já está aberta **recolhe** o
  painel.
- Os painéis laterais e o painel inferior são **redimensionáveis**: arraste
  a borda entre eles e o editor.
- A aba **Símbolos** (borda direita do editor; `Alt+7`) nasce **recolhida**:
  uma aba estreita no centro da borda direita; clicar nela (ou `Alt+7`) abre
  o painel. Sem texto no campo, ele mostra a **estrutura do arquivo** aberto;
  com texto, busca **declarações por nome no projeto inteiro** (o índice da
  IDE, sem esperar language server) — primeiro as da **pasta do arquivo
  aberto**, depois as do projeto — e `Enter`/clique abre `arquivo:linha`.
  O painel pode ser redimensionado e recolhido pelo botão próprio; a escolha
  fica salva e **não muda sozinha** com a largura da janela.
- **Ajuda → Manual da IDE** abre este mesmo `DocsPublic/manual.md` numa visualização
  Markdown renderizada dentro da Kinein; não abre editor externo nem mantém
  uma segunda documentação divergente.

---

## 3. Editor

### 3.1 Edição

| Atalho | Ação |
| --- | --- |
| `Ctrl+S` | Salvar |
| `Ctrl+Shift+S` | **Salvar tudo** (respeita format-on-save, em fila) |
| `Ctrl+D` | Duplicar linha atual ou seleção |
| `Alt+Shift+↑` / `↓` | Mover linha (ou seleção) para cima/baixo |
| `Ctrl+/` | Comentar/descomentar linha ou seleção |
| `Ctrl+Y` | Deletar linha atual |
| `Ctrl+G` | Ir para linha (aceita `linha` ou `linha:coluna`) |
| `Ctrl+F` | **Localizar no arquivo** (abre a barra de busca) |
| `Ctrl+H` | **Substituir no arquivo** (busca + substituição) |
| `F3` ou `Ctrl+Alt+G` | Próxima ocorrência |
| `Shift+F3` ou `Ctrl+Alt+Shift+G` | Ocorrência anterior |
| `Ctrl+W` / `Ctrl+Shift+W` | Expandir / encolher seleção (palavra → parênteses → linha → bloco → arquivo) |
| `Home` / `Shift+Home` | Início do texto da linha ↔ coluna 0 (alterna; com `Shift`, seleciona) |
| `Tab` / `Shift+Tab` | Indentar / desindentar seleção |
| `Ctrl+Alt+L` | **Formatar o arquivo** (rustfmt/clang-format do projeto) |

Ao digitar `(`, `[`, `{`, `"` ou `'`, o par fecha sozinho com o cursor no
meio; digitar o fechador "pula por cima" do que já existe; `Backspace`
entre um par vazio apaga os dois; e digitar um abridor com texto
**selecionado** envolve a seleção (ex.: selecionar `nome` e digitar `"`
vira `"nome"`). Aspas coladas em palavras não duplicam (para não quebrar
apóstrofos).

`Enter` preserva a indentação da linha. Entre `{` e `}`, ele abre uma linha
interna indentada e deixa o `}` na linha seguinte; em comentários iniciados
por `//` ou dentro de `/* ... */`, continua o prefixo do comentário.

Digitar `}` numa linha vazia (só espaços) alinha o fechador sozinho com a
linha do `{` correspondente — fechar um bloco à mão não exige acertar a
indentação.

**Rede de segurança contra perda de dado.** Suas edições não salvas são
autosalvas localmente enquanto você digita (num banco `SQLite` em
`.kinein/kinein.db`, sem rede). Se a IDE fechar de forma abrupta (crash,
queda de energia) **antes de você salvar**, ao reabrir o projeto os
arquivos afetados voltam com as alterações **marcadas como não salvas**
(aba modificada) — é só conferir e salvar (`Ctrl+S`) ou reverter. Salvar
ou fechar a aba normalmente limpa o rascunho. Além disso, o salvamento em
disco é **atômico**: uma interrupção no meio de um `Ctrl+S` nunca deixa o
arquivo truncado/zerado — ele fica com o conteúdo antigo ou o novo, nunca
pela metade.

Se Git, um formatter ou outro editor mudar no disco um arquivo aberto, a IDE
detecta automaticamente. Uma aba sem edições locais é recarregada preservando
o cursor. Se também houver mudanças locais, aparece um aviso sem descartar o
buffer: **Recarregar do disco** aceita a versão externa; **Manter local** torna
a versão externa a nova base e permite um save explícito por cima dela. Mesmo
se o monitor do sistema falhar, `Ctrl+S` compara o snapshot e recusa
sobrescrever silenciosamente um arquivo mais novo.

O realce de sintaxe é imediato: palavras-chave, tipos e funções da
biblioteca padrão (em C/C++: `std`, `cout`/`cin`, `printf`, `string`,
`vector`, `size_t`...; em Rust: `Vec`, `Option`, `println!`, tipos
primitivos...) já aparecem coloridos, e as **variáveis ficam claras/quase
brancas** para se distinguir na hora. Alguns segundos depois de abrir um
arquivo Rust/C++, as **cores semânticas** do servidor de linguagem
(clangd / rust-analyzer) refinam ainda mais o realce (tipos, funções,
campos, lifetimes, membros...).

A base imediata é o **Tree-sitter incremental**: ela também alimenta folds e
o Outline em árvore mesmo quando clangd/rust-analyzer estiver reiniciando. O
LSP continua sendo a autoridade para tipos, referências e refatorações.

Ao completar código, as **sugestões do servidor de linguagem** aparecem
sozinhas enquanto você digita (também depois de `.` e `::`); `Tab` ou
`Enter` aceita a selecionada, `↑`/`↓` navega e `Esc` fecha — sem
atrapalhar quem só quer digitar. `Ctrl+Space` força a lista a qualquer
momento.

Digitar `<` logo depois de `#include ` já fecha em `<>` com o cursor no
meio (só nesse contexto — em comparações e templates o `<` fica normal).

### 3.2 Inteligência de código (LSP)

Funciona em Rust (rust-analyzer) e C/C++ (clangd). Os servidores sobem
sozinhos ao abrir o primeiro arquivo da linguagem — a primeira resposta
pode demorar alguns segundos enquanto o projeto é indexado. Nesse intervalo,
o autocomplete já mostra símbolos locais do Tree-sitter; quando o LSP responde,
a lista é enriquecida/substituída pelo resultado semântico autoritativo.

| Atalho | Ação |
| --- | --- |
| `Ctrl+B` | Ir para a definição do símbolo sob o cursor |
| `Ctrl+Q` | Documentação/tipo do símbolo (hover) |
| `Ctrl+Space` | Completar código (também abre sozinho ao digitar) |
| `Alt+F7` ou `Ctrl+Shift+U` | Encontrar usos do símbolo |
| `Shift+F6` ou `Ctrl+Shift+R` | Renomear símbolo (em todos os arquivos afetados) |
| `Alt+Enter` | **Quick fixes / ações de código** no cursor (↑↓ escolhe, Enter aplica) |
| `Alt+O` | **C/C++: alternar header/source** (`.h` ↔ `.cpp` do mesmo componente, via clangd) |
| `F2` ou `Ctrl+Alt+E` | Ir para o **próximo problema** do arquivo |
| `Shift+F2` ou `Ctrl+Alt+Shift+E` | Ir para o **problema anterior** |

Erros e avisos do servidor de linguagem aparecem **na hora, enquanto você
digita** (uma fração de segundo depois de parar): sublinhado ondulado sob
o trecho com problema (vermelho = erro, âmbar = aviso, azul = nota), uma
**marca colorida na régua de linhas** (passe o mouse nela para ler a
mensagem) e a lista completa na aba **Problemas**. Na aba Problemas cada
item mostra o arquivo:linha, o **código/regra** (ex.: `E0425`,
`unused_variables`) e a mensagem; clique para saltar até ele.

`Alt+O` só vale para C/C++ e depende do clangd; se não houver contraparte
(um `.h` sem `.cpp`, por exemplo), nada acontece.

Rename e quick fixes que afetam arquivos mostram primeiro um **preview lado a
lado**. **Aplicar** revalida todos os snapshots e grava tudo como uma única
transação; **Cancelar** não toca no disco. Se algum arquivo mudou desde o
preview, a operação inteira é recusada e nenhum arquivo fica parcialmente
alterado.

### 3.3 Navegação e busca

| Atalho | Ação |
| --- | --- |
| `Ctrl+Shift+N` ou `Ctrl+Shift+A` | **Search Everywhere**: arquivos por nome e comandos da IDE |
| `Ctrl+E` | Arquivos recentes, no mesmo Search Everywhere |
| `Alt+7` | Aba **Símbolos**: estrutura do arquivo e busca de declarações por nome no projeto |
| — digite `@` | ...símbolos do arquivo atual (estrutura); `@nome` filtra |
| — digite `#nome` | ...símbolos do workspace inteiro (structs, funções...) |
| `Ctrl+Shift+F` | Buscar texto em todos os arquivos (aba Busca) |
| `Ctrl+Shift+H` | **Substituir texto no projeto** (resumo + confirmação) |
| `Ctrl+F` / `Ctrl+H` | Buscar/substituir **dentro do arquivo aberto** (barra no editor) |

No Search Everywhere: `↑↓` navegam, `Enter` abre, `Esc` fecha.

**Busca no arquivo (`Ctrl+F`) vs. busca no projeto (`Ctrl+Shift+F`):** a
primeira procura só no arquivo que está na tela e realça as ocorrências ali
mesmo; a segunda varre todos os arquivos do projeto e lista os resultados na
aba Busca.

Na substituição do projeto, informe busca/substituição na aba Busca e confirme
o resumo. A IDE exige salvar ou descartar buffers sujos antes; o core aplica
todos os arquivos de forma transacional e faz rollback em qualquer falha.

Na barra do `Ctrl+F`: o texto **selecionado** (ou a palavra sob o cursor) já
entra preenchido. `Enter` vai para a próxima ocorrência, `Shift+Enter` para a
anterior, `Esc` fecha e devolve o foco ao editor. O contador mostra **"3 de
17"**. Os três botões ligam **`Aa`** (diferenciar maiúsculas), **`W`** (só
palavra inteira) e **`.*`** (expressão regular). Com `Ctrl+H`, aparece o campo
de substituição: **Substituir** troca a ocorrência atual e avança, **Tudo**
troca todas de uma vez. Em modo regex, `$1` no campo de substituição reusa o
grupo capturado.

---

## 4. Build, testes e análise

| Atalho | Ação | Onde ver o resultado |
| --- | --- | --- |
| `Ctrl+F9` ou `Ctrl+Alt+B` | **Build** do projeto (cargo build / cmake) | aba Build + Problemas |
| `Ctrl+Shift+F9` ou `Ctrl+Alt+T` | **Testes** (cargo test / ctest / `python -m pytest -v`) | aba Testes |
| `Ctrl+Shift+L` | **Análise de qualidade** (clippy / clang-tidy / `ruff check`) | aba Problemas |

> Todos esses também são **botões na barra superior** — atalho nenhum é
> obrigatório. Onde houver tecla `F1`–`F12` há sempre uma alternativa sem
> ela (para notebooks onde F-keys exigem `Fn`).

Tudo roda em segundo plano como *job*: a barra de status mostra o
progresso e um **×** para cancelar; a aba **Jobs** guarda o histórico.
Clicar num problema abre o arquivo na linha exata.

**Projetos Rust:** além do build/clippy, há o **"Cargo: Check"** no Search
Everywhere — feedback de tipos/borrow bem mais rápido que o build, com os
problemas caindo na mesma aba Problemas. Se o `cargo metadata` do projeto
estiver quebrado (Cargo.toml inválido), o banner de Project Health avisa
com a mensagem do cargo e um botão para tentar de novo.

**A árvore de testes.** Na aba **Testes**, **Listar testes** mostra os casos
**sem rodar nenhum** (`pytest --collect-only`, `cargo test -- --list`,
`ctest -N`), e cada linha tem **Rodar só este teste**. O caso que roda pinta a
própria linha com o resultado; a saída bruta do runner fica logo abaixo — e
se o runner nem chegou a correr (sem `pytest` no ambiente, por exemplo), a aba
mostra **o motivo e o passo para instalar**, em vez de "0 testes".

**gtest e Catch2 dentro dos binários.** Num projeto CMake, **Listar testes**
pergunta a cada executável do `ctest` quais casos ele tem (`--gtest_list_tests`
ou `--list-tests` do Catch2) e os lista abaixo da linha do `ctest`, como
`unit_tests::Math.Adds`; **Rodar só este teste** roda o binário com o filtro
do framework, e cada `[ OK ]`/`[ FAILED ]` pinta a linha do caso.

**Análise em C/C++.** O clangd sobe com `--clang-tidy` e lê o `.clang-tidy`
do projeto: os avisos aparecem no arquivo aberto como qualquer diagnóstico.
A **Análise de qualidade** (`Ctrl+Shift+L`) num CMake ou Makefile roda o
clang-tidy do projeto inteiro pela `compile_commands.json` (`run-clang-tidy`
quando existe) e lista os avisos em Problemas — sem CDB, ela diz "configure";
sem `clang-tidy`, diz o pacote.

**A lâmpada.** Na linha do cursor com um diagnóstico, a calha mostra 💡 antes
de você apertar `Alt+Enter`: é onde os quick fixes do clangd, rust-analyzer,
basedpyright e ruff moram. Clicar nela é o mesmo que `Alt+Enter`.

**Cobertura dos testes** (menu Build → **Cobertura dos testes**, ou a
paleta): Rust pelo `cargo llvm-cov` (`cargo install cargo-llvm-cov` e
`rustup component add llvm-tools-preview`), Python pelo `coverage.py` do
ambiente do projeto (`uv add --dev coverage pytest`). O job escreve
`.kinein/coverage.lcov`, o resumo sai na aba Jobs e a **calha do editor**
ganha uma barra ao lado da do git: verde, linha executada pelos testes;
vermelha, instrumentada e nunca executada. C/C++ ainda não: exige compilar
com `--coverage`, e a IDE não muda o seu build por conta própria.

**Scripts do projeto:** arquivos `.sh`, `.bash`, `.zsh` e `.py` mostram um
botão de executar ao passar o mouse na árvore. A mesma ação fica no clique
direito como **Executar script** (e **Depurar**, num `.py`). A saída abre
numa **aba do Terminal** com o nome do comando (`▶ bash -- 'x.sh'`) e pode
ser interrompida pelo controle normal de Run (ou pelo × da aba). Só arquivos dentro do workspace são aceitos; não é necessário abrir um
terminal e digitar o caminho. O que a árvore aceita como executável é o core
quem diz — a lista não mora na tela.

---

### 4.1 Depurar (debugger)

Funciona em C/C++ e Rust via `lldb-dap` (instalado junto com o lldb), em
**Python via `debugpy`** (o módulo do interpretador do projeto — veja §10) e,
em projetos embarcados, pelo `gdb -i dap` contra o servidor de debug do kit
(OpenOCD, QEMU, `probe-rs`) — veja §9, *Embarcados*.

1. **Compile antes** (`Ctrl+F9`) — o debug usa o binário já compilado.
2. Clique na **gutter** (a coluna dos números de linha) para marcar
   breakpoints — a bolinha vermelha. Pode marcar antes mesmo de iniciar.
3. Clique em **[Debug]** na barra superior (ou `Shift+F9` /
   `Ctrl+Alt+D`). O programa
   roda e **para no primeiro breakpoint**: a linha fica destacada e o
   arquivo abre sozinho na posição.
4. Na aba **Debug** do painel inferior ficam os controles e a saída do
   programa.

**Conectar a um Python já iniciado.** No ambiente do serviço, rode:

```bash
python -m debugpy --listen 127.0.0.1:5678 --wait-for-client app.py
```

Abra as fontes correspondentes na IDE e marque os breakpoints. Na aba **Debug**,
preencha **Host do Python** e **Porta** e clique **Conectar ao Python**.
Frames, variáveis e watches usam os controles habituais. **Desconectar** encerra
apenas a sessão de depuração; o serviço continua rodando. O adaptador precisa
informar caminhos acessíveis neste workspace; SSH e mapeamento de caminhos
remotos continuam na etapa de Linux embarcado.

Todos os controles são **botões clicáveis na aba Debug** — dá para
depurar inteiro sem atalho. Para quem prefere teclado (as duas colunas
valem sempre; a segunda não depende de `Fn`):

| Atalho clássico | Sem F-keys | Ação |
| --- | --- | --- |
| `Shift+F9` | `Ctrl+Alt+D` | Inicia o debug (o botão vira ■ para parar) |
| `F9` | `Ctrl+Alt+C` | Continuar até o próximo breakpoint |
| `F8` | `Ctrl+Alt+N` | Step over (executa a linha, sem entrar em funções) |
| `F7` | `Ctrl+Alt+I` | Step into (entra na função da linha) |
| `Shift+F8` | `Ctrl+Alt+U` | Step out (sai da função atual) |

O alvo é escolhido automaticamente (o binário único de `target/debug` no
Rust, ou de `.kinein/build` no CMake; num projeto Python, o ponto de entrada —
`main.py`, `app.py`, `__main__.py` ou um pacote com `__main__.py`, lançado
como `-m pacote`). Se houver mais de um, a mensagem de erro lista os
candidatos.

**Inspecionar o estado:** com o programa pausado, a aba Debug mostra os
**Frames** (a pilha de chamadas — clique num frame para abrir o código
dele e ver as variáveis daquele nível) e as **Variáveis** locais.
Variáveis com **▸** são structs/objetos: clique para expandir os campos.
Acima do editor, uma trilha (breadcrumbs) mostra onde o arquivo atual
está no projeto; a linha do cursor fica levemente destacada.

---

### 4.2 Git

Se o projeto for um repositório git, a IDE mostra sem você pedir:

- **Barra de status** (embaixo): a branch atual (`⎇ main`), setas
  `↑`/`↓` quando há commits à frente/atrás do remoto e o total de
  alterações.
- **Árvore de arquivos**: nomes coloridos — **verde** = arquivo novo
  (untracked/adicionado), **azul** = modificado/renomeado, **apagado** =
  deletado, **vermelho** = conflito.

- **Gutter do editor**: uma barra fina colorida ao lado dos números de
  linha mostra o que mudou desde o último commit — verde = linhas
  novas, azul = linhas alteradas, um traço vermelho = linhas removidas
  ali. Atualiza ao salvar e ao trocar de aba (edição ainda não salva
  não aparece).
- **Diff do arquivo**: rode **"Git: Diff do arquivo"** no Search
  Everywhere (`Ctrl+Shift+A`) para ver o diff completo do arquivo atual
  contra o HEAD, **no lugar do editor**, como uma aba de visualização
  ("diff: caminho"); o × (ou `Esc`) devolve o editor como estava.

- **Janela do Git** (à esquerda, **no lugar do explorer** — o ícone Git do
  trilho, o widget da barra, **Exibir → Git** ou "Git: Commit..." abrem;
  o ícone Projeto traz o explorer de volta; o ícone do que está aberto
  fecha o slot — como Project/Commit nas IDEs JetBrains). Em pé, com
  duas abas no topo:
  - **Commit**: a linha do branch (branch · pull · push · stash · pop) e a
    lista de tudo que mudou, **agrupada por pasta** (o quadradinho da
    pasta marca/desmarca todos). Clique no **quadradinho** de um arquivo
    para marcar/desmarcar para commit (stage); clique no nome mostra o
    **diff** no editor; duplo clique/“abrir” abre o arquivo; passando o
    mouse aparece **↩** (descartar — pede confirmação, porque não tem
    desfazer). Digite a mensagem embaixo e clique **Commit (N)** — commita
    só o que está marcado; **Commit e Push** pede um segundo clique;
    **Amend** reescreve o último commit.
  - **Log**: o filtro (texto; o branch de onde partir) e a lista de
    commits com o grafo, os refs (chips) e a idade. Clique num commit
    para ver, no editor, o que ele mudou (autor, arquivos, patch).
    "Git: Historico" no Search Everywhere abre direto aqui; **atualizar**
    recarrega a lista.

- **Blame** (autoria por linha): rode **"Git: Blame do arquivo"** no
  Search Everywhere para ligar/desligar uma coluna ao lado dos números
  de linha mostrando **quem** mudou cada linha e **há quanto tempo**.
  Linhas ainda não commitadas aparecem como "não commitado". Acompanha o
  arquivo que você está editando.

O status atualiza sozinho ao salvar, criar, renomear ou excluir arquivos pela
IDE **e também** quando o watcher detecta mudanças externas (por exemplo, um
`git pull` no terminal). **"Git: Atualizar status"** no Search Everywhere
continua disponível para atualização manual.

Na janela do Git também é possível listar/trocar/criar branches, fazer **Pull** e
**Push** como jobs (saída e cancelamento na aba Jobs) e guardar/restaurar o
trabalho com **Stash push/pop**. Checkout, criação de branch e stash são
recusados enquanto houver buffers sujos, evitando misturar estado do editor
com mudanças do worktree. Metadados `.kinein` nunca entram no stash.

---

## 5. Terminal (shell e execução)

A aba **Terminal** tem várias sessões, alternáveis pelos chips no topo —
shells e execuções, todas terminais de verdade:

- **Terminal 1, Terminal 2, …**: shells completos e independentes em
  **terminais de verdade** (PTY real, `TERM=xterm-256color`), abertos na raiz
  do projeto. Têm **cores**, cursor, refluem ao redimensionar e rodam programas
  interativos/full-screen (vim,
  htop, o progresso do `cargo`). A digitação é **caractere a caractere**
  (como no VS Code/JetBrains): `Ctrl+C` interrompe, setas navegam o
  histórico, `Tab` completa — não é mais "escreva a linha e aperte Enter".
  O cursor vertical acompanha a próxima célula de digitação. As cores ANSI
  seguem a paleta visual da Kinein; prompts que marcam usuário, máquina e
  pasta continuam sendo produzidos pelo shell, com peso moderado pela IDE.
  Clique em **+** para abrir outro terminal e em **×** no chip para fechar só
  aquela sessão; programas e histórico das outras abas continuam vivos.
  A barra de rolagem permanece visível desde o início, acompanha a saída ao
  vivo e coalesce movimentos rápidos. Ao digitar, o terminal retorna
  imediatamente ao prompt atual; rolar manualmente para cima preserva a leitura
  do histórico até o usuário voltar ao fundo ou começar um novo comando.
- **▶ `<comando>`**: a execução do seu projeto (`cargo run`, o executável do
  CMake, o `main.py` do interpretador do projeto) abre numa aba própria, um
  PTY como os shells — stdin, cores e programas de tela cheia funcionam;
  a bolinha verde acompanha enquanto roda. Quando termina, **a aba fica**,
  com o desfecho no nome (`✓` ou `✗ 101`), para você ler a saída; o × a
  fecha. (Até 2026-09-18 havia uma sessão "Execução" separada, por pipes,
  sem TTY — saiu: o terminal integrado já faz tudo o que ela fazia.)

**Configurações de execução:** na barra superior há um seletor (começa em
"Automático" — o menu abre por cima de tudo e fecha clicando fora). "Nova configuração..." salva um comando com nome (ex.:
`cargo run --bin servidor`); a configuração ativa é o que o ▶ executa. Dá
para editar/excluir a atual pelo mesmo menu, e a escolha fica salva com o
projeto.

| Atalho | Ação |
| --- | --- |
| `Shift+F10` ou `Ctrl+Alt+R` | Executa o projeto — abre uma aba **▶ comando** no Terminal |
| `Ctrl+F2` ou `Ctrl+Alt+X` | Fecha a aba da execução em curso (mata o processo) |
| `Alt+F12` ou ``Ctrl+` `` | Abre a aba Terminal; cria o primeiro shell se necessário |

Enquanto um processo estiver rodando, a aba mostra **Terminal ●** e o chip
da execução, a bolinha. (O botão **limpar** saiu junto com a sessão
"Execução": num terminal de verdade, `clear` ou `Ctrl+L`.)

## 6. Usar uma CLI de IA (Claude, Codex...)

Abra o terminal integrado (`Alt+F12`) e rode a ferramenta como faria fora da
IDE:

```bash
claude
codex
```

É isso. Não há painel dedicado, perfil a escolher nem configuração: para a
Kinein, uma CLI de IA é **um programa como outro qualquer**. Ela roda no mesmo
PTY real e no mesmo emulador de terminal de um `ls` ou de um `vim`, então a
interface da ferramenta aparece exatamente como aparece no seu terminal.

A IDE não instala, não autentica e não envia nada: rede, credenciais e política
de dados pertencem à CLI que você iniciou.

> **Não existe painel de IA, e isso é decisão, não pendência.** Um atalho visual
> dedicado para essas CLIs foi construído em 2026-07-17 e removido no mesmo dia,
> depois de rodar: ele só poupava digitar uma palavra, e cobrava por isso um
> seletor, um rótulo, uma aba e um ícone. Você roda o agente no terminal, como
> roda qualquer outro programa. A aba **Ferramentas** mostra se o binário está
> instalado.

---

## 7. Configurações (`Ctrl+Alt+S`)

Abra as configurações com `Ctrl+Alt+S` (ou "Configurações" no Search
Everywhere). No v1 há quatro opções, que valem para **todos os projetos**:

- **Tamanho da fonte do editor**: `−`/`+` (8 a 40), aplica na hora.
- **Formatar ao salvar**: quando ligado, `Ctrl+S` formata o arquivo
  (rustfmt/clang-format) e **então** salva. Se o formatador falhar, o
  arquivo é salvo assim mesmo.
- **Fechar pares automaticamente**: liga/desliga o auto-fechamento de
  `( [ { " '` ao digitar.
- **Perfil de rigor**: regula o quanto os botões **[Verificar]** e
  **[Compilar]** apertam **o seu projeto** (nunca o próprio Kinein).
  Três níveis:
  - **Estrito** (padrão): clippy com `pedantic` + `nursery` e warning
    vira erro (`cargo build` também falha em warning). É a régua da casa
    — o código sai afiado.
  - **Equilibrado**: clippy padrão (bugs, estilo, performance), warnings
    aparecem mas não travam o build.
  - **Relaxado**: só o essencial (`clippy::correctness` — bugs reais);
    o resto é permitido. Bom para prototipar rápido.
  Só afeta projetos **Rust** no v1; trocar de perfil recompila o projeto
  (o `cargo` reavalia as flags).

As configurações ficam em `~/.config/kinein-vectis/settings.json`. Um
projeto pode ter ajustes próprios em `.kinein/settings.json` (têm
prioridade sobre o global) — por enquanto editando o arquivo à mão.

---

## 8. Aba Ferramentas e saúde do ambiente

A aba **Ferramentas** mostra o que a IDE encontrou na sua máquina (cargo,
clangd, cmake...), com versão e, se faltar algo, a sugestão de pacote para
instalar. O botão **Redetectar** atualiza a lista. O scan completo também
pode ser disparado pelo banner de Project Health.

---

## 9. Menu **Ambiente** — preparar o projeto antes de compilar

Este menu existe porque configurar o ambiente **não é "ferramenta"**: é o que se
faz antes de compilar. São nove painéis, e todos seguem a mesma regra — a IDE
**mostra o que vai fazer e espera você aceitar**; nenhum deles escreve no seu
projeto sozinho. Os quatro que se usam todo dia também estão na **barra lateral
esquerda**, nesta ordem: Banco de dados, Containers, Observabilidade e, por
último, Ferramentas.

### Bibliotecas

Um catálogo **curado** de bibliotecas C/C++ — hoje 15 —, cada uma com a licença
verificada na fonte, a versão fixada e a frase do que ela faz. A bolinha ao lado
do nome diz o que está **no seu projeto**, não o que está instalado na máquina.

Ativar escreve no `CMakeLists.txt` por meio de uma ação de configuração, com
prévia. **Desativar existe** e usa o mesmo caminho — o botão diz qual dos dois
vai acontecer.

Uma biblioteca só entra no catálogo depois de auditada. Duas já foram
**recusadas** por licença, e a recusa fica registrada.

### Ações de configuração

São **18**, e elas respondem duas perguntas diferentes: *o que o projeto tem*
(dependência, feature, edição, alvo, preset) e *como ele compila* (rigor,
sanitizers, OpenMP, `.hex`/`.bin`, alvo embarcado).

**Nenhuma escreve sem prévia.** A caixa mostra o arquivo como ele ficará, e o
botão de aceitar só vale para o que está na tela. Os campos explicam o que
esperam e sugerem valores lidos do **seu** projeto — não exemplos genéricos.

### Banco de dados

A IDE guarda o **perfil** da conexão: motor, endereço, porta, base e usuário.
**Nunca a senha** — quando ela é necessária, a IDE pede na hora, e nada de
credencial vai para o disco. Três motores:

```text
PostgreSQL   e tudo que fala o protocolo dele — TimescaleDB incluso
SQLite       um ARQUIVO: sem servidor, sem porta, sem usuario
MongoDB      colecao -> documento, e a tela dele e' OUTRA
```

O MongoDB tem uma forma de exibição própria de propósito: uma coluna de tabela
garante que existe em toda linha, tem um tipo e não aninha, e **nenhuma das três
vale para um documento**. Desenhar documento como linha faria a tela afirmar três
coisas falsas.

Além de testar a conexão, o painel **lê o banco**: esquemas, tabelas e colunas —
ou coleções e a forma dos documentos.

**Executar** (desde 0.121.0): o campo **Consulta** aceita o que você escrever
— `Ctrl+Enter` ou **Executar**. Uma leitura (`SELECT`, `WITH`, `VALUES`,
`SHOW`, `EXPLAIN`) roda com teto de 500 linhas (a IDE põe o `LIMIT` por
fora; "teto atingido" avisa) e **de verdade só lê**: no PostgreSQL vai numa
transação `READ ONLY`, no SQLite o arquivo abre só para leitura — um `WITH …
INSERT` disfarçado é recusado pelo próprio motor. Uma instrução que
**escreve** (`INSERT`, `UPDATE`, `DELETE`, DDL) não roda de primeira: aparece
o botão *"Esta instrução ESCREVE — executar mesmo assim"*, e só o clique nele
executa; o resultado diz quantas linhas foram afetadas. A grade mostra
toda célula como texto e `NULL` como *null* em itálico. No MongoDB a
consulta é `<coleção> <filtro JSON>` (ex.: `sensores {"placa": "esp32"}`), só
leitura, e cada chave de primeiro nível vira coluna.

**TLS** (PostgreSQL): o chip **TLS verificado (verify-full)** cifra a conexão
e confere a cadeia **e** o nome do host — para um servidor com certificado
próprio, aponte o PEM no campo que aparece. Não existe "cifra sem conferir":
é a opção que dá sensação de segurança sem a garantia.

### Alvo remoto (SSH)

Uma Raspberry Pi, uma placa com imagem própria (Yocto, Buildroot) — um Linux
que você alcança por SSH — vira **recurso do projeto**: menu Ambiente →
**Alvo remoto (SSH)...** (ou `remote.list` na paleta). A IDE guarda o
**perfil** (nome, host, usuário, porta, chave privada, pasta de deploy) em
`.kinein/remotes.json`. **Não há campo de senha, por desenho**: SSH aqui é
por chave — copie a sua com `ssh-copy-id usuario@host` uma vez; o que o
`ssh` do sistema precisar perguntar (o host key novo, a senha da chave),
pergunta no terminal da IDE. A IDE nunca digita nem guarda senha.

```text
Sondar          ssh -o BatchMode=yes -o ConnectTimeout=5 … 'uname -m; uname -sr; command -v …'
                -> arquitetura (aarch64), kernel e o que o alvo TEM: gdbserver, python3, rsync
                   sem chave: falha em segundos e diz o `ssh-copy-id`
Enviar (deploy) rsync -az --delete <origem> alvo:<pasta>/   (ou scp -r sem rsync)
                origem padrao build/, pasta padrao ~/kinein/<projeto>
Rodar em…       configuracao de execucao "Rodar em <nome>": ssh -tt alvo '<programa>'
gdbserver → kit o kit ganha debugServer = ssh -tt alvo 'gdbserver :2345 <programa>' e
                remoteTarget = host:2345 — o [Debug] de sempre sobe o servidor e conecta
debugpy → config "debugpy em <nome>": python3 -m debugpy --listen 0.0.0.0:5678 --wait-for-client
                <script>; depois, o attach TCP da aba Debug em host:5678
Shell no terminal ssh alvo, no terminal da IDE
```

O campo **Programa no alvo** é o caminho DO OUTRO LADO: relativo entra na
pasta de deploy (`app` → `~/kinein/<projeto>/app`), absoluto vai como está.
Sondar e enviar só valem para um alvo **salvo** — o core só conhece o que
está no arquivo. Cada botão mostra a linha que compôs (`$ …`), e o que ela
virou (a configuração salva, o kit gravado).

**Abrir a pasta do alvo como espelho** (desde 0.122.0): digite a pasta
(`/home/pi/projeto`) e clique **Abrir espelho**. A IDE puxa a árvore por
`rsync` para `~/.cache/kinein-vectis/remote/…` e abre esse espelho como um
workspace comum — editor, busca, git, LSP, tudo funciona nele. A partir daí
**salvar um arquivo empurra só ele para o alvo** (um job "Empurrar
main.c para pi", visível na aba Jobs). O painel Remoto mostra "este
workspace é um espelho de pi:/home/pi/projeto" com **Puxar do alvo** (o que
mudou lá) e **Empurrar tudo**. Nada apaga do outro lado: renomear ou apagar
no espelho não propaga — é gesto seu, pelo shell. O que muda no alvo só
aparece ao Puxar; se os dois lados editarem o mesmo arquivo, o rsync mais
recente vence. O LSP resolve contra a sua máquina: para C/C++ cross, o
sysroot do kit; um interpretador Python do alvo ainda não. Dica: com o
espelho aberto, "Rodar em pi" roda o que você acabou de salvar.

### Observabilidade (Grafana)

A IDE conversa com o Grafana pela **HTTP API** e nunca o embute — a licença dele
(AGPL-3.0) decide essa forma. Ela guarda o endereço e a política; **o token não
tem onde ser gravado**, e isso é garantia estrutural, não disciplina.

### Instalar ferramentas

Falta o `cmake`, o `clangd` ou o `gdb`? Este painel mostra o passo a passo
**oficial** para a sua distro — copiado da documentação do próprio projeto, com a
URL e a data em que foi conferida. Onde a fonte oficial não cobre a sua família
de distro, **a entrada não existe**: a IDE mostra o link e diz que não tem passo
a passo, em vez de traduzir um comando de outra distro.

**A IDE não instala nada sozinha.** O botão escreve os comandos no terminal
dela, visível, e quem aperta Enter é você.

O painel cobre o banco, o Grafana, as ferramentas Python e — desde
2026-09-17 — as de embarcados: `esptool`, `mpremote`, `espflash`,
`probe-rs`, `picotool`, `dfu-util`, `tio`, `picocom`, o GCC ARM
(`arm-none-eabi` + GDB), o QEMU (ARM e RISC-V) e o OpenOCD. Cada uma diz se
já está instalada (a mesma detecção do resto da IDE) e, quando há guia para
a sua distro, os passos com a fonte. As regras udev de sonda e do
ModemManager não estão aqui: são o botão **Permissões** do painel de
Embarcados, que as mede na sua máquina.

### Toolchain e kits

Qual executável cumpre cada papel — compilador C/C++, gerador, `cmake`, `cargo`.
Sem escolha, o `PATH` decide, que é o comportamento de sempre. A escolha é do
**kit**, e um kit é um preset mais o sysroot e o *triple* do alvo — é assim que
compilação cruzada e embarcados entram sem um segundo mecanismo.

A IDE escolhe automaticamente quando dá, e **mostra que escolheu**.

**O kit manda no configure e nos servidores.** Com um kit ativo, o configure
(automático ao abrir, ou o botão Configurar) usa o preset dele; sem kit, o
preset padrão do projeto — o primeiro do seu `CMakeUserPresets.json`, senão
do `CMakePresets.json` — e a aba IDE diz qual (`preset de: kit |
CMakeUserPresets.json | CMakePresets.json`). O `targetTriple` do kit vai ao
`cargo build` e ao rust-analyzer (que reinicia ao trocar de kit); o
compilador cross vai ao clangd. **Escolher o preset** (desde 2026-09-18):
num projeto com `CMakePresets.json`, o painel de Embarcados → Alvo do kit
mostra os presets como chips (`padrão`, `dev`, `release`…); clicar num
torna o kit dele o ativo, e o próximo configure usa esse preset.

**Projeto só com Makefile.** É reconhecido como "Make": Build roda `bear --
make` quando o `bear` está instalado — é ele que escreve o
`compile_commands.json` que o clangd lê — e `make` a seco quando não, dizendo
isso; o Project Health nomeia o `bear` como ferramenta ausente e o painel
Instalar ferramentas tem o passo. Um Makefile ao lado de um `CMakeLists.txt`
continua sendo um projeto CMake.

**Sysroot e SDK do alvo.** O campo "Pasta/SDK" aceita digitação ou
**Escolher pasta…** — o mesmo navegador de pastas da Start Screen, não um
diálogo do sistema. "Ler sysroot" diz o que a pasta contém; "Importar kit"
lê um SDK Yocto (`environment-setup-*`), uma árvore Buildroot
(`output/host`), um **Zephyr SDK** (a raiz `zephyr-sdk-*`, com as toolchains
que o `setup.sh` instalou — a IDE propõe a `arm-zephyr-eabi` quando há
várias e lista as outras) ou uma pasta de toolchain, e mostra a proposta
antes de "Aplicar proposta ao kit". Nada é gravado sem o segundo clique.

### Embarcados (`Ctrl+Alt+M`, ou o chip no rail)

O painel de quem escreve firmware, em **quatro abas** (desde a Etapa 3)
com o veredito da placa no cabeçalho ("ESP32-D0WD-V3 em /dev/ttyUSB0",
ou "1 porta(s) serial(is); nenhuma placa identificada ainda", ou
"nenhuma placa"): **Placa** (portas seriais, identificar, permissões,
sonda, arquivos na placa) · **Projeto** (o framework e o modelo, o tamanho
do binário) · **Gravar** (o motor, a prévia, o firmware) · **Kit** (chip,
alvo, sysroot, SVD, o depurador, instalar toolchain, importar de SDK —
**Aplicar ao kit** no pé). Cada aba cabe sem rolar numa janela de 800 px.
Ele **lê**, e diz o que leu:

```text
o projeto        o framework pelos marcadores dele (ESP-IDF, Zephyr, pico-sdk,
                 PlatformIO, STM32Cube, Rust bare metal, MicroPython, Yocto,
                 Buildroot), o alvo deduzido, o que falta na maquina com o
                 passo oficial, e o que o modelo NAO conseguiu decidir
compilar pelo    o botao Compilar usa o WRAPPER do framework quando ele tem
framework        um: `pio run` (platformio.ini — vence tudo), `idf.py build`
                 dentro do ambiente ativado (o export.sh de IDF_PATH/
                 ~/esp/esp-idf, ou o activate_idf_<versao>.sh do EIM em
                 ~/.espressif/tools — a IDE faz o `source` por voce, no
                 job), `west build -d build -b <placa>` (a placa do build
                 anterior ou de `west config build.board`; sem ela o west
                 diz o que falta), e o CMake de sempre com -DPICO_SDK_PATH
                 no pico-sdk. Framework reconhecido sem a ferramenta: a IDE
                 recusa ANTES de rodar e diz o passo (eim install, pipx
                 install west/platformio). Em Gravar, os motores `idf.py`,
                 `west` e `platformio` aparecem ao lado do esptool; o
                 monitor num projeto ESP-IDF e' o IDF Monitor e num
                 PlatformIO o `pio device monitor`
portas seriais   vistas pelo sysfs, SEM abrir nenhuma: quem faz a ponte
                 (CP2102, CH340, USB Serial/JTAG...), se voce tem acesso e o
                 aviso do ModemManager quando ele pode ocupar a porta
monitor serial   um botao por porta abre tio/picocom/minicom (ou o REPL do
                 mpremote num projeto MicroPython) NUMA ABA DE TERMINAL —
                 e' um processo, como qualquer outro
permissoes       o botao "Permissoes" mede cada canal e diz o que falta: o no'
                 serial (grupo dono, ou a ACL que o udev deu pela tag uaccess
                 — e' assim que voce pode ter acesso SEM estar no dialout), o
                 ModemManager (rodando e sem regra de ignorar, ele segura a
                 porta por segundos depois do plug) e a regra udev das sondas
                 (a distro pode ja' ter posto a do OpenOCD/ST-Link). Para o
                 que falta, o passo OFICIAL com a fonte e a data; "Escrever no
                 terminal" poe o comando no terminal da IDE e o prompt de
                 senha do sudo aparece la'. A IDE nunca roda sudo
identidade       a lupa ao lado de cada porta pergunta ao `esptool` o que ha'
pelo canal       do outro lado (chip, flash, MAC) — ABRE a porta e a placa
                 RESETA, por isso e' um clique, nunca automatico. O painel
                 mostra o que leu e o kit que isso sugere; "Usar chip no kit"
                 grava so' o chip. Sem esptool, a linha diz `pipx install
                 esptool`; sem acesso a porta, o passo oficial do grupo
gravar           e' uma CONFIGURACAO DE EXECUCAO, nao um botao magico: escolha
                 a porta, (opcional) o motor — solto, o modelo do projeto decide
                 — e peca a Previa. A IDE monta a linha do esptool a partir da
                 receita que o build do ESP-IDF escreveu (flasher_args.json),
                 do probe-rs com o chip do kit e o ELF, do picotool com o UF2,
                 do dfu-util (so' STM32) — e mostra de onde veio cada pedaco e
                 o que voce deve saber antes (imagem cifrada, flash menor que
                 a receita). "Gravar agora" roda a linha na aba de execucao;
                 "Salvar" a guarda como configuracao ativa: dai' em diante o
                 botao Executar grava, e a linha e' editavel como qualquer
                 configuracao (e' assim que um esptool antigo troca
                 write-flash por write_flash). Sem build, a IDE diz "compile";
                 sem porta, "escolha a porta"; sem a ferramenta, o passo
arquivos na      a pasta ao lado de cada porta abre "Arquivos na placa"
placa            (MicroPython, pelo `mpremote fs`): a lista da raiz, entrar
                 em pastas, "Baixar" (para `placa/<caminho>` no seu projeto —
                 o espelho da placa — e o arquivo abre no editor), "Enviar
                 arquivo aberto" (o `.py` do editor vai para a placa),
                 "Apagar". Tudo que ESCREVE na placa pede um segundo clique.
                 Cada gesto interrompe o programa em execucao (raw REPL) e a
                 placa reinicia ao fim; se a primeira conexao falhar porque o
                 firmware inunda a serial, a IDE tenta de novo uma vez. Sem
                 mpremote, o passo de instalacao.
firmware         no catalogo de instalacao (secao "Toolchains instalaveis")
MicroPython      ha' o firmware oficial do micropython.org v1.29.0 para
                 ESP32, ESP32-C3, ESP32-S3, Pico e Pico W, marcado "firmware":
                 "Baixar" o traz para a pasta da IDE com o SHA-256 conferido
                 (a fonte nao publica checksum: o pinado foi medido em
                 2026-09-17 e a tela o diz). Depois, em Gravar, escolha o
                 firmware baixado no lugar do build: a linha e' a da pagina
                 da placa (`write-flash 0x1000` no ESP32 classico, `0x0` em
                 C3/S3; `picotool load -f -x` no Pico), com o aviso de
                 apagar a flash na primeira instalacao. Nada grava sem o seu
                 clique — gravar o firmware APAGA o que esta' na placa.
porta do         o chip "Executar" ao lado de cada porta a ESCOLHE para o
Executar         Executar de um projeto MicroPython (`mpremote connect
                 <porta> run`); clicar de novo desfaz. Sem escolha, o
                 mpremote usa a primeira porta que achar. A escolha vale
                 para o botao Executar (main.py) e para "Executar" num .py,
                 nao para um comando digitado nem para uma configuracao de
                 execucao salva — esses rodam como foram escritos. Uma porta
                 que some da lista (placa desplugada, painel reaberto) deixa
                 de ser a escolhida; trocar de projeto tambem limpa
sonda e kit      a sonda pelo `probe-rs list`; o chip, o alvo (triple), o
                 sysroot e o depurador ficam no KIT do projeto
tamanho          "Medir" le o ELF e mostra uma barra por regiao do linker
                 script (flash, RAM) depois do build
toolchains       o catalogo do que a IDE sabe instalar NA PASTA DELA
instalaveis      (~/.local/share/kinein-vectis/toolchains): URL, tamanho e
                 SHA-256 visiveis ANTES do clique; o download e' um job e o
                 checksum e' conferido antes de desempacotar. Nada no sistema
sysroot e SDK    "Ler sysroot" diz o que uma pasta contem (headers,
                 bibliotecas, libc, .pc do pkg-config) e se ela serve;
                 "Importar kit" le um SDK Yocto, uma arvore Buildroot ou uma
                 pasta de toolchain e PROPOE o kit — voce aplica
```

**O GDB certo para o ELF** (desde 2026-09-18): o `gdb` do Ubuntu não fala
ARM, Xtensa nem RISC-V — com ele escolhido e um ELF de placa, o attach
ficava mudo. Agora a IDE lê a arquitetura do ELF antes de subir o
depurador: se é de outra máquina, entra o GDB de alvo instalado
(`arm-none-eabi-gdb`, `gdb-multiarch`, `xtensa-esp-elf-gdb`,
`riscv32-esp-elf-gdb`) e o console de debug diz qual; sem nenhum, a
recusa diz o pacote (`apt install gdb-multiarch`).

Depurar um alvo embarcado é o mesmo `[Debug]` de sempre: o kit diz o
servidor (OpenOCD, QEMU ou `probe-rs`) e a IDE o sobe e conecta o `gdb -i dap`.
Nada aqui roda como root; permissão de porta e de sonda é mostrada, com o
passo oficial da distro, nunca executada.

**O que o depurador de embarcado mostra** (na aba Debug, a coluna entre as
variáveis e os watches, com a sessão parada): **Escopos** → *Listar* traz os
escopos do frame como o adaptador os nomeia — `Locals`, `Registers` e, com o
**SVD** do chip no kit (campo no painel de Embarcados) e o `probe-rs` como
adaptador, `Peripherals` (os registradores de periférico; o chip ⏱ marca o
escopo caro) — e o clique num escopo mostra as variáveis dele. **Memória**
→ digite um endereço (`0x3ff00000`) e leia 64 bytes em hexadecimal + ASCII;
**Desmontar** → as instruções a partir do endereço, com bytes e símbolo. Os
dois são o `readMemory`/`disassemble` do próprio DAP, que o `probe-rs` e o
GDB respondem. **RTT/defmt**: com o `probe-rs`, o console da placa pelos
canais RTT aparece na saída de debug (`canal RTT 0 \`Terminal\` aberto`, e as
linhas do canal) — o defmt já chega decodificado.

### Containers (`Ctrl+Alt+W`)

Docker **ou** Podman — o que responder nesta máquina (no Fedora, `docker`
costuma ser o `podman-docker`, e o painel diz isso). A primeira linha é o
**motor**: versão, rootless ou com daemon, o socket, se responde e qual
`compose` existe. Se algo falta, o painel imprime o passo oficial (grupo
`docker`, `systemctl`, `podman.socket`) — **e não o executa**.

Abaixo, os containers numa **grade** (estado · nome · imagem · portas ·
status; os parados também, pelo chip **parados também**), com um **filtro**
por nome, imagem ou id. Clique numa linha para escolhê-la: a **barra de
ações** logo abaixo oferece o que o estado dela permite — **Iniciar** o que
parou, **Parar/Reiniciar** o que roda, **Remover** só o parado (remover o
que roda é dois gestos, de propósito). **Logs** e **Shell** abrem numa aba
de terminal — por isso pedem um projeto aberto: a aba é do projeto (o
botão desligado diz isso ao pairar). As imagens locais fecham a lista.

**compose up / compose down** são do **projeto**: só acendem quando há motor
respondendo, uma ferramenta de compose **e um arquivo de compose na raiz do
workspace** (`compose.yaml`, `docker-compose.yml`...). Sem um deles, a linha do
motor diz o que falta. `up` é `-d`; a saída viva mora nos logs de cada
container. Toda ação é um job cancelável, e o que falhou vira motivo no topo
do painel.

---

## 10. Python

Python é vertical **nativa** da IDE, não um plugin — e a tela só diz "Python"
porque a cadeia inteira funciona: ambiente, linguagem, executar, testar,
depurar, e a placa.

**O interpretador é o `compile_commands.json` do Python.** Ao abrir um projeto
com `pyproject.toml`, `requirements.txt` ou `setup.py`, a IDE resolve **qual
Python** o projeto usa — o `VIRTUAL_ENV` ativo, o `.venv/` (ou `venv/`,
`env/`) do projeto, o ambiente do Poetry, o `python3` do sistema em último
caso, com aviso — e mostra na **barra de status**
(`python: .venv · 3.14.7`). Sem ambiente, o banner de Project Health oferece
**Criar .venv com uv** (ou `python3 -m venv`, se o `uv` não estiver na
máquina), com o comando visível; nada é criado sem clique.

```text
linguagem     realce e outline pelo Tree-sitter; completar, navegar, renomear,
              diagnosticos pelo basedpyright — que SOBE COM O INTERPRETADOR DO
              PROJETO e reinicia sozinho quando o .venv nasce
correcoes     Alt+Enter = acoes do basedpyright e do `ruff server`, quando Ruff
              e' encontrado pelo detector. Diagnosticos dos dois convivem na
              aba Problemas; remover import inutilizado, organizar imports e
              corrigir pelo Ruff usam o mesmo preview confirmavel do editor
formatar      Ctrl+Alt+L = `ruff format`
analise       Ctrl+Shift+L = `ruff check`, com o mesmo perfil de rigor dos
              outros (os problemas caem na aba Problemas)
executar      o botao Executar (Shift+F10) roda o PONTO DE ENTRADA do projeto —
              main.py, app.py, __main__.py, um pacote com __main__.py (como
              `-m pacote`) ou o script de [project.scripts] instalado —, com o
              interpretador do projeto (`uv run` quando ha' uv.lock). Qualquer
              .py da arvore tambem roda pelo botao ou pelo clique direito
testar        Ctrl+Shift+F9 = `python -m pytest -v`; a arvore de casos antes de
              rodar e "rodar so' este" (§4). Sem pytest no ambiente, a aba diz
              o passo para instalar NELE, nao no sistema
depurar       o mesmo [Debug]: o `debugpy` e' modulo do interpretador do
              projeto (`-m debugpy.adapter`); sem ele, a IDE diz `pip install
              debugpy` no ambiente certo. Breakpoints, frames e variaveis como
              em C/C++/Rust
modulo nativo um projeto com pybind11, nanobind ou PyO3 (maturin, scikit-build)
              e' reconhecido, e a barra diz qual e como se constroi — o C/C++
              ou Rust dentro dele e' lido pelo mesmo indice
novo projeto  Arquivo -> Novo projeto -> template "Python": layout plano,
              pyproject PEP 621, pytest em [dev], ruff configurado
```

**MicroPython.** Num projeto MicroPython, o **monitor serial** do painel de
Embarcados abre o **REPL do `mpremote`** na porta do botão, e **Executar** num
`.py` (ou o botão Executar, com o `main.py`) roda o arquivo **na placa**
(`mpremote run`). Com mais de uma placa plugada, escolha a porta pelo chip
**Executar** ao lado dela no painel de Embarcados: o comando vira `mpremote
connect <porta> run`, e a linha abaixo da lista diz qual porta está valendo.
Sem escolha, o `mpremote` usa a primeira porta que achar. Se o `mpremote` não
está na máquina, a aba de execução diz o que instalar em vez de rodar um
`import machine` no Python do desktop. O **firmware oficial** se baixa e se
grava pelo painel de Embarcados (catálogo de instalação → Gravar), e os
**arquivos da placa** se leem, enviam e apagam pela pasta ao lado da porta
(veja a seção Embarcados). **Stubs da placa:** num projeto MicroPython com o
chip conhecido (kit ou identidade pelo canal), a faixa de saúde oferece
**Instalar stubs** — `micropython-<port>-<placa>-stubs` em `typings/` na raiz
do projeto, pelo `uv` (ou o `pip` do interpretador); o basedpyright passa a
receber a pasta e o `import machine` completa e deixa de ser marcado como
módulo inexistente. Apague `typings/` para remover.

**O que a IDE não faz:** instalar o `uv`, o `basedpyright`, o `ruff` ou o
`debugpy` por conta própria. O painel **Instalar ferramentas** (`Ctrl+Alt+H`)
mostra o passo oficial de cada um; o botão escreve o comando no terminal da
IDE, e quem aperta Enter é você.

---

## 11. Quando algo der errado

1. **Aba "IDE"** (painel inferior): fluxo interno da IDE ao vivo — o que
   foi pedido e o que respondeu, e o que os servidores de linguagem escrevem
   no stderr deles (`lsp cpp · …`): a versão do clangd, onde ele procurou o
   `compile_commands.json`, o traceback de um basedpyright que não subiu.
   Um servidor que sai deixa o motivo na mesma aba.
2. **Log de erros persistente**:
   `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt` — anexe este arquivo ao
   reportar um bug.
3. **LSP demorou?** A primeira indexação de um projeto grande leva tempo
   (especialmente rust-analyzer). O completion sintático local aparece antes;
   navegação, tipos e resultados semânticos completos chegam quando o servidor
   termina a primeira preparação.
4. **Ferramenta faltando?** Veja a aba Ferramentas e siga a sugestão exibida
   para o seu sistema.
5. **"O adapter não respondeu"?** A aba Debug mostra, em vermelho, o que o
   próprio adaptador (debugpy, lldb-dap, probe-rs, gdb) escreveu antes de
   morrer, e a mensagem de erro traz as últimas linhas dele. Num projeto com
   `debugServer` no kit, o que o QEMU/OpenOCD disse antes de sair também
   vem na mensagem.
6. **O motor (core) caiu?** A IDE se recupera sozinha: a barra de status
   mostra "recuperando..." por um instante e reconecta ao mesmo projeto,
   mantendo suas abas abertas — sem reiniciar a janela. Se ele cair várias
   vezes seguidas, a recuperação pausa e a status bar avisa; aí veja o log
   de erros.

### O que reportar num bug

- O que você fez, o que esperava, o que aconteceu.
- O arquivo de log acima + a saída do terminal se houver.
- Distro, e se o projeto era Rust, C/C++, Python ou embarcado (qual placa).

---

## 12. Tabela-resumo de todos os atalhos

| Categoria | Atalho | Ação |
| --- | --- | --- |
| Arquivo | `Ctrl+S` | Salvar |
| Arquivo | `Ctrl+Shift+S` | Salvar tudo |
| Workspace | `Ctrl+O` | Abrir pasta |
| Edição | `Ctrl+D` | Duplicar linha/seleção |
| Edição | `Alt+Shift+↑/↓` | Mover linha(s) |
| Edição | `Ctrl+/` | Comentar/descomentar |
| Edição | `Ctrl+Y` | Deletar linha |
| Edição | `Ctrl+G` | Ir para linha |
| Edição | `Ctrl+F` / `Ctrl+H` | Localizar / substituir no arquivo |
| Edição | `F3` ou `Ctrl+Alt+G` | Próxima ocorrência (`Shift+` = anterior) |
| Edição | `Ctrl+W` / `Ctrl+Shift+W` | Expandir / encolher seleção |
| Edição | `Home` / `Shift+Home` | Início do texto ↔ coluna 0 |
| Edição | `Ctrl+Alt+L` | Formatar arquivo |
| Código | `Ctrl+B` | Ir para definição |
| Código | `Ctrl+Q` | Hover/documentação |
| Código | `Ctrl+Space` | Completar |
| Código | `Alt+F7` ou `Ctrl+Shift+U` | Encontrar usos |
| Código | `Shift+F6` ou `Ctrl+Shift+R` | Renomear símbolo |
| Código | `Alt+Enter` | Quick fixes / ações |
| Código | `Alt+O` | C/C++: alternar header/source |
| Código | `F2` / `Shift+F2` (ou `Ctrl+Alt+E` / `Ctrl+Alt+Shift+E`) | Próximo / anterior problema |
| IDE | `Ctrl+Alt+S` | Abrir configurações |
| Busca | `Ctrl+Shift+N` / `Ctrl+Shift+A` | Search Everywhere (`@` símbolos do arquivo, `#` do workspace) |
| Busca | `Ctrl+E` | Arquivos recentes |
| Busca | `Alt+7` | Aba Símbolos (estrutura do arquivo; declarações do projeto por nome) |
| Busca | `Ctrl+Shift+F` | Buscar nos arquivos |
| Busca | `Ctrl+Shift+H` | Substituir no projeto |
| Build | `Ctrl+F9` ou `Ctrl+Alt+B` | Build |
| Build | `Ctrl+Shift+F9` ou `Ctrl+Alt+T` | Testes |
| Build | `Ctrl+Shift+L` | Análise (clippy) |
| Debug | `Shift+F9` ou `Ctrl+Alt+D` | Iniciar debug |
| Debug | `F9` ou `Ctrl+Alt+C` | Continuar |
| Debug | `F8` ou `Ctrl+Alt+N` | Step over |
| Debug | `F7` ou `Ctrl+Alt+I` | Step into |
| Debug | `Shift+F8` ou `Ctrl+Alt+U` | Step out |
| Terminal | `Shift+F10` ou `Ctrl+Alt+R` | Executar projeto (aba ▶ no Terminal) |
| Terminal | `Ctrl+F2` ou `Ctrl+Alt+X` | Parar execução |
| Terminal | `Alt+F12` ou ``Ctrl+` `` | Terminal integrado (sessão Shell) |
| Ambiente | `Ctrl+Alt+K` | Bibliotecas |
| Ambiente | `Ctrl+Alt+P` | Ações de configuração |
| Ambiente | `Ctrl+Alt+J` | Banco de dados |
| Ambiente | `Ctrl+Alt+O` | Observabilidade (Grafana) |
| Ambiente | `Ctrl+Alt+M` | Embarcados |
| Ambiente | `Ctrl+Alt+W` | Containers |
| Ambiente | `Ctrl+Alt+H` | Instalar ferramentas |
