# 34 — A simulação por conceito: arquitetura

> **Classe: PLANO, com a primeira fatia JÁ IMPLEMENTADA** (`docs/README.md`).
> Este documento é a saída da etapa 28: as sete perguntas do
> [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md) §5 estão
> todas respondidas, e isto é a arquitetura que as respostas produzem.
>
> **O que EXISTE em código, medido em 2026-09-05 com o gate verde:**
>
> ```text
> crates/kinein-protocol/src/sim.rs        os tipos do dominio
> crates/kinein-core/src/sim/catalogo.rs   14 conceitos, todos ALGEBRICOS
> crates/kinein-core/src/sim/formula.rs    ligacao explicita, checagem, avaliacao
> crates/kinein-core/src/handlers/sim.rs   sim.catalog · sim.inspectFormula ·
>                                          sim.checkFormula · sim.evaluate
> crates/kinein-core/src/sim/integrador.rs os tres metodos, o teto de passos e
>                                          a magnitude que denuncia crescimento
> crates/kinein-core/src/sim/corrida.rs    liga formula + integrador + oraculo
> crates/kinein-core/src/sim/             `.kinein/simulacoes/`, uma por arquivo,
>   persistencia.rs                       e o nome do autor que NAO vira caminho
> crates/kinein-core/src/tests/sim.rs      27 testes, 6 deles provados por MUTACAO
> crates/kinein-core/src/tests/            a VERIFICACAO por ordem de
>   sim_integrador.rs                      convergencia (ASME V&V 20)
> ui/src/core_client_sim.cpp               os quatro metodos no cliente Qt
> ui/qml/sim/                              17 componentes: catalogo, campo da
>                                          formula, lista de problemas, TABELA
>                                          DE LIGACAO, valores, acoes, calculo,
>                                          controles da corrida, estimativa,
>                                          resultado e o GRAFICO 2D (Canvas
>                                          raster — sem GPU, invariante intacto)
> ui/qml/ipc/Sim{Event,Request}Router.qml  as duas pontas do IPC
> scripts/qml-harness/tst_sim_controller   a logica, provada por 3 mutacoes
> scripts/qml-harness/tst_sim_layout       a GEOMETRIA, provada por 2 mutacoes
> protocolo 0.82.0 -> 0.83.0
> ```
>
> **O integrador entrou em 2026-09-05**, com as formas `EDO_1` e `EDO_2`, os
> três métodos, o teto de passos, e o **oráculo de exatidão** — mas o oráculo
> desta fatia é a solução fechada ESCRITA NO CATÁLOGO, e não o SymPy. Três
> conceitos a têm: oscilador amortecido, queda livre e decaimento exponencial.
>
> **O que NÃO existe ainda:** `SISTEMA_EDO`, o motor de EDP, o **SymPy** como
> oráculo (o que estenderia a comparação a qualquer equação, e não só aos três
> conceitos com fórmula fechada escrita à mão), a janela em resolução cheia, o
> processo `kinein-sim` e a vista 3D de trajetória. O gráfico 2D de **y(t) e
> y'(t)** já existe, desenhado em `Canvas` raster — sem GPU, invariante do
> AppImage intacto.
>
> O teste `o_catalogo_so_oferece_forma_que_tem_motor` é o que impede uma forma
> sem motor de aparecer no catálogo — ele **evoluiu** quando o integrador entrou,
> em vez de sumir.
>
> **A vista 3D e os gráficos 2D continuam fora:** a trilha já existe, mas
> desenhá-la é a fatia do `kinein-sim`, e a §10.1 mediu que ela cabe no JSON-RPC
> antes de valer a pena o processo separado.
>
> Toda medição citada foi feita em **2026-09-05**, nesta máquina, e mora em
> [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md) §8 a §14.
> Números aqui são referências, não medições novas.
>
> **O que este documento NÃO faz:** não reabre nenhuma decisão registrada, e não
> cria fila. A fila é o `../roadmaps/30-caminho-para-o-mvp.md`.

## 1. O que se está construindo, em uma frase

Uma superfície onde o autor **escolhe um conceito de física ou matemática de
engenharia, digita a equação dele, e vê o resultado calculado, desenhado e
conferível** — com a IDE dizendo, o tempo todo, o que ela sabe e o que ela não
sabe.

O objetivo declarado pelo autor em 2026-09-05 é **aprender e visualizar**, com
peso em **aeroespacial e automotivo**, do básico ao avançado. Não há ferramenta
incumbente a substituir: esta máquina não tem nenhuma instalada
(`roadmaps/31` §8.6).

## 2. As nove decisões que fecham a etapa 28

Todas do autor, em 2026-09-03 e 2026-09-05. **Nenhuma se reabre aqui.**

```text
§5.1  processo `kinein-sim` separado calcula E desenha; a IDE pinta o frame
      como IMAGEM 2D. Invariante do verificar-appimage.sh INTACTO
§5.2  catalogo de DUAS CAMADAS: o nome que o usuario conhece por cima, a forma
      matematica que a IDE resolve por baixo. "Todos os conceitos" e' viavel
      porque conceito novo e' ENTRADA, nao codigo
§5.2b o usuario DIGITA a equacao dentro do conceito, e a IDE alerta NA HORA da
      digitacao quando os dois nao batem
§5.3  interpretar E compilar. A IDE MOSTRA a estimativa; QUEM ESCOLHE o motor
      e' o usuario — revisado em 2026-09-05 pelo principio da §2.1
§5.4  `.kinein/simulacoes/`, um arquivo por simulacao, com schemaVersion
§5.5  unidade e' declarada E CHECADA em runtime, pelo `check_dimensions` do
      SymPy. REVISADA em 2026-09-05 — ver §7.2
§5.6  o autor e' o primeiro usuario; o criterio de sucesso e' aprender
§5.7  o passo 1 ja' inclui TELA: formula com alerta ao vivo, grafico, e o
      `kinein-sim` nasce nesta fatia
§13   o CAS externo (SymPy) entra como ORACULO DE EXATIDAO, nao como narrador
§15   a EDP entra como QUINTA forma, ja' nesta etapa, para cobrir a parte de
      campo da Fisica II, III e IV
§16   a tela tem QUATRO paineis: vista 3D orbitavel, graficos 2D por variavel,
      painel do calculo e painel do metodo
§17   DOIS caminhos: o catalogo com formula digitada, E o usuario escrevendo o
      proprio codigo. No segundo, o mapeamento das colunas e' DECLARADO na run
      config — a IDE nao adivinha, e nao toca no codigo dele
§2.1  NADA E' ADIVINHADO. Campo comeca vazio, o usuario preenche tudo que
      decide o resultado, e a IDE nunca altera um numero que ele escreveu
```

## 2.1 O princípio que governa todos os outros — autor, 2026-09-05

> *"nada deve ser adivinhado[.] o usuario mesmo que selecionando visualmente o
> conceito e inserindo a formula, tudo deve ser detalhado e selecionado de forma
> explicita."*

**Este princípio é hierarquicamente superior às outras decisões**, e ele revogou
duas delas no mesmo dia em que foram tomadas (§6 e §4.1). Ele se lê assim:

```text
A IDE CALCULA E MOSTRA          o numero, o limite, a estimativa, a consequencia
O USUARIO ESCOLHE E PREENCHE    metodo, dt, motor, amostragem, o que e' cada
                                variavel
A IDE NUNCA                     preenche por voce, deduz do dado, casa por
                                semelhanca, ou muda um numero que voce escreveu
```

**Campo começa VAZIO.** Não há valor pré-preenchido, nem mesmo "sugerido": a
simulação não roda enquanto os campos que decidem o resultado estiverem em
branco. Isso é mais estrito que o *"toolchain automático e VISÍVEL"* de
2026-09-04, e é deliberado — lá a IDE escolhe uma ferramenta e mostra qual;
aqui ela não escolhe nada.

**A distinção que o princípio preserva:** a IDE continua produzindo **informação**
— "esta simulação tem 4,0×10⁷ avaliações", "o dt máximo estável é 6,25×10⁻⁶",
"acima de 21,5 milhões compilar compensa". Informar não é decidir. O que ela não
faz é transformar essa informação em ação sem você mandar.

## 3. A forma geral, e por que ela é a do resto da casa

```text
QML (apresenta e solicita)
     · escolhe conceito · digita formula · ve alerta ao vivo · ve o grafico
        ↓ CoreClient (JSON-RPC stdio) — o cliente que ja' existe
kinein-protocol::sim   descriptor de conceito, formula, plano, resultado, frame
        ↓
kinein-core/src/sim/   catalogo · checagem · planejamento · orquestracao
        ↓                              ↓                       ↓
   motor interpretado          motor compilado          processo kinein-sim
   (exmex, em processo)        (toolchain do usuario)   (calcula + desenha GL)
        ↓                              ↓                       ↓
              eventos tipados  →  resultado · trilha · frame · erro
```

**Nada disso é mecanismo novo.** É a forma do terminal (backend computa, UI
pinta 2D), a forma dos jobs (operação longa é cancelável), a forma das
Configuration Actions (catálogo declarado), e a forma do GDB (ferramenta externa
executada, nunca linkada).

## 4. O catálogo de duas camadas

### 4.1 A camada de baixo: as FORMAS, que são poucas e fechadas

É o que o motor sabe resolver. Cresce raramente, e cada entrada custa um estudo
de convergência (§8).

```text
ALGEBRICA        y = f(parametros)            avaliacao direta, sem integrador
EDO_1            dy/dt = f(t, y)              uma variavel de estado
EDO_2            d2y/dt2 = f(t, y, dy/dt)     a forma da mecanica newtoniana
SISTEMA_EDO      dY/dt = F(t, Y), Y vetorial  varios corpos, varios graus,
                                              e a trajetoria em x, y, z
EDP              du/dt = L(u) no espaco       campo: onda, calor, Laplace,
                                              Schrodinger. Diferencas finitas
```

**A quinta forma foi decisão do autor em 2026-09-05**, com a objeção
apresentada antes: o motor de EDP é maior que os outros quatro somados. Ela é o
que faz "básica, I, II, III e IV — ou seja tudo" ser verdade, porque a parte de
**campo** da Física II, III e IV não cabe em nenhuma das outras quatro
(`../roadmaps/31` §15.3).

**E ela tem regras próprias, todas medidas** (`../roadmaps/31` §16):

```text
ESTABILIDADE E' GATE      r = alpha*dt/dx^2 <= 0,5 (1D) ou 0,25 (2D). Medido:
                          r=0,500 da' max|u|=0,34; r=0,510 da' 1,5 MILHOES.
                          Dois por cento acima e o resultado perde relacao com
                          fisica — e o codigo NAO avisa, entrega o numero.
                          A IDE calcula o criterio ANTES, RECUSA a rodar, e
                          MOSTRA o limite. Ela NAO corrige o seu dt (§2.1):
                          "com dx=5e-3 e alfa=1 o dt maximo estavel e' 6,25e-6;
                          voce pediu 1e-5; explodiria" — e quem ajusta e' voce
EDP SEMPRE COMPILA        o menor caso 2D util (3,25e7 atualizacoes) ja' passa
                          do ponto de virada de 21,5 milhoes. Para esta forma a
                          estimativa da §6 nao tem o que decidir
O CUSTO DA MALHA APARECE  201x201 por 1 s = 3,9 s; 801x801 = 987 s. Refinar
                          custa ao QUADRADO, e o numero vai na tela ANTES
A TRILHA E' QUADRO        guardar a historia de uma EDP 2D e' 4,1 GB (101x101)
                          a 16 TB (801x801). Impossivel por ordem de grandeza,
                          nao por aperto — ver §7.1
```

### 4.2 A camada de cima: os CONCEITOS, que são muitos e crescem por entrada

É o que o usuário escolhe pelo nome. **Adicionar um conceito é uma entrada de
catálogo, não código novo** — é isso que faz "todos os conceitos" ser sustentável.

Cada entrada declara, no mínimo:

```text
nome              "queda livre", "oscilador amortecido", "arrasto aerodinamico"
forma             qual das quatro formas da §4.1 resolve
variaveis         exigidas e opcionais, cada uma com nome, rotulo de unidade e
                  uma frase do que ela e'
estado inicial    quais grandezas precisam de valor em t=0
solucao fechada   quando existe: a formula exata, para o oraculo da §7
faixa de dt util  medida, nao suposta (§8) — abaixo dela a IDE avisa
vista natural     2D, 3D ou mapa de cor. DECISAO DO AUTOR em 2026-09-05: o
                  conceito DECLARA qual e' a vista dele, e o usuario PODE
                  trocar. Queda livre e RLC nascem 2D; orbita e lancamento
                  obliquo nascem 3D; campo/EDP nasce mapa de cor. A IDE abre
                  na vista certa sem perguntar, e nao impede ver de outro jeito
                  — querer x(t) de uma orbita 3D e' legitimo
fonte             de onde veio a formulacao, com data. Mesma regra do
                  `setup.list`: sem fonte, a IDE nao afirma
```

**O risco, dito de frente:** catálogo é dívida **por entrada**. Ele envelhece
sozinho e cada nome que entra é uma afirmação de que a conta está certa. É
exatamente o risco aceito de olhos abertos no catálogo de bibliotecas
(`roadmaps/35` §2.1), e a defesa é a mesma: **entrada não auditada não entra.**

## 5. A checagem de conceito — o coração da decisão do autor

> *"caso o usuario erre a selecao de conceito, a IDE deve alertar na hora que o
> usuario inserir a formula."*

**Mecanismo, medido e funcionando** (`roadmaps/31` §9.2): o `exmex` devolve
`var_names()` — as variáveis que a fórmula realmente usa. A checagem compara isso
com o que o conceito declara.

### 5.0 A ligação é EXPLÍCITA — revisão de 2026-09-05

> **DECIDIDO: o usuário liga cada variável da fórmula à grandeza que ela é.** A
> IDE **não** casa por nome.

O desenho anterior comparava `var_names()` com os nomes declarados no conceito e
os casava por igualdade de string. **Isso é dedução**, e ela erra calada: se você
chamar de `x` alguma coisa que não é a posição, a tabela fica com cara de certa e
a física sai errada.

```text
1. voce digita          -(k/m)*x - (c/m)*v
2. a IDE EXTRAI         encontrei 5 variaveis: c, k, m, v, x
3. voce LIGA cada uma   x = posicao             (m)
                        v = velocidade          (m/s)
                        k = constante elastica  (N/m)
                        m = massa               (kg)
                        c = amortecimento       (N.s/m)
4. a IDE confere        as grandezas exigidas por "oscilador amortecido" estao
                        todas ligadas? sobra variavel sem papel? as dimensoes
                        fecham?
```

**Este passo já existia por outro motivo** — é onde a unidade é declarada
(§7.2) —, então explicitar a ligação **não acrescenta uma tela**: dá função a
uma que já ia existir de qualquer forma.

**E ele resolve por construção as duas piores armadilhas do `exmex`:**

```text
a ordem ALFABETICA   deixa de importar: o vetor de avaliacao se monta pela
                     LIGACAO, nao pela posicao. A falha silenciosa medida na
                     ../roadmaps/31 §8.2 desaparece por DESENHO, nao por
                     disciplina de quem escreve o codigo
o `pi` minusculo     vira uma variavel sem papel na lista, que voce teria de
                     ligar a alguma grandeza — e nao ha' o que ligar. Vira
                     pergunta na tela em vez de incognita silenciosa
```

**O que a IDE responde depois da ligação:**

```text
formula nao fecha                     -> NAO COMPILA, com mensagem PROPRIA
grandeza exigida pelo conceito sem    -> "oscilador amortecido precisa de
nenhuma variavel ligada a ela            constante elastica, e nenhuma variavel
                                         da sua formula foi ligada a ela"
variavel da formula sem papel         -> "voce nao disse o que `F` e'"
dimensao incoerente (§7.2)            -> "nao da' para somar (m) com (s)"
tudo ligado e coerente                -> segue
```

**Custo medido: 5,3 µs por checagem completa.** Checar a cada tecla é de graça.

**E a terceira armadilha do `exmex`, que a ligação não resolve:** a mensagem de
erro do crate **não vai à tela**. O texto tem **endereço de ponteiro** dentro
(356 caracteres, com `0x...`). A IDE classifica o erro e escreve a própria
mensagem — como já faz com `secretRequired` e com o `code` do `requestFailed`.

**Gate provado por mutação, e ele continua obrigatório:** trocar duas variáveis
de papel na ligação tem de mudar o resultado. Se não mudar, a ligação não está
sendo usada e o código voltou a casar por posição sem ninguém notar.

### 5.1 O aviso que o autor pediu explicitamente

> **Conceito certo e fórmula válida NÃO significam resultado certo.** Uma fórmula
> errada dentro do conceito certo usa as variáveis certas e produz um número — e
> o número está errado. A IDE não tem como saber, e diz isso na tela.

Mesma família do *"custo de leitura sempre visível"* da etapa 27: a IDE afirma o
que sabe e declara o que não sabe.

## 6. Os dois motores, e o que impede eles de virarem dois sistemas

A decisão da §5.3 é **interpretar e compilar, com a IDE estimando a escala**. A
objeção — a §8.1 do `ARCHITECTURE.md` proíbe "novo executor de processo" — foi
apresentada antes da escolha e o autor decidiu assim mesmo
(`roadmaps/31` §10.3). O que segue é o desenho que **paga essa decisão sem
duplicar sistema**:

```text
UM contrato de execucao      `SimEngine`: recebe o plano, devolve trilha e
                             resultado. Interpretar e compilar sao duas
                             IMPLEMENTACOES dele, nao dois caminhos paralelos
UM caminho de checagem       o parser roda SEMPRE. Compilar nao substitui a
                             analise da formula: um compilador nunca diz
                             "esta formula nao e' de oscilador"
UM job cancelavel            a operacao longa e' Job, com a politica de
                             cancelamento que ja' existe. Nao nasce executor novo
A ESTIMATIVA E' VISIVEL      a IDE MOSTRA o custo dos dois caminhos e QUEM
E A ESCOLHA E' SUA           ESCOLHE e' o usuario (§2.1, revisto em 2026-09-05):
                             "4,0e7 avaliacoes: interpretado ~1,0 s; compilado
                             ~0,1 s + 490 ms de build". O campo comeca vazio
```

**Por que a revisão não é capricho, e o argumento que ela recusou.** Havia um
argumento honesto para deixar a IDE decidir o motor: **a escolha do motor não
muda o resultado**, só o tempo — a `../roadmaps/31` §8.3 mediu que os
avaliadores concordam até a nona casa. Escolher motor não é escolher método de
integração, que muda o número.

**O autor recusou a distinção**, e o princípio da §2.1 vale inteiro. O ganho é
coerência: um único critério (*"a IDE mostra, você escolhe"*) em vez de uma
fronteira que alguém teria de manter — e fronteira mal mantida é onde a exceção
vira regra sem ninguém decidir.

**O ponto de virada é 21,5 milhões de avaliações**, medido: `490 ms ÷ 22,75 ns`,
onde 490 ms é o `cargo build` incremental real e 22,75 ns é o que cada avaliação
economiza compilada.

```text
pendulo 10 s, dt=1e-4              1e5 avaliacoes    interpretar
3 corpos acoplados 60 s, dt=1e-5   1,8e7            interpretar
campo 2D 200x200, 1.000 passos     4,0e7            COMPILAR
```

**E a estimativa tem de saber da §8.2 do roadmap 31:** passo menor **não** é
sempre melhor. RK4 com `dt=1e-7` é **108× pior** que com `dt=1e-5`, porque o
arredondamento passa a dominar. Uma estimativa que só olha velocidade mandaria
compilar para entregar mais rápido um resultado pior.

## 7. "Exato e preciso": o que a IDE pode honestamente afirmar

**Integração numérica não é exata, por construção.** O que existe é erro
conhecido e declarado. A arquitetura torna isso mecanismo, em quatro peças que a
tela mostra juntas (decisão do autor, 2026-09-05):

```text
1. TRILHA DO QUE RODOU    metodo e dt no topo; por passo, t · x · v · a. Para o
                          RK4, um passo aberto com os quatro estagios k1..k4.
                          E' o que executou — nao existe segunda conta para
                          discordar dele
2. A RESPOSTA EXATA       a solucao fechada, o valor exato no instante pedido,
                          o valor numerico, e a DIFERENCA
3. O METODO NOMEADO       "EDO linear de 2a ordem, coeficientes constantes,
                          homogenea" — e por que a equacao e' desse tipo
4. O AVISO DE PISO        quando o dt pedido cai abaixo da faixa util, a IDE diz
                          que passo menor vai PIORAR, em vez de obedecer calada
```

### 7.1 A trilha não cabe na RAM, e isso é decisão de desenho — medido

A peça 1 exige **guardar** os passos. Medido nesta máquina, com pico de RSS real:

```text
um passo (t, x, v, a em f64) = 32 bytes

cenario                        passos       trilha    pico RSS medido
pendulo 10 s, dt=1e-4         100.000       3,2 MB          5 MB
1 milhao de passos          1.000.000      32,0 MB         33 MB
3 corpos 60 s, dt=1e-5     18.000.000     576,0 MB        552 MB
campo 2D, 1.000 passos     40.000.000   1.280,0 MB      1.223 MB
```

**A trilha completa de um campo 2D pede 1,28 GB** — uma simulação, sem o frame e
sem a UI, numa máquina de 21 GiB. E o cenário do campo 2D é justamente o que a
§6 manda **compilar**, ou seja, o caminho rápido é o que estoura a memória.

**O desenho que isso obriga:**

```text
a trilha e' AMOSTRADA              1 a cada N, e QUEM DEFINE N e' o usuario
                                   (§2.1). A IDE mostra o custo de cada escolha
                                   ANTES: 10.000 pontos de 40 milhoes = 0,32 MB;
                                   guardar tudo = 1,28 GB. O campo comeca vazio
                                   e a corrida nao parte sem ele preenchido
a amostragem e' VISIVEL            "mostrando 1 de cada 4.000 passos" na tela.
                                   Mesmo idioma do `$sample` da etapa 27: a IDE
                                   diz o que ela FEZ, nao entrega uma tabela que
                                   parece completa e nao e'
a janela abre em resolucao CHEIA   pedir os passos 3.900 a 4.100 re-executa
                                   aquele trecho e devolve todos. E' como um
                                   depurador: nao se guarda todo o estado, se
                                   reproduz o trecho pedido
```

**E isso depende do determinismo.** Re-executar a janela só devolve os mesmos
números se a mesma entrada produzir sempre o mesmo resultado — que é a pergunta
que a §5.5 fazia (*"dois runs com a mesma entrada produzem o mesmo resultado?"*)
e que aqui deixa de ser filosófica: **sem determinismo, a janela em resolução
cheia mostra outra simulação.** Medido na `roadmaps/31` §8.3: os quatro
avaliadores testados dão o mesmo resultado até a nona casa, então o determinismo
está disponível — mas ele passa a ser **requisito**, não propriedade feliz.

**Por que a peça 2 é a que muda tudo, e de onde ela vem.** Sem oráculo, a IDE só
sabe o erro dos conceitos cuja solução fechada alguém escreveu à mão. Com o
**SymPy como oráculo de exatidão** (decisão do autor, 2026-09-05), ela sabe o
erro de **qualquer equação que o usuário digitar** e que o `dsolve` resolva.

Medido: o `dsolve` do SymPy 1.14.0 resolveu a EDO do oscilador amortecido e deu
`x(10) = -0.275886266940654`, contra `-0.275886266940653` da analítica derivada à
mão — **conferem até a 14ª casa**.

**O que o oráculo NÃO é, e a §13 do roadmap 31 registra por quê:** ele **não
narra** a resolução. O passo a passo algébrico de uma EDO não existe em ferramenta
auditável — o Maxima marcou o pedido como *"won't fix"* (exigiria reimplementar
as funções internas), o SymPy não tem, e o Wolfram tem com heurística
proprietária **gerada por fora do motor que calcula**. O que o SymPy tem de
narração real é para **integrais** (`integral_steps`, estilo estudante), e isso
entra onde couber.

**Fronteira, como o GDB:** o SymPy é processo externo orquestrado, detectado pelo
`ToolDetector`, e **ausente ele não derruba nada** — a simulação roda, e o que
some é a coluna do erro, com a IDE dizendo que sumiu e por quê.

### 7.2 As unidades passam a ser CHECADAS — decisão revisada em 2026-09-05

A decisão original desta etapa foi **rótulo sem checagem**, e ela foi tomada
sobre uma medição minha: o `uom` checa em tempo de **compilação**, e uma fórmula
digitada pelo usuário não tem tipo Rust nenhum (`../roadmaps/31` §8.4).

**A premissa mudou quando o SymPy entrou por outro motivo.** O
`check_dimensions` dele checa em tempo de **execução** — que é quando a fórmula
do usuário existe. Medido (`../roadmaps/31` §15.6):

```text
F = m*a       aceitou     ok
E = m*c^2     aceitou     ok
m + v         RECUSOU     ok
t + x         RECUSOU     ok
F = m*v       aceitou     <- dimensionalmente COERENTE, fisicamente ERRADO
```

**O limite vai na tela junto com o recurso:** checagem dimensional pega
**incoerência**, nunca pega **fórmula errada**. `F = m·v` passa. Isso não
enfraquece o aviso da §5.1 — reforça: a IDE continua dizendo que conceito certo e
fórmula válida não significam resultado certo.

**Duas armadilhas do mecanismo, medidas e evitáveis:**

1. **A API ingênua mente.** `get_dimensional_expr(100*m + 9.58*s)` devolve
   `length` para uma soma **inválida** — pegou o primeiro termo e calou. Só o
   `check_dimensions` recusa. Quem validar pela primeira valida nada.
2. **O oráculo é opcional, e a checagem segue o oráculo.** Sem SymPy presente, a
   tela diz *"unidades não verificadas nesta sessão"* e por quê — em vez de parar
   de checar em silêncio. Mesmo tratamento da coluna de erro na §7.

### 7.3 O que a IDE mostra NO LUGAR do raciocínio — a pergunta do autor

> *"Se da para mostrar apenas o resultado e nao o raciocinio, poderiamos adaptar
> algumas coisas para a IDE?"*

**Dá, e a divisão é limpa — ela não é por conceito, é por FORMA.** Medido em
`../roadmaps/31` §15.4:

```text
FORMA                 o raciocinio existe?    o que a tela mostra
ALGEBRICA             parcial                 substituicao numerica por extenso
EDO_1 / EDO_2         NAO                     trilha + solucao fechada + erro
SISTEMA_EDO           NAO                     idem, por variavel de estado
EDP                   NAO                     quadro amostrado + estabilidade
integral (Calc I-III) SIM, estilo estudante   as REGRAS aplicadas, em ordem:
                                              `Parts`, `Rewrite`, `Arctan`...
```

**As quatro adaptações que substituem o raciocínio ausente, e por que cada uma é
honesta:**

```text
1. SUBSTITUICAO         a formula com os valores dentro, por extenso. Nao e'
   NUMERICA             algebra — e' literalmente a conta que rodou. Impossivel
                        de divergir do resultado, porque E' o resultado
2. A RESPOSTA VERDADEIRA a solucao fechada do oraculo ao lado do numero
   AO LADO              calculado, com a diferenca. Substitui "confie no
                        raciocinio" por "confira o resultado" — que e' mais
                        forte, e e' o que a engenharia chama de VERIFICACAO (§8)
3. O METODO NOMEADO     "EDO linear de 2a ordem, coeficientes constantes,
   E JUSTIFICADO        homogenea" + por que a sua equacao e' desse tipo. E' o
                        `classify_ode`, e e' a parte pedagogica que EXISTE
4. AS REGRAS, ONDE      em integral, o `integral_steps` da o passo a passo real.
   ELAS EXISTEM         Entra so' onde existe, e a IDE nao finge que tem nas
                        outras formas
```

**A regra que amarra as quatro:** onde o raciocínio não existe, a IDE **diz que
não existe** — não preenche com narrativa plausível. Uma derivação inventada que
não é a conta executada é a forma exata do que a §11.5 do `../roadmaps/31`
encontrou no Wolfram, e do anti-padrão da §8.1 do `ARCHITECTURE.md`: dois
caminhos para a mesma verdade, e o dia em que discordarem a IDE mostra uma conta
e entrega outro número.

## 7.4 O SEGUNDO caminho: o usuário escreve o próprio código

> Decisão do autor em 2026-09-05: *"gostaria de facilitar e deixar as duas
> opções"* — o catálogo com fórmula digitada, **e** o usuário escrevendo a
> simulação em C++ (ou no que for) e clicando para executar.

**Medição primeiro, e ela encurtou a frente** (`../roadmaps/31` §17.1): rodar o
programa do usuário **já funciona hoje**. `run.start`, `run.stop`, `run.stdin`,
`runConfig.*`, `job.*`, `event.run.output` e o `fswatch` estão de pé. A frente
não é "construir execução"; é **ligar uma saída que já atravessa a fronteira a um
desenho**.

### 7.4.1 O contrato: o usuário DECLARA, a IDE não adivinha

> **DECIDIDO: o mapeamento das colunas é declarado na run config.** Sem
> heurística, sem cabeçalho no código do usuário, sem biblioteca.

```text
o programa do usuario     imprime `t x y z` e mais nada. Zero header, zero
                          biblioteca, zero linha que exista por causa da IDE.
                          Funciona em C, C++, Rust, ou qualquer coisa que
                          saiba imprimir
a run config              guarda "a saida deste alvo e' trajetoria; coluna 1 =
                          t, 2 = x, 3 = y, 4 = z". Declarado uma vez, fica
o core                    parseia, AMOSTRA, e manda so' a amostra
a vista                   desenha
```

**Por que não a heurística, mesmo tendo medido bem.** A `../roadmaps/31` §17.4
mediu o reconhecimento automático contra a saída **real** de `cmake`, `cargo`,
`g++`, `ping` e `df`: **zero falso positivo**, e até o caso misto (simulação que
também imprime log) foi reconhecido. **O autor escolheu declarar mesmo assim**, e
o registro importa: a opção foi medida e recusada por preferência de não ter
mágica, não por ela ter falhado. O que a medição não resolvia continua verdade —
a heurística reconhece a **forma**, nunca a **semântica**: ela não sabe se a
terceira coluna é `z` ou é energia.

### 7.4.2 O core parseia e amostra — a UI nunca vê o volume bruto

Medido sobre 46 MB reais de saída de um programa C++ (`../roadmaps/31` §17.3):

```text
parsear o texto no core                    96 ms    481 MB/s
mandar TUDO por event.run.output           38 ms    47,21 MB, 5.613 eventos
mandar TUDO como pontos                   286 ms    47,98 MB de JSON
mandar 10.000 pontos amostrados            1,6 ms    0,46 MB   <- e' isto
```

**Uma tela de 1920 px não distingue mais que ~2.000 pontos numa curva.** Mandar
um milhão é mandar 500× o que o olho resolve. A divisão é a mesma do terminal: o
core computa, a UI desenha.

**E o texto cobra do programa DO USUÁRIO:** imprimir 1M de linhas custa 512 ms a
ele, contra 28 ms em binário — imposto de 18×. Por isso o contrato da run config
aceita dizer que a saída é binária, em vez de obrigar texto.

### 7.4.3 O que a IDE NÃO faz neste caminho

```text
NAO linka biblioteca no programa do usuario
NAO exige header, macro, anotacao ou main() de formato especial
NAO injeta codigo, NAO reescreve o arquivo dele
NAO adivinha o significado das colunas
NAO ocupa o stdout dele: o log continua sendo log, e aparece como log
```

**A IDE orquestra e desenha. O código é dele.** É a mesma linha do `arquitetura/27`
§6: detectar é capacidade, ativar é política, **executar é fronteira**.

## 8. O gate de cada conceito: ordem de convergência, e a faixa importa

A pergunta *"esta conta está certa?"* tem procedimento com norma. A **ASME V&V
20** separa duas coisas que costumam virar uma:

```text
VERIFICACAO   as equacoes foram implementadas corretamente?  -> mede-se, sem
              experimento: compara-se com solucao exata e mede-se a ORDEM
VALIDACAO     as equacoes descrevem a realidade?  -> exige experimento. FORA
              do alcance de uma IDE, e a IDE nao vai fingir que faz
```

**O critério de entrada de cada conceito do catálogo:**

```text
COM solucao fechada   compara contra ela E mede a ordem de convergencia
SEM solucao fechada   Metodo das Solucoes Manufaturadas (MMS): escolhe-se a
                      solucao, substitui-se na equacao, e o resto vira
                      termo-fonte, de modo que a solucao satisfaca exatamente
                      a equacao modificada
que nao passa         NAO ENTRA no catalogo
```

**E a armadilha que esse gate tem de evitar, medida:** a ordem só é mensurável
numa faixa de `dt`. Medida do RK4 nesta máquina:

```text
de 1e-1 para 1e-2   ordem 3.85   confirmada
de 1e-2 para 1e-3   ordem 3.99   confirmada
de 1e-3 para 1e-4   ordem 2.21   CONTAMINADA pelo piso do f64
de 1e-4 para 1e-5   ordem 0.34   CONTAMINADA
```

Um teste que verificasse "RK4 é ordem 4" escolhendo `dt=1e-4` e `dt=1e-5`
**reprovaria um código correto**. A faixa é parte do teste.

## 9. Onde a simulação mora

`.kinein/simulacoes/<nome>.json`, um arquivo por simulação, com `schemaVersion`,
e arquivo inválido tratado como vazio em vez de quebrar — o precedente de
`runconfigs.json` e `settings.json`.

**O que o arquivo NÃO guarda: o resultado.** Essa é a lição precisa do `.ipynb`,
que é texto, é diffável, e mesmo assim falha — porque mistura o que o usuário
escreveu com o que a máquina produziu: diff ilegível, metadado que muda sozinho,
imagem em base64 no diff, e conflito de merge que quebra o JSON a ponto do
arquivo não abrir mais. Nasceu o `nbdime` só para remediar.

```text
GUARDA    conceito escolhido · formula · parametros · estado inicial · rotulos
          de unidade · metodo e dt escolhidos · janela de tempo
NAO GUARDA  a trilha · os numeros · a imagem
```

## 10. O desenho, e o invariante que não se toca

`scripts/verificar-appimage.sh` reprova `ShaderEffect|QOpenGL|QRhi|QtQuick3D` em
`ui/`, porque o AppImage força renderer por software — é isso que faz a IDE abrir
em qualquer máquina.

```text
kinein-sim (processo)              kinein-vectis (IDE)
  calcula                            recebe o frame PRONTO
  desenha OpenGL OFFSCREEN    -->    pinta como IMAGEM 2D no layout
  le o framebuffer                   sem GPU, sem QRhi, sem ShaderEffect
```

**Pintar imagem é 2D. O invariante sobrevive porque a GPU está no outro
processo.** Uma GPU quebrada derruba o simulador, nunca a IDE.

### 10.1 O transporte: medido, e a premissa antiga não sobreviveu

A `roadmaps/31` §5.1.1 dizia que o frame *"exige um segundo canal — memória
compartilhada"*. Isso vale para **frame cru**. Medido com o `serde_json` e o
`base64` do próprio projeto:

```text
grade do terminal 200x50, hoje, a 30fps            0,45 MB/s
frame CRU 1920x1080 em base64, a 30fps           331,8 MB/s
frame COMPRIMIDO (grafico de linha) 1920x1080      1,42 MB/s   <- 176x menor
frame COMPRIMIDO (heatmap 1280x720), pior caso    13,44 MB/s   <- 8x menor
```

**Um gráfico 1920×1080 comprimido a 30fps são três vezes a grade do terminal**,
que o JSON-RPC já carrega hoje. **A memória compartilhada deixa de ser exigência
do transporte** — ela era exigência do frame cru, e frame cru é escolha.

O gargalo passa a ser o **tempo de comprimir** (14,7 ms a 1080p no `zlib` do
Python; em Rust ainda não medido). **E se o render for sob demanda, a pergunta
some.**

## 10.2 A tela: quatro painéis, decididos pelo autor em 2026-09-05

> *"Estava pensando em ter a exibicao da parte da coordenadas x, y e z. E ter um
> layout ao lado mostrando o calculo e o resultado e afins."*

```text
+---------------------------------+---------------------------------+
|  VISTA 3D  (x, y, z)            |  PAINEL DO CALCULO              |
|  a trajetoria nos tres eixos,   |  a formula como voce digitou    |
|  camera orbitavel pelo usuario  |  os valores substituidos por    |
|                                 |  extenso, e o resultado         |
|  desenhada pelo kinein-sim,     |                                 |
|  pintada como IMAGEM 2D         +---------------------------------+
|                                 |  PAINEL DO METODO               |
+---------------------------------+  a solucao fechada do SymPy     |
|  GRAFICOS 2D por variavel       |  o nome do metodo (classify_ode)|
|  x(t), v(t), a(t) em curvas     |  o valor numerico e a DIFERENCA |
|  separadas, sincronizadas com   |  o aviso de piso do dt          |
|  a posicao mostrada no 3D       |  (e o de estabilidade, na EDP)  |
+---------------------------------+---------------------------------+
```

**Quem desenha o quê, e por que o invariante sobrevive:**

```text
VISTA 3D          kinein-sim desenha em OpenGL OFFSCREEN e manda o frame; a IDE
                  PINTA A IMAGEM. Renderiza SOB DEMANDA — girou a camera,
                  redesenha. Sob demanda, a §10.1 diz que nem compressao e'
                  necessaria: o custo por frame deixa de importar
GRAFICOS 2D       a UI desenha sozinha, sem GPU, sem processo. E' curva sobre
                  eixos — o que ela ja' sabe fazer
PAINEL DO CALCULO texto. A substituicao numerica E' a conta que rodou, e por
                  isso e' honesta: nao existe segunda derivacao para discordar
PAINEL DO METODO  texto vindo do oraculo, e some com ele — dizendo que sumiu
```

**A divisão de trabalho é a mesma do resto:** o 3D mostra **onde**, o 2D mostra
**como varia**, e os dois painéis de texto mostram **por que aquele número**. O
`kinein-sim` nasce nesta fatia (decisão do autor, §5.7) e é o único que toca GPU
— num processo que, se cair, derruba o desenho e não a IDE.

## 11. O que esta arquitetura NÃO decide, e fica para quando houver código

```text
- o formato exato do arquivo de simulacao (campos e nomes)
- os nomes dos metodos IPC e dos eventos do dominio `sim.*`
- quais conceitos entram PRIMEIRO no catalogo — depende do autor dizer quais
  ele quer ver funcionando, e a §5.6 ja' diz que ele e' o usuario
- o formato do frame na fronteira (cru, comprimido, ou sob demanda) — decide-se
  medindo a compressao em RUST, que a §8.8 registra como nao medida
- se o `integral_steps` do SymPy entra ja' na primeira fatia (ele e' o UNICO
  lugar onde o raciocinio estilo estudante existe — ../roadmaps/31 §15.4)
- o esquema numerico da EDP alem do explicito (implicito/Crank-Nicolson tira a
  parede de estabilidade da §4.1 e cobra sistema linear por passo)
- como a janela em resolucao cheia da §7.1 se comporta na forma EDP, onde
  reproduzir um trecho custa a malha inteira
```

## 12. As referências consultadas em 2026-09-05

Estudadas como comportamento, **nunca transplantadas** — a regra da §2.1 do
`ARCHITECTURE.md`.

```text
Modelica / OpenModelica   motor generico: linguagem de equacoes nao-causais,
                          achatada e ordenada antes do solver. O compilador
                          gera C e e' GPL-3/OSMC-PL — prateleira do GDB
Simulink                  catalogo de blocos + escotilha. Proprietario: entra
                          como referencia FUNCIONAL apenas (licao do Pylance).
                          O .slx e' zip de XML tratado como BINARIO, e o proprio
                          fabricante vende a ferramenta de merge a parte
Jupyter / nbdime          o fracasso que ensina a §9: texto nao basta se o
                          modelo e o resultado moram juntos
Wolfram|Alpha             passo a passo por heuristica "como um humano faria",
                          gerado POR FORA do motor que calcula
Maxima                    CAS GPL, binario de terminal. Passo a passo: won't fix
SymPy                     CAS BSD. dsolve, classify_ode, e passo a passo real
                          para integrais
ASME V&V 20 / MMS         o procedimento que separa verificacao de validacao, e
                          o metodo para verificar quando nao ha solucao exata
```

## 13. A forma SISTEMA_EDO — desenho, 2026-09-06

> **Escolha do autor em 2026-09-06**, sobre a medição das três candidatas
> (`../roadmaps/31` §19): o `SISTEMA_EDO` primeiro, o oráculo depois, a EDP por
> último. Esta seção é o desenho da forma, escrito **antes** do código, e cada
> decisão abaixo carrega o número que a obriga.
>
> **ENTREGUE no mesmo dia**, protocolo `0.86.0`, com `sim.checkSystem` e
> `sim.runSystem`. O desenho abaixo sobreviveu inteiro à implementação; o que a
> implementação acrescentou está na §13.8, e são três defeitos que o desenho não
> previu.

### 13.1 Ela é de PRIMEIRA ORDEM, e a segunda entra pela porta do usuário

```text
dY/dt = F(t, Y)     Y = [y1, y2, ..., yn]
```

**Não há forma de segunda ordem vetorial**, e isso é decisão, não omissão. Um
sistema de segunda ordem entra escrevendo as velocidades como componentes — a
redução padrão, que todo livro faz:

```text
orbita, como o usuario a escreve
  x'  = vx           <- 4 componentes, 4 formulas, primeira ordem
  y'  = vy
  vx' = -x/(x^2+y^2)^1.5
  vy' = -y/(x^2+y^2)^1.5
```

O motivo é o de sempre neste projeto: **um mecanismo, não dois**. A forma
`EDO_2` escalar continua existindo porque ela é a da mecânica newtoniana de um
grau e economiza a redução manual num caso muito frequente; acrescentar uma
`SISTEMA_EDO_2` seria um terceiro caminho para o mesmo cálculo.

### 13.2 O conceito DECLARA os componentes; o usuário escreve uma fórmula para cada

O catálogo ganha, na entrada, a lista **ordenada** de componentes de estado:

```text
SimSystemComponent   id · label · unit · a frase do que ele e'
```

E o pedido de corrida traz **uma fórmula e uma tabela de ligação por
componente**. Nada é adivinhado, e em particular:

```text
a IDE nao supoe que a formula do componente `x` seja a que menciona `x`
a IDE nao supoe a ordem das formulas: cada uma diz de que componente ela e'
a IDE nao supoe estado inicial: sao n numeros, e o campo comeca vazio
```

A armadilha da ordem alfabética do avaliador (ADR-0006, armadilha 1) fica
resolvida do mesmo jeito da forma escalar: **o vetor de avaliação é montado pela
LIGAÇÃO**, componente a componente, nunca pela posição.

### 13.3 O método simplético exige um PAREAMENTO, e quem o declara é o conceito

Esta é a decisão mais consequente da forma, e ela vem de medição
(`../roadmaps/31` §19.1.1). Numa órbita circular de raio verdadeiro 1:

```text
Euler explicito     dt=0,01, 10 voltas  ->  raio 1,647957   deriva de E 2,032e-01
Euler simpletico    dt=0,01, 10 voltas  ->  raio 1,000024   deriva de E 2,800e-10
```

**Oito ordens de grandeza na energia, com o mesmo passo.** Jogar o simplético
fora seria jogar fora o achado.

Mas o simplético **não é definível num sistema de primeira ordem qualquer**: ele
precisa saber quais componentes são posição e quais são a velocidade
correspondente, porque o método é *atualize a velocidade, depois ande com a
posição usando a velocidade NOVA*. Num `dY/dt = F(t,Y)` genérico esse par não
existe.

**Duas saídas, e a escolhida:**

```text
(a) a IDE DEDUZ o par           REPROVADA pelo principio da §2.1: adivinhar
    (por nome, por ordem)       qual componente e' velocidade de qual e'
                                exatamente a deducao que erra calada
(b) o CONCEITO declara o par    ESCOLHIDA. E' declaracao auditada, do mesmo
                                tipo que ja' sustenta `quantities` e `fonte`
```

**E a consequência é dita na tela, não escondida:** um conceito sem pareamento
declarado **não oferece o simplético**, e a IDE diz por quê — *"este conceito não
declara quais componentes são posição e velocidade, então o método simplético
não se aplica a ele"*. Melhor recusar com o motivo do que aceitar e integrar
outra coisa.

```text
orbita          pareado: (x, vx) e (y, vy)          -> simpletico DISPONIVEL
pendulo duplo   pareado: (t1, w1) e (t2, w2)        -> simpletico DISPONIVEL
sistema linear  sem pareamento                      -> Euler e RK4, e a frase
generico
```

### 13.4 O gate de convergência mede o VETOR — e diz que mede

A §8 já exige estudo de ordem de convergência por forma. A medição de
2026-09-06 achou como ele erra:

```text
ordem medida NO RAIO da orbita circular (dt: 0,02 -> 0,01 -> 0,005 -> 0,0025)
  Euler explicito     0,98   0,98   0,99     bate com o teorico
  Euler simpletico    1,32   1,09   3,21     ruidosa
  Runge-Kutta 4       5,00   4,99   5,30     ordem 5 num metodo de ordem 4
```

**O RK4 mede 5 porque a grandeza medida cancela erro:** a órbita circular é
solução especial e o raio não vê o erro de fase. É a mesma família da armadilha
da faixa de `dt` (`../roadmaps/31` §11.3), num eixo diferente — lá era *onde*
medir, aqui é *o quê*.

**A regra que sai daqui, e ela é gate:**

```text
o estudo mede o ERRO DO VETOR DE ESTADO em norma do maximo, contra uma
referencia, e a assercao ESCREVE qual grandeza foi medida.
Grandeza derivada (raio, energia, modulo) NAO serve de medida de ordem.
```

Sem isso, um estudo de convergência aprova código errado e reprova código certo
— e o faz mostrando um número bonito.

### 13.5 O oráculo: solução fechada onde é honesto, e INVARIANTE onde não é

A forma escalar compara com a solução fechada quando ela existe. Num sistema,
isso cobre pouco: a órbita circular tem forma fechada, a elíptica exige resolver
a equação de Kepler (transcendental), e o pêndulo duplo **não tem** — medido,
ele é caótico acima de uma energia (`../roadmaps/31` §19.1.4).

**Então a forma ganha um segundo sinal de exatidão, que a escalar não tinha:**

```text
SOLUCAO FECHADA   quando existe. O erro absoluto contra a verdade, como hoje
INVARIANTE        uma grandeza que a fisica CONSERVA — energia, momento angular.
                  O conceito declara; a IDE mostra o valor inicial, o final e a
                  DERIVA. Nao e' o erro, e a tela nao chama de erro
```

**Por que a deriva vale, mesmo não sendo o erro:** ela é medida sem oráculo
nenhum, ela pega exatamente o modo de falha que a integração de sistema tem, e
o número mede o que se quer saber. Na órbita medida: `2,032e-01` com Euler
explícito contra `2,800e-10` com simplético — a diferença entre uma órbita que
espirala para fora e uma que fecha.

**E o limite vai junto, como sempre:** invariante conservado **não** significa
resultado certo. Um erro que respeita a simetria conservada passa por ele. A IDE
mostra o que sabe e nomeia o que não sabe.

### 13.6 A trilha e a tela

```text
uma amostra   t + n valores   ->  (1 + n) * 8 bytes
```

A amostragem é a mesma da forma escalar, e o `sample_every` continua indo à tela
— tabela amostrada parece completa.

**A tela desenha dois modos, os dois em `Canvas` raster 2D**, sem GPU e sem tocar
o invariante do AppImage:

```text
COMPONENTES NO TEMPO   n curvas sobre t. Serve a qualquer sistema
TRAJETORIA NO PLANO    um componente contra outro. O conceito declara o par de
                       plano (na orbita, x contra y), e e' o modo em que a
                       orbita que NAO fecha aparece como o que ela e'
```

**A vista 3D continua fora desta fatia.** Ela depende do `kinein-sim` com OpenGL
offscreen (§10), e a trajetória de uma órbita no plano é 2D de verdade — não é
uma projeção de conveniência.

### 13.7 O motor continua interpretado

Medido em 2026-09-06 (`../roadmaps/31` §19.1.3): 584 ns por passo de RK4 em
quatro dimensões, 36,5 ns por avaliação. As corridas que a forma libera custam:

```text
orbita 10 voltas, dt=1e-3        0,04 s
orbita 100 voltas, dt=1e-4       3,67 s
pendulo duplo 60 s, dt=1e-5      3,50 s
```

**Nenhuma justifica compilar** (o ponto de virada é 21,5 milhões de avaliações),
e por isso esta forma **não abre o motor compilado**. A `sim.estimate` passa a
receber quantas equações o sistema tem, porque o custo escala com elas — e ela
continua **informando**, não decidindo.

### 13.8 O que a implementação achou, e o desenho não previa — 2026-09-06

**Um defeito que a forma escalar carregava desde 2026-09-05, sem aparecer.** O
oráculo era perguntado pela `duracao` **pedida**, e a integração para em
`passos * dt`:

```text
duracao = 2*pi = 6,283185...   dt = 0,01   ->  628 passos, para em 6,28
diferenca                                      3,2e-3
```

Nesse instante a órbita ainda não fechou, e o "erro" mostrado passava a incluir
uma diferença de **tempo** que não é erro de integração nenhum. **Fez o estudo de
ordem medir 0,81 num RK4.** Na forma escalar ele nunca apareceu porque
`duracao = 10` com `dt = 0,1` dá exatamente 100 passos — o defeito precisava de
uma duração irracional para se mostrar, e a órbita tem uma. Corrigido nas duas.

**O invariante pegou o que nenhum outro gate pegaria.** As equações de movimento
do pêndulo duplo, escritas de memória, passaram na compilação e no checador de
fórmula — elas usam as variáveis certas e produzem números. **A energia derivou
2,18 onde deveria ficar parada.** Rederivadas com o SymPy a partir da
lagrangiana e conferidas contra a derivação em 2.000 pontos aleatórios (maior
diferença 7,1e-15).

Isto é exatamente a §5.1 acontecendo: *fórmula válida dentro do conceito certo
produz um número, e o número está errado.* A diferença é que aqui **a IDE tinha
como saber** — porque o conceito declara uma grandeza conservada.

**E um gate que já existia pegou a IDE decidindo física.** A primeira versão do
pêndulo duplo normalizava `g`, `l` e `m` a 1 dentro do catálogo. O
`catalogo_declara_conceitos_com_fonte_datada` cobra que todo conceito declare
grandeza, e o pêndulo não declarava nenhuma — porque elas estavam embutidas.
Normalização embutida é a IDE escolhendo por você (§2.1), e as duas viraram
grandezas declaradas que aparecem nas equações.

**O que o gate do QML ensinou sobre gates.** Três mutações no `SimPlotSystem`, e
**duas não mataram**:

```text
teto de comprimento     `indice < values.length` era REDUNDANTE: o acesso fora
                        da faixa ja' chega como `undefined`. SAIU do codigo
guarda de `undefined`   tambem nao mata — em JS `undefined < x` e `undefined > x`
                        sao os DOIS falsos, e o valor e' pulado de qualquer
                        jeito. FICOU, com o motivo escrito no arquivo: a
                        alternativa e' depender de comportamento implicito da
                        linguagem, que este projeto nao aceita como defesa
```

A linha que **de fato** defende é o `!isFinite` nos extremos, e foi preciso mutar
três candidatas para descobrir qual era. **Mutação não serve só para provar que
o gate pega: serve para descobrir qual linha está segurando o peso.**

### 13.9 A tela foi ligada no motor — 2026-09-07

**O motor vetorial ficou um dia inteiro sem porta, e nada reclamou.** Em
2026-09-06 saíram `sim.checkSystem`, `sim.runSystem`, o integrador, o oráculo, os
invariantes e 649 testes verdes. O `SimPlotSystem.qml` estava no `QML_FILES`,
tinha harness próprio que passava — e **não era instanciado em lugar nenhum do
app**. O `SimSystemController` nascia no `AppDomains` e ninguém o lia.

**O efeito para quem usa era pior que a ausência**, e foi medido contra o
binário real antes de qualquer conserto:

```text
sim.catalog        devolve os 3 conceitos odeSystem, e a lista NAO filtra por forma
sim.checkFormula   concept=orbita-dois-corpos, formula="mu*2"  ->  {"ok": true}
sim.evaluate       -> "Orbita de dois corpos = mu*2"  =  2
```

A IDE oferecia três conceitos, deixava a borda ficar verde numa fórmula
algébrica qualquer e devolvia um número — enquanto o integrador que resolve
aqueles três, e que mede raio final `1,000024` no simplético, ficava
inalcançável. É a §5.1 acontecendo com a IDE do lado errado do aviso.

**A ligação, e as três decisões que ela obrigou:**

```text
o painel escolhe o RAMO    `SimPanel` le `system.isSystem`, que vem do
pelo CATALOGO              CATALOGO — nunca de olhar a formula. O ramo vetorial
                           mora em arquivo proprio (`SimSystemAuthoring`,
                           `SimSystemResultView`, `SimComponentEquation`)
o ramo vetorial recebe o   e nao vinte propriedades soltas. O que a tela le e'
CONTROLLER                 POR COMPONENTE — a ligacao daquela equacao, as
                           variaveis daquela formula —, e achatar isso produz
                           uma lista de propriedades que cresce com `n`. Ha'
                           precedente: o `GrafanaPanel` ja' recebe o dele
os dois SINAIS de          o `SimSystemAccuracy` saiu do `SimSystemResultView`
exatidao sao arquivo       quando a catraca reprovou 324/300. O corte e' por
proprio                    RESPONSABILIDADE: "o quanto isto erra" e "como isto
                           se desenha" sao duas perguntas, e a §13.5 ja' as
                           separava. Nenhum limite foi levantado
a numerica tem UM dono     metodo, passo, duracao e amostragem sao os mesmos nas
para as duas formas        duas formas, e ficam no `SimRunController`. O que
                           difere e' o estado inicial — um numero contra `n` —, e
                           ele fica com quem o guarda. Quem junta as duas metades
                           e' o host, que e' onde fiacao mora
```

**E uma recusa, com o motivo na tela:** salvar um sistema **não** entrou. O
`SimSaved` do protocolo carrega UMA fórmula e UMA ligação (§9), e um sistema tem
uma de cada por componente. Gravar assim escreveria uma montagem que não volta,
então o botão fica desabilitado e a tela diz por quê. Persistir um sistema é
fatia própria: muda o protocolo, o `persistencia.rs` e os testes dele.

**O gate que nasceu daqui, e por que ele não existia.** Nenhuma das dezoito
verificações via o buraco, e a razão é estrutural: **cada uma confere o
componente por si**. O `qmllint` lê um arquivo; o de propriedades confere o
binding onde ele está escrito; o harness instancia o que o teste pediu. Faltava
alguém perguntando se **alguma tela chega ali** — que é a pergunta do usuário.

```text
verificar-qml-alcance.sh   todo .qml do QML_FILES e' instanciado em outro
                           arquivo do modulo, ou — se for `pragma Singleton` —
                           usado pelo nome. `Main.qml` e' a raiz, e quem a
                           instancia e' o C++. Sem baseline: entregue e
                           inalcancavel e' ZERO
```

Ele foi provado por mutação nos dois sentidos: tirar o `SimPlotSystem` da tela
reproduz o achado original, e tirar o `pragma Singleton` do `StatusColors` pega
o outro caminho. Junto veio o `tst_sim_system_panel`, que cobra a tela em si —
conceito de sistema não cai na tela escalar, há uma equação e um estado inicial
por componente, o gráfico vetorial aparece com o resultado, e o simplético é
recusado **com motivo** quando o conceito não declara o pareamento.

**O que a medição achou de quebra, e não é desta fatia:** o
`EditorUnsavedChangesDialog.qml` (198 linhas, do commit de fundação) não está no
`QML_FILES` e não é referenciado por ninguém. Ele não chega ao binário, então o
gate novo não o vê — fica registrado aqui para alguém decidir entre ligá-lo ou
removê-lo.

### 13.10 O oráculo entrou — 2026-09-10

**A segunda escolha da §7 do [`../roadmaps/40`](../roadmaps/40-estado-e-continuidade.md)**,
e o que ela conserta é a §19.0 do [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md):
a coluna `exato` respondia por outra equação.

**A decisão que governa o desenho, e ela não era óbvia:** o conserto **não é
esconder o número**. Sem SymPy na máquina, a solução do conceito continua sendo
a melhor resposta disponível — o que faltava era **dizer que é ela**. Por isso
nasceu `SimAccuracySource`, e não um `if` que apaga a coluna.

```text
oracle    a IDE resolveu a EQUACAO QUE VOCE DIGITOU. A tela nomeia quem
          resolveu e mostra a solucao fechada ao lado do numero calculado
concept   a solucao do CONCEITO. Continua na tela, com a ressalva dizendo que
          e' de outra pergunta se voce mudou a equacao — e por que o oraculo
          nao respondeu (falta o SymPy, ele nao soube, ou passou do tempo)
```

**A fronteira é a do GDB, como a §7 já mandava:** processo externo orquestrado,
nunca linkado. Uma ida por **fórmula**, não por ponto — o SymPy resolve, a IDE
guarda a expressão, e o `exmex` avalia a trilha inteira. Medido em 2026-09-06:
os dois concordam até `6,9e-18`.

**As cinco armadilhas medidas, e onde cada defesa mora:**

```text
o `dsolve` TRAVA         teto de 5 s no `oraculo::executar`. O pendulo nao
                         linearizado nao volta em 20 s, e a IDE precisa
                         responder "nao sei" em vez de congelar
o float QUEBRA           os parametros viajam como TEXTO decimal e o outro lado
e quebra DEVAGAR         faz `Rational("0.5")`, que e' exato. Com float,
                         `RecursionError` depois de 4,2 s
`**` nao e' `^`          troca textual no `oraculo::portao`
o `pi` e' SILENCIOSO     o portao recusa toda expressao cujo `var_names()` nao
                         seja exatamente `["t"]`. Preencher o vetor pelo
                         tamanho devolve 0,086 onde a resposta e' 0,100, SEM
                         ERRO NENHUM
vocabulario alheio       coberto pela MESMA trava: o que o exmex nao le' morre
                         no parse (alto), o que ele le' errado vira variavel
                         livre (silencioso). Uma trava, dois modos de falha
```

**A dependência é INJETADA, não descoberta** (`Core::set_oraculo`). O motivo é o
mesmo do `ToolDetector` com caminho de busca vazio: sem isso, o resultado de
`sim.run` passaria a depender de a máquina ter SymPy, e a suíte ficaria verde
aqui e vermelha ali — a classe de falha da qual o `verificar-shell.sh` nasceu.
O teste aponta para um `Python` **falso** que grava o que recebeu, e o cenário
viaja por **argumento**: este repositório proíbe `unsafe`, e escrever variável
de ambiente virou `unsafe` na edição 2024.

**O que ele NÃO cobre, e está dito na tela:** a forma **vetorial**. O `dsolve`
sobre sistema não foi medido, e entrar sem medir é o oposto do que este domínio
faz — então a tela do sistema carrega a mesma ressalva, e o sinal honesto dela
continua sendo o **invariante**, que é medido na trajetória do próprio autor.

### 13.11 O que o oráculo custa, e o que ele não resolve

```text
425-506 ms por corrida     dos quais ~200 ms sao o `import sympy`. E' uma vez
                           por "Integrar", que e' um gesto explicito do autor
bloqueia o laco do core    `sim.run` ja' era sincrono e ja' podia levar
                           minutos (o teto e' 100 milhoes de passos). O oraculo
                           acrescenta 5 s no PIOR caso, dentro de um desenho que
                           ja' era assim. Virar JOB e' fatia propria, e vale
                           para os dois
sem cache                  duas corridas com a mesma equacao perguntam duas
                           vezes. Nao doi hoje; se doer, a chave e'
                           (formula, parametros, condicoes iniciais)
```

### 13.12 As unidades passaram a ser CHECADAS — 2026-09-10

**A decisão é de 2026-09-05 e dependia do oráculo existir** (§7.2). Ela entrou na
mesma ida: o processo custa ~200 ms de `import` antes de qualquer conta, e
perguntar duas vezes por corrida seria desenho ruim.

**O que a IDE confere, em três camadas:**

```text
1. argumento de funcao   `sin(x)` com `x` em metros nao e' fisica: e' erro de
   transcendente          unidade que produz numero
2. os TERMOS entre si     `x + x^3` nao se soma. Pega o expoente errado
3. o LADO ESQUERDO        a equacao tem de ser da grandeza do estado dividida
                          pelo tempo elevado a ordem. E' esta camada que pega
                          `-(k/m)*x*x*x` SOZINHO, que e' coerente consigo mesmo
                          e nao e' uma aceleracao
```

**Na forma vetorial ela vale mais, e a razão é aritmética:** são `n` equações, e
cada uma tem o próprio lado esquerdo. Medido — trocar a derivada de uma POSIÇÃO
pela de uma VELOCIDADE **passa no `sim.checkSystem`**, porque ele confere ligação
e não física, e **não passa aqui**.

**Três armadilhas medidas antes do código** (`../roadmaps/31` §19.5), e as três
produziriam veredito errado em silêncio:

```text
substituir pela UNIDADE   `a*x - b*v` vira `u - u = 0`, e a dimensao de zero e'
crua faz cancelar         1. Cada variavel ganha um SIMBOLO POSITIVO proprio
o `check_dimensions`      medido: ele ACEITA `length^3/time^2 + length/time^2`
fica CEGO com simbolo     quando ha' simbolo livre. A defesa NAO e' usa-lo: e'
livre                     decompor cada parcela em dimensoes de BASE e comparar
o expoente volta FLOAT    `(x^2+y^2)^1.5` da' `length^1.00000000000000`, que num
                          dicionario nao e' igual a `length^1` — e as quatro
                          equacoes CERTAS da orbita foram reprovadas por isso
```

**E uma quarta, que mudou o protocolo com o processo:** o `dsolve` do pêndulo não
linearizado não volta, o teto o mata, e **o veredito de unidade morria junto** —
pronto em 3 ms, dizendo exatamente o que estava errado. O processo passou a
responder em **duas linhas, a barata primeiro**, e o core lê linha a linha. O
teto não descarta mais o que já chegou.

**O limite vai na tela junto com o recurso, e é o mesmo da §7.2:** checagem
dimensional pega incoerência, nunca pega fórmula errada. `E = m·v²` sem o meio
passa. Isso reforça — nunca substitui — o aviso da §5.1.

**O que ficou de fora, e por quê:** a forma **algébrica**. Ela não declara a
unidade do RESULTADO (o `energia-cinetica` declara `m` e `v`, não o joule),
então só metade da checagem seria possível — e meia checagem numa tela que
promete conferir é pior que nenhuma. Fechar isso é acrescentar unidade de
resultado ao catálogo, que é trabalho de tabela, não de motor.

## Apêndice A — O catálogo mapeado, e "tudo" virando número

> **Rascunho de ESCOPO, não catálogo auditado.** Levantado em 2026-09-05 a partir
> do pedido do autor (*"matematica/fisica basica, I, II, III e IV. Ou seja
> tudo"*). Cada entrada ainda tem de passar pelo gate da §8 — estudo de
> convergência na faixa mensurável — antes de existir de verdade. **Isto diz o
> tamanho da coisa, não que ela está pronta.**
>
> **A tabela da A.4 é a FONTE.** Toda contagem abaixo se reconta a partir dela,
> para nenhum número desta seção envelhecer sozinho:
>
> ```bash
> sed -n '/^DISCIPLINA /,/^```$/p' docs/arquitetura/34-simulacao-por-conceito.md \
>   | grep -E '^[A-Z][a-z]+ ' | awk '{print $NF}' | sort | uniq -c   # fechada sim/nao
> sed -n '/^DISCIPLINA /,/^```$/p' docs/arquitetura/34-simulacao-por-conceito.md \
>   | grep -cE '^[A-Z][a-z]+ '                                        # total
> ```

**100 conceitos**, distribuídos assim:

```text
  Matematica basica        8
  Calculo I                7
  Calculo II               6
  Calculo III             10
  Calculo IV               9
  Fisica basica           16
  Fisica I                12
  Fisica II                9
  Fisica III              12
  Fisica IV               11
```

### A.1 O que isto revela sobre o MOTOR

```text
forma   conceitos      %     o que ela custa
ALG          66     66.0%   nenhum integrador — avaliar e pronto
EDO2         14     14.0%   integrador de 2a ordem (RK4)
EDP           9      9.0%   malha, contorno, estabilidade — a mais cara de todas
SIST          7      7.0%   integrador vetorial
EDO1          4      4.0%   integrador de 1a ordem
```

**Dois terços de "tudo" são a forma mais barata.** A `ALGEBRICA` — que não
tem integrador nenhum — carrega 66 dos 100 conceitos: todo o Cálculo I e III,
a maior parte da Física básica, II, III e IV.

```text
  ate' ALG      66 de 100 conceitos  ( 66.0%)
  ate' EDO2     80 de 100 conceitos  ( 80.0%)
  ate' EDP      89 de 100 conceitos  ( 89.0%)
  ate' SIST     96 de 100 conceitos  ( 96.0%)
  ate' EDO1    100 de 100 conceitos  (100.0%)
```

**E a EDP, que o autor decidiu incluir nesta etapa, destrava 9%** — sendo a
forma que custa mais que as outras quatro somadas (§4.1). Isso **não reabre a
decisão**, que é dele e está registrada: informa a ORDEM de construção dentro
da etapa, que continua em aberto (§11).

### A.2 A vista, e por que o 2D domina
```text
  2D      68     68.0%
  3D      24     24.0%
  MAPA     8      8.0%
```

### A.3 O oráculo tem com que comparar em 88 dos 100

```text
  com solucao fechada conhecida     88   88%
  sem                               12   12%
```

**Este é o número que sustenta "exato e preciso" da §7.** Em 88 dos 100
conceitos a IDE tem o valor verdadeiro para pôr ao lado do numérico e mostrar
a diferença. Nos outros 12 — pêndulo não-linear, órbitas, corpo rígido,
momento angular, e as EDPs — o gate é o MMS da §8, não a comparação direta.

### A.4 A tabela

```text
DISCIPLINA          CONCEITO                                    FORMA  VISTA  FECH.
------------------------------------------------------------------------------------

Matematica basica   Funcao afim e linear                        ALG    2D     sim
Matematica basica   Funcao quadratica                           ALG    2D     sim
Matematica basica   Exponencial e logaritmo                     ALG    2D     sim
Matematica basica   Trigonometria                               ALG    2D     sim
Matematica basica   Geometria analitica: reta e conicas         ALG    2D     sim
Matematica basica   Vetores no plano e no espaco                ALG    3D     sim
Matematica basica   Matrizes e sistemas lineares                ALG    2D     sim
Matematica basica   Numeros complexos                           ALG    2D     sim

Calculo I           Limite                                      ALG    2D     sim
Calculo I           Derivada                                    ALG    2D     sim
Calculo I           Reta tangente e taxa de variacao            ALG    2D     sim
Calculo I           Maximos e minimos                           ALG    2D     sim
Calculo I           Integral definida e area                    ALG    2D     sim
Calculo I           Teorema fundamental do calculo              ALG    2D     sim
Calculo I           Solido de revolucao                         ALG    3D     sim

Calculo II          Tecnicas de integracao                      ALG    2D     sim
Calculo II          Integral impropria                          ALG    2D     sim
Calculo II          Sequencias e series                         ALG    2D     sim
Calculo II          Serie de Taylor e Maclaurin                 ALG    2D     sim
Calculo II          Coordenadas polares                         ALG    2D     sim
Calculo II          EDO de 1a ordem separavel e linear          EDO1   2D     sim

Calculo III         Funcoes de varias variaveis e superficies   ALG    3D     sim
Calculo III         Derivadas parciais                          ALG    3D     sim
Calculo III         Gradiente e derivada direcional             ALG    3D     sim
Calculo III         Multiplicadores de Lagrange                 ALG    3D     sim
Calculo III         Integral dupla                              ALG    3D     sim
Calculo III         Integral tripla                             ALG    3D     sim
Calculo III         Campos vetoriais                            ALG    3D     sim
Calculo III         Integral de linha                           ALG    3D     sim
Calculo III         Integral de superficie                      ALG    3D     sim
Calculo III         Green, Stokes e Gauss                       ALG    3D     sim

Calculo IV          EDO de 2a ordem homogenea                   EDO2   2D     sim
Calculo IV          EDO de 2a ordem nao homogenea               EDO2   2D     sim
Calculo IV          Sistemas de EDO                             SIST   3D     sim
Calculo IV          Solucao por serie de potencias              ALG    2D     sim
Calculo IV          Transformada de Laplace                     ALG    2D     sim
Calculo IV          Serie de Fourier                            ALG    2D     sim
Calculo IV          EDP: equacao do calor                       EDP    MAPA   -
Calculo IV          EDP: equacao da onda                        EDP    MAPA   -
Calculo IV          EDP: equacao de Laplace                     EDP    MAPA   -

Fisica basica       MRU                                         ALG    2D     sim
Fisica basica       MRUV                                        ALG    2D     sim
Fisica basica       Queda livre e lancamento vertical           EDO2   2D     sim
Fisica basica       Lancamento obliquo                          SIST   3D     sim
Fisica basica       Leis de Newton e plano inclinado            EDO2   2D     sim
Fisica basica       Atrito                                      EDO2   2D     sim
Fisica basica       Trabalho, energia e potencia                ALG    2D     sim
Fisica basica       Quantidade de movimento e colisoes          SIST   2D     sim
Fisica basica       Estatica e equilibrio                       ALG    2D     sim
Fisica basica       Hidrostatica                                ALG    2D     sim
Fisica basica       Termometria e dilatacao                     ALG    2D     sim
Fisica basica       Calorimetria                                ALG    2D     sim
Fisica basica       Gases ideais                                ALG    2D     sim
Fisica basica       Optica geometrica: espelhos e lentes        ALG    2D     sim
Fisica basica       Ondulatoria basica                          ALG    2D     sim
Fisica basica       Eletrostatica e circuitos simples           ALG    2D     sim

Fisica I            Cinematica vetorial                         SIST   3D     sim
Fisica I            Dinamica: segunda lei                       EDO2   3D     -
Fisica I            Trabalho-energia e conservacao              ALG    2D     sim
Fisica I            Momento linear e colisoes                   SIST   3D     sim
Fisica I            Rotacao de corpo rigido                     EDO2   3D     -
Fisica I            Momento angular                             SIST   3D     -
Fisica I            Gravitacao e orbitas                        SIST   3D     -
Fisica I            Movimento harmonico simples                 EDO2   2D     sim
Fisica I            Oscilador amortecido                        EDO2   2D     sim
Fisica I            Oscilador forcado e ressonancia             EDO2   2D     sim
Fisica I            Pendulo simples linearizado                 EDO2   2D     sim
Fisica I            Pendulo nao-linear                          EDO2   2D     -

Fisica II           Hidrodinamica e Bernoulli                   ALG    2D     sim
Fisica II           Ondas mecanicas                             EDP    MAPA   -
Fisica II           Ondas estacionarias                         ALG    2D     sim
Fisica II           Som e efeito Doppler                        ALG    2D     sim
Fisica II           Temperatura, calor e primeira lei           ALG    2D     sim
Fisica II           Conducao de calor                           EDP    MAPA   -
Fisica II           Segunda lei e entropia                      ALG    2D     sim
Fisica II           Maquinas termicas                           ALG    2D     sim
Fisica II           Teoria cinetica dos gases                   ALG    2D     sim

Fisica III          Campo eletrico e lei de Coulomb             ALG    3D     sim
Fisica III          Lei de Gauss                                ALG    3D     sim
Fisica III          Potencial eletrico                          EDP    MAPA   -
Fisica III          Capacitores                                 ALG    2D     sim
Fisica III          Corrente e resistencia                      ALG    2D     sim
Fisica III          Circuito RC                                 EDO1   2D     sim
Fisica III          Campo magnetico: Biot-Savart e Ampere       ALG    3D     sim
Fisica III          Inducao de Faraday                          EDO1   2D     sim
Fisica III          Circuito RL                                 EDO1   2D     sim
Fisica III          Circuito RLC                                EDO2   2D     sim
Fisica III          Corrente alternada                          EDO2   2D     sim
Fisica III          Ondas eletromagneticas                      EDP    MAPA   -

Fisica IV           Optica: matrizes ABCD                       ALG    2D     sim
Fisica IV           Interferencia: fenda dupla                  ALG    2D     sim
Fisica IV           Difracao                                    ALG    2D     sim
Fisica IV           Polarizacao                                 ALG    2D     sim
Fisica IV           Relatividade restrita                       ALG    2D     sim
Fisica IV           Radiacao de corpo negro                     ALG    2D     sim
Fisica IV           Efeito fotoeletrico                         ALG    2D     sim
Fisica IV           Atomo de Bohr                               ALG    2D     sim
Fisica IV           Dualidade e de Broglie                      ALG    2D     sim
Fisica IV           Schrodinger: poco de potencial              EDP    MAPA   sim
Fisica IV           Atomo de hidrogenio                         EDP    3D     sim
```
