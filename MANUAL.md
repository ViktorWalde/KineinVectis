# Manual do Kinein Vectis

Guia completo para usar e testar a IDE depois que ela estiver aberta. Este
documento cobre os fluxos, recursos, atalhos e problemas encontrados durante o
uso.

> **O que é:** Kinein Vectis é uma IDE para C, C++, Rust, sistemas embarcados
> e simulação. Ela orquestra ferramentas maduras (clangd, rust-analyzer,
> cargo, CMake, clang-format...) em vez de reimplementá-las. Os fluxos da IDE
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

## 2. O layout

```text
┌──────────────────────────────────────────────────────────────┐
│ App Bar: Arquivo · Editar · Exibir · Navegar · Código ...    │
│ Toolbar: target · perfil · Configurar · Build · Run · Debug  │
├───┬───────────────┬─────────────────────────────┬────────────┤
│ R │ Projeto       │ Editor (abas + código)      │ Estrutura  │
│ a │ (árvore de    │                             │ (símbolos  │
│ i │  arquivos)    │                             │  do arquivo)│
│ l │               │                             │            │
├───┴───────────────┴─────────────────────────────┴────────────┤
│ Painel inferior: Build | Jobs | Problemas | Testes |         │
│                  Terminal | Debug | Git | Busca | IDE |      │
│                  Ferramentas                                 │
├──────────────────────────────────────────────────────────────┤
│ Barra de status: progresso de build/testes/análise + cancelar│
└──────────────────────────────────────────────────────────────┘
```

- **Rail** (coluna fininha à esquerda): liga/desliga Projeto, Busca, Git,
  Build, Debug e Ferramentas.
- Clicar numa aba do painel inferior que já está aberta **recolhe** o
  painel.
- Os painéis laterais e o painel inferior são **redimensionáveis**: arraste
  a borda entre eles e o editor.
- A aba **Estrutura** mostra os símbolos do arquivo, pode ser redimensionada e
  recolhida pelo botão próprio. Recolhida, vira uma aba estreita no centro da
  borda direita do editor; clicar nela restaura o painel. O primeiro layout é
  calculado pelo tamanho da janela e os ajustes posteriores ficam salvos.
- **Ajuda → Manual da IDE** abre este mesmo `MANUAL.md` numa visualização
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
| `Ctrl+Shift+F9` ou `Ctrl+Alt+T` | **Testes** (cargo test / ctest) | aba Testes |
| `Ctrl+Shift+L` | **Análise de qualidade** (clippy; Cppcheck + clang-tidy) | aba Problemas |

> Todos esses também são **botões na barra superior** — atalho nenhum é
> obrigatório. Onde houver tecla `F1`–`F12` há sempre uma alternativa sem
> ela (para notebooks onde F-keys exigem `Fn`).

Tudo roda em segundo plano como *job*: a barra de status mostra o
progresso e um **×** para cancelar; a aba **Jobs** guarda o histórico.
Clicar num problema abre o arquivo na linha exata.

### O que a análise de qualidade roda

Em projeto **Rust/Cargo**: `cargo clippy`.

Em projeto **C/C++ (CMake)**: dois analisadores no mesmo passo, porque eles
enxergam coisas diferentes. O **Cppcheck** lê o código sem precisar saber como
ele é compilado — funciona até antes de configurar o CMake. O **clang-tidy**
(Clang Static Analyzer) usa o comando de compilação real de cada arquivo, então
percebe problemas que dependem de macro, include e flag. Ele só entra depois de
o projeto ter sido configurado uma vez, e a análise avisa quando pulou.

Se o seu projeto tiver um `.clang-tidy` próprio, ele manda: a Kinein não
sobrepõe os seus checks. Se não tiver, o conjunto vem do perfil de rigor
configurado. Ter só uma das duas ferramentas instaladas não impede a análise —
a que existir roda.

### Análise dinâmica de memória

Em projeto C/C++ (CMake), o menu **Build → Análise dinâmica (memória)** roda os
**testes do projeto sob o Valgrind** e traz para a aba **Problemas** o que só
aparece com o programa rodando: escrita fora do bloco alocado, leitura de valor
não inicializado e vazamento de memória. Clicar leva à linha exata.

Acesso inválido aparece como **erro** e vazamento como **aviso** — ler fora do
bloco corrompe o programa agora, vazar é uma dívida que talvez nunca seja
cobrada. Exige `valgrind` instalado e o projeto configurado pelo menos uma vez;
sem isso, a análise diz o que falta em vez de falhar em silêncio.

> Esta é a única análise que **executa** o seu código. As outras leem. Se os
> seus testes tiverem efeitos colaterais, eles vão acontecer.

### Auditoria de segurança

Em projeto Rust, o menu **Build → Auditoria de segurança** verifica as suas
dependências contra a base de advisories do RustSec (`cargo-audit`). Os achados
aparecem na aba **Problemas**: vulnerabilidade como erro, pacote abandonado ou
retirado como aviso. Clicar leva ao `Cargo.lock`, na linha da dependência.

**A Kinein não acessa a internet por conta própria.** Por padrão a auditoria
roda offline, usando a cópia da base que já exista no seu computador — se não
houver nenhuma, ela diz isso em vez de tentar baixar. Para autorizar a
atualização pela rede, grave na configuração da integração `cargo-audit` a
chave `allowNetwork` com o valor `true`. Só esse valor exato autoriza.

A auditoria também verifica a **política de dependências** do seu projeto —
licenças aceitas, crates banidos, fontes permitidas — com o `cargo-deny`. Isso
só acontece se o seu projeto tiver um `deny.toml`: sem ele, a Kinein não impõe
uma política de licença que você nunca declarou. Essa parte é sempre offline, e
roda mesmo quando a verificação de vulnerabilidades não pode.

O resultado sempre informa **de quando é a base** e se ela foi atualizada
naquela execução. Auditoria com base velha apresentada como recente é pior que
auditoria nenhuma: ela tranquiliza sem motivo.

### Cobertura de testes

No painel **Testes** há o botão **Cobertura**, ao lado do título. Ele mede
quanto do código os testes tocaram e mostra o total no rodapé do painel, com a
lista por arquivo abaixo — verde acima de 80%, vermelho abaixo de 50%.

O que cada linguagem precisa:

| Projeto | Ferramenta | Como preparar |
| --- | --- | --- |
| Rust / Cargo | `cargo-llvm-cov` | `cargo install cargo-llvm-cov` |
| C / C++ / CMake | `lcov` | compilar o projeto **com `--coverage`** |

A Kinein **não** acrescenta `--coverage` ao seu build. Rigor de compilação é
decisão do projeto, não da IDE — a mesma regra vale para `-Werror`. Quando não
houver dados para medir, o painel diz **por quê** em vez de mostrar `0%`: um
zero silencioso seria indistinguível de um projeto sem nenhum teste.

**Projetos Rust:** além do build/clippy, há o **"Cargo: Check"** no Search
Everywhere — feedback de tipos/borrow bem mais rápido que o build, com os
problemas caindo na mesma aba Problemas. Se o `cargo metadata` do projeto
estiver quebrado (Cargo.toml inválido), o banner de Project Health avisa
com a mensagem do cargo e um botão para tentar de novo.

**Scripts do projeto:** arquivos `.sh`, `.bash` e `.zsh` mostram um botão de
executar ao passar o mouse na árvore. A mesma ação fica no clique direito como
**Executar script**. A saída abre na sessão **Execução** do Terminal e pode ser
interrompida pelo controle normal de Run. Só arquivos dentro do workspace são
aceitos; não é necessário abrir um terminal e digitar o caminho.

---

### 4.1 Depurar (debugger)

Funciona em C/C++ e Rust via `lldb-dap` (instalado junto com o lldb).

1. **Compile antes** (`Ctrl+F9`) — o debug usa o binário já compilado.
2. Clique na **gutter** (a coluna dos números de linha) para marcar
   breakpoints — a bolinha vermelha. Pode marcar antes mesmo de iniciar.
3. Clique em **[Debug]** na barra superior (ou `Shift+F9` /
   `Ctrl+Alt+D`). O programa
   roda e **para no primeiro breakpoint**: a linha fica destacada e o
   arquivo abre sozinho na posição.
4. Na aba **Debug** do painel inferior ficam os controles e a saída do
   programa.

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
Rust, ou de `.kinein/build` no CMake). Se houver mais de um, a mensagem
de erro lista os candidatos.

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
  contra o HEAD, com + e − coloridos. `Esc` fecha.

- **Aba Git** (painel inferior): tem duas vistas, alternáveis pelos
  chips no topo dela:
  - **Mudanças**: a lista de tudo que mudou. Clique no **quadradinho**
    para marcar/desmarcar o arquivo para commit (stage); clique no nome
    abre o arquivo; passando o mouse aparecem **diff** e **↩**
    (descartar — pede confirmação, porque não tem desfazer). Digite a
    mensagem embaixo e clique **Commit (N)** — commita só o que está
    marcado. "Git: Commit..." no Search Everywhere abre essa vista.
  - **Histórico**: a lista de commits (hash curto, resumo, autor e
    idade). Clique num commit para ver o diff dele. "Git: Historico" no
    Search Everywhere abre direto aqui; **atualizar** recarrega a lista.

- **Blame** (autoria por linha): rode **"Git: Blame do arquivo"** no
  Search Everywhere para ligar/desligar uma coluna ao lado dos números
  de linha mostrando **quem** mudou cada linha e **há quanto tempo**.
  Linhas ainda não commitadas aparecem como "não commitado". Acompanha o
  arquivo que você está editando.

O status atualiza sozinho ao salvar, criar, renomear ou excluir arquivos pela
IDE **e também** quando o watcher detecta mudanças externas (por exemplo, um
`git pull` no terminal). **"Git: Atualizar status"** no Search Everywhere
continua disponível para atualização manual.

Na aba Git também é possível listar/trocar/criar branches, fazer **Pull** e
**Push** como jobs (saída e cancelamento na aba Jobs) e guardar/restaurar o
trabalho com **Stash push/pop**. Checkout, criação de branch e stash são
recusados enquanto houver buffers sujos, evitando misturar estado do editor
com mudanças do worktree. Metadados `.kinein` nunca entram no stash.

---

## 5. Terminal (shell e execução)

A aba **Terminal** tem várias sessões de shell, alternáveis pelos chips no
topo, além da sessão separada **Execução**:

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
- **Execução**: o processo do seu projeto (`cargo run` ou o executável do
  CMake), com saída ao vivo e envio de entrada (stdin) pelo campo de baixo —
  digitar um comando ali com nada rodando executa esse comando.

**Configurações de execução:** na barra superior há um seletor (começa em
"Automático" — o menu abre por cima de tudo e fecha clicando fora). "Nova configuração..." salva um comando com nome (ex.:
`cargo run --bin servidor`); a configuração ativa é o que o ▶ executa. Dá
para editar/excluir a atual pelo mesmo menu, e a escolha fica salva com o
projeto.

| Atalho | Ação |
| --- | --- |
| `Shift+F10` ou `Ctrl+Alt+R` | Executa o projeto — abre a aba Terminal já na sessão **Execução** |
| `Ctrl+F2` ou `Ctrl+Alt+X` | Para o processo em execução |
| `Alt+F12` ou ``Ctrl+` `` | Abre a aba Terminal; cria o primeiro shell se necessário |

Enquanto um processo estiver rodando, a aba mostra **Terminal ●**. O botão
**limpar** zera a saída da sessão ativa (só dela).

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

## 9. Quando algo der errado

1. **Aba "IDE"** (painel inferior): fluxo interno da IDE ao vivo — o que
   foi pedido e o que respondeu.
2. **Log de erros persistente**:
   `~/.cache/kinein-vectis/logs/kinein-ui-erros.txt` — anexe este arquivo ao
   reportar um bug.
3. **LSP demorou?** A primeira indexação de um projeto grande leva tempo
   (especialmente rust-analyzer). O completion sintático local aparece antes;
   navegação, tipos e resultados semânticos completos chegam quando o servidor
   termina a primeira preparação.
4. **Ferramenta faltando?** Veja a aba Ferramentas e siga a sugestão exibida
   para o seu sistema.
5. **O motor (core) caiu?** A IDE se recupera sozinha: a barra de status
   mostra "recuperando..." por um instante e reconecta ao mesmo projeto,
   mantendo suas abas abertas — sem reiniciar a janela. Se ele cair várias
   vezes seguidas, a recuperação pausa e a status bar avisa; aí veja o log
   de erros.

### O que reportar num bug

- O que você fez, o que esperava, o que aconteceu.
- O arquivo de log acima + a saída do terminal se houver.
- Distro, e se o projeto era Rust ou C++.

---

## 10. Tabela-resumo de todos os atalhos

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
| Terminal | `Shift+F10` ou `Ctrl+Alt+R` | Executar projeto (sessão Execução) |
| Terminal | `Ctrl+F2` ou `Ctrl+Alt+X` | Parar execução |
| Terminal | `Alt+F12` ou ``Ctrl+` `` | Terminal integrado (sessão Shell) |
