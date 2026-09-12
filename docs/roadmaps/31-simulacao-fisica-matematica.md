# Simulação física/matemática por layout, com exibição em OpenGL

> **FORA DO FOCO desde 2026-09-12 — decisão do autor** (*"esquece a parte de
> simulação física/matemática; vamos refinar ao máximo para sistemas embarcados
> e desenvolvimento de software"*, registrada no
> [`40`](40-estado-e-continuidade.md) §5). O código do domínio
> `sim` e o `kinein-sim` ficam como estão e o gate continua a testá-los;
> nenhuma fatia nova sai daqui. Este documento permanece como registro para o
> dia em que o autor reabrir.

> **Classe: PLANO** (`docs/README.md`).
>
> **ATUALIZADO em 2026-09-05, e o cabeçalho antigo virou mentira nesse dia.** Ele
> dizia *"nenhuma das perguntas da §5 tem resposta hoje"*. **As sete estão
> respondidas**, a medição que as sustenta está nas §8 a §13, e a arquitetura que
> elas produzem está em
> [`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md).
>
> **O que continua verdade:** nenhuma linha de código foi escrita, e isto não
> cria fila — a fila é o `30-caminho-para-o-mvp.md`.
>
> **Como ler:** a §5 abaixo é o texto ORIGINAL das perguntas, preservado, com a
> resposta marcada em cada uma. As §8 a §13 são a medição e as decisões, em ordem
> cronológica. Se você só quer o desenho, vá direto para o `arquitetura/34`.
>
> **Pedido do autor, registrado em 2026-09-01**, ao fechar a etapa 2 do
> `30-caminho-para-o-mvp.md`:
>
> > *"para facilitar o uso e desenvolvimento de simulações físicas/matemáticas,
> > gostaria que fosse via layout a configuração e o inserimento das fórmulas
> > matemáticas, e a IDE fazer o cálculo com base no conceito matemático/físico
> > selecionado e a equação informada pelo usuário; e exibir via OpenGL a
> > simulação."*
>
> O pedido veio com a instrução explícita de **documentar como etapa futura a
> ser analisada e arquitetada**. Este documento cumpre isso e mais nada:
> registra o que foi pedido, o que já estava decidido, o que o pedido de hoje
> muda, e as perguntas que precisam de resposta **antes** de qualquer código.

## 1. Por que este documento existe agora, e não depois

Porque a §1.1 do `ARCHITECTURE.md` documenta o custo de descobrir tarde: uma
decisão de hoje pode fechar uma porta que o simulador vai precisar aberta. Já
aconteceu uma vez neste projeto — o AppImage forçar backend de software é uma
garantia conquistada que **colide** com o simulador, e isso só foi visto porque
alguém registrou (`arquitetura/27` §6.2).

Registrar cedo não obriga a construir cedo. Obriga a não fechar a porta por
acidente.

## 2. Onde isto entra na ordem — que já está decidida e não muda aqui

A ordem é do autor, registrada em 2026-07-16 e reafirmada no mesmo pedido:

```text
1. Solidificar C/C++ e Rust na IDE            <- e onde o projeto esta
2. Solidificar embarcados, de forma profissional
3. So entao simulacao fisica/matematica
```

Isso põe a simulação **depois** do MVP (`30-caminho-para-o-mvp.md`, cujas
etapas 3–10 são o passo 1) e **depois** da escada L1–L6 do
`28-plataforma-de-plugins-e-verticais.md` (`L6` são os embarcados). A spec de
fechamento já a classifica como não-MVP: *"simulador OpenGL"* está literalmente
na lista §11.2 de
`docs/specs/KINEIN_VECTIS_FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST.md`.

**Consequência prática:** este documento não compete por prioridade com nada. Se
uma sessão futura o ler como fila, está lendo errado — a fila é o
`30-caminho-para-o-mvp.md`.

## 3. O que JÁ estava registrado e continua valendo

`docs/arquitetura/27-modulos-por-dominio.md` §6 é a fonte, e não se reescreve
aqui. O resumo do que ela fixa, para não obrigar o leitor a abrir os dois:

- **A linha de responsabilidade**: detectar é capacidade (`ToolDetector`),
  ativar é política (settings + UI), executar é fronteira (fora do núcleo).
- **A colisão medida**: o AppImage roda com `QT_QUICK_BACKEND=software` de
  propósito, porque driver de host quebrava a abertura da IDE; isso só funciona
  porque **a UI hoje é 100% 2D** — zero `ShaderEffect`, zero
  `QQuickFramebufferObject`, zero OpenGL em `ui/`. O simulador precisa de GPU.
- **As duas saídas**, com recomendação: **(a)** simulador em processo próprio
  com contexto GL próprio, falando o mesmo JSON-RPC stdio — a IDE continua
  abrindo em qualquer máquina, e uma GPU quebrada derruba o simulador, não a
  IDE; **(b)** a UI inteira passa a hardware quando o simulador liga, mais
  simples de escrever e mais cara de manter.
- **Opt-in significa custo zero**: sem instância, sem thread, sem binding
  quando desligado. Fronteira, não `if simulador_ativo` espalhado.
- **Nome já escolhido**: nasce como `crates/kinein-core/src/sim/` e vira crate
  `kinein-sim` quando ganhar corpo.

## 4. O que o pedido de 2026-09-01 ACRESCENTA — e por que muda o desenho

A §6 do `arquitetura/27` tratava o simulador como **uma ferramenta que a IDE
detecta e lança**. O pedido de hoje descreve três responsabilidades, e só uma
delas é essa:

```text
AUTORIA    montar a simulacao POR LAYOUT, e digitar a formula ali
CALCULO    a IDE calcula, a partir do conceito selecionado + a equacao do usuario
EXIBICAO   a simulacao aparece em OpenGL
```

**"A IDE fazer o cálculo" está em tensão direta com a saída (a) da §6.3.** Se o
cálculo é da IDE, ele mora no core (nunca na UI — `ARCHITECTURE.md` §2 proíbe
regra de negócio na camada de apresentação). Se o desenho é de um processo
separado com GPU, então o resultado do cálculo precisa atravessar a fronteira a
cada passo de tempo — e o `arquitetura/04` §9 já mede o preço disso: *uma
serialização por mensagem*, hoje paga pelo `event.terminal.render` a ~30fps com
um grid de spans. Um campo de partículas a 60fps não é um grid de spans.

Essa tensão é o primeiro achado deste documento e a primeira pergunta da §5.
Ela não tem resposta óbvia, e é exatamente o tipo de decisão que fica cara se
for tomada por acidente dentro de uma fatia.

## 5. As perguntas que precisam de resposta antes de arquitetar

Cada uma está aqui porque a resposta **muda o desenho**, não porque é um detalhe
a preencher depois. Nenhuma tem resposta hoje.

### 5.1 Quem calcula, e onde o resultado é desenhado — RESPONDIDA em 2026-09-03

> **DECIDIDO: (ii) — processo `kinein-sim` separado calcula E desenha, e a IDE
> exibe o frame como IMAGEM 2D dentro do layout.** Visualmente embutido, GPU no
> outro processo, invariante do AppImage **intacto**. Detalhe em §5.1.1.

Três formas, com custos diferentes:

```text
(i)   core calcula, processo separado desenha
      -> o resultado atravessa IPC a cada passo. Mede-se antes de escolher.
(ii)  processo `kinein-sim` calcula E desenha; o core so orquestra
      -> preserva a garantia do AppImage; o "a IDE calcula" vira "a IDE
         orquestra quem calcula", que e o idioma do resto do projeto
(iii) a UI da IDE desenha (QQuickFramebufferObject/RHI) e o core calcula
      -> a mais direta visualmente, e a que devolve a IDE inteira a
         dependencia de driver (§6.2/6.3(b) do arquitetura/27)
```

**O que decide não é gosto: é medição.** Qual o volume de dados por frame de uma
simulação real do autor? Antes de escrever qualquer coisa, esse número existe ou
não existe.

### 5.1.1 Como "processo separado" e "visualmente embutido" convivem

Pergunta do autor em 2026-09-03: *dá para ser processo separado mas visualmente
dentro da IDE?* **Dá — e não pelo caminho óbvio.**

**O caminho óbvio está fechado.** Embutir a janela de outro processo é XEmbed,
que é mecanismo do X11 (reparenting). **Wayland rejeitou deliberadamente** um
equivalente; a orientação é "escreva um compositor embutido". As alternativas —
`wl_subsurface` e `xdg-foreign` — ou exigem que a IDE **seja** um compositor, ou
só definem parentesco entre janelas de topo, sem embutir num layout. O autor usa
Wayland (`XDG_SESSION_TYPE=wayland`, medido em 2026-09-03), então esse caminho
não serve nem em teoria nem na prática.

**O caminho que funciona:**

```text
kinein-sim (processo)            kinein-vectis (IDE)
  calcula                          recebe o frame PRONTO
  desenha OpenGL OFFSCREEN   -->   pinta como IMAGEM 2D no layout
  le o framebuffer                 sem GPU, sem QRhi, sem ShaderEffect
```

**Pintar imagem é 2D**, e por isso o invariante sobrevive: a GPU está no *outro*
processo. E a forma não é nova aqui — **é a do terminal**: o core computa a
grade, emite `event.terminal.render` a ~30fps, o QML desenha. "Backend calcula,
UI pinta 2D" é idioma existente.

**Os dois custos, ditos de frente:**

1. **O render não passa pelo JSON-RPC.** Um frame 1920×1080 RGBA é ~8 MB; a
   30fps, ~250 MB/s. A grade do terminal é minúscula em comparação. Isso exige
   um **segundo canal — memória compartilhada** — ao lado do protocolo de linha,
   e o `arquitetura/04` §1 diz que stdout é o canal de dados e nada mais escreve
   ali. É adição arquitetural real, não detalhe.
2. **Ler o framebuffer da GPU é um stall de pipeline** no processo da simulação.

**E o que muda o cálculo inteiro:** simulação física/matemática raramente precisa
de 60fps contínuo. Se o render for **sob demanda** — o usuário rotaciona, muda um
parâmetro, e só então redesenha —, o custo por frame deixa de importar e a
memória compartilhada pode nem ser necessária. **É esse o número que a medição
da §5.1 tem que produzir antes de escolher o transporte.**

### 5.2 O que é, exatamente, um "conceito matemático/físico selecionado" — RESPONDIDA em 2026-09-05

> **DECIDIDO: catálogo de DUAS CAMADAS** — o nome que o usuário conhece por
> cima, a forma matemática que a IDE resolve por baixo; e o usuário **digita a
> equação** dentro do conceito, com a IDE alertando na hora quando os dois não
> batem. Detalhe em §9.1 e §10.2.

Duas leituras, e elas produzem produtos diferentes:

```text
CATALOGO FECHADO   uma lista de conceitos nomeados (queda livre, MRU, oscilador
                   harmonico, pendulo, campo vetorial, EDO de 1a ordem, ...),
                   cada um com parametros declarados e um integrador proprio
MOTOR GENERICO     um solver que aceita qualquer sistema que o usuario descreva
```

O precedente deste repositório aponta para o **catálogo fechado**: é a forma das
Configuration Actions (16 ações declaradas, `catalog.rs`), e o anti-padrão que a
§8 do `ARCHITECTURE.md` proíbe é justamente *"mecanismo GENÉRICO sem usuário"* —
com a lição de 2026-07-17 anexada, quando o mecanismo sobreviveu ao único
chamador e teve de ser removido. Um catálogo pequeno com conceitos que o autor
realmente usa vale mais que um solver universal sem usuário.

Mas isso é uma **inclinação, não uma decisão**. Quem decide é o autor, e a
pergunta que responde é: *quais três simulações você faria na primeira semana?*

### 5.3 Como a equação do usuário vira cálculo — RESPONDIDA em 2026-09-05

> **DECIDIDO: os dois — interpretar (`exmex`) E compilar —, com a IDE estimando
> a escala e escolhendo.** O parser entra sempre, porque a checagem de conceito
> precisa da árvore da fórmula. Medição em §8.1 a §8.3, decisão em §10.3.

Aqui há uma armadilha e uma oportunidade.

**A armadilha:** escrever um parser + avaliador de expressões é reimplementar
algo — e "não reimplementa o que já existe consolidado" é a frase de uma linha
que define esta IDE (`LEITURA_TECNICA` §1). Adotar um crate de expressões é
possível, mas é adoção de componente: passa pelo `docs/integracoes/README.md`
(modos A–D, gate de auditoria), entra no
`docs/tooling/OPEN_COMPONENT_REGISTRY.json` e tem de passar no `deny.toml`
(licenças permissivas apenas).

**A oportunidade, e ela é o idioma da casa:** a IDE **já dirige um compilador**.
Gerar código a partir da equação, compilá-lo com o toolchain do usuário e rodar
como job é exatamente o que o projeto faz com `cmake`, `cargo` e `lldb-dap` —
sem motor de cálculo novo, com desempenho nativo, e com o compilador do usuário
como autoridade sobre a linguagem da equação. O custo é latência de compilação a
cada edição da fórmula, que é medível.

Nenhuma das duas está escolhida. **Ambas precisam da §5.2 respondida antes**: o
que se avalia depende do que se declara.

### 5.4 Onde a simulação mora no projeto do usuário — RESPONDIDA em 2026-09-05

> **DECIDIDO: `.kinein/simulacoes/`**, um arquivo por simulação, com
> `schemaVersion` — e o **resultado nunca no mesmo arquivo** (a lição do
> `.ipynb`). Detalhe em §9.4.

Uma simulação montada por layout é **conteúdo do usuário**, não configuração da
IDE: precisa ser versionável, diffável e sobreviver a uma troca de máquina. O
precedente existe (`.kinein/runconfigs.json` e `.kinein/settings.json`, ambos
com `schemaVersion`, ambos tratando arquivo inválido como vazio em vez de
quebrar), mas nenhum deles guarda conteúdo autoral.

Perguntas abertas: formato texto (diffável) ou binário? Dentro de `.kinein/`
(que hoje também guarda build dir e o SQLite de rascunhos) ou em pasta própria
do projeto? Um arquivo por simulação ou um catálogo?

### 5.5 Determinismo, unidades e precisão — RESPONDIDA em 2026-09-05

> **DECIDIDO: unidade é RÓTULO declarado, sem checagem dimensional** (§9.5) — o
> `uom` não entra, e a §8.4 explica por que ele não alcançaria uma fórmula
> digitada em runtime. E **"exato" não existe**: o que a IDE mostra é o erro,
> medido contra a solução fechada. Ver §11.

Uma simulação que não diz seu passo de integração, suas unidades e sua precisão
não é resultado — é animação. Antes da primeira linha: o método de integração é
escolha do usuário ou da IDE? Unidades são declaradas ou implícitas? Dois runs
com a mesma entrada produzem o mesmo resultado?

Isso não é rigor acadêmico: é a mesma regra que já vale no resto do projeto —
número sem procedência mente (`docs/README.md`), e ferramenta que mente é pior
que ferramenta ausente.

### 5.6 O risco de virar uma IDE dentro da IDE — RESPONDIDA em 2026-09-05

> **O autor é o primeiro usuário, e o que se paga é APRENDER** — não há
> incumbente a substituir, porque esta máquina não tem nenhuma ferramenta de
> simulação instalada (§8.6). Domínio: engenharia, com peso em aeroespacial e
> automotivo. Detalhe em §10.1.

Um editor visual de layout com campos, ligações e uma superfície de render é uma
frente de UI grande — comparável ao editor de texto, que já é o maior débito do
projeto (`EditorController.qml`, 1.070/400). Antes de começar, vale a pergunta
que o `ARCHITECTURE.md` §9 deixou registrada em sangue:

> *"rigor arquitetural não torna útil uma feature que não se paga. Antes da
> fatia, pergunte o que ela substitui e quanto isso custava."*

O seletor de agente de IA passou em todos os itens do checklist arquitetural e
foi removido no mesmo dia, porque o usuário já podia digitar `claude` no
terminal. A pergunta equivalente aqui: **o que o autor usa hoje para fazer essas
simulações, e o que exatamente dói nisso?** A resposta define o escopo mínimo
que se paga — e pode ser muito menor que "um simulador".

### 5.7 A ordem interna das três responsabilidades — RESPONDIDA em 2026-09-05

> **DECIDIDO: o passo 1 já inclui TELA** — fórmula com alerta ao vivo, gráfico, e
> o `kinein-sim` nasce nesta fatia. A escada "cálculo sem tela primeiro" abaixo
> está **superada** por esta decisão. Detalhe em §10.4.

Se a resposta da §5.6 for "vale a pena", a ordem de construção ainda é uma
decisão. A escada natural, do mais barato ao mais caro:

```text
1. CALCULO sem tela      um conceito, uma equacao, resultado em numeros/CSV,
                         testado como o resto do core e' testado
2. EXIBICAO 2D           o que a UI ja sabe desenhar hoje, sem tocar em GPU
                         nem na garantia do AppImage
3. LAYOUT                a superficie de autoria, so depois de 1 e 2 provarem
                         que o modelo de dados esta certo
4. OpenGL                so quando 1-3 existirem e o volume por frame (§5.1)
                         justificar
```

Isso é sugestão, não decisão — mas tem a forma que este repositório usa: a rede
de segurança antes do corte (2026-08-30), o teste antes da superfície.

## 6. Referência obrigatória quando a vez chegar

A §2.1 do `ARCHITECTURE.md` e a §2 do
`KINEIN_VECTIS_OPEN_PLUGIN_ADAPTATION_ROADMAP.md` valem aqui como valem para
qualquer feature: **estudar o que existe, registrar a revisão consultada, e
adaptar a invariante — nunca transplantar código, runtime ou modelo interno.**

Neste domínio as referências **não são IDEs**, e isso é uma diferença que muda a
pesquisa. Candidatos a estudar (nenhum auditado ainda; a lista é ponto de
partida, não adoção): Jupyter + matplotlib, GNU Octave, Scilab/Xcos,
OpenModelica, ParaView/VTK. O Simulink entra como referência **funcional
apenas**: é proprietário, e a lição do Pylance (`roadmaps/29`) vale igual —
licença que só serve ao produto de quem a escreveu não entra, nem como
dependência, nem como "inspiração de código".

## 7. O que este documento NÃO decide

```text
- NAO decide que a feature sera feita.
- NAO decide (i), (ii) ou (iii) da §5.1.
- NAO decide catalogo fechado vs motor generico (§5.2).
- NAO adota nenhum componente novo (§5.3) — adocao passa pelo docs/integracoes.
- NAO reabre a garantia do AppImage nem a ordem 1-2-3 da §2.
- NAO cria fila: a fila e o 30-caminho-para-o-mvp.md.
```

E o que ele **firma**: quando esta etapa for analisada, o ponto de partida são as
sete perguntas da §5, e a primeira coisa a produzir é uma **medição** — o volume
de dados por frame e a resposta de "o que dói hoje" —, não um diagrama.

## 8. A medição de 2026-09-05, antes de qualquer arquitetura

> A §7 fecha dizendo que *"a primeira coisa a produzir é uma **medição**"*. Esta
> seção é essa medição. Ela não decide nada — as decisões da §5 continuam sendo
> do autor. Ela troca **suposição por número** em cinco das sete perguntas, e em
> dois casos o número **contradiz o que estava escrito**.
>
> Máquina: AMD Ryzen 7 7735HS (8C/16T), 21 GiB RAM, Radeon 680M, Fedora 44,
> Wayland. `rustc` 1.96.1, `gcc` 16.2.1, `clang` 22.1.8.

### 8.0 O gate não estava verde ao abrir a sessão — e não era código

A regra zero mandou medir antes de aceitar estado, e a primeira coisa medida foi
o próprio gate. O [`40`](40-estado-e-continuidade.md) o registrava verde em
2026-09-04. Em 2026-09-05 ele **reprovava**, no último passo:

```text
ninja: error: '/usr/lib64/libQt6Quick.so.6.11.1', needed by 'ui/kinein-vectis',
       missing and no known rule to make it
```

A causa, medida:

```bash
rpm -q --qf "%{INSTALLTIME:date}\n" qt6-qtdeclarative   # sex 04 set 2026 22:21
```

```text
o build/dev-local guardava caminho ABSOLUTO   libQt6Quick.so.6.11.1
o sistema atualizou o Qt depois da medicao    libQt6Quick.so.6.11.2
```

**`cmake --preset <nome>` seguido de rebuild resolve** — confirmado, 401 alvos,
verde. Não houve mudança de código.

> **Resolvia o GATE, não a IDE — medido em 2026-09-11.** Reconfigurar troca o
> caminho da biblioteca e não invalida objeto nenhum: onze objetos compilados
> antes da troca de Qt ficaram no link (o rpm instala header com mtime de maio,
> e o ninja compara mtime), e o binário abortava ao abrir com o gate verde. A
> metade **silenciosa** desta falha nasceu gate, o 20º —
> [`40`](40-estado-e-continuidade.md) §7.7.

**E resolver um não resolve o outro.** Reconfigurado só o `dev-local`, o gate
avançou e reprovou de novo, no passo seguinte, com a mesma mensagem — porque
`build/dev-local-release` guardava a mesma referência velha. **Cada árvore de
build carrega o caminho absoluto por si**, e o `verificar.sh` usa duas.

O que isto ensina, e vale registrar: **o gate não é hermético contra atualização
de biblioteca do sistema**, e a mensagem que ele dá nesse caso é do `ninja`, não
da Kinein — ela diz o arquivo que falta, não diz *"reconfigure, o Qt do sistema
mudou"*. Não nasce gate daqui (a falha é ruidosa, não silenciosa — e a regra é
que gate nasce de falha silenciosa), mas nasce a observação de que
`docs/roadmaps/40` afirmando "gate verde" tem prazo de validade de uma
atualização de sistema.

### 8.1 §5.3, primeira metade: os crates de expressão contra o `deny.toml` REAL

Não contra a licença que o crates.io anuncia: contra o `cargo-deny` 0.20.2 deste
repositório, com o `deny.toml` de 2026-09-04, num projeto de teste isolado.

```text
crate       licenca declarada     +crates  ultimo release  deny.toml
exmex       MIT OR Apache-2.0       7      2026-05-23      PASSA
fasteval    MIT                     1      2020-01-25      PASSA
rsc         MIT                     2      2024-03-31      PASSA
meval       Unlicense/MIT           3      2018-09-30      PASSA
mexprp      MPL-2.0                12      2022-11-25      REPROVA
evalexpr    AGPL-3.0-only           1      2025-11-26      REPROVA
```

**O `mexprp` reprova por onde ninguém olharia.** A licença dele é MPL-2.0 — que
o `docs/integracoes/README.md` admite "após revisão". Não é a dele que reprova:
é a das **transitivas**. Ele puxa `rug` e `gmp-mpfr-sys`, que embrulham GMP e
MPFR:

```text
error[rejected]: LGPL-3.0-or-later   <- rug
error[rejected]: LGPL-3.0-or-later   <- gmp-mpfr-sys
```

**E o `evalexpr` é a lição do Grafana repetida dentro de um crate.** Ele é o
avaliador de expressões mais usado do ecossistema Rust — 9,96 milhões de
downloads, 2,11 milhões recentes, mais que os outros cinco somados. E:

```text
0.4.4    2019-03-15   MIT
12.0.0   2024-10-17   AGPL-3.0-only     <- trocou de licenca sob quem ja o usava
```

Quem adotou o `evalexpr` antes de outubro de 2024 e depois rodou
`cargo update` **passou a distribuir AGPL sem tomar decisão nenhuma**. É a forma
exata do risco que o `verificar-deny.sh` existe para pegar, e a prova de que
pegar licença **na transitiva e a cada build** não é zelo: é a única forma que
funciona. Fixar a licença uma vez, na adoção, não teria pegado isto.

Comando que reproduz (num projeto de teste com `license` e `publish = false`,
senão o próprio harness reprova como `unlicensed` e mascara o resultado):

```bash
cargo add evalexpr@13 && cargo deny check licenses
```

### 8.2 O `exmex` exercitado — quatro armadilhas que a documentação não conta

O `exmex` é o único candidato **mantido** que passa no `deny.toml` (release de
2026-05-23, contra 2020 do `fasteval` e 2018 do `meval`). Exercitado contra ele
mesmo, achou quatro coisas, e a primeira é grave.

**1. A ordem das variáveis é ALFABÉTICA, não a da fórmula.**

```text
"a*t + v0"         -> ["a", "t", "v0"]
"v0 + a*t"         -> ["a", "t", "v0"]     <- as duas dao a MESMA ordem
"k*x/m - c*v"      -> ["c", "k", "m", "v", "x"]
"theta + omega*t"  -> ["omega", "t", "theta"]
```

`eval()` recebe um **slice posicional**. Quem montar esse slice na ordem em que
leu a fórmula calcula a física errada **sem erro nenhum** — o comprimento bate,
os tipos batem, o número sai. É falha silenciosa, e é a categoria que este
repositório já pagou três vezes (`ARCHITECTURE` §8: *"ligar no lugar errado não
falha no build, deixa de funcionar em silêncio"*).

**A defesa existe e é do chamador:** `var_names()` devolve a ordem real, e o
vetor se monta por nome. Medido, dá o resultado certo:

```text
var_names() = ["c","k","m","v","x"]   ->  montado por nome  ->  Ok(5.0)
```

Se o `exmex` for adotado, **isso é gate por mutação**, não comentário: um teste
que troca duas variáveis de nome e exige que o resultado mude.

**2. `pi` minúsculo não é constante — é variável livre.**

```text
PI   -> constante 3.141592653589793
E    -> constante 2.718281828459045
e    -> constante 2.718281828459045
pi   -> VARIAVEL       tau -> VARIAVEL       inf -> VARIAVEL       nan -> VARIAVEL
sin(2*pi*t)  -> vars ["pi", "t"]
```

Um usuário que escreve `sin(2*pi*f*t)` — a forma como se escreve física — ganha
`pi` como incógnita a preencher. A IDE teria de pedir o valor de π.

**3. Divisão por zero é aceita, e vira `inf`/`NaN` calados.**

```text
1/x     com x=0    -> Ok(inf)
x/x     com x=0    -> Ok(NaN)
log(x)  com x=0    -> Ok(-inf)
sqrt(x) com x=-1   -> Ok(NaN)
```

Nenhum é erro. Uma simulação que divide por zero no passo 4.000 continua
rodando e entrega uma curva de `NaN`. Quem tem de decidir o que fazer com isso é
a IDE, não o crate — e §5.5 (*"simulação que não diz sua precisão é animação"*)
vira código aqui, não parágrafo.

**4. A mensagem de erro do parser não é mostrável ao usuário.**

```text
tamanho: 356 caracteres        contem "0x": SIM
"a binary operator cannot be on the right of another operator,
 Operator { repr: "*", bin_op: Some(BinOp { apply: 0x5638306fe1d0, prio: 2, ...
```

Um **endereço de ponteiro** dentro do texto do erro. A IDE não pode repassar
essa string; teria de classificar o erro e escrever a própria mensagem — que é o
que ela já faz com `secretRequired` e com o `code` do `requestFailed`.

Do lado bom, medido: derivada parcial funciona e confere numericamente
(`d/dx (x²y + sin x)` em (2,3) = 11.583853163452858, esperado 11.583853), e o
vocabulário de funções está completo para física — `sin cos tan asin acos atan
sinh cosh tanh exp log log2 log10 sqrt abs signum floor ceil round`.

**Detalhe de adoção que custou uma compilação para descobrir:** a derivada
parcial está atrás da feature `partial`, que **não** vem por padrão, e o método
não é inerente — é `exmex::Differentiate::partial(expr, i)`. A feature não traz
dependência nenhuma (`partial = []`), então os 7 crates da tabela continuam
valendo com ela ligada.

### 8.3 §5.3, segunda metade: interpretar contra compilar — o número que decide

A §5.3 põe duas saídas frente a frente e diz que o custo da segunda *"é latência
de compilação a cada edição da fórmula, que é medível"*. Foi medido.

**Oscilador harmônico amortecido**, `a = -(k/m)x - (c/m)v`, Euler
semi-implícito, 1.000.000 de passos, `dt = 1e-4`, `--release`:

```text
caminho               us/1M passos   vezes   resultado x
nativo (compilado)            4434    1.0x   -0.000003690
exmex                        33566    7.6x   -0.000003690
meval                        41503    9.4x   -0.000003690
fasteval                     55576   12.5x   -0.000003690
```

Os quatro dão **o mesmo resultado até a nona casa** — determinismo entre
avaliadores, confirmado, não suposto. (O `fasteval` sai por último por causa da
API: ela quer um `BTreeMap<String,f64>` por passo, e a alocação da chave domina.)

**E o outro prato da balança, nesta máquina:**

```text
gcc -O0    sim.c        42 ms      pico de RAM   28 MB
gcc -O2    sim.c        41 ms                    28 MB
clang -O2  sim.c        55 ms                    88 MB
rustc -O   sim.rs       66 ms                    94 MB
cargo build --release   490 ms     (1 arquivo mudou; e' o caminho REAL da IDE)
```

**O ponto de equilíbrio, e é ele que responde a §5.3:**

```text
interpretar 1M passos com exmex          33,5 ms
o mesmo, compilado                        4,4 ms
o que compilar ECONOMIZA                 29,1 ms por 1M passos
o que compilar CUSTA (gcc cru)           41   ms  -> empata em ~1,4M passos
o que compilar CUSTA (cargo, real)      490   ms  -> empata em ~17M passos
```

Uma simulação de pêndulo de 10 segundos com `dt = 1e-4` são **10⁵ passos**:
3,4 ms interpretando. Pelo caminho de compilar, a IDE pagaria **490 ms de espera
a cada edição da fórmula** para economizar 3 ms de conta.

**Compilar só ganha a partir de dezenas de milhões de passos** — e ali o gargalo
já não é a expressão. A §5.3 chama o caminho de compilar de *"o idioma da casa"*,
e é: mas o idioma da casa aqui **cobra 140× mais caro do que rende**, e a medição
é que diz isso, não gosto.

### 8.4 §5.5: o preço de unidades e de integrador, medido no mesmo gate

```text
crate         licenca               +crates   ultimo release   deny.toml
uom           Apache-2.0 OR MIT        5       2026-02-14      PASSA
ode_solvers   Apache-2.0              28       2026-06-07      PASSA
nalgebra      Apache-2.0              20       2026-05-24      PASSA
```

Base de comparação, medida: o workspace tem hoje **244 crates de terceiros**. O
`uom` custa +2%; `ode_solvers` custa +11% (e arrasta `nalgebra`).

E o que **já está na árvore** e um simulador usaria sem adotar nada:
`num-traits`, `num-conv`, `rand`, `rand_chacha`, `rand_core`, `typenum`.

**Mas o `uom` não faz o que a §5.5 parece pedir, e isso muda a pergunta.** Ele é
análise dimensional **em tempo de compilação, custo zero** — a checagem é do
`rustc`, pelo sistema de tipos. Verificado por mutação nesta máquina: somar
comprimento com tempo não compila.

```text
let d = Length::new::<meter>(100.0);
let t = Time::new::<second>(9.58);
let v: Velocity = d / t;      ->  10.438 m/s      (compila e confere)
let _erro = d + t;            ->  error[E0308]: mismatched types
```

**A consequência é estrutural:** uma fórmula que o usuário **digita em tempo de
execução** não tem tipo Rust nenhum. As dimensões de `k*x/m - c*v` só existem
depois que alguém disser o que são `k`, `x`, `m`, `c` e `v` — e isso acontece
depois do build. O `uom` protege **a física que a IDE escreveu**; ele não
alcança a que o usuário escreveu.

```text
catalogo fechado, equacao FIXA em Rust   -> uom serve, e serve bem
formula digitada pelo usuario            -> uom nao alcanca. Checar exigiria
                                            analise dimensional em RUNTIME, que
                                            e' mecanismo novo a escrever
```

Ou seja: **a §5.5 não é independente da §5.2 e da §5.3.** Responder "unidades
checadas" junto com "o usuário digita a fórmula" pede um mecanismo que não existe
em nenhum crate auditado aqui.

### 8.5 §5.1.1 revisitada: a premissa da memória compartilhada não sobreviveu

A §5.1.1 diz, sobre exibir o frame: *"Um frame 1920×1080 RGBA é ~8 MB; a 30fps,
~250 MB/s. Isso exige um **segundo canal — memória compartilhada**"*. O número
está certo **para frame cru**. Medido com o `serde_json` 1.0.150 e o `base64`
0.22.1 do próprio projeto:

```text
a grade do TERMINAL, que ja' atravessa o JSON-RPC hoje
   80x24      4.453 bytes   serializar  81 us     a 30fps =   0,13 MB/s
  200x50     15.134 bytes   serializar  94 us     a 30fps =   0,45 MB/s

um FRAME CRU pelo mesmo cano (base64 dentro do JSON)
   640x480    1,23 MB -> 1,64 MB (+33%)           a 30fps =  49,2 MB/s
  1920x1080   8,29 MB -> 11,06 MB (+33%)          a 30fps = 331,8 MB/s
              e so' serializar come 72,3% de um quadro de 33 ms
```

**Mas frame de simulação não é ruído** — é fundo liso com curvas em cima. Com
`zlib -6` sobre um quadro realista (fundo, grade, duas curvas do oscilador):

```text
                             cru      comprimido   razao    30fps
grafico de linha  640x480    1,23 MB    0,012 MB    99x    0,37 MB/s
grafico de linha 1920x1080   8,29 MB    0,047 MB   176x    1,42 MB/s
20k particulas   1280x720    3,69 MB    0,043 MB    86x    1,29 MB/s
200k particulas  1280x720    3,69 MB    0,169 MB    22x    5,08 MB/s
heatmap          1280x720    3,69 MB    0,448 MB     8x   13,44 MB/s   <- pior caso
```

**Um gráfico 1920×1080 comprimido a 30fps são 1,42 MB/s — três vezes a grade do
terminal, que o JSON-RPC já carrega hoje sem reclamar.** Mesmo o pior caso
realista (campo escalar contínuo) fica em 13,4 MB/s, longe dos 250.

A conclusão que a medição autoriza, e que **muda a §5.1.1**: a memória
compartilhada não é exigência do transporte — é exigência do **frame cru**, e
frame cru é uma escolha, não um dado. O gargalo passa a ser o **tempo de
comprimir** (14,7 ms a 1080p, 34 ms no heatmap, medidos no `zlib` do Python; em
Rust seria mais rápido, e isso ainda não foi medido). E se o render for **sob
demanda**, como a própria §5.1.1 previu, a pergunta some.

Isto **não reabre a §5.1** — o processo separado continua calculando e
desenhando, e a IDE continua pintando imagem 2D. Muda só o transporte, que a
§5.1.1 deixou explicitamente em aberto.

### 8.6 §5.6: o que esta máquina tem hoje para simular

A §5.6 pergunta *"o que o autor usa hoje para fazer essas simulações, e o que
exatamente dói nisso?"*. Metade disso é medível, e o resultado é seco:

```text
octave AUSENTE   scilab AUSENTE   gnuplot AUSENTE   julia AUSENTE   R AUSENTE
maxima AUSENTE   paraview AUSENTE   openmodelica/omc AUSENTE   freecad AUSENTE
python3 presente  ->  numpy 2.4.6 | scipy AUSENTE | matplotlib AUSENTE
                      sympy AUSENTE | pandas AUSENTE
```

**Nenhuma ferramenta de simulação instalada.** Isso não diz que a feature não se
paga — diz que **não há incumbente medido para comparar**, e que a outra metade
da §5.6 (*o que dói*) só o autor responde. É o oposto do caso do seletor de IA,
que foi removido porque o usuário já podia digitar `claude` no terminal: aqui
não há nada digitando.

### 8.7 O que o mercado faz, nas três perguntas que ele de fato responde

Consultado em 2026-09-05, e limitado ao que muda o desenho:

**§5.2 — catálogo ou motor genérico?** As duas formas existem e são produtos
diferentes. O **Modelica** é motor genérico: linguagem de equações não-causais,
achatada em constantes/variáveis/equações, ordenada, e resolvida por solver
simbólico e/ou numérico. O **Simulink/Xcos** é catálogo de blocos com escotilha
de fuga (o bloco de função). Nenhum dos dois é "o certo" — mas o Modelica é uma
**linguagem**, com especificação de centenas de páginas, e o catálogo não é.

**§5.3 — como a equação vira cálculo?** O OpenModelica **gera C e compila**. Vale
notar a licença, porque ela decide a forma como decidiu no Grafana: o compilador
é GPL-3 / OSMC-PL, o que o põe na mesma prateleira do GDB — **executável
orquestrado, nunca linkado** (MODE-A possível, MODE-C proibido).

**§5.4 — onde a simulação mora?** Aqui o mercado ensina por **fracasso**, e são
dois:

```text
Simulink .slx    zip de XML tratado como BINARIO. Git nao faz merge; o proprio
                 fabricante manda registrar a extensao como binaria e vende uma
                 ferramenta de diff/merge a parte (Simulink Report Generator).
Jupyter .ipynb   JSON com a SAIDA embutida junto do codigo. Diff ilegivel,
                 metadado que muda sozinho (execution_count), imagem em base64
                 dentro do diff, e conflito de merge que quebra o JSON a ponto
                 de o arquivo nao abrir mais. Nasceu o nbdime so' para remediar.
```

A lição das duas não é "use texto" — é mais estreita e mais útil: **o modelo
autorado e o resultado da execução não moram no mesmo arquivo.** O `.ipynb`
é texto e mesmo assim falha, porque mistura os dois.

### 8.8 O que esta medição NÃO mediu

```text
- compressao em RUST (o 8.5 usou zlib do Python; a ordem de grandeza vale,
  o tempo absoluto nao)
- custo de ler o framebuffer da GPU (o "stall de pipeline" da §5.1.1) —
  exige o processo kinein-sim existir
- o exmex sob formula GRANDE (as medidas usam expressao de 5 termos)
- quanto custa a fatia de UI da §5.6 — nao ha' o que medir antes de decidir
  a §5.2
- a outra metade da §5.6: o que doi hoje. Nao e' medivel aqui, e' do autor.
```

## 9. As decisões do autor em 2026-09-05

Tomadas depois da medição da §8, e cada uma fecha uma pergunta da §5.

### 9.1 §5.2 — RESPONDIDA, e a resposta não era nenhuma das três opções

> **DECIDIDO: catálogo AMPLO de conceitos, e o usuário digita a equação dentro
> do conceito que escolheu. A IDE confere uma coisa contra a outra NA HORA em
> que a fórmula é digitada, e avisa quando não batem.**

Nas palavras do autor:

> *"vai ter todos os conceitos, porem o usuario vai selecionar um conceito e dai
> sim colocar uma equacao para a IDE resolver que ta de acordo com o conceito
> pedido, caso o usuario erre a selecao de conceito, a IDE deve alertar na hora
> que o usuario inserir a formula. Mas deixe um aviso tambem que se ele errar a
> formula, ou o calculo nao vai ser feito, ou o resultado pode sair errado."*

**Isto não é "catálogo" nem "motor genérico" — é uma terceira forma, e ela é mais
exigente que as duas.** O catálogo não escolhe a conta: ele **declara o
contrato** que a fórmula do usuário tem de cumprir. O conceito diz quais
grandezas participam; a fórmula diz como. E a IDE é responsável por dizer,
enquanto se digita, que as duas discordam.

Isso produz três obrigações que nenhuma das opções descartadas produziria:

```text
1. o conceito DECLARA suas variaveis   exigidas e opcionais, nomeadas
2. a formula e' ANALISADA, nao so' avaliada   a IDE precisa saber quais
                                              variaveis a formula usa ANTES
                                              de calcular qualquer coisa
3. o alerta e' na DIGITACAO             nao no resultado, nao no botao de rodar
```

**E o aviso que o autor pediu explicitamente**, que é sobre o que a IDE *não*
consegue garantir:

> Conceito certo e fórmula sintaticamente válida **não** significam resultado
> certo. Uma fórmula errada dentro do conceito certo usa as variáveis certas e
> produz um número — e o número está errado. A IDE **não tem como saber**. Isso
> vai dito ao usuário, na tela, e não em nota de rodapé.

Este aviso é a mesma família do *"custo de leitura sempre VISÍVEL"* da etapa 27 e
do *"comando de instalação só com fonte oficial"*: **a IDE diz o que sabe e diz
o que não sabe.**

### 9.2 A medição que a decisão 9.1 obrigou, e ela mudou a §5.3

A decisão acima diz que a fórmula é **analisada antes de ser calculada**. Isso
foi exercitado contra o `exmex` real, com um catálogo de brinquedo de três
conceitos, e ele entrega o que a decisão pede — por `var_names()`:

```text
[queda livre        ] -g                    -> OK
[queda livre        ] -(k/m)*x              -> FALTA ["g"]          <- conceito errado
[oscilador          ] -(k/m)*x - (c/m)*v    -> OK
[oscilador          ] -(k/m)*x              -> OK                   <- `c` e' opcional
[oscilador          ] -g                    -> FALTA ["k","m","x"]  <- conceito errado
[oscilador          ] -(k/m)*x + F          -> ESTRANHA ["F"]       <- variavel nao declarada
[oscilador          ] -(k/m)*x - (c/m)*v +  -> NAO COMPILA
[MRU                ] v                     -> OK
```

**E ela custa 5,3 µs por checagem completa** (parse + comparação, média de 1.000
execuções). Checar **a cada tecla** é de graça.

**Efeito colateral medido, e é bom:** a checagem de conceito **desarma sozinha a
armadilha nº 2 da §8.2**, a do `pi`.

```text
[oscilador] -(k/m)*x*sin(2*pi*t)   -> ESTRANHA ["pi","t"]   <- o `pi` aparece
[oscilador] -(k/m)*x*sin(2*PI*t)   -> ESTRANHA ["t"]        <- com PI, so' o `t`
```

`pi` minúsculo é variável livre para o `exmex`; para a checagem de conceito ele é
**variável não declarada**, e vira alerta na tela em vez de incógnita silenciosa.
A decisão do autor tornou o defeito do crate visível de graça.

### 9.3 §5.3 — o que a decisão 9.1 faz com a escolha

**A pergunta mudou de forma.** A §5.3 comparava "adotar um crate de expressão"
*contra* "gerar código e compilar". Depois da 9.1, **os dois deixam de ser
alternativas**: para alertar o conceito errado na digitação, a IDE precisa da
**árvore da fórmula** — quais variáveis ela usa — e isso é o parser, não o
compilador. Um compilador não diz *"esta fórmula não é de oscilador"*; ele diz
`error:` ou nada.

```text
ANTES da 9.1   interpretar  OU  compilar
DEPOIS da 9.1  interpretar (obrigatorio)  E, talvez, compilar por cima
```

Compilar deixa de ser um caminho e vira uma **otimização opcional** do caminho
único. E aí a pergunta é só de escala. Medida:

```text
cenario                              avaliacoes    exmex   compilar(cargo)  ganha
pendulo 10 s, dt=1e-4                   100.000     3 ms      490 ms     interpretar
3 corpos acoplados 60 s, dt=1e-5     18.000.000   483 ms      554 ms     interpretar
campo 2D 200x200, 1.000 passos       40.000.000  1.048 ms     628 ms     COMPILAR
```

**O ponto de virada fica em ~21,5 milhões de avaliações** — `490 ms ÷ (26,2 ns −
3,45 ns)`, com os dois números medidos nesta máquina.

### 9.4 §5.4 — RESPONDIDA

> **DECIDIDO: `.kinein/simulacoes/`.**

Segue o precedente de `runconfigs.json` e `settings.json`: `schemaVersion`, e
arquivo inválido tratado como vazio em vez de quebrar.

**O que a decisão de LOCAL não decidiu, e continua valendo da §8.7:** o modelo
autorado e o resultado da execução **não moram no mesmo arquivo**. Essa é a lição
precisa do `.ipynb` — que é texto, é diffável, e mesmo assim falha, porque mistura
o que o usuário escreveu com o que a máquina produziu. Um arquivo por simulação
dentro de `.kinein/simulacoes/`; resultado, se for gravado, em outro lugar.

### 9.5 §5.5 — RESPONDIDA

> **DECIDIDO: unidade é RÓTULO declarado, sem checagem.**

A IDE mostra a unidade ao lado do campo, grava no arquivo e imprime no resultado
— e **não** verifica dimensão. O `uom` **não entra**, e a §8.4 explica por que
essa combinação seria incoerente de qualquer forma: o `uom` checa em tempo de
compilação, e a fórmula da decisão 9.1 só existe em tempo de execução.

O rótulo continua cumprindo o que a §5.5 exige de fato — *"uma simulação que não
diz suas unidades não é resultado, é animação"* —, sem prometer uma verificação
que não existe.

## 10. As decisões de 2026-09-05, segunda rodada — e o pedido que muda tudo

### 10.1 §5.6 — RESPONDIDA, e não por medição: pelo autor

A §5.6 perguntava *"o que o autor usa hoje para fazer essas simulações, e o que
exatamente dói nisso?"*. A §8.6 mediu a metade que era medível e achou **nenhuma
ferramenta de simulação instalada** nesta máquina. A outra metade veio do autor:

> *"eu vou ser o usuario desde ja, quero usar para aprender e visualizar essa
> parte, deve representar todos os conceitos de fisica e matematica dentro da
> engenharia que aborda e usa os conceitos, ainda mais voltado para aeroespacial
> ou automotivo. Desde o basico ate o avancado."*

**Não há incumbente porque não há uso hoje — o que se paga é APRENDER.** Isso é
diferente de toda frente anterior deste projeto, e muda o critério de sucesso:
não é "substituiu a ferramenta X"; é "o autor entendeu o conceito olhando".

**E dá dono ao `docs-privada/diario/19-registro-de-saidas.md`**, vazio desde que
nasceu: o autor passa a ser o primeiro usuário desta frente, e as frases de uso
dele são o que ordena o resto — como já foram a origem dos seis defeitos reais
da sessão anterior.

**Domínio declarado:** engenharia, com peso em **aeroespacial e automotivo**, do
básico ao avançado.

### 10.2 §5.2 — o catálogo tem DUAS CAMADAS

> **DECIDIDO: nome por cima, forma matemática por baixo.**

```text
CAMADA DE CIMA    o nome que o usuario conhece e escolhe
                  queda livre · MRU · MRUV · lancamento obliquo · pendulo
                  simples · oscilador amortecido · arrasto aerodinamico · ...

CAMADA DE BAIXO   a forma matematica que a IDE sabe resolver
                  equacao algebrica · EDO de 1a ordem · EDO de 2a ordem ·
                  sistema de EDOs
```

**N nomes sobre M formas.** É o que faz "todos os conceitos" ser possível sem
que o motor cresça: adicionar um conceito é uma **entrada de catálogo**, não
código novo. O motor fica fechado e pequeno; o catálogo cresce por cima dele.

E é a camada de cima que carrega as **variáveis declaradas** de que a checagem da
§9.1 depende — o conceito diz quais grandezas participam, a forma diz como se
resolve.

### 10.3 §5.3 — DECIDIDO: os dois, com a IDE estimando a escala

> **DECIDIDO pelo autor: interpretar E compilar, com a IDE estimando a escala e
> escolhendo.**

**Registro do que foi dito ANTES da escolha, porque este repositório muda de
regra por registro novo, nunca por silêncio:** esta opção foi apresentada com a
objeção anexada — construir dois mecanismos de execução, sendo que o segundo
nasce sem usuário medido, é o que a §8.1 do `ARCHITECTURE.md` proíbe
nominalmente (*"novo executor de processo"*), e a razão é que sistema duplicado
divide o lugar onde um bug pode estar. Os cenários medidos na §9.3 põem pêndulo
(10⁵) e três corpos acoplados (1,8×10⁷) do lado do interpretador; só o campo 2D
200×200 (4×10⁷) cruza o ponto de virada.

**O autor escolheu assim mesmo, com a objeção à vista.** A escolha vale, e a
objeção fica registrada aqui em vez de desaparecer — que é a diferença entre uma
decisão e um acidente. O domínio declarado na §10.1 (aeroespacial e automotivo,
*"até o avançado"*) é o argumento a favor dela: campo 2D e varredura de
parâmetros são o pão desses domínios, e ali o interpretador perde.

**O que a decisão obriga, e vai no desenho:**

```text
a estimativa e' VISIVEL     a IDE diz qual caminho vai tomar e por que, ANTES
                            de tomar — mesmo idioma do `$sample` da etapa 27
um caminho, dois motores    o contrato de execucao e' UM; interpretar e compilar
                            sao implementacoes dele, nao dois sistemas paralelos
o parser entra sempre       a checagem de conceito da §9.1 precisa da arvore da
                            formula, e compilar nao a fornece
```

### 10.4 §5.7 — a ordem: cálculo e checagem na TELA, juntos

> **DECIDIDO: o passo 1 inclui o campo de fórmula com o alerta ao vivo.**

O motivo é o da própria decisão da §9.1: *"a IDE deve alertar **na hora** que o
usuário inserir a fórmula"* é exigência de **interface**, e núcleo testado sem
tela não prova que ela acontece. O `roadmaps/40` registrava a etapa 28 como
*"cálculo sem tela"* — **essa linha está superada por esta decisão.**

### 10.5 O pedido que chegou por último, e é o maior

> *"mas quero que mostre a simulacao grafica, e exiba os resultados do calculo e
> todo o passo a passo feito para o usuario checar as respostas. E nesse caso
> deve ser pesquisado como e' feito cada conta corretamente para ser tudo exato
> e preciso."*

São **três exigências**, e a do meio não estava em nenhuma das sete perguntas da
§5:

```text
GRAFICO       a simulacao aparece desenhada
RESULTADO     os numeros do calculo
PASSO A PASSO todo o caminho da conta, para o usuario CONFERIR
```

**O passo a passo não é apresentação — é arquitetura.** Um avaliador que devolve
um número não tem como mostrar como chegou nele. Exibir a derivação exige que o
motor **saiba explicar**, e isso é uma capacidade que se decide antes, não um
painel que se acrescenta depois. É a mesma diferença entre um terminal que mostra
saída e um depurador que mostra estado.

**E "exato e preciso" tem uma tensão que precisa ser dita de frente**, medida na
§11: integração numérica **não é exata**, por definição. O que existe é erro
conhecido e declarado. A pesquisa pedida está na §11.

## 11. "Exato e preciso": a pesquisa pedida, e a medição que a acompanha

> Pedido do autor em 2026-09-05: *"deve ser pesquisado como e' feito cada conta
> corretamente para ser tudo exato e preciso."* Esta seção é essa pesquisa. Ela
> tem uma má notícia logo no começo, e é melhor que ela venha agora do que
> depois de escrito o simulador.

### 11.1 A má notícia, medida e não opinada: **exato não existe**

Integração numérica **não é exata por construção** — ela troca a equação
diferencial por uma aproximação, e a diferença é o erro de truncamento. O que
existe é **erro conhecido, limitado e declarado**.

Medido nesta máquina, no oscilador amortecido `m x'' + c x' + k x = 0` com
`m=2, c=0.5, k=10, x(0)=1, v(0)=0`, que **tem solução fechada**:

```text
x(10) pela solucao fechada = -0.275886266940653

dt          Euler explicito   Euler simpletico          RK4      passos
1e-1                 3.11e0            3.96e-2      4.91e-5         100
1e-2                7.54e-2            2.69e-3      6.88e-9       1.000
1e-3                6.72e-3            2.59e-4     7.06e-13      10.000
1e-4                6.64e-4            2.58e-5     4.33e-15     100.000
1e-5                6.63e-5            2.58e-6     2.00e-15   1.000.000
                (erro absoluto contra a solucao fechada)
```

**Leia a primeira célula.** Euler explícito com `dt=0.1` erra por **3,11** —
onde a resposta certa é **−0,276**. O erro é **onze vezes o valor da resposta**.
Não é "um pouco impreciso": é um gráfico bonito que não tem relação com a
física. É precisamente o que a §5.5 chama de **animação**.

E o mesmo problema com RK4 e `dt=0.01` erra por 6,9×10⁻⁹. **O método importa
mais que o passo:** RK4 com `dt=0.1` (100 passos) é mais exato que Euler com
`dt=1e-5` (um milhão de passos) — **por 3 ordens de grandeza, com 10.000 vezes
menos conta**.

### 11.2 E mais passos pode PIORAR — o piso do `f64`

```text
RK4 dt=1e-5  (      999.999 passos)   erro = 1.998e-15
RK4 dt=1e-6  (   10.000.000 passos)   erro = 5.995e-14     <- 30x PIOR
RK4 dt=1e-7  (  100.000.000 passos)   erro = 2.154e-13     <- 108x PIOR
```

Abaixo de certo ponto o erro **para de cair e volta a subir**: o acúmulo de
arredondamento em `f64` passa a dominar o erro de truncamento. Cada passo a mais
soma mais um arredondamento.

**Isso tem consequência direta na decisão da §10.3.** A IDE vai estimar escala
para escolher entre interpretar e compilar; essa mesma estimativa tem de saber
que **passo menor não é sempre melhor**, ou a IDE vai gastar 100 milhões de
avaliações para entregar um resultado 108× pior — e compilando, para ser rápida
nisso.

### 11.3 A ordem do método: verificada, e a armadilha de verificar errado

A ordem `p` de um método diz que dividir `dt` por 10 divide o erro por `10^p`.
Medido, sem afirmar:

```text
de 1e-1 para 1e-2:  erro 4.91e-5 -> 6.88e-9    ordem medida 3.85   confirmada
de 1e-2 para 1e-3:  erro 6.88e-9 -> 7.06e-13   ordem medida 3.99   confirmada
de 1e-3 para 1e-4:  erro 7.06e-13 -> 4.33e-15  ordem medida 2.21   CONTAMINADA
de 1e-4 para 1e-5:  erro 4.33e-15 -> 2.00e-15  ordem medida 0.34   CONTAMINADA
```

Euler explícito e simplético medem **1,00** — exatamente a ordem 1 que a teoria
dá.

**A armadilha:** medir a ordem do RK4 abaixo do piso do `f64` devolve **2,21** e
depois **0,34** — a ordem errada, sem aviso nenhum. Um teste que verificasse
"RK4 é ordem 4" escolhendo `dt=1e-4` e `dt=1e-5` **reprovaria um código
correto**. A faixa de medição é parte do teste, não detalhe.

### 11.4 Como a engenharia responde "esta conta está certa" — e ela tem norma

A resposta profissional não é "revisar a fórmula". É um procedimento com nome,
norma e ordem, e ele separa duas perguntas que costumam virar uma só:

```text
VERIFICACAO   as equacoes foram implementadas CORRETAMENTE no codigo?
              -> nao precisa de experimento. Compara-se com solucao exata, e
                 mede-se a ORDEM DE CONVERGENCIA. E' o que a 11.3 fez.
VALIDACAO     as equacoes descrevem a REALIDADE?
              -> exige dado experimental. Esta' FORA do alcance de uma IDE.
```

A referência é a **ASME V&V 20**, que trata verificação e validação em mecânica
computacional e quantifica o grau de exatidão comparando solução e dado num
ponto de validação, com as incertezas dos dois lados.

**E quando não existe solução fechada?** Existe procedimento para isso também: o
**Método das Soluções Manufaturadas (MMS)**. Escolhe-se uma solução arbitrária,
substitui-se ela na equação governante, e o resto vira um termo-fonte — de modo
que a solução escolhida satisfaça exatamente a equação modificada. Com refino
sistemático, produz verificação robusta e com ponto de término claro.

**Isto encaixa no idioma desta casa sem adaptação nenhuma.** "Gate provado por
mutação" e "estudo de ordem de convergência" são a mesma ideia: não se afirma
que o código está certo, mede-se. E dá o critério de entrada de cada conceito do
catálogo:

```text
conceito COM solucao fechada    o gate compara contra ela E mede a ordem, na
                                faixa de dt onde a ordem e' mensuravel (11.3)
conceito SEM solucao fechada    MMS: solucao manufaturada + termo-fonte
conceito que nao passa          NAO ENTRA no catalogo
```

É a mesma regra do catálogo de bibliotecas da §2.1 do `roadmaps/35`: **oferecer
é afirmar que serve.** Aqui, mostrar um passo a passo é afirmar que a conta é
aquela.

### 11.5 O que o mercado faz no passo a passo — e a armadilha que ele tem

Consultado em 2026-09-05. O Wolfram|Alpha é a referência do gênero, e a forma
como ele funciona tem um detalhe que **decide o desenho**:

> O motor calcula com os algoritmos rápidos e sofisticados dele; o **passo a
> passo é gerado à parte**, por heurísticas que seguem *"o caminho que um humano
> mais provavelmente tomaria"* — procurar substituição, integrar por partes, e
> assim por diante.

**Ou seja: no Wolfram, os passos exibidos NÃO são os passos executados.** São
uma segunda derivação, feita para ensinar, ao lado da primeira, feita para
responder.

Isso é ótimo pedagogicamente e é **exatamente o anti-padrão da §8.1** aplicado à
matemática: dois caminhos para a mesma verdade, e o dia em que discordarem, a
IDE mostra uma conta e entrega outro número. *"Sistema duplicado divide o lugar
onde um bug pode estar."*

O outro caminho — **os passos SÃO a execução** — é honesto por construção: o que
se vê é o que rodou. E ele mostra a trilha numérica (t, x, v, a a cada passo, os
`k1..k4` do RK4), não a álgebra. Menos bonito, impossível de mentir.

**Esta é a última pergunta em aberto da etapa 28**, e está na §12.

### 11.6 O que a decisão da §10.5 já obriga, independentemente da resposta

```text
o metodo aparece            "RK4, dt=1e-3" na tela, sempre. Numero sem
                            procedencia mente (docs/README)
o erro aparece quando da'    concepto com solucao fechada -> a IDE mostra o
                            valor exato E a diferenca. E' a unica forma
                            honesta da palavra "preciso"
o piso aparece              quando o dt pedido cai abaixo da faixa util, a IDE
                            diz que passo menor vai PIORAR — e nao obedece calada
a ordem e' gate             cada conceito do catalogo entra com estudo de
                            convergencia, na faixa de dt onde ele e' mensuravel
```

## 12. O passo a passo: as quatro saídas, com o custo de cada uma medido

A §11.5 mostrou que o Wolfram gera os passos **à parte** do cálculo. Para
decidir, faltava saber o que custa cada caminho **aqui**. Levantado em
2026-09-05.

### 12.1 (A) Os passos SÃO a execução — trilha numérica

O que a tela mostra é o que rodou: o método e o `dt` escolhidos, os valores a
cada passo (`t`, `x`, `v`, `a`), e para o RK4 os quatro estágios `k1..k4` de um
passo aberto por extenso. Onde existe solução fechada, o valor exato e a
diferença ao lado.

```text
custo de dependencia   ZERO. Sai do que o motor ja' calcula.
honestidade            por construcao: nao ha' segunda conta para discordar
o que NAO mostra       a algebra. Nao diz "isole x", diz "no passo 4.000, x
                       valia isto"
```

### 12.2 (B) Escrever a derivação simbólica na Kinein

Um motor de álgebra simbólica próprio: simplificação, substituição, regras de
derivação e integração, e as heurísticas de "como um humano resolveria".

**É reimplementar o que já existe consolidado** — a frase de uma linha que define
esta IDE (`LEITURA_TECNICA` §1). Décadas de trabalho existem em Maxima, SymPy e
Wolfram. **Descartado por regra registrada, não por gosto.**

### 12.3 (C) Adotar o `symbolica` — PROIBIDO, e é a lição do Pylance

O único CAS sério em Rust é o `symbolica` (2026-07-22, "blazing fast computer
algebra system"). A licença aparece no crates.io como **`non-standard`**, e o
texto dela diz o que isso significa:

> *"The source code of Symbolica is publicly available. It is not permitted to
> copy or distribute any part of the Symbolica code without express prior
> permission."* — com licenças **hobbyist**, **professional non-commercial** e
> **commercial** a adquirir.

**Código aberto para ler, proprietário para usar.** É exatamente a categoria do
Pylance, já **PROIBIDA** por decisão registrada (`roadmaps/29`): licença que só
serve ao produto de quem a escreveu não entra, nem como dependência, nem como
inspiração de código. E o `deny.toml` reprovaria de qualquer forma.

### 12.4 (D) Orquestrar um CAS externo como processo — o idioma do GDB

O projeto já executa ferramenta de licença copyleft **como processo, nunca
linkada** — é o que o `deny.toml` diz do GDB, que é GPL-3. Dois candidatos,
verificados na fonte em 2026-09-05:

```text
Maxima   GPL. "manipulacao de expressoes simbolicas e numericas, incluindo
         diferenciacao, integracao, series de Taylor, transformadas de Laplace,
         equacoes diferenciais ordinarias..." e "o programa maxima roda numa
         janela de terminal comum" -> executavel, MODE-A possivel
SymPy    BSD (OSI approved), versao 1.14.0. Biblioteca Python -> exigiria
         Python em runtime, que a decisao de 2026-07-16 adiou como linguagem
```

**Custo medido nesta máquina (§8.6): nenhum dos dois está instalado.** Entrar por
aqui significa a IDE detectar a ausência e explicar o que falta — que ela já sabe
fazer (`ToolDetector`, `setup.list` com fonte oficial datada), mas é fatia.

### 12.5 O que isto reduz a decisão a

```text
(B) descartado por regra   reimplementar consolidado
(C) descartado por regra   proprietario, categoria Pylance
```

Sobram **(A)**, **(D)**, e a ordem entre as duas.

## 13. O passo a passo simbólico não existe como o pedido supõe — medido

> **Esta seção corrige uma premissa da §12, e a correção é minha.** A §12.4
> ofereceu o Maxima como caminho para a álgebra do passo a passo. O autor
> escolheu essa opção em 2026-09-05. **Antes de escrever qualquer código,
> exercitar as duas ferramentas mostrou que a premissa estava errada.**

### 13.1 O Maxima **não mostra passos**, e a recusa é oficial

O pedido de funcionalidade nº 103 do projeto Maxima — *"Show step by step
solution"* — está marcado **"won't fix"**. A justificativa registrada é técnica,
não de prioridade:

> Exibir passos intermediários para funções como `ode2` é considerado
> **impossível de implementar sem reimplementar muitas das funções internas**;
> não dá para escrever uma função única que mostre os passos — **toda** função
> teria de ser modificada para imprimir os intermediários.

O `ode2` *"resolve a equação diferencial num único passo"*: faz a integração e a
resolução e entrega **a resposta final**, sem a derivação.

**Medido nesta máquina, o que existe é o pacote:**

```text
maxima 5.49.0-2.fc44   repositorio `fedora`   GPL-2.0-only
depende de: maxima-runtime, gnuplot, rlwrap, emacs-filesystem, hicolor-icon-theme
```

Instalável por fonte oficial, licença compatível com o idioma do GDB (executado,
nunca linkado). **Mas ele não entrega o que a opção prometia.**

### 13.2 O SymPy entrega mais — e mesmo assim não entrega a EDO passo a passo

Exercitado de verdade, SymPy 1.14.0 num venv desta máquina:

```text
1. dsolve na EDO do oscilador amortecido:
   x(t) = (0.0559892510955854*sin(2.23257138743647*t)
           + 1.0*cos(2.23257138743647*t)) * exp(-0.125*t)
   x(10) = -0.275886266940654

   A analitica derivada a mao na §11.1 deu  -0.275886266940653
   CONFEREM ate' a 14a casa (1 ulp de diferenca). Verificacao independente.

2. classify_ode diz QUAL metodo resolveu:
   ('factorable', 'nth_linear_constant_coeff_homogeneous',
    '2nd_power_series_ordinary')

3. passo a passo de EDO:  NAO EXISTE no SymPy 1.14.0

4. passo a passo de INTEGRAL: EXISTE e funciona (integral_steps):
   integral de x*exp(x)   -> regra `Parts`    -> x*exp(x) - exp(x)
   integral de sin(x)**2  -> regra `Rewrite`  -> x/2 - sin(2*x)/4
   Ele imita o que um estudante faria a mao, e e' o que o sympy_gamma formata.
```

### 13.3 O que isso deixa de pé, e é bastante

**Nenhuma ferramenta auditável entrega a derivação algébrica de uma EDO passo a
passo.** O Wolfram entrega — e o faz com heurísticas próprias, proprietárias, e
**por fora do motor que calcula** (§11.5). Construir isso é a saída (B) da §12.2,
proibida por regra.

O que as duas ferramentas **de fato** entregam, e que é muito para o objetivo
declarado de *aprender e visualizar*:

```text
A RESPOSTA EXATA        a solucao fechada da EDO que o usuario escreveu — nao
                        so' dos conceitos que a IDE codificou a mao. Isso
                        transforma o "exato e preciso" da §11 de promessa em
                        mecanismo: para QUALQUER formula que o SymPy resolva,
                        a IDE tem o valor verdadeiro para comparar
O NOME DO METODO        `nth_linear_constant_coeff_homogeneous` — a IDE diz por
                        que a equacao e' desse tipo e qual tecnica a resolve
A DERIVACAO, em INTEGRAL a unica parte onde o passo a passo estilo humano existe
                        aberto e auditado
A TRILHA DE EXECUCAO    (A) da §12.1, que nao depende de ferramenta nenhuma
```

**Isto reordena o valor do CAS externo.** Ele não é o "motor do passo a passo" —
ele é o **oráculo de exatidão**. E nessa função ele vale mais do que valeria como
narrador: sem ele, a IDE só sabe o erro dos conceitos cuja solução fechada
alguém escreveu à mão; com ele, sabe o erro de qualquer equação que o usuário
digitar.

### 13.4 SymPy contra Maxima, agora que a função mudou

```text                 SymPy 1.14.0              Maxima 5.49.0
licenca                 BSD (OSI)                 GPL-2.0-only
forma                   biblioteca Python         binario de terminal
solucao fechada         sim (dsolve)              sim (ode2)
nome do metodo          sim (classify_ode)        parcial
passo a passo integral  SIM (integral_steps)      nao ("won't fix")
nesta maquina           AUSENTE                   AUSENTE (no repo fedora)
custo de runtime        exige Python              binario + gnuplot + rlwrap
```

**O SymPy ganha em licença e em capacidade.** O que ele cobra é **Python em
runtime** — e há uma decisão registrada de 2026-07-16 que diz *"Python: adiado"*.

**Essa decisão não é obviamente violada aqui, e a diferença precisa ser dita:**
ela trata Python como **linguagem que a IDE suporta** (LSP, debug, build). Usar
uma ferramenta *escrita* em Python como processo orquestrado é outra coisa — a
IDE já orquestra o GDB sem "suportar C++ o GDB". Mas é uma dependência de runtime
nova, e quem decide é o autor.

## 14. A trilha não cabe na RAM — medido em 2026-09-05

A decisão da §12/§13 põe a **trilha do que rodou** na tela. Trilha na tela é
trilha **guardada**, e isso tem preço. Medido com pico de RSS real
(`/proc/self/status`, `VmHWM`), um passo sendo `(t, x, v, a)` em `f64` = 32 bytes:

```text
cenario                        passos       trilha    pico RSS medido    tempo
pendulo 10 s, dt=1e-4         100.000       3,2 MB           5 MB         2 ms
1 milhao de passos          1.000.000      32,0 MB          33 MB        19 ms
3 corpos 60 s, dt=1e-5     18.000.000     576,0 MB         552 MB       326 ms
campo 2D, 1.000 passos     40.000.000   1.280,0 MB       1.223 MB       724 ms
```

**1,28 GB para uma simulação**, numa máquina de 21 GiB — sem o frame, sem a UI, e
sem uma segunda simulação aberta. E a ironia importa: o campo 2D é justamente o
cenário que a §9.3 manda **compilar**. **O caminho rápido é o que estoura a
memória.**

**Amostrar resolve o tamanho:**

```text
guardar   1.000 pontos de 40.000.000 (1 a cada 40.000) =  0,03 MB
guardar  10.000 pontos de 40.000.000 (1 a cada  4.000) =  0,32 MB
guardar 100.000 pontos de 40.000.000 (1 a cada    400) =  3,20 MB
```

**E cria um problema novo, que é o de sempre nesta casa:** uma tabela amostrada
**parece completa**. Se o defeito estiver no passo 3.987 e a IDE guardou 1 a cada
4.000, a trilha não o contém e nada avisa. É a forma exata do `$sample` da etapa
27, e a resposta é a mesma: **a amostragem aparece na tela**, e existe um jeito
de pedir a janela em resolução cheia — que se resolve **re-executando o trecho**,
como um depurador, em vez de guardar tudo.

**Isso promove o determinismo de pergunta a requisito.** A §5.5 perguntava se
dois runs com a mesma entrada dão o mesmo resultado; com a janela reproduzida sob
demanda, **se não derem, a IDE mostra outra simulação e diz que é a mesma.** A
§8.3 mediu que os quatro avaliadores concordam até a nona casa, então determinismo
está disponível — mas ele deixa de ser propriedade feliz e passa a ser condição.

O desenho que sai disso está em
[`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md) §7.1.

## 15. "Tudo": a grade inteira medida contra o SymPy, em 2026-09-05

> Pedido do autor: *"quero os conceitos de matematica/fisica basica, I, II, III e
> IV. Ou seja tudo."* Antes de prometer, foi medido — o SymPy 1.14.0 exercitado
> tópico a tópico, nesta máquina.

### 15.1 O Cálculo inteiro passa

```text
CALCULO I     limite sin(x)/x           OK      derivada x^3*e^x        OK
              integral definida         OK      integral por partes     OK
CALCULO II    serie de Taylor           OK      soma de 1/n^2 = pi^2/6  OK
              integral impropria        OK      EDO 1a ordem            OK
CALCULO III   derivada parcial          OK      gradiente               OK
              integral dupla e tripla   OK      divergente e rotacional OK
              jacobiano                 OK
CALCULO IV    EDO 2a ordem              OK      sistema de EDOs         OK
              transformada de Laplace   OK      Laplace inversa         OK
              serie de Fourier          OK      EDP de 1a ordem simples OK
```

### 15.2 A física passa — até bater na EDP

```text
FISICA I      queda com arrasto (EDO 1a)      OK
              oscilador forcado (EDO 2a)      OK
              modulo `mechanics` (Lagrange)   EXISTE
FISICA II     lei dos gases (algebrica)       OK
              equacao da ONDA 1D              FALHA  NotImplementedError
FISICA III    circuito RLC (EDO 2a)           OK
              campo de Coulomb (integral)     OK
              EDP de LAPLACE 2D               FALHA  NotImplementedError
FISICA IV     matriz de lente fina (optics)   OK
              fator de Lorentz                OK
              Ket/Bra (quantum)               OK
              SCHRODINGER 1D                  FALHA  NotImplementedError
```

**E o SymPy traz mais módulo de física do que se esperava**, o que engorda a
camada de cima do catálogo sem engordar o motor:

```text
biomechanics · continuum_mechanics · control · hep · hydrogen · matrices
mechanics · optics · paulialgebra · pring · qho_1d · quantum · secondquant
sho · units · vector · wigner
```

O **`control`** merece nota: é exatamente o vocabulário de aeroespacial e
automotivo que o autor declarou na §10.1.

### 15.3 A fronteira do escopo é a EDP, e ela é dura

**Todas as quatro EDPs de verdade falharam** — onda, Laplace, calor, Schrödinger.
O `pdsolve` do SymPy resolve EDP de 1ª ordem simples e para aí. **E isso não é
defeito do SymPy: EDP não tem solução fechada em geral.** Resolver EDP é
discretizar o espaço (diferenças finitas, elementos finitos) e integrar — outro
motor, com malha, condição de contorno e estabilidade.

```text
o que as QUATRO formas da arquitetura/34 §4.1 cobrem
  ALGEBRICA · EDO_1 · EDO_2 · SISTEMA_EDO
  -> Fisica I inteira, boa parte da II e da III, Calculo I a IV

o que elas NAO cobrem, e e' onde "tudo" esbarra
  EDP: onda, calor, Laplace/Poisson, Schrodinger
  -> a parte de CAMPO da Fisica II, III e IV
```

### 15.4 O raciocínio existe para **integrais**, e só

```text
integral x*e^x       -> regra `Parts`     (integracao por partes)
integral sin(x)^3    -> regra `Rewrite`
integral 1/(x^2+1)   -> regra `Arctan`
integral log(x)      -> regra `Parts`
passo a passo de EDO    -> NAO EXISTE
passo a passo de serie  -> NAO EXISTE
```

**Esta é a resposta precisa à pergunta do autor** (*"se dá para mostrar apenas o
resultado e não o raciocínio..."*): **depende do tópico, e a divisão é limpa.**
Em integral, o raciocínio estilo estudante existe e é auditado. Em todo o resto,
só o resultado. A IDE mostra o que tem e **diz** onde não tem.

### 15.5 Duas armadilhas do oráculo, medidas

**1. O `dsolve` quebra com coeficiente `float`** — numa EDO que qualquer aluno de
Física I resolve:

```text
u'' + 4.905*u = 0   com float          -> RecursionError
u'' + Rational(981,200)*u = 0          -> OK
u'' + 5*u = 0       com inteiro        -> OK
```

**A IDE tem de racionalizar os coeficientes ANTES de perguntar ao oráculo.** Isso
não é detalhe de implementação: sem isso, o oráculo falha justamente nos números
que um usuário digita (`9.81`, `0.5`).

**2. A API ingênua de dimensão MENTE:**

```text
get_dimensional_expr(100*meter + 9.58*second)  ->  length
```

Ela devolve `length` para uma soma **inválida** — pegou o primeiro termo e calou.
Quem validar por ela valida nada. A função correta é `check_dimensions`, que
recusa com `addends have incompatible dimensions`.

### 15.6 E a surpresa que mexe com a §5.5: o SymPy checa dimensão em RUNTIME

A §8.4 mediu que o `uom` **não alcança** fórmula digitada pelo usuário, porque
checa em tempo de compilação. Com base nisso o autor decidiu, em 2026-09-05,
**rótulo sem checagem** (§9.5). **A medição de agora muda a premissa:** o
`check_dimensions` do SymPy — que já está entrando como oráculo — checa em tempo
de **execução**, que é quando a fórmula do usuário existe.

```text
F = m*a       aceitou    ok
E = m*c^2     aceitou    ok
m + v         RECUSOU    ok
t + x         RECUSOU    ok
F = m*v       aceitou    <- dimensionalmente COERENTE, fisicamente ERRADO
```

> **REVISADO em 2026-09-06 (§19.3.6): a última linha desta tabela está
> errada.** Medido, `check_dimensions(newton - kilogram*meter/second)`
> **RECUSA** — `F = m·v` é dimensionalmente incoerente (N contra kg·m/s), e a
> checagem o pega. A tese continua de pé; o exemplo precisa de uma fórmula que
> **passe**, como `E = m·v²` sem o meio. E `check_dimensions` não é importável
> de `sympy.physics.units` na 1.14.0: ela mora em `.util`.

**O limite honesto está na última linha:** `F = m*v` passa na checagem
dimensional e continua sendo física errada. Checagem de unidade pega incoerência,
**nunca** pega fórmula errada — e isso reforça, não substitui, o aviso da §9.1.

## 16. A EDP entra na etapa — e o que ela cobra, medido em 2026-09-05

> **DECIDIDO pelo autor: a EDP entra como quinta forma, já nesta etapa.** A
> objeção foi apresentada antes (o motor de EDP é maior que os outros quatro
> somados) e ele escolheu assim mesmo. Esta seção mede o que a escolha cobra,
> para o desenho não descobrir tarde.

### 16.1 A condição de estabilidade é uma parede, e ela explode em silêncio

> **COMPLETADO em 2026-09-06 (§19.2.1): esta tabela não registrou a condição
> inicial, e ela decide o número.** Repetido com quatro iniciais, `r=0,510` em
> 500 passos dá `2,9e+6` com degrau ou pico — que é a ordem da tabela abaixo — e
> `2,99e-1` com uma gaussiana, que **parece física** e está a 911 passos de
> virar 1e+19. A parede é a mesma; o que muda é **quando** ela aparece. Isso
> reforça o gate: não dá para detectar instabilidade olhando a saída de uma
> corrida curta.

Calor 1D explícito, `nx=101`, `α=1.0`, 500 passos. O parâmetro é `r = α·dt/dx²`,
e o limite teórico é `r ≤ 0,5`:

```text
dt            r        max|u| depois     veredito
2.000e-5    0.200          5.203e-1      estavel
4.000e-5    0.400          3.827e-1      estavel
5.000e-5    0.500          3.449e-1      estavel      <- o limite exato
5.100e-5    0.510          1.492e+6      EXPLODIU     <- 2% acima
6.000e-5    0.600         6.021e+70      EXPLODIU
1.000e-4    1.000        2.391e+236      EXPLODIU
```

**Dois por cento a mais no `dt` transforma 0,34 em 1.500.000.** A solução certa é
uma gaussiana que decai e nunca passa de 1. Não é "impreciso" — é resultado sem
relação com física nenhuma, e **o código não avisa**: ele entrega o número.

**Isto é o gate da quinta forma**, e ele não é opcional: a IDE calcula o critério
de estabilidade **antes** de rodar, e recusa ou corrige o `dt` em vez de obedecer.
É a mesma família do aviso de piso da §11.2, com a diferença de que aqui o erro
não é grande — é infinito.

### 16.2 Refinar o espaço custa ao QUADRADO

`dt` máximo cai com `dx²`. Dobrar a malha **quadruplica** os passos:

```text
malha       dx        dt maximo   passos p/ 1 s   atualizacoes
51x51     2.00e-2      1.00e-4          12.500        3.25e7
101x101   1.00e-2      2.50e-5          50.000        5.10e8
201x201   5.00e-3      6.25e-6         200.000        8.08e9
401x401   2.50e-3      1.56e-6         800.000        1.29e11
801x801   1.25e-3      3.91e-7       3.200.000        2.05e12
```

### 16.3 O tempo real, medido nesta máquina

> **CORRIGIDO em 2026-09-06 (§19.2.2): a coluna "interpretado" desta seção usa
> fator 7,6x, e o medido é 90,3x.** O `0,5 ns` do núcleo fixo **reproduz**
> (0,45–0,47 ns); o que não reproduz é o custo da fórmula do usuário no `exmex`,
> medido em **44,6 ns por célula**. `201x201` por 1 s de física são **6 minutos**
> interpretados, não 29,5 s; `801x801` são **25,4 h**, não 2 h. A regra "EDP
> sempre compila" deixa de ser preferência e vira **pré-requisito** — e o motor
> compilado não existe.

Calor 2D explícito, `--release`. Medido: **~0,5 ns por atualização de célula**.

```text
malha       celulas   passos   atualizacoes    tempo    ns/celula
51x51         2.601    2.000        5.20e6       4 ms       0.70
101x101      10.201    4.000        4.08e7      20 ms       0.49
201x201      40.401    8.000        3.23e8     161 ms       0.50
401x401     160.801    4.000        6.43e8     309 ms       0.48
```

E o que isso dá para **1 segundo de tempo físico**:

```text
malha        atualizacoes     compilado    interpretado (7,6x)
51x51              3.25e7        0,0 s              0,1 s
101x101            5.10e8        0,2 s              1,9 s
201x201            8.08e9        3,9 s             29,5 s
401x401            1.29e11      61,9 s            470,1 s   (~8 min)
801x801            2.05e12     987,2 s           7502,7 s   (~2 h)
```

**Três regras caem daqui, e nenhuma é opinião:**

1. **EDP SEMPRE compila.** O menor caso 2D útil (3,25×10⁷ atualizações) já está
   acima do ponto de virada de 21,5 milhões medido na §9.3. A estimativa da §10.3
   não precisa nem pensar: para a forma EDP, o motor compilado é o único.
2. **O tamanho da malha aparece com o custo ANTES de rodar.** 201×201 são 4
   segundos; 801×801 são duas horas. Idioma do `$sample` da etapa 27.
3. **A malha grande é onde o interpretador vira inviável**, não só lento: 8
   minutos contra 1 minuto em 401×401.

### 16.4 A trilha da EDP não pode existir como a das EDOs

```text
malha      1 instante        TODOS os passos de 1 s
101x101      0,08 MB                    4,1 GB
201x201      0,32 MB                   64,6 GB
401x401      1,29 MB                1.029,1 GB
801x801      5,13 MB               16.425,0 GB
```

**Guardar a história completa de uma EDP 2D é impossível por ordem de grandeza,
não por aperto** — 16 TB para uma simulação de um segundo numa malha de 801².

A §14 já tinha decidido que a trilha é amostrada; aqui isso deixa de ser economia
e vira **a única forma possível**. Para a EDP, a "trilha" é **quadro amostrado**,
e o painel de cálculo mostra a conta de **uma célula** num instante, não a
história de todas.

## 17. O segundo caminho: o usuário escreve o código — medido em 2026-09-05

> Pedido do autor: *"o usuario vai desenvolver o codigo tambem e executar ele
> dai exibir em 3d... eu gostaria de facilitar e deixar as duas opcoes... o que
> pode de fato ser feito dentro do possivel para auxiliar o usuario e nao
> atrapalhar ou ser invasivo?"*

### 17.1 Regra zero: metade disso JÁ EXISTE, e a medição encurtou a frente

```bash
grep -rhoE '"(run|runConfig|job)\.[a-zA-Z.]+"' crates/kinein-core/src/handlers/ | sort -u
grep -rhoE '"event\.(run|job)[a-zA-Z.]*"' crates/kinein-core/src/ | sort -u
```

```text
run.start · run.stop · run.stdin · run.script
runConfig.save · runConfig.list · runConfig.delete · runConfig.setActive
job.cancel · job.list
event.run.started · event.run.output · event.run.finished
event.job.created · event.job.output · event.job.progress · event.job.finished
fswatch.rs  — observador com debounce de 180 ms (notify, ADR-0001)
```

**"O usuário escreve C++ e clica para executar" já funciona hoje.** Compilar,
rodar, capturar a saída, cancelar — está tudo de pé. **O que falta é um pedaço
só:** a saída do programa dele chegar à vista 3D.

Isso muda o tamanho da frente: não é "construir execução de código do usuário",
é "ligar a saída que já atravessa a fronteira a um desenho".

### 17.2 O que o programa do usuário custa para produzir a saída

Programa real de órbita de dois corpos, escrito como um usuário escreveria, sem
biblioteca nenhuma da IDE, compilado com `g++ -O2` desta máquina:

```text
saida            1M passos      bytes        tempo do PROGRAMA
texto "t x y z"  1.000.000    45,98 MB           512 ms
binario (4 f64)  1.000.000    32,00 MB            28 ms      <- 18x mais rapido
```

**O `printf` custa mais que a física.** Para uma simulação de um milhão de passos,
imprimir texto é meio segundo do tempo do usuário — um imposto de 18× sobre a
alternativa binária.

### 17.3 Como a saída chega à tela — três contratos, medidos

Sobre os 46 MB reais gerados acima:

```text
1. PARSEAR o texto no core           96 ms   481 MB/s, 96 ns por linha
2. mandar tudo por event.run.output  38 ms   47,21 MB, 5.613 eventos de 8 KB
3. mandar tudo como pontos          286 ms   47,98 MB de JSON
```

**E o que a vista 3D de fato precisa:**

```text
 1.000 pontos (1 a cada 1.000) = 0,05 MB    0,2 ms
10.000 pontos (1 a cada   100) = 0,46 MB    1,6 ms
50.000 pontos (1 a cada    20) = 2,34 MB    7,9 ms
```

**Uma tela de 1920 px não mostra mais que ~2.000 pontos distinguíveis numa
curva.** Mandar um milhão é mandar 500× o que o olho resolve.

**O desenho que cai daqui:** o core **parseia e amostra**; só a amostra atravessa
para a UI. É a mesma divisão do terminal — o core computa a grade, o QML desenha —
e ela custa 1,6 ms em vez de 286 ms.

### 17.4 A IDE consegue RECONHECER a saída sozinha? Medido contra ferramenta real

A opção menos invasiva de todas seria a IDE reconhecer a trajetória sem o usuário
declarar nada. O risco é o falso positivo: confundir log de build com dado. A
heurística ingênua — *"é trajetória se a maioria das linhas for só números, com 3
ou mais colunas"* — testada contra a saída **real** das ferramentas que o usuário
roda todo dia nesta IDE:

```text
arquivo      linhas   % so-numeros   veredito
traj.txt       5.000        100,0%   TRAJETORIA   <- certo
cmake.txt          2          0,0%   nao          <- certo
cargo.txt         10          0,0%   nao          <- certo
gcc.txt           36          0,0%   nao          <- certo
ping.txt          10          0,0%   nao          <- certo
df.txt            13          0,0%   nao          <- certo
tabela.txt        12          0,0%   nao          <- certo (df -k, tabela numerica)
misto.txt      5.002         99,96%  TRAJETORIA   <- certo, com log no meio
```

**Zero falso positivo contra `cmake`, `cargo`, `g++`, `ping` e `df`.** E o caso
misto — simulação que também imprime log — continua sendo reconhecido, porque as
linhas de texto são minoria.

**Mas reconhecer não é o mesmo que saber.** A heurística diz *"isto parece
trajetória"*; ela não sabe se a terceira coluna é `z` ou é energia. **Adivinhar a
SEMÂNTICA é onde a IDE mentiria** — e a saída disso é a de sempre nesta casa: a
IDE propõe o que achou, mostra em que se baseou, e o usuário confirma ou corrige.

## 18. "Nada adivinhado": o princípio que revogou duas decisões do mesmo dia

> Autor, 2026-09-05, depois de todas as decisões acima:
>
> > *"nada deve ser adivinhado[.] o usuario mesmo que selecionando visualmente o
> > conceito e inserindo a formula, tudo deve ser detalhado e selecionado de
> > forma explicita."*

Este princípio é **superior às outras decisões**, e o registro de que ele revogou
duas tomadas horas antes fica aqui, porque decisão que muda tem de aparecer:

```text
REVOGADA   "a IDE ESTIMA a escala e ESCOLHE entre interpretar e compilar"
NOVA       a IDE MOSTRA o custo dos dois; quem escolhe e' o usuario

REVOGADA   na EDP, "a IDE recusa OU CORRIGE o dt instavel"
NOVA       a IDE RECUSA e MOSTRA o limite. Nunca altera um numero seu

REVISTO    a checagem de conceito casava formula e conceito pelo NOME da
           variavel — que e' deducao. Agora o usuario LIGA cada variavel a'
           grandeza que ela e'

CONFIRMADO campo comeca VAZIO. Nao ha' valor pre-preenchido nem sugerido:
           metodo, dt, motor e amostragem sao preenchidos pelo usuario, e a
           simulacao nao parte com eles em branco
```

### 18.1 O argumento que foi recusado, registrado porque era bom

Havia um argumento honesto para a IDE escolher o motor sozinha: **a escolha do
motor não muda o resultado**, só o tempo — a §8.3 mediu que os avaliadores
concordam até a nona casa. Escolher motor é diferente de escolher método de
integração, que muda o número.

**O autor recusou a distinção.** O ganho de recusá-la é ter **um critério só**
(*"a IDE mostra, você escolhe"*) em vez de uma fronteira entre "muda o resultado"
e "muda só o tempo" que alguém teria de manter — e fronteira mal mantida é onde a
exceção vira regra sem ninguém decidir.

### 18.2 O que o princípio dá de graça, e não era o objetivo dele

A ligação explícita de variáveis foi decidida por coerência, e ela **mata por
desenho as duas piores armadilhas do `exmex`** medidas na §8.2:

```text
ordem ALFABETICA   o vetor de avaliacao passa a ser montado pela LIGACAO. A
                   falha silenciosa — casar por ordem de leitura e calcular a
                   fisica errada sem erro nenhum — deixa de ser possivel, em
                   vez de depender de quem escreve o codigo lembrar
`pi` minusculo     vira variavel sem papel, que pede ligacao e nao tem a que
                   ligar. Aparece como pergunta em vez de incognita livre
```

E o passo **não custa uma tela nova**: é o mesmo lugar onde a unidade é
declarada, que a §15.6 já tinha posto no desenho por outro motivo.

### 18.3 O que fica pendente de uma frase sua

**A vista 2D/3D.** Você decidiu, antes de enunciar o princípio, que *"o conceito
declara e o usuário pode trocar"*. Isso deixa a vista como o único campo que
começa **preenchido**. A justificativa para manter é que ela não muda o
resultado — só o desenho —, e que o valor vem do catálogo, que é declaração
auditada e não palpite. **Mas é uma exceção ao princípio, e ela está aqui em vez
de escondida.** Se você quiser a vista também em branco, é uma frase.

## 19. As três candidatas da etapa 28, MEDIDAS antes da escolha — 2026-09-06

> **Pedido do autor, 2026-09-06:** *"PROXIMO: escolha minha, e quero as opções
> medidas antes."* Esta seção é a medição, e só ela: nenhuma linha de código de
> forma nova foi escrita. Cada número abaixo saiu de ferramenta real nesta
> máquina, e cada um traz o comando que o reproduz.
>
> **Estado no momento da medição, confirmado por `bash scripts/verificar.sh`
> (verde, 2026-09-06):** protocolo 0.85.0, 639 testes Rust, 28 harnesses QML, 18
> verificações, 130 métodos IPC, 36 eventos, catraca com 1 arquivo em débito.

### 19.0 O que a medição achou primeiro, e não estava na lista

> **CONSERTADO em 2026-09-10**, e a medição abaixo virou o teste que o prova.
> O conserto **não foi esconder o número** — foi dar-lhe procedência
> (`SimAccuracySource`). Com o oráculo presente, a coluna passa a responder pela
> equação DIGITADA; sem ele, a solução do conceito continua na tela **dizendo
> que é ela**.
>
> Exercitado contra o binário real em 2026-09-10, com SymPy 1.14.0 numa venv:
>
> ```text
> formula digitada          exato agora        proced.    erro abs     erro rel
> -(k/m)*x - (c/m)*v          0.032128320      oracle    3.732e-06    1.161e-04
> (k/m)*x - (c/m)*v       83178.343751029      oracle    1.474e+00    1.773e-05
> -(k/m)*x - 2*(c/m)*v        0.006879277      oracle    3.222e-07    4.684e-05
> -(k/m)*x*x*x - (c/m)*v      0.032128320     concept    8.634e-02    2.687e+00
> ```
>
> **As três primeiras linhas batem com a coluna "erro REAL" da tabela abaixo**,
> que foi medida com o `dsolve` resolvendo à mão o que tinha sido digitado. A
> mentira de 78.000x acabou. A quarta é a resposta certa do §19.3.5 acontecendo:
> o `dsolve` diz `NotImplementedError`, a IDE cai na solução do conceito **e
> avisa**.
>
> **O segundo achado desta seção também entrou:** o erro relativo vai à tela ao
> lado do absoluto. Repare na segunda linha — `1,474` de erro absoluto sobre um
> valor de 83 mil é `1,8e-5` relativo, uma integração excelente; a mesma
> tela mostrava só o absoluto e deixava a conta para quem lê.
>
> Custo medido: 425–506 ms por corrida, dos quais ~200 ms são o `import sympy`.
> Uma ida por FÓRMULA, nunca por ponto.

**A IDE mostrava um "valor exato" que podia ser de OUTRA equação.** Medido
contra o binário real por stdio, em 2026-09-06:

```text
conceito escolhido: "oscilador amortecido"   k=2  m=1  c=0,5  y0=1  dy0=0
metodo RK4, dt=0,1, t=10 — e todas as variaveis LIGADAS, checker aprovando

formula digitada            check.ok   numerico      "exato" da IDE   "erro" na tela
-(k/m)*x - (c/m)*v            true      0,032132        0,032128       3,73e-06
(k/m)*x - (c/m)*v             true     83176,869269     0,032128       8,32e+04
-(k/m)*x*x*x - (c/m)*v        true      0,118467        0,032128       8,63e-02
-(k/m)*x - 2*(c/m)*v          true      0,006880        0,032128       2,52e-02
```

A coluna `"exato"` **não muda**, porque ela vem de `catalogo::exata(conceito.id)`
e **não olha a fórmula que o usuário escreveu** (`sim/corrida.rs`, `fn exatidao`).
O `sim.checkFormula` aprova as quatro: ele confere ligação, não física — como a
`../arquitetura/34` §5.1 diz que ele faz.

**O tamanho do estrago, medido com o `dsolve` resolvendo o que foi de fato
digitado:**

```text
formula digitada            "erro" que a IDE mostra    erro REAL da integracao
-(k/m)*x - (c/m)*v                 3,732e-06                 3,732e-06   (igual)
(k/m)*x - (c/m)*v                  8,318e+04                 1,474e+00
-(k/m)*x - 2*(c/m)*v               2,525e-02                 3,222e-07   (78.000x)
```

**A terceira linha é a pior:** a IDE acusa erro de 2,5e-2 numa integração que
está certa até 3,2e-7 — **setenta e oito mil vezes** o que ela reporta. Ela culpa
o integrador por uma divergência que é da própria pergunta, e o número tem cara
de resultado.

Isto **não é a advertência da §5.1**, que fala do *resultado*. Aqui é a coluna
`exato` que afirma o que a IDE não sabe. A regra da casa (`docs/README.md`:
número sem procedência mente) está sendo quebrada pela própria coluna que existe
para dar procedência.

**E ela chega à tela com esse nome.** O `SimRunResultView.qml` escreve três
linhas — `valor exato`, `calculado` e `erro` — e a primeira é a que responde por
outra equação.

**Um segundo achado, menor, na mesma medição:** o erro é só **absoluto**. Na
linha do sinal trocado, `1,474` sobre um valor de `83.178` é erro relativo de
1,8e-5 — uma integração excelente —, e um erro absoluto de `1,474` ao lado de um
resultado de `0,032` seria catástrofe. A mesma tela mostra os dois números sem
distinguir, e quem lê tem de fazer a conta de cabeça.

**Não corrigido em 2026-09-06, e de propósito na época:** o conserto era decisão
de desenho, não de código. O catálogo **não guarda a fórmula** por decisão registrada
(`../arquitetura/34` §4.2), então a IDE não tem como saber que a fórmula digitada
deixou de ser a canônica — a menos que alguém resolva a equação digitada, que é
exatamente a candidata 19.3.

```bash
# reproduz: exercita sim.checkFormula e sim.run contra target/release/kinein-core
python3 scripts/exercitar-sim-oraculo.py
```

### 19.1 Candidata A — SISTEMA_EDO

**O que existe hoje:** `SimForm::OdeSystem` está declarado no protocolo e
**nenhum conceito do catálogo o usa**. Medido no binário, 2026-09-06:

```text
17 conceitos:  algebraic 14 · ode2 2 · ode1 1 · odeSystem 0 · pde 0
integraveis:   oscilador-amortecido, queda-livre, decaimento-exponencial
```

A forma é uma promessa do protocolo sem motor atrás.

#### 19.1.1 Por que ela muda o produto: a órbita, medida

Órbita circular (`mu=1`, `Y = [x, y, vx, vy]`, início `x=1, vy=1`), cujo raio
verdadeiro é **exatamente 1** em qualquer instante. Medido em 2026-09-06:

```text
metodo                 dt    voltas    raio final    deriva de energia
Euler explicito      0,01        10      1,647957            2,032e-01
Euler explicito     0,001        10      1,110807            5,057e-02
Euler explicito     0,001       100      1,686773            2,030e-01
Euler simpletico     0,01        10      1,000024            2,800e-10
Euler simpletico    0,001        10      1,000000            1,057e-13
Euler simpletico    0,001       100      1,000000            1,627e-13
Runge-Kutta 4        0,01        10      1,000000            8,727e-11
Runge-Kutta 4       0,001       100      1,000000            7,105e-14
```

**O Euler explícito transforma um círculo de raio 1 num de raio 1,65 em dez
voltas** — 65% de erro numa órbita que deveria fechar. O simplético, com o mesmo
passo, erra 2,4e-5 e conserva energia oito ordens de grandeza melhor.

**Isto dá função nova a uma escolha que já existe.** Hoje o `SimMethod` do
usuário decide entre 3,11 e 4,91e-5 de erro num oscilador (§11.1); no sistema ele
decide se a órbita **fecha ou não**. A tela não muda; a consequência sim.

#### 19.1.2 A armadilha de verificar a ordem no lugar errado — medida

A ordem de convergência medida **no raio da órbita circular**, em 2026-09-06:

```text
metodo                ordem medida (dt: 0,02 -> 0,01 -> 0,005 -> 0,0025)
Euler explicito       0,98    0,98    0,99      <- bate com o teorico (1)
Euler simpletico      1,32    1,09    3,21      <- ruidosa
Runge-Kutta 4         5,00    4,99    5,30      <- ordem 5 num metodo de ordem 4
```

**O RK4 mede 5, e ele é de ordem 4.** O motivo é que a órbita circular é uma
solução especial e o raio é uma grandeza que cancela parte do erro. É a mesma
família da armadilha da §11.3 — lá era a *faixa* de `dt`, aqui é a *grandeza
medida* —, e a consequência é a mesma: **o estudo de convergência do
`SISTEMA_EDO` tem de declarar em que grandeza ele mede**, ou aprova código errado
e reprova código certo sem ninguém notar.

#### 19.1.3 O custo: o motor de hoje BASTA

`exmex` interpretado, quatro fórmulas (uma por componente), RK4 de quatro
estágios = 16 avaliações por passo. Medido em 2026-09-06:

```text
584 ns por passo de RK4 em 4 dimensoes   (36,5 ns por avaliacao)

orbita 10 voltas, dt=1e-3          62.832 passos  ->  0,04 s
orbita 100 voltas, dt=1e-4      6.283.185 passos  ->  3,67 s
pendulo duplo 60 s, dt=1e-5     6.000.000 passos  ->  3,50 s
```

**Nenhum desses casos passa do ponto de virada de 21,5 milhões de avaliações**
(§9.3) de forma que justifique compilar. O `SISTEMA_EDO` roda no motor
interpretado que já existe — **ele não arrasta subsistema nenhum**.

#### 19.1.4 O pêndulo duplo não tem oráculo, e dá para DIZER a partir de quando

Duas corridas que diferem por 1e-12 no ângulo inicial, RK4 com `dt=1e-5`. Medido
em 2026-09-06:

```text
inicial           t=0      t=10     t=20      t=25      t=30
theta=2,0 rad   1,0e-12   1,2e-12  1,7e-12   7,9e-12   5,2e-12   <- NAO caotico
theta=3,0 rad   1,0e-12   1,3e-11  1,5e-09   1,6e-08   9,1e-08   <- caotico
```

**O pêndulo duplo só é caótico acima de uma energia**, e isso é medido, não
citado. No regime caótico o expoente é ~0,38/s: uma incerteza de 1e-16 — que é o
piso do `f64`, não escolha de ninguém — vira 1e-2 em ~85 s.

**Isso é uma coisa nova que a IDE pode HONESTAMENTE mostrar**, e que nenhuma das
outras candidatas oferece: em vez de "não há solução fechada, não sei o erro",
ela diz **até quando** a trajetória significa alguma coisa. É o idioma do custo
visível da etapa 27, aplicado ao tempo.

### 19.2 Candidata B — o motor de EDP

> A decisão de que a EDP **entra na etapa** é do autor (2026-09-05) e não se
> reabre. O que se mede aqui é a **ordem dentro da etapa**, e o que ela cobra.

#### 19.2.1 A parede da estabilidade é real — e o §16.1 não é reproduzível como está

Calor 1D, `nx=101`, `alpha=1`. A §16.1 mediu `r=0,510` → `max|u| = 1,492e+6` em
500 passos, **sem dizer de que condição inicial partiu**. Repetindo em
2026-09-06, com quatro iniciais:

```text
inicial        r      passos   max|u| ao fim   explodiu no passo
gaussiana    0,500       500        3,013e-01          -
gaussiana    0,510       500        2,986e-01          -        <- parece estavel!
gaussiana    0,510      2000        2,271e+19        911
seno         0,510       500        7,775e-01          -        <- parece estavel!
seno         0,510      2000        1,193e+17       1041
degrau       0,510       500        2,936e+06        164
pico         0,510       500        5,917e+06        146
gaussiana    0,600       500        4,233e+58        101
```

**A parede continua de pé, e a lição fica pior.** Com `r=0,510` e uma inicial
suave, 500 passos entregam `0,2986` — um número que **parece física** e está a
911 passos de virar 1e+19. A §16.1 mediu o caso `degrau`/`pico`; com `gaussiana`
o mesmo `r` passa despercebido.

```text
o que a §16.1 dizia    r > 0,5 explode, e o codigo nao avisa
o que se mede agora    r > 0,5 explode SEMPRE, mas QUANDO depende do espectro
                       da condicao inicial: 146 passos com um pico, 911 com uma
                       gaussiana. Uma corrida curta instavel entrega numero
                       plausivel e NAO explode
```

**Isto reforça o gate em vez de amolecê-lo:** não dá para detectar instabilidade
olhando a saída. O critério tem de ser calculado **antes**, do jeito que a
`../arquitetura/34` §4.1 já exige. E a §16.1 precisa da inicial escrita ao lado
do número — número sem procedência de novo, desta vez dentro do próprio
documento que criou a regra.

#### 19.2.2 O custo por célula reproduz — o fator do interpretado NÃO

Calor 2D explícito, `--release`, 201×201. Medido em 2026-09-06:

```text
nucleo fixo em Rust, indexacao direta     0,47 ns/celula
nucleo fixo em Rust, por fatias           0,45 ns/celula
a §16.3 documentou                        0,50 ns/celula      <- reproduz
```

Mas a mesma conta **escrita pelo usuário e interpretada** pelo `exmex`
(`m + a*(n + s + l + o - 4*m)`, seis variáveis):

```text
interpretado pelo exmex                  44,61 ns/celula
RAZAO interpretado / compilado            90,3x
a §16.3 usou                               7,6x
```

**A §16.3 subestima o interpretado por doze vezes.** A tabela dela refeita com
90,3x:

```text
malha      atualizacoes   nucleo fixo   interpretado (§16.3)   interpretado (medido)
51x51            3,25e7        0,0 s              0,1 s                    1,4 s
101x101          5,10e8        0,3 s              1,9 s                   22,8 s
201x201          8,08e9        4,0 s             29,5 s                  6,0 min
401x401          1,29e11      63,7 s              8 min                   1,6 h
801x801          2,05e12      16,9 min            2 h                    25,4 h
```

**A consequência é dura e é o custo real desta candidata:** a regra "EDP SEMPRE
COMPILA" da §16.3 deixa de ser preferência e vira **pré-requisito**, porque
`201x201` por 1 segundo de física são **6 minutos** no motor que existe hoje.

**E o motor compilado NÃO EXISTE.** Verificado em 2026-09-06: `crates/kinein-core/src/sim/`
tem `catalogo`, `corrida`, `formula`, `integrador` e `persistencia`. O único
vestígio de compilação é `compiling_would_pay` — uma **estimativa que informa**,
sem nada atrás. A EDP não é uma quinta forma somada às quatro: é uma quinta forma
**mais um motor de execução novo**, com `cargo build` por edição de fórmula
(490 ms medidos na §9.3), e a §8.1 do `ARCHITECTURE.md` proíbe executor novo — a
objeção já foi apresentada e o autor decidiu que entra (§10.3), mas o preço é
este.

#### 19.2.3 O transporte do CAMPO — medido pela primeira vez

A §8.5 mediu o transporte de um **gráfico 2D** e enterrou a memória
compartilhada. **Campo é outro dado**, e não tinha número. Medido em 2026-09-06:

```text
malha       f64 em JSON     serializar   u8 + base64    a 30 fps: f64  /   u8
51x51          49.488 B        61,04 µs      3.468 B      1,5 MB/s   0,10 MB/s
101x101       199.335 B       295,15 µs     13.604 B      6,0 MB/s   0,41 MB/s
201x201       791.638 B         1,12 ms     53.868 B     23,7 MB/s   1,62 MB/s
401x401     3.156.463 B         5,06 ms    214.404 B     94,7 MB/s   6,43 MB/s
```

**O quadro de campo cabe no canal que já existe, se for quantizado.** Um campo
`201x201` a 30 fps custa **1,62 MB/s** como mapa de cor de 256 níveis — a mesma
ordem do 1,42 MB/s que a §8.5 mediu para o gráfico, e que o JSON-RPC já carrega.
Em `f64` cru seriam 23,7 MB/s e **1,12 ms de serialização por quadro**, que come
3,4% do orçamento de um frame a 30 fps só para virar texto.

**E a quantização não custa informação que a tela use:** o mapa de cor tem 256
níveis de qualquer jeito, e a §16.4 já decidiu que o painel de cálculo mostra
**uma célula**, não a história de todas — essa célula vem em `f64`, por pedido,
como a janela em resolução cheia da `../arquitetura/34` §7.1.

**Uma incógnita a menos para a EDP**, e ela vale para o `SISTEMA_EDO` também: a
trajetória 3D é ainda menor que o campo.

### 19.3 Candidata C — o SymPy como oráculo

#### 19.3.1 Regra zero: a ferramenta NÃO está nesta máquina

```text
python3 -c "import sympy"    ModuleNotFoundError: No module named 'sympy'
rpm -q python3-sympy         o pacote python3-sympy nao esta instalado
```

**Medido em 2026-09-06.** A medição da §15 foi feita com o SymPy presente em
2026-09-05; hoje ele não está. O que o Fedora 44 oferece:

```text
python3-sympy   1.14.0-11.fc44   noarch   84,3 MiB instalado
licenca         BSD-3-Clause AND MIT      <- sem problema de licenca
```

**Isso é fatia, não detalhe:** o oráculo depende de um processo externo que pode
não existir, e a IDE precisa **detectar e dizer**, do jeito que o `setup.list` e
o `ToolDetector` já fazem. As medições abaixo foram feitas numa venv descartável
com SymPy 1.14.0.

#### 19.3.2 O custo por pergunta, medido

```text
import sympy (a frio)                  186,0 ms
dsolve oscilador amortecido + ics      177,9 ms
dsolve queda livre                      34,7 ms
dsolve decaimento                       22,1 ms
dsolve RLC subamortecido                68,4 ms
dsolve logistica                       229,4 ms
avaliar a solucao em t=10 (sp.N)         1,4 ms
lambdify + chamar                        4,2 ms
```

**Um processo por pergunta custa ~200 ms de `import` antes de qualquer conta.** É
o número que decide entre processo por chamada e processo vivo — e o idioma do
GDB (§12.4) já é o segundo.

#### 19.3.3 O `dsolve` TRAVA — e isso não estava medido

```text
pendulo NAO linearizado   y'' = -sin(y)      NAO TERMINOU em 20 s
arrasto quadratico (sem ics)                  1.839,5 ms   respondeu
arrasto quadratico (COM ics)                       -        NotImplementedError
Duffing  y'' = -2y^3 - y'/2                     137 ms     NotImplementedError
dsolve com coeficiente float 4.905            4.188,6 ms   RecursionError
```

Três achados novos:

```text
1. TRAVA, nao falha    o pendulo simples NAO linearizado — a primeira equacao
                       de Fisica I que nao e' de brinquedo — nao volta em 20 s.
                       A IDE precisa de TETO DE TEMPO e de dizer "nao sei", ou
                       ela congela numa pergunta que o usuario tem direito de
                       fazer
2. ics podem MATAR     o arrasto quadratico RESOLVE sem condicao inicial (1,8 s)
   uma solucao que     e da' NotImplementedError com ela. Saber a solucao geral
   existia             nao e' saber a resposta do usuario
3. o float custa 4 s   a armadilha da §15.5 nao so' falha: ela falha DEVAGAR.
   ATE' falhar         `nsimplify(4.905)` custa 8,9 ms e resolve — a
                       racionalizacao e' obrigatoria e e' barata
```

#### 19.3.4 O que o SymPy devolve, o `exmex` LÊ — e o valor bate no último bit

Esta é a medição que decide o custo de integrar o oráculo:

```text
sympy:  (sqrt(31)*sin(sqrt(31)*t/4)/31 + cos(sqrt(31)*t/4))*exp(-t/4)
exmex:  parse OK, var_names = ["t"]
exmex  em t=10:  0,0321283198320319
sympy  em t=10:  0,0321283198320319
diferenca:       6,939e-18
```

**Uma ida ao oráculo por FÓRMULA, não por ponto.** O SymPy resolve, a IDE guarda
a expressão, e o Rust avalia a trilha inteira. Sem isso seriam 500 idas de 1,4 ms
por corrida.

**E há quatro pontos de contato entre os dois, todos medidos — três exigem
conversão e um já funciona:**

```text
`**` -> `^`        o sstr do SymPy escreve `t**2`; o exmex RECUSA (alto). A
                   troca e' segura: `**` so' significa potencia
`pi` fica          `sp.N()` NAO substitui pi: `0.1*cos(pi*t)` sai assim mesmo, e
                   o exmex le `pi` como VARIAVEL LIVRE. E' a armadilha 2 do
                   ADR-0006 reaparecendo no caminho do oraculo, onde nao ha'
                   usuario para ligar nada
`E` funciona       o exmex conhece `E` como Euler: `E*t` em t=2 da' 5,436564
expoente `E-13`    `4.6788114844200873E-13` — o exmex RECUSA (alto)
```

**E a armadilha do `pi` é a única SILENCIOSA, medida:**

```text
exmex.eval(&[10.0])          Err("expression contains 2 vars ... length 1")   alto
exmex.eval(&[10.0, 10.0])    Ok(0.086232)                                     MENTIRA
o valor certo                0.100000
```

Preencher o vetor pelo tamanho de `var_names()` — que é o que um código
descuidado faz — **entrega 0,086 no lugar de 0,100 sem erro nenhum**. O gate é:
**o oráculo recusa toda expressão cujo `var_names()` não seja exatamente `["t"]`**,
e a mutação que prova o gate é injetar `pi` na expressão devolvida.

**E há um vocabulário a recusar**, porque o `dsolve` emite o que o `exmex` não
lê. Medido: `Abs`, `LambertW`, `erf`, `Piecewise`, `re`, `log(t,2)` são
**recusados alto**; `I` (imaginário) é lido como **variável livre** — silencioso,
e a mesma defesa cobre os dois.

#### 19.3.5 O que o oráculo ganha HOJE: zero conceitos, e um defeito

**Contado no catálogo real, 2026-09-06:** os três conceitos que integram
(`oscilador-amortecido`, `queda-livre`, `decaimento-exponencial`) **já têm** a
solução fechada escrita à mão. Os outros 14 são algébricos, onde "exato" é a
própria fórmula avaliada.

```text
conceitos que o oraculo cobriria e a IDE ainda nao cobre:  0
```

**O valor dele não está na cobertura — está na §19.0.** O oráculo resolve **a
equação que o usuário digitou**, e é o único caminho medido que faz a coluna
`exato` parar de responder por outra pergunta. Medido: nos três casos em que a
IDE mente hoje, o `dsolve` acerta em 87–185 ms, e no quarto (Duffing) ele diz
`NotImplementedError` — que é a resposta **certa**, e melhor que o 8,63e-02 que a
IDE mostra.

#### 19.3.6 As unidades: a API muda de lugar, e o exemplo da §15.6 está errado

```text
check_dimensions       NAO e' importavel de `sympy.physics.units` na 1.14.0.
                       Ela mora em `sympy.physics.units.util` e nao esta em
                       nenhum `__all__` — API publica de fato, nao de contrato
get_dimensional_expr   confirmada mentindo: `100*m + 9.58*s` -> `length`
custo por checagem     0,36 ms (sympify + subs + check), em processo
```

**E o exemplo que a §15.6 usa para ilustrar o limite honesto não se sustenta:**

```text
a §15.6 afirma    `F = m*v` aceitou  <- dimensionalmente COERENTE, fisicamente errado
medido            check_dimensions(newton - kilogram*meter/second)  RECUSOU
```

`F = m·v` é **dimensionalmente incoerente** (N contra kg·m/s), então a checagem
o pega. **A tese da §15.6 continua de pé** — checagem de unidade nunca pega
fórmula errada —, mas o exemplo precisa ser um que **passe**: `E = m·v²` sem o
meio, ou um coeficiente errado em fórmula coerente. Coerência dimensional não vê
constante adimensional.

### 19.5 As unidades, medidas em 2026-09-10 — e as TRÊS armadilhas do caminho

A §15.6 e a §19.3.6 deixaram a checagem dimensional pronta para entrar. Medi-la
antes de escrever código achou três coisas, e **as três teriam produzido veredito
errado em silêncio**.

#### 19.5.1 O vocabulário fecha, e é pequeno

As **24 unidades distintas** que o catálogo declara hoje viram objeto do SymPy
com um vocabulário de **12 símbolos** (`m s kg N J C K mol A rad Hz ohm`), depois
de trocar `.` por `*` e `^` por `**`. Zero falhas.

```text
m/s^2      -> Dimension(length/time**2)      N.s/m  -> Dimension(force*time/length)
rad        -> Dimension(1)                   rad/s  -> Dimension(1/time)
J/(mol.K)  -> Dimension(energy/(amount_of_substance*temperature))
```

**E o vocabulário virou gate**, porque ele mora do lado do Python e o catálogo
mora do lado do Rust: `todo_conceito_declara_unidade_que_o_oraculo_conhece`. Sem
ele, um `furlong` no catálogo vira **símbolo livre**, símbolo livre é
**adimensional** para o SymPy, e o veredito sai "coerente" sem nada reclamar.

#### 19.5.2 Substituir a variável pela UNIDADE faz os termos se CANCELAREM

```text
formula          -(k/m)*x - (c/m)*v
substituido      (N/m)/kg*m - (N.s/m)/kg*(m/s)   =   N/kg - N/kg   =   0
dimensao de 0    1  (adimensional)
```

A conta é **exata**, os dois termos somem, e a dimensão da equação vira
adimensional. A defesa é dar a cada variável um **símbolo positivo próprio**:
`x → kv0·m`, e aí `kv0·N/kg − kv1·N/kg` não cancela.

#### 19.5.3 O `check_dimensions` fica CEGO quando há símbolo livre

Esta é a pior, porque a defesa da anterior a causa. Medido:

```text
termo cubico     (N/m)/kg * m^3   ->  base {length: 3, time: -2}
termo do atrito  (N.s/m)/kg * m/s ->  base {length: 1, time: -2}

check_dimensions(soma)      SEM simbolos livres  ->  RECUSOU (certo)
check_dimensions(soma)      COM simbolos livres  ->  ACEITOU (errado)
get_dimensional_expr(soma)                       ->  force/mass
```

Ele pega **um** dos termos e cala — a mesma mentira que a §15.5 atribui ao
`get_dimensional_expr`, agora dentro da função que existe para não cometê-la.

**A defesa não é usar o `check_dimensions`.** É decompor cada parcela em
dimensões de BASE (`get_dimensional_dependencies`) e comparar as parcelas entre
si, na mão. Determinístico, e não depende de uma função que emudece.

#### 19.5.4 O expoente volta como FLOAT, e reprova equação certa

```text
-mu*x/(x^2+y^2)^1.5   com mu = m^3/s^2, x,y = m
esquerda  {length: 1,               time: -2}
direita   {length: 1.00000000000000, time: -2.00000000000000}
```

São a mesma dimensão. Comparadas por dicionário, **não são iguais** — e as
quatro equações CERTAS da órbita foram reprovadas por isso na primeira medição.
A comparação passou a ser numérica, com folga de `1e-9`.

#### 19.5.5 O que a checagem pega, e o que ela não pega

Contra o SymPy real, no oscilador amortecido (`y=m, dy=m/s, k=N/m, m=kg,
c=N.s/m`, lado esquerdo `m/s²`):

```text
-(k/m)*x - (c/m)*v         coherent
-(k/m)*x - 2*(c/m)*v       coherent      <- o LIMITE: coeficiente errado PASSA
-(k/m)*x*x*x - (c/m)*v     incoherent    length*time^-2 != length^3*time^-2
-(k/m)*x - (c/m)*v + k     incoherent    length*time^-2 != mass*time^-2
-(k/m)*x*x*x               wrongSide     length*time^-2 != length^3*time^-2
-(k/m)*sin(x)              dimensionalArgument   (sin)
```

E na órbita, um veredito **por componente** — que é onde ela vale mais, porque
são quatro equações:

```text
as quatro equacoes CERTAS                          x, y, vx, vy   coherent
`vx' = x` (a POSICAO no lugar da velocidade)       vx   wrongSide  length*time^-2 != length
```

**Essa última é o caso que justifica a feature.** Trocar a derivada de uma
posição pela de uma velocidade **passa no `sim.checkSystem`**, porque ele confere
ligação e não física — e não passa aqui.

**O limite honesto, e ele vai na tela junto com o recurso:** `E = m·v²` sem o
meio **passa**. Coerência dimensional não vê constante adimensional, e por isso
ela reforça — nunca substitui — o aviso de que conceito certo e fórmula válida
não significam resultado certo.

#### 19.5.6 O `dsolve` que trava levava junto o veredito que já estava pronto

Medido contra o binário real: `-(k/m)*sin(x)` é o pêndulo não linearizado, o
`dsolve` não volta, e o teto de 5 s mata o processo. **Com uma resposta só, o
veredito de unidade morria junto** — e ele estava pronto em 3 ms, dizendo
`dimensionalArgument`, que é exatamente o que o autor precisa ler.

A defesa é o processo responder em **duas linhas, a barata primeiro**, e o core
ler linha a linha. Medido depois do conserto:

```text
formula                  unidades              ms      procedencia do exato
-(k/m)*sin(x)            dimensionalArgument   5024    concept (com a ressalva)
-(k/m)*x*x*x             wrongSide             5018    concept (com a ressalva)
```

Os 5 s são o teto fazendo o que ele existe para fazer. O veredito chega.

### 19.4 O quadro para a escolha

```text
                      SISTEMA_EDO          EDP                 SymPy oraculo
motor novo            nao (interpretado    SIM: motor          nao: um processo
                      basta, 3,67 s p/     COMPILADO, que      externo, uma ida
                      100 voltas)          nao existe          por formula
subsistema arrastado  nenhum               cargo build por     deteccao de
                      trilha vetorial      edicao + malha +    ferramenta ausente
                      (protocolo)          contorno            + teto de tempo
gate proprio          ordem de converg.,   estabilidade ANTES  var_names()==["t"]
                      com a GRANDEZA       de rodar (r<=0,5)   e vocabulario
                      declarada
o que so' ele da'     orbita, pendulo      onda, calor,        a coluna `exato`
                      duplo, e a base      Laplace,            parar de responder
                      da vista 3D          Schrodinger         por outra equacao
                                           (a Fisica de CAMPO)
o que ele NAO da'     campo                nada de campo       nada para EDP
                                           fica de fora        (pdsolve nao
                                                               resolve as quatro)
conceitos novos que   orbita, pendulo      onda 1D/2D, calor,  0 hoje; todos os
ele libera            duplo, 3 corpos,     Laplace, Schrod.    futuros
                      massa-mola acoplada
custo relativo        o mais barato        maior que as        o menor em codigo,
medido                das tres             outras quatro       maior em fronteira
                                           formas somadas
```

**A recomendação, e ela é só isso — a escolha é do autor:**

1. **`SISTEMA_EDO`**, porque foi medido como o mais barato (motor de hoje basta),
   é o único que abre a vista 3D, e porque a órbita dá função nova à escolha de
   método que já está na tela — 65% de erro contra 2,4e-5, com o mesmo passo.
2. **O oráculo**, porque ele é o conserto do defeito da §19.0 e porque seu valor
   cresce com o catálogo: fazê-lo depois do `SISTEMA_EDO` o entrega já com
   órbitas e osciladores acoplados para medir.
3. **A EDP por último**, porque a medição de 19.2.2 transformou "EDP sempre
   compila" de preferência em pré-requisito, e o motor compilado é fatia própria
   antes de a primeira onda aparecer na tela.

**O que NÃO depende da ordem, e é barato:** a §19.0 é um defeito de veracidade
hoje, com ou sem oráculo. Um recorte mínimo — a IDE deixar de mostrar `exato`
quando não pode garantir que a fórmula é a do conceito — é decisão de uma frase
sua, não de uma etapa.
