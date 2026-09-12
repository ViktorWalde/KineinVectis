# Simulação física/matemática — o registro do que existiu (2026-09-01 → 2026-09-12)

> **Histórico, não estado.** Em 2026-09-12 o autor decidiu tirar a simulação
> física/matemática do produto: *"esquece a parte de simulação
> física/matemática; vamos refinar ao máximo para sistemas embarcados e
> desenvolvimento de software"* e, à noite, *"vamos remover também tudo sobre
> a parte de simulação"* — quem quiser simular escreve o próprio código e a IDE
> o roda e mostra como qualquer programa. O código (domínio `sim` no core e no
> protocolo, `kinein-sim`, os painéis QML, 7 harnesses, o oráculo SymPy, o
> `exmex`) saiu no mesmo commit; a história fica no git e aqui. Ao lado, os
> documentos inteiros que a descreviam: `31-simulacao-fisica-matematica.md`
> (o estudo), `34-simulacao-por-conceito.md` (a arquitetura),
> `ADR-0006-exmex-avaliador-de-expressao.md`.
>
> Abaixo, **verbatim**, os blocos que estavam na fila viva
> (`roadmaps/40-estado-e-continuidade.md`) e foram cortados de lá em
> 2026-09-12, na ordem em que apareciam. Nada foi reescrito.



---

## Extraído de: 40 §3.5

## 3.5 O que a sessão de 2026-09-05 entregou: a etapa 28, em duas metades

**A etapa 28 pedia responder as sete perguntas da §5 do
[`31`](31-simulacao-fisica-matematica.md) e PRODUZIR ARQUITETURA.** A sessão fez
isso e foi além, em duas metades separadas por uma instrução do autor.

**Primeira metade — medir e perguntar, com código ZERO.** Por instrução dele:
medição antes, perguntas com opções depois, e nenhuma linha escrita até as sete
respostas existirem. A arquitetura saiu daí, em
[`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md).

**Segunda metade — a primeira fatia de código**, quando ele mandou seguir:
domínio `sim` com nove métodos, catálogo de 17 conceitos, ligação explícita de
variáveis, integrador verificado por ordem de convergência, tela com gráfico 2D,
e persistência em `.kinein/simulacoes/`.

**O que a medição achou, e que nenhuma suposição teria achado:**

```text
evalexpr trocou de licenca     o avaliador de expressao mais usado do
                               ecossistema Rust (9,96M downloads) era MIT ate' a
                               v11 e virou AGPL-3.0-only na v12 (2024-10-17).
                               Quem adotou antes e rodou `cargo update` passou a
                               distribuir AGPL sem decidir nada. O deny.toml
                               reprova — e so' pega porque roda a cada build
mexprp reprova pela transitiva a licenca DELE e' MPL-2.0; quem reprova e' o
                               `rug`/`gmp-mpfr-sys` (GMP/MPFR), LGPL-3.0+
compilar custa 140x mais       do que rende: 490 ms de `cargo` por edicao da
                               formula para economizar 3 ms de conta num
                               pendulo. O ponto de virada e' 21,5 MILHOES de
                               avaliacoes
o exmex tem 4 armadilhas       e uma e' SILENCIOSA: `var_names()` devolve em
                               ordem ALFABETICA, nao a da formula. Quem casar
                               por ordem de leitura calcula a fisica errada sem
                               erro nenhum
a memoria compartilhada        nao e' exigencia do transporte. A §5.1.1 dizia
nao era necessaria             "250 MB/s, exige segundo canal" — isso vale para
                               frame CRU. Comprimido, um grafico 1920x1080 a
                               30fps sao 1,42 MB/s: tres vezes a grade do
                               terminal, que o JSON-RPC ja' carrega hoje
uom nao serve com formula      ele checa em tempo de COMPILACAO; formula digitada
digitada pelo usuario          pelo usuario nao tem tipo Rust nenhum
"exato" nao existe             Euler com dt=0.1 erra por 3,11 onde a resposta e'
                               -0,276 — onze vezes a propria resposta. E mais
                               passos pode PIORAR: RK4 com dt=1e-7 e' 108x pior
                               que com dt=1e-5
o Maxima nao mostra passos     pedido oficial #103: "won't fix". Nem o SymPy,
                               para EDO. A narracao algebrica de EDO nao existe
                               em ferramenta auditavel — o Wolfram tem, com
                               heuristica propria e POR FORA do motor que calcula
symbolica e' Pylance de novo   unico CAS serio em Rust, "fonte publica, proibido
                               copiar ou distribuir sem permissao expressa"
```

**E "tudo" virou número.** O pedido *"matematica/fisica basica, I, II, III e IV
— ou seja tudo"* foi mapeado contra as cinco formas do motor, no Apêndice A do
[`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md), e a tabela
de lá é a fonte que se reconta sozinha:

```text
100 conceitos          8 disciplinas, de Matematica basica a Fisica IV
 66% forma ALGEBRICA   dois tercos de "tudo" NAO precisa de integrador nenhum
 88% com solucao       o oraculo tem com que comparar em 88 dos 100 — e' este
     fechada           numero que sustenta o "exato e preciso"
  9% forma EDP         a que o autor pos na etapa, e a que custa mais que as
                       outras quatro somadas. Nao reabre a decisao: informa a
                       ORDEM de construcao dentro da etapa, que segue aberta
```

**Uma correção minha, registrada porque premissa errada não some sozinha:**
ofereci o Maxima como caminho para o passo a passo algébrico e o autor escolheu
com base nisso. Exercitar as duas ferramentas **antes de escrever código** mostrou
que a premissa estava errada. O CAS entrou na função que ele de fato cumpre —
**oráculo de exatidão**, não narrador. Registro em
[`31`](31-simulacao-fisica-matematica.md) §13.


---

## Extraído de: 40 §4 (item 28 e as entradas sim:)

28  simulacao fisica/matematica         as SETE perguntas da §5 do roadmaps/31
                                          respondidas, ARQUITETURA em
                                          ../arquitetura/34, e a PRIMEIRA FATIA
                                          DE CODIGO entregue em 2026-09-05:
                                          forma algebrica + integrador (EDO 1a e
                                          2a ordem) + tela + grafico 2D +
                                          persistencia. Medicao em roadmaps/31
                                          §8 a §18. O que falta esta' abaixo
--  sim: SISTEMA_EDO                      FECHADA. O motor saiu em 2026-09-06
                                          (§7.1) e a TELA foi ligada nele em
                                          2026-09-07 (§7.2) — antes disso o motor
                                          existia sem porta, e os tres conceitos
                                          apareciam na lista caindo na tela
                                          ESCALAR. Falta so' salvar, abaixo
--  sim: salvar um SISTEMA                 RECUSADO com motivo na tela, em
                                          2026-09-07. O `SimSaved` do protocolo
                                          carrega UMA formula e UMA ligacao, e um
                                          sistema tem uma de cada por componente:
                                          gravar assim escreveria uma montagem
                                          que nao volta. Fatia propria — muda o
                                          protocolo, o `persistencia.rs` e os
                                          testes dele
--  sim: motor de EDP                     onda, calor, Laplace, Schrodinger — a
                                          parte de CAMPO da Fisica II, III e IV.
                                          DECISAO DO AUTOR de que entra nesta
                                          etapa (2026-09-05). Custo medido em
                                          roadmaps/31 §16: malha, condicao de
                                          contorno, e a ESTABILIDADE como gate.
                                          REMEDIDA em 2026-09-06 (§19.2), e o
                                          preco subiu: o interpretado e' 90,3x o
                                          compilado, nao 7,6x, entao 201x201 por
                                          1 s sao SEIS MINUTOS no motor de hoje.
                                          "EDP sempre compila" virou PRE-REQUISITO
                                          e o motor compilado NAO EXISTE. Duas
                                          boas noticias na mesma medicao: o quadro
                                          de campo CABE no canal atual se for
                                          quantizado (1,62 MB/s a 30 fps em
                                          201x201), e a parede de estabilidade so'
                                          aparece depois de N passos que dependem
                                          da inicial — 146 com um pico, 911 com
                                          uma gaussiana
--  sim: o oraculo SymPy                  FECHADO em 2026-09-10 (§7.3). A coluna
                                          `exato` passou a ter PROCEDENCIA, e com
                                          o SymPy presente ela responde pela
                                          equacao DIGITADA. Exercitado contra o
                                          binario real: os tres casos em que a
                                          IDE mentia agora batem com o erro de
                                          verdade, e o quarto (nao linear) cai na
                                          solucao do conceito AVISANDO. Falta so'
                                          o que esta abaixo
--  sim: o oraculo da forma VETORIAL       o `dsolve` sobre SISTEMA nao foi
                                          medido, e entrar sem medir e' o oposto
                                          do que este dominio faz. Hoje a tela do
                                          sistema carrega a ressalva, e o sinal
                                          honesto dela e' o INVARIANTE
--  sim: `sim.run` deveria ser JOB         ele e' sincrono e pode levar minutos
                                          (teto de 100 milhoes de passos); o
                                          oraculo acrescenta 5 s no pior caso.
                                          Nao e' regressao desta fatia — e' um
                                          desenho que ficou visivel por causa
                                          dela
--  sim: o SymPy como oraculo (registro)  hoje a IDE so' sabia o erro dos TRES
                                          conceitos cuja solucao fechada alguem
                                          escreveu a mao. Com o SymPy, ela
                                          saberia o de qualquer equacao que o
                                          `dsolve` resolva. Decidido em
                                          2026-09-05; falta a deteccao do
                                          processo e a racionalizacao dos
                                          coeficientes (roadmaps/31 §15.5: o
                                          `dsolve` quebra com float).
                                          MEDIDA em 2026-09-06 (§19.3): o SymPy
                                          NAO ESTA nesta maquina (a medicao de
                                          2026-09-05 usou o que hoje falta), o
                                          `dsolve` TRAVA no pendulo nao
                                          linearizado (>20 s, exige teto de
                                          tempo), e o `exmex` LE a saida dele com
                                          diferenca de 6,9e-18 — uma ida por
                                          FORMULA, nao por ponto. Cobertura nova
                                          no catalogo de hoje: ZERO. O valor dele
                                          e' consertar o defeito da §19.0
--  sim: unidades CHECADAS                FECHADO em 2026-09-10 (§7.4) para as
                                          formas que INTEGRAM. Falta a ALGEBRICA,
                                          e o motivo e' de catalogo: ela nao
                                          declara a unidade do RESULTADO
                                          (`energia-cinetica` declara `m` e `v`,
                                          nao o joule), entao so' metade da
                                          checagem seria possivel — e meia
                                          checagem numa tela que promete conferir
                                          e' pior que nenhuma
--  sim: `kinein-sim` e a vista 3D        o processo separado com OpenGL
                                          offscreen. Depende do SISTEMA_EDO ou
                                          da EDP existirem — antes disso nao ha'
                                          trajetoria 3D nem campo para desenhar
--  sim: janela em resolucao cheia        a trilha e' amostrada; pedir um trecho
                                          em detalhe re-executa aquele pedaco
                                          (arquitetura/34 §7.1)


---

## Extraído de: 40 §5 (decisoes de simulacao)

simulacao: quem calcula      processo `kinein-sim` separado calcula E desenha; a
                             IDE pinta o frame como IMAGEM 2D (2026-09-03)
simulacao: catalogo          DUAS CAMADAS — o nome que o usuario conhece por
                             cima, a forma matematica por baixo (2026-09-05)
simulacao: a formula         o usuario DIGITA a equacao dentro do conceito que
                             escolheu, e a IDE alerta NA HORA da digitacao
                             quando os dois nao batem (2026-09-05)
simulacao: exato             nao existe. Integracao numerica tem erro, e a IDE
                             MOSTRA o erro em vez de prometer exatidao. O SymPy
                             entra como ORACULO, nunca como narrador (2026-09-05)
simulacao: a grade           basica + I a IV, matematica E fisica. A EDP entra
                             como QUINTA forma ja' nesta etapa, porque a parte
                             de campo da Fisica II/III/IV nao cabe nas outras
                             quatro (2026-09-05)
simulacao: unidades          declaradas E CHECADAS em runtime pelo
                             `check_dimensions` do SymPy. REVERTE a decisao de
                             rotulo-sem-checagem tomada horas antes: ela se
                             apoiava no `uom`, que checa em COMPILACAO; o SymPy
                             checa em EXECUCAO, que e' quando a formula do
                             usuario existe (2026-09-05)
simulacao: o raciocinio      so' existe para INTEGRAL (`integral_steps`). Nas
                             demais formas a IDE mostra substituicao numerica,
                             solucao fechada com o erro, e o metodo nomeado —
                             e DIZ que nao tem derivacao, em vez de inventar
simulacao: DOIS caminhos     o catalogo com formula digitada, E o usuario
                             escrevendo o proprio codigo. Rodar o codigo dele
                             JA' FUNCIONA (run.*, runConfig.*, event.run.output,
                             fswatch): falta so' ligar a saida ao desenho
                             (2026-09-05)
simulacao: a saida do codigo DECLARADA na run config, nunca adivinhada. A
                             heuristica de reconhecimento foi MEDIDA (zero falso
                             positivo contra cmake, cargo, g++, ping, df) e
                             recusada por preferencia do autor: sem magia. A IDE
                             nao linka, nao exige header, nao toca no codigo dele
simulacao: 2D ou 3D          o CONCEITO declara a vista natural, e o usuario
                             pode trocar (2026-09-05). E' a UNICA excecao ao
                             principio abaixo, e esta' registrada como excecao
simulacao: NADA ADIVINHADO   principio SUPERIOR as outras decisoes (2026-09-05).
                             Campo comeca VAZIO; o usuario preenche metodo, dt,
                             motor, amostragem e o papel de cada variavel; a IDE
                             calcula e MOSTRA, e nunca altera um numero dele.
                             REVOGOU duas decisoes do mesmo dia: a IDE escolher
                             o motor, e a IDE corrigir o dt instavel da EDP


---

## Extraído de: 40 §7 (introducao) e §7.1–§7.4

## 7. A escolha do autor para a etapa 28 — 2026-09-06

**Ele escolheu a recomendação medida.** A ordem, registrada:

```text
1. SISTEMA_EDO       a forma vetorial. ENTREGUE em 2026-09-06 (§7.1), e a TELA
                     ligada nela em 2026-09-07 (§7.2)
2. o oraculo SymPy   ENTREGUE em 2026-09-10 — ver §7.3
3. a EDP             por ultimo, e o motor compilado e' fatia PROPRIA antes dela
```

**O que sustenta a ordem está medido em [`31`](31-simulacao-fisica-matematica.md)
§19**, e o resumo de uma linha por candidata:

```text
SISTEMA_EDO   o motor interpretado de hoje BASTA (3,67 s para 100 voltas de
              orbita, 3,50 s para 60 s de pendulo duplo). Nao arrasta subsistema
              nenhum, e e' o unico que abre a vista 3D
oraculo       cobertura NOVA no catalogo de hoje: zero. O valor dele e' consertar
              o defeito da §19.0, e ele cresce com o catalogo — por isso depois
EDP           o interpretado e' 90,3x o compilado, nao 7,6x. "EDP sempre compila"
              virou PRE-REQUISITO, e o motor compilado nao existe
```

**O pedido veio com uma condição, e ela foi cumprida antes:** *"atualize/sincronize
toda a documentação antes"*. O que a sincronização de 2026-09-06 fez:

```text
arquitetura/03   os CINCO dominios ausentes ganharam secao (command, setup,
                 datasource, grafana, sim), mais o `core.*` que faltava. As
                 duas listas foram REFEITAS a partir do codigo: a de metodos
                 listava 66 de 128 sem dizer que era parcial
o par de numeros CORRIGIDO pela segunda vez, e desta vez o erro estava no
                 COMANDO: 128 metodos e 41 eventos (§1)
LEITURA_TECNICA  classe ESTADO, e estava muito velho: core listado com 25.823
                 linhas e tem 43.750; o par de numeros errado com data ao lado
arquitetura/04   413 testes -> 639, em dois lugares
```

### 7.1 O que a forma SISTEMA_EDO entregou — 2026-09-06

**Dois métodos novos, `sim.checkSystem` e `sim.runSystem`**, e o protocolo subiu
de `0.85.0` para `0.86.0`. O desenho está em
[`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md) §13, escrito
**antes** do código.

```text
tres conceitos novos   orbita de dois corpos, pendulo duplo, massa-mola acoplada
tres metodos           Euler, simpletico e RK4, os tres vetoriais
oraculo                solucao fechada onde e' honesto (a orbita CIRCULAR), e
                       INVARIANTE onde nao ha' — energia e momento angular
grafico                componentes no tempo, e trajetoria no plano declarado
```

**O core reproduz a medição que guiou o desenho**, exercitado por stdio contra o
binário de verdade em 2026-09-06 — órbita circular de raio verdadeiro 1, dez
voltas, `dt=0,01`:

```text
metodo                raio final     deriva de energia   deriva de |L|
Euler explicito         1.647957            2,032e-01       2,975e-01
Euler simpletico        1.000024            2,800e-10       1,554e-15
Runge-Kutta 4           1.000000            8,727e-11       8,727e-11
```

Os três números batem com o que a §19.1.1 do
[`31`](31-simulacao-fisica-matematica.md) mediu **antes de existir código**.

**A decisão que mais custou, e o que ela paga:** o método simplético exige saber
qual componente é posição de qual velocidade, e num sistema de primeira ordem
genérico esse par não existe. Deduzi-lo seria a dedução que erra calada, então
**o conceito declara o pareamento** — e um conceito que não declara **não oferece
o método**, com o motivo na recusa. É o que separa `1,647957` de `1,000024`.

**Três defeitos apareceram durante a fatia, e nenhum veio de gate:**

```text
o oraculo era perguntado    a corrida para em `passos * dt`, e o oraculo era
pelo instante ERRADO        avaliado na `duracao` PEDIDA. Com duracao = 2*pi e
                            dt = 0,01 a diferenca e' 3,2e-3 — e o "erro"
                            mostrado passava a incluir uma diferenca de TEMPO
                            que nao e' erro de integracao. Fez um RK4 medir
                            ordem 0,81. CORRIGIDO nas duas formas, escalar e
                            vetorial: a escalar carregava o mesmo defeito sem
                            aparecer, porque `duracao = 10` com `dt = 0,1` da'
                            exatamente 100 passos
as equacoes do pendulo      escritas de memoria, passaram na compilacao e no
duplo estavam ERRADAS       checador, e reprovaram no INVARIANTE: a energia
                            derivou 2,18 onde deveria ficar parada. Derivadas
                            de novo com o SymPy e conferidas contra a
                            lagrangiana em 2.000 pontos (maior diferenca
                            7,1e-15). **O invariante pegou o que nenhum outro
                            gate pegaria** — e' exatamente para isso que ele
                            existe
o conceito normalizava      `g`, `l` e `m` iguais a 1 embutidos no catalogo e'
a fisica em silencio        a IDE decidindo fisica, que a §2.1 proibe. Achado
                            por um gate que ja' existia: `catalogo_declara_
                            conceitos_com_fonte_datada` cobra grandeza
                            declarada, e o pendulo nao declarava nenhuma
```

**Cinco mutações provam os gates do core** (simplético vira explícito, ligação
casada por nome, pareamento deixa de ser exigido, tolerância do oráculo frouxa,
oráculo no instante pedido) **e três provam o do QML**. Uma sexta mutação foi
recusada por dar falso negativo — o compilador a pegou antes do teste, que é a
armadilha já registrada.

**E o gate do QML ensinou uma coisa nova sobre gates.** Duas mutações **não
mataram**, e as duas foram registradas:

```text
teto de comprimento     `indice < values.length` era REDUNDANTE — o acesso fora
                        da faixa ja' chega como `undefined`. SAIU do codigo:
                        linha que nenhuma mutacao mata nao defende nada
guarda de `undefined`   tambem nao mata, porque em JS `undefined < x` e
                        `undefined > x` sao os dois falsos. FICOU, com o motivo
                        escrito: a alternativa e' depender de comportamento
                        implicito da linguagem
```

A linha que **de fato** defende é `!isFinite` nos extremos, e foi preciso mutar
três candidatas para descobrir qual era.

### 7.2 A tela foi ligada no motor — 2026-09-07

**A fatia anterior entregou motor sem porta, e o gate inteiro ficou verde.** O
`SimPlotSystem.qml` estava no `QML_FILES`, tinha harness próprio que passava, e
não era instanciado em lugar nenhum do app; o `SimSystemController` nascia no
`AppDomains` e ninguém o lia. Medido contra o binário real antes do conserto:

```text
sim.catalog        devolve os 3 conceitos odeSystem, e a lista NAO filtra por forma
sim.checkFormula   concept=orbita-dois-corpos, formula="mu*2"  ->  {"ok": true}
sim.evaluate       -> "Orbita de dois corpos = mu*2"  =  2
```

**Pior que a ausência:** a IDE oferecia órbita, pêndulo duplo e massa-mola
acoplada, deixava a borda ficar verde numa fórmula algébrica qualquer e devolvia
um número — enquanto o integrador que resolve os três ficava inalcançável.

**O que a ligação trouxe:**

```text
o painel escolhe o RAMO pelo CATALOGO   `system.isSystem`, nunca olhando a formula
4 arquivos novos de tela                SimSystemAuthoring (n equacoes, n
                                        ligacoes, n estados iniciais),
                                        SimComponentEquation (uma equacao),
                                        SimSystemAccuracy (os DOIS sinais de
                                        exatidao) e SimSystemResultView (a
                                        procedencia e os dois modos de grafico)
1 dono para o formato do numero         `SimFormat`, singleton. A regra ia ser
                                        copiada para o segundo arquivo, e copia
                                        de derivacao e' o que o
                                        verificar-qml-duplicacao.sh persegue
o simpletico RECUSA com motivo          conceito sem pareamento declarado nao
                                        oferece o metodo, e a tela diz por que
salvar um sistema RECUSA com motivo     o `SimSaved` carrega UMA formula; um
                                        sistema tem uma por componente
```

**O gate que nasceu, e por que ele não existia.** Nenhuma das dezoito
verificações via o buraco, e a razão é estrutural: **cada uma confere o
componente por si.** O `qmllint` lê um arquivo; o de propriedades confere o
binding onde ele está escrito; o harness instancia o que o teste pediu. Faltava
alguém perguntando se **alguma tela chega ali** — que é a pergunta do usuário.

```text
verificar-qml-alcance.sh   19o gate. Todo .qml do QML_FILES e' instanciado em
                           outro arquivo do modulo, ou — se for `pragma
                           Singleton` — usado pelo nome. Sem baseline:
                           entregue e inalcancavel e' ZERO
tst_sim_system_panel       30o harness. Conceito de sistema nao cai na tela
                           escalar, ha' uma equacao e um estado inicial por
                           componente, o grafico vetorial aparece com o
                           resultado, e o simpletico e' recusado com motivo
```

**A catraca reprovou no meio da fatia, e o corte foi por RESPONSABILIDADE:** o
`SimSystemResultView` chegou a 324/300 e o que estava misturado eram duas
perguntas — *"o quanto isto erra"* e *"como isto se desenha"*. A §13.5 já as
tratava separadas no desenho, e o `SimSystemAccuracy` nasceu daí. Nenhum limite
foi levantado.

Os dois foram provados por mutação. No gate, nos dois sentidos: tirar o
`SimPlotSystem` da tela **reproduz o achado original**, e tirar o `pragma
Singleton` do `StatusColors` pega o outro caminho. No harness, três mutações —
sumir com o `SimSystemAuthoring`, deixar o campo escalar visível sempre, e calar
a recusa do simplético — derrubam três assertivas diferentes.

**A lição, e ela generaliza:** *"o motor passa nos testes"* e *"o usuário alcança
o motor"* são afirmações diferentes, e este repositório só tinha rede para a
primeira. Uma fatia que entrega core, protocolo, ponte C++, roteador e
controller **ainda não entregou nada** enquanto nenhuma tela instancia o
componente — e é o tipo de coisa que uma frase de uso acharia em cinco segundos
(§6) e que dezoito gates não acharam em um dia.

**Achado de quebra, e não é desta fatia:** o `EditorUnsavedChangesDialog.qml`
(198 linhas, do commit de fundação) não está no `QML_FILES` e ninguém o
referencia. Ele não chega ao binário, então o gate novo não o vê. Fica
registrado para decisão: ligar ou remover.

### 7.3 O oráculo entrou, e a coluna `exato` parou de mentir — 2026-09-10

**A segunda escolha da §7.** O que ela conserta é o defeito de veracidade da
§19.0 do [`31`](31-simulacao-fisica-matematica.md): a coluna `exato` vinha de
`catalogo::exata(conceito.id)` e **não olhava a fórmula digitada**, enquanto o
`sim.checkFormula` a aprovava porque confere ligação, não física.

**A decisão de desenho, e ela não era óbvia: o conserto não é esconder o
número.** Sem SymPy na máquina, a solução do conceito continua sendo a melhor
resposta disponível — o que faltava era **dizer que é ela**. Nasceu daí o
`SimAccuracySource`, e não um `if` que apaga a coluna.

**Exercitado contra o binário real**, com SymPy 1.14.0 numa venv:

```text
formula digitada          exato agora        proced.    erro abs     erro rel
-(k/m)*x - (c/m)*v          0.032128320      oracle    3.732e-06    1.161e-04
(k/m)*x - (c/m)*v       83178.343751029      oracle    1.474e+00    1.773e-05
-(k/m)*x - 2*(c/m)*v        0.006879277      oracle    3.222e-07    4.684e-05
-(k/m)*x*x*x - (c/m)*v      0.032128320     concept    8.634e-02    2.687e+00
```

**As três primeiras batem com a coluna "erro REAL" da §19.0**, que tinha sido
medida com o `dsolve` resolvendo à mão o que fora digitado. **A mentira de
78.000x acabou.** A quarta é a §19.3.5 acontecendo: o `dsolve` responde
`NotImplementedError`, a IDE cai na solução do conceito **e avisa**.

**O segundo achado da §19.0 também entrou:** o erro relativo, ao lado do
absoluto. Repare na segunda linha — `1,474` sobre 83 mil é `1,8e-5`, uma
integração excelente; a mesma tela mostrava só o absoluto.

**O que a fatia trouxe:**

```text
sim/oraculo.rs           processo externo, teto de 5 s, racionalizacao e o
                         PORTAO: `var_names()` tem de ser exatamente ["t"]
Core::set_oraculo        a dependencia e' INJETADA, nao descoberta — senao a
                         suite passa a depender de a maquina ter SymPy
SimAccuracySource        a procedencia, no protocolo. Foi a AUSENCIA deste campo
                         que fez a coluna mentir
SimAccuracyProvenance    a linha na tela, num dono so' para as duas formas
fake_sympy_oracle.py     um `python3` FALSO que grava o que recebeu. O cenario
                         viaja por ARGUMENTO: `unsafe` e' proibido aqui, e
                         escrever variavel de ambiente virou `unsafe` na edicao
                         2024 — e a trava esta certa, porque ambiente e' estado
                         global e teste que o escreve contamina o vizinho
```

**Cinco mutações provam os gates do core** (o portão para de conferir
`var_names`; some a troca de `**`; a exatidão ignora o oráculo; o teto vira 20x
maior; os parâmetros viajam como float) **e três provam o do QML** (some o
`SimAccuracyProvenance`; a tela para de reconhecer a procedência do oráculo;
some o erro relativo).

**Custo medido:** 425–506 ms por corrida, dos quais ~200 ms são o `import
sympy`. Uma ida por **fórmula**, nunca por ponto.

**O que ele NÃO cobre, e está dito na tela:** a forma vetorial. O `dsolve` sobre
sistema não foi medido, e entrar sem medir é o oposto do que este domínio faz —
a tela do sistema carrega a ressalva, e o sinal honesto dela continua sendo o
**invariante**, medido na trajetória do próprio autor.

**E uma coisa que a fatia tornou visível sem ser culpa dela:** `sim.run` é
síncrono e bloqueia o laço do core. Ele já podia levar minutos (o teto é 100
milhões de passos); o oráculo acrescenta 5 s no pior caso. Virar job é fatia
própria, e vale para os dois.

### 7.4 As unidades passaram a ser CHECADAS — 2026-09-10

**Decisão do autor em 2026-09-05, e ela dependia do oráculo existir.** A decisão
original desta etapa era rótulo SEM checagem, tomada sobre a medição de que o
`uom` checa em tempo de COMPILAÇÃO e uma fórmula digitada não tem tipo Rust
nenhum. O SymPy checa em EXECUÇÃO, que é quando a fórmula do usuário existe.

Entrou **na mesma ida do oráculo**: o processo custa ~200 ms de `import` antes
de qualquer conta.

**Três camadas, e a terceira é a que pega o caso difícil:**

```text
1. argumento de transcendente   `sin(x)` com `x` em metros
2. os TERMOS entre si           `x + x^3` nao se soma
3. o LADO ESQUERDO              a equacao tem de ser da grandeza do estado
                                dividida pelo tempo elevado a ordem — e' o que
                                pega `-(k/m)*x*x*x` SOZINHO, coerente consigo
                                mesmo e que nao e' uma aceleracao
```

**Na forma vetorial ela vale mais**, e a razão é aritmética: são `n` equações.
Medido — `vx' = x` (a posição no lugar da velocidade) **passa no
`sim.checkSystem`**, porque ele confere ligação e não física, e sai `wrongSide`
aqui.

**A medição achou TRÊS armadilhas antes do código, e as três dariam veredito
errado em silêncio** (`31` §19.5):

```text
substituir pela UNIDADE crua   `a*x - b*v` vira `u - u = 0`, e a dimensao de
faz os termos CANCELAREM       zero e' 1
o `check_dimensions` fica      medido: ele ACEITA `length^3/time^2 +
CEGO com simbolo livre         length/time^2`. A defesa NAO e' usa-lo
o expoente volta FLOAT         `length^1.00000000000000` != `length^1` num
                               dicionario, e as quatro equacoes CERTAS da
                               orbita foram reprovadas por isso
```

**E uma quarta, que mudou o protocolo com o processo.** O `dsolve` do pêndulo
não linearizado não volta, o teto de 5 s o mata, e **o veredito de unidade
morria junto** — pronto em 3 ms, dizendo exatamente o que estava errado. O
processo passou a responder em **duas linhas, a barata primeiro**, e o core lê
linha a linha; o teto não descarta mais o que já chegou. Medido contra o
binário real, com SymPy 1.14.0:

```text
formula                  unidades              ms      procedencia do exato
-(k/m)*sin(x)            dimensionalArgument   5024    concept (com a ressalva)
-(k/m)*x*x*x             wrongSide             5018    concept (com a ressalva)
```

**O limite vai na tela junto com o recurso:** unidade que fecha não quer dizer
física certa — `E = m·v²` sem o meio passa, porque coerência dimensional não vê
constante adimensional.

**A catraca reprovou dois arquivos, e os dois cortes foram por
RESPONSABILIDADE:** o `oraculo.rs` em 714/500 virou pasta (`mod` a fachada,
`programa` o Python embutido, `processo` o transporte, `portao` o que se
aceita de volta), e o `sim_corrida.rs` do protocolo em 509/500 perdeu a forma
vetorial para o `sim_sistema.rs` — o mesmo corte que o core já tinha entre
`corrida.rs` e `corrida_sistema.rs`, pela mesma razão. Nenhum limite levantado.

**O que ficou de fora, e por quê:** a forma **algébrica**. Ela não declara a
unidade do RESULTADO (`energia-cinetica` declara `m` e `v`, não o joule), então
só metade da checagem seria possível — e meia checagem numa tela que promete
conferir é pior que nenhuma. Fechar isso é trabalho de tabela, não de motor.

  protocolo  0.88.0 — `SimDimensionCheck` e `SimDimensionVerdict`
  testes     666 Rust (+8) e 31 harnesses; 3 mutacoes no core, 2 no QML


---

## Extraído de: 40 §1 (as notas de medição da etapa 28, 2026-09-05/06)

> **RECONFERIDO em 2026-09-06** com `bash scripts/verificar.sh` verde. O que
> bateu com o disco: 0.85.0, 639 testes, 28 harnesses, 18 verificações, catraca
> com 1 arquivo. O que **não** bateu foi o par `metodos`/`eventos`, e a causa
> está na nota acima. A medição das três candidatas está na
> [`31`](31-simulacao-fisica-matematica.md) §19, e a **escolha do autor** está
> na §7 deste documento.
>
> **Medido em 2026-09-05**, com o gate completo verde e a etapa 28 com CODIGO:
> o dominio `sim`, a tela dele e o INTEGRADOR. Os tres harnesses QML novos sao
> `tst_sim_controller` (a logica de montar a equacao), `tst_sim_run_controller`
> (a escolha numerica: metodo, passo, amostragem), `tst_sim_layout` (a GEOMETRIA
> da tela, na forma do `tst_configaction_layout`: a tabela de ligacao nao pode
> sumir) e `tst_sim_plot` (a ESCALA do grafico nos casos degenerados: trilha
> vazia, um ponto so', e a curva CONSTANTE, onde `max - min` vale zero e um
> grafico ingenuo divide por zero e some sem erro).
>
> **Nove metodos `sim.*`:** `catalog`, `inspectFormula`, `checkFormula`,
> `evaluate`, `estimate`, `run`, `list`, `save` e `forget`. O `estimate` existe porque a IDE MOSTRA o
> custo antes de rodar e contar passos e' regra de negocio — a UI pergunta e
> desenha, nao calcula (`ARCHITECTURE` §2).
>
> **O integrador e' verificado por ORDEM DE CONVERGENCIA**, que e' o
> procedimento da ASME V&V 20 e nao "o resultado parece razoavel": Euler mede
> 1,00, RK4 mede 3,85 na faixa onde isso e' mensuravel. E a faixa esta escrita
> na assercao, porque medir o RK4 entre 1e-4 e 1e-5 daria 0,34 e reprovaria um
> codigo correto.
>
> **E o core foi exercitado por STDIO, contra o binario de verdade**, em
> 2026-09-05. Ele reproduz as medicoes que guiaram o desenho inteiro:
>
> ```text
> metodo                calculado          erro contra o exato
> Euler explicito     -3.382195262            3.106309e+00
> Euler simpletico    -0.236244794            3.964147e-02
> Runge-Kutta 4       -0.275935335            4.906781e-05
> exato               -0.275886266940653
> ```
>
> Os tres erros batem com o que foi medido no rascunho ANTES de existir codigo
> (§8 do roadmaps/31: 3,11 / 3,96e-2 / 4,91e-5), e o valor exato bate com o
> `dsolve` do SymPy ate' a 14a casa. **A IDE mede o que a analise prometeu.**
>
> **E a simulacao SOBREVIVE a sessao**, em `.kinein/simulacoes/`, uma por
> arquivo, com `schemaVersion` e arquivo invalido tratado como ausente. O
> arquivo guarda o que o autor MONTOU — conceito, formula, ligacao, valores,
> metodo, passo — e **nenhuma linha do que a maquina produziu**. Isso e' gate,
> nao intencao: uma mutacao que injeta `"trail"` no arquivo derruba o teste
> `o_arquivo_guarda_o_que_o_autor_montou_e_nada_do_que_a_maquina_produziu`.
>
> **A catraca reprovou durante esta fatia**, e o corte foi por RESPONSABILIDADE.
> Medicao observada em 2026-09-05: ao ganhar o painel de simulacao o
> `ShellOverlays.qml` chegou a 309 linhas, acima do limite de 300 que a catraca
> guarda. Os cinco paineis de "Ambiente do projeto" (bibliotecas, banco,
> simulacao, observabilidade, instalacao) sairam para o
> `ShellEnvironmentOverlays.qml` — eles tem a mesma forma, o mesmo ciclo
> abrir/fechar, e sao o agrupamento que o menu ja' usa.
> O salto de 581 para 608 testes e de 121 para 125 metodos e' a **primeira
> fatia de codigo da etapa 28**: a forma ALGEBRICA, com `sim.catalog`,
> `sim.inspectFormula`, `sim.checkFormula` e `sim.evaluate`. O protocolo subiu
> de `0.82.0` para `0.83.0` pela mesma regra que o MongoDB e o Grafana seguiram:
> dominio novo sobe o minor.


---

## Extraído de: 35 §6 (FRENTE G — simulação)

## 6. FRENTE G — simulação

> **DEIXOU DE SER ESTUDO em 2026-09-05.** As sete perguntas da §5 do
> [`31`](31-simulacao-fisica-matematica.md) estão respondidas, a arquitetura
> está em [`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md), e
> o domínio `sim` existe em código: catálogo de 17 conceitos, checagem de
> conceito com ligação explícita, integrador verificado por ordem de
> convergência, gráfico 2D e persistência em `.kinein/simulacoes/`.
>
> O texto abaixo é de 2026-09-03 e fica como registro de quando a frente ainda
> era estudo.

Continua sendo a etapa 18 do roadmap 34, e era **ESTUDO** até 2026-09-05. As
sete perguntas de [31-simulacao-fisica-matematica.md](31-simulacao-fisica-matematica.md)
precisavam de resposta antes de existir arquitetura.

**A primeira pergunta foi RESPONDIDA em 2026-09-03**, e o invariante fica de pé.

`scripts/verificar-appimage.sh` reprova `ShaderEffect|QOpenGL|QRhi|QtQuick3D`
porque o AppImage força renderer por software — é isso que faz a IDE abrir em
qualquer máquina. **Decisão do autor: processo `kinein-sim` separado calcula e
desenha; a IDE exibe o frame como IMAGEM 2D dentro do layout.**

```text
kinein-sim   calcula + OpenGL offscreen + le o framebuffer
kinein-vectis  pinta a imagem 2D no layout — sem GPU, invariante intacto
```

**Visualmente embutido, GPU no outro processo.** Pintar imagem é 2D, e é a mesma
forma que o terminal já usa (`event.terminal.render`: o core computa a grade, o
QML desenha a ~30fps).

**Embutir a JANELA do outro processo está fora**, e não por escolha: XEmbed é
mecanismo do X11, e o Wayland rejeitou deliberadamente um equivalente. O autor
usa Wayland (medido em 2026-09-03). Detalhe e os dois custos — o render não cabe
no JSON-RPC, e ler o framebuffer é stall de pipeline — em
[`roadmaps/31`](31-simulacao-fisica-matematica.md) §5.1.1.

**A medição de 2026-09-05 desmentiu metade disso**, e o registro fica em
[`roadmaps/31`](31-simulacao-fisica-matematica.md) §8. Em uma linha: *"o render
não cabe no JSON-RPC"* vale para frame **cru**; um quadro realista de simulação
comprime de 8× (heatmap) a 176× (gráfico de linha), e um gráfico 1920×1080
comprimido a 30fps são **1,42 MB/s** — três vezes a grade do terminal, que o
JSON-RPC já carrega hoje. A memória compartilhada deixa de ser exigência do
transporte. **A §5.1 não reabre**: muda só o transporte, que a §5.1.1 já deixara
em aberto.

**E a §8 mediu as outras perguntas antes de arquitetar**, que é o que a etapa 28
pede: os crates de expressão contra o `deny.toml` real (o `evalexpr`, o mais
usado do ecossistema, **trocou de MIT para AGPL-3.0 em 2024-10-17** e reprova; o
`mexprp` reprova por LGPL na transitiva, via `rug`/`gmp-mpfr-sys`), o
`exmex` exercitado (quatro armadilhas, uma delas **silenciosa**: a ordem das
variáveis é alfabética, não a da fórmula), e o ponto de equilíbrio entre
interpretar e compilar — **compilar custa 490 ms por edição de fórmula para
economizar 3 ms de conta**.


---

## Extraído de: arquitetura/03 (a seção `sim.*`)

## `sim.*` — a simulação por conceito

Domínio da etapa 28. **Onze métodos**, nenhum evento — as corridas de hoje
terminam dentro da resposta. O desenho está em
[`34-simulacao-por-conceito.md`](34-simulacao-por-conceito.md), e os tipos em
`crates/kinein-protocol/src/sim.rs`.

```text
sim.catalog        { course? }                     -> { concepts: [SimConcept] }
sim.inspectFormula { formula }                     -> { variables: [String] }
sim.checkFormula   { concept, formula, bindings }  -> SimCheckResult
sim.evaluate       { concept, formula, bindings, values }        -> SimEvaluateResult
sim.estimate       { duration, step, samples }     -> SimEstimateResult
sim.run            { concept, formula, bindings, values, initial,
                     duration, step, method, samples }           -> SimRunResult
sim.list           {}                              -> { simulations: [SimSaved] }
sim.save           { simulation }
sim.forget         { name }

sim.checkSystem    { concept, equations }          -> SimCheckSystemResult
sim.runSystem      { concept, equations, values, initial,
                     duration, step, method, samples }  -> SimRunSystemResult
```

**Os dois últimos são a forma VETORIAL** (`dY/dt = F(t, Y)`), entrada em
2026-09-06 e desenhada em [`34`](34-simulacao-por-conceito.md) §13. O
`equations` traz **uma fórmula por componente**, cada uma com a ligação dela, e
a ORDEM da lista não importa: o core casa pelo campo `component`, porque supor
que a n-ésima fórmula é do n-ésimo componente seria adivinhar.

**O método `eulerSymplectic` é recusado quando o conceito não declara o
pareamento posição/velocidade**, com `reason: "noPairing"`. Não é limitação a
contornar: sem o par, o método não está definido. E ele importa — medido numa
órbita circular de raio verdadeiro 1 com `dt=0,01` por dez voltas, o Euler
explícito termina com raio `1,647957` e o simplético com `1,000024`.

**O `SimRunSystemResult` traz DOIS sinais de exatidão**, e o segundo não existia
na forma escalar:

```text
accuracy     o erro contra a solucao fechada, quando ela existe. Na orbita ela
             vale so' no caso CIRCULAR — a eliptica exige a equacao de Kepler,
             que e' transcendental, e o oraculo recusa em vez de aproximar
invariants   a DERIVA de cada grandeza que a fisica conserva. Existe mesmo sem
             solucao fechada, e e' o unico sinal que um sistema caotico admite.
             A tela chama de DERIVA e nunca de erro: invariante conservado nao
             significa resultado certo
```

**Os seis primeiros não exigem workspace** — montar e conferir uma fórmula não
depende de projeto aberto. Os três últimos exigem, porque a persistência mora em
`.kinein/simulacoes/`.

**A decisão que governa cada tipo deste domínio: nada é adivinhado.** Todo campo
que decide um resultado é **obrigatório** — não há método padrão, passo padrão
nem amostragem padrão. A IDE calcula e MOSTRA; quem escolhe é o usuário.

**A ligação é dado do usuário, nunca casamento por nome.** O `SimBinding` diz
qual grandeza cada variável da fórmula é. Isso não é rigor: o avaliador devolve
as variáveis em ordem **alfabética**, e montar o vetor de avaliação pela ordem de
leitura da fórmula produz um número com a física errada e **sem erro nenhum**
(ADR-0006, armadilha 1).

**O `sim.estimate` existe porque contar passos é regra de negócio.** A IDE mostra
o custo antes de rodar — quantos passos, quanto a trilha ocuparia inteira e
amostrada, e se compilar valeria a pena nesta escala. A UI pergunta e desenha;
ela não faz a conta (`ARCHITECTURE.md` §2).

**O `SimCheckResult` devolve TODOS os problemas, não o primeiro**, cada um como
uma variante tipada — `parseFailed`, `missingQuantity`, `unboundVariable`,
`unknownQuantity`, `duplicateQuantity`, `variableNotInFormula` — para a UI nunca
casar por texto de mensagem. E o erro de parse carrega a mensagem **da IDE**: o
texto do avaliador tem endereço de ponteiro dentro e nunca é repassado.

**O `SimRunResult.accuracy` só existe quando o conceito tem solução fechada**, e
a tela **diz** quando não tem, em vez de omitir a coluna e deixar parecer que o
número é exato.

> **Defeito conhecido, medido em 2026-09-06 e registrado em `../roadmaps/31`
> §19.0:** o `accuracy` vem do CONCEITO e não olha a fórmula digitada. Quando as
> duas divergem — o que o `sim.checkFormula` permite, porque ele confere ligação
> e não física — o `absoluteError` é calculado contra a solução de outra equação.
> Reproduz com `python3 scripts/exercitar-sim-oraculo.py`. O conserto depende de
> decisão de desenho e está na fila do `../roadmaps/40` §4.


---

## Extraído de: arquitetura/03 (cabeçalho, as notas do 0.87.0 e 0.88.0)

> **O `0.88.0` (2026-09-10) acrescentou o veredito de UNIDADE:**
> `SimDimensionCheck` e `SimDimensionVerdict`, no `dimensions` do `sim.run`
> (um) e do `sim.runSystem` (um por componente). Tipos novos no contrato sobem
> o minor. A medicao que os sustenta esta no
> [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md) §19.5 — e o
> que ela achou primeiro foram TRES armadilhas que dariam veredito errado em
> silencio.
>
> **RECONFERIDO em 2026-09-10**, com o gate completo verde: `0.87.0`, **130
> métodos**, **41 eventos**, **30 domínios**, e os 30 com seção aqui.
>
> **O `0.87.0` não trouxe método novo — trouxe PROCEDÊNCIA.** O `SimAccuracy` do
> `sim.run` ganhou `source` (`concept` ou `oracle`), `relativeError`,
> `solvedBy` e `closedForm`, e o `SimRunResult`/`SimRunSystemResult` ganharam
> `oracleNote`. O `source` é campo **obrigatório**, e é por isso que o minor
> sobe: quem ler a resposta antiga não o encontra. A razão de ele existir está
> medida no [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md)
> §19.0 — sem ele a coluna `exato` respondia por outra equação, e errava por
> 78.000x.
>


---

## Extraído de: LEITURA_TECNICA §3 (a linha do domínio sim)

```text
            sim       simulacao por conceito (0.83.0-0.88.0): catalogo de duas
                      camadas, ligacao EXPLICITA de variaveis, integrador
                      escalar e vetorial verificados por ORDEM DE CONVERGENCIA,
                      e o ORACULO — que desde 2026-09-10 resolve a equacao que
                      o usuario DIGITOU num processo externo opcional. A coluna
                      `exato` tem PROCEDENCIA: sem ela, ela mentia por 78.000x.
                      E desde 2026-09-10 as UNIDADES sao checadas nas formas que
                      integram — argumento de transcendente, os termos entre si,
                      e o lado esquerdo. O limite vai na tela junto: unidade que
                      fecha nao quer dizer fisica certa
```


---

## Extraído de: arquitetura/27 §6 (o simulador OpenGL como subsistema opcional)

## 6. Horizonte registrado — subsistema opcional (o simulador OpenGL)

> **ATUALIZADO em 2026-09-05: isto deixou de ser horizonte.** O domínio `sim`
> existe em código, com nove métodos, catálogo, integrador verificado por ordem
> de convergência e tela. O desenho está em
> [`34-simulacao-por-conceito.md`](34-simulacao-por-conceito.md); a medição que
> o sustenta, em [`../roadmaps/31`](../roadmaps/31-simulacao-fisica-matematica.md)
> §8 a §18.
>
> **O que esta seção ainda decide, e continua valendo:** a colisão da §6.2 (o
> AppImage força renderer por software, e é isso que faz a IDE abrir em qualquer
> máquina). Ela segue intacta — o gráfico 2D de hoje é `Canvas` raster, e a GPU
> só aparece quando o processo `kinein-sim` nascer, que ainda não aconteceu.
>
> **O texto abaixo é de 2026-07-16 e fica como registro** de quando o simulador
> era menção de exemplo, não pedido de trabalho. Ele explica por que a colisão
> foi encontrada antes de custar caro.
>
> **Ordem real do projeto, decidida pelo autor:**
>
> ```text
> 1. Solidificar C/C++ e Rust na IDE          <- e aqui que a arquitetura mira AGORA
> 2. Solidificar embarcados, de forma profissional
> 3. So entao simulacao fisica/matematica
> ```
>
> Até o passo 3 vai levar tempo. As frentes 1 e 2 deste documento (UI por
> domínio, catraca no core) servem ao passo 1 e não dependem de nada daqui.

O que o autor descreveu quando chegar a vez: simulador integrado usando OpenGL,
**desativado por padrão**, que o usuário instala/ativa.

**Atualização de 2026-09-01:** o autor acrescentou ao pedido a **autoria por
layout** (montar a simulação e digitar a fórmula na tela) e o **cálculo feito
pela IDE** a partir de um conceito físico/matemático selecionado. Isso não
invalida nada desta seção — mas põe em tensão a saída (a) da §6.3, porque
"calcular" e "desenhar com GPU" passariam a viver em processos diferentes. O
estudo, com as perguntas ainda em aberto, está em
`docs/roadmaps/31-simulacao-fisica-matematica.md`. Continua **fora de escopo
atual**.

O valor de registrar agora é um só: a 6.2 mostra que essa feature **colide com
uma garantia já conquistada** do AppImage. Saber disso desde já evita que as
decisões dos passos 1 e 2 fechem a porta — não obriga a abri-la hoje.

### 6.1 A linha, e ela já tem precedente

```text
DETECTAR o simulador existe?    → CAPACIDADE. ToolDetector, como claude/codex.
ATIVAR   o usuario quer?        → POLITICA. Settings + UI.
EXECUTAR como roda?             → FRONTEIRA. Fora do nucleo.
```

O passo 1 do §0.2f já provou o padrão: o core detecta `claude` como detecta
`cargo`, sem ramo por programa. Simulador é o mesmo caso.

### 6.2 A descoberta que muda o desenho

**O AppImage força `QT_QUICK_BACKEND=software` por padrão, de propósito.**
`packaging/appimage/kinein-portable-graphics-hook.sh` faz
`KINEIN_GRAPHICS_BACKEND:-software`, e a razão está registrada: o driver do host
não criava contexto RHI/OpenGL e a UI abortava antes do primeiro frame. A raster
oficial do Qt Quick desacoplou a abertura de EGL/Mesa/NVIDIA em Wayland e X11. E
isso só é possível porque **a UI hoje é 100% 2D** — não há uma linha de
`ShaderEffect`, `QQuickFramebufferObject` ou OpenGL em `ui/`.

O simulador precisa de GPU. Logo:

> Ativar o simulador **colide de frente com a garantia de abertura do AppImage**.
> Não é detalhe de implementação: é decisão de arquitetura, e tem de ser tomada
> antes da primeira linha de simulador.

### 6.3 As duas saídas reais

**(a) Simulador em processo próprio, com contexto GL próprio — RECOMENDADA.**
A UI da IDE continua raster e continua abrindo em qualquer máquina. O simulador é
um binário separado (`kinein-sim`), detectado como ferramenta, lançado como job,
falando o **mesmo JSON-RPC stdio** que o core já fala. Se a GPU do usuário falhar,
falha o simulador — não a IDE. É a lição do host de extensões do VS Code
(isolamento por processo) sem importar a máquina dele. Pela §6 da
`ARCHITECTURE.md`, nasce como `crates/kinein-core/src/sim/` e vira crate
`kinein-sim` quando ganhar corpo — o nome já fica certo desde o início.

**(b) A UI inteira passa a hardware quando o simulador é ativado.**
`KINEIN_GRAPHICS_BACKEND=hardware` já existe como opt-in reversível, então o
mecanismo está pronto — mas exige reinício e devolve a IDE inteira à dependência
de driver que o AppImage evitou. Um bug de GPU volta a impedir a IDE de abrir,
não só o simulador.

A (a) preserva o que já foi conquistado. A (b) é mais simples de escrever e mais
cara de manter.

### 6.4 Regra do opt-in, independente da saída escolhida

```text
- Desativado por padrao significa CUSTO ZERO: sem instancia, sem thread, sem
  binding. Precedente: TerminalGeometryOverlay atras de Loader (sem a env, o
  Loader nao instancia).
- O nucleo nao pode ter `if simulador_ativo` espalhado. Ou o modulo existe e
  responde, ou nao existe. Fronteira, nao condicional.
- Embarcados e simulacao entram como DOMINIOS (`sim/`, `target/`), no mesmo
  molde dos demais — nao como camada nova nem como excecao.
```


---

## Extraído de: GUIAIA §5.9b (o mapa do domínio sim)

### 5.9b Simulação por conceito (etapa 28)

```text
ui/qml/sim/SimController.qml         o conceito, a formula ESCALAR, a ligacao,
                                     os VALORES dos parametros e as salvas
ui/qml/sim/SimRunController.qml      a escolha NUMERICA — metodo, passo, duracao,
                                     amostragem. Vale para as DUAS formas
ui/qml/sim/SimSystemController.qml   a autoria VETORIAL: `n` equacoes, `n`
                                     ligacoes, `n` estados iniciais
ui/qml/ipc/{SimEventRouter,SimRequestRouter}.qml
ui/src/core_client_sim.cpp           pedidos + dispatch do dominio
ui/qml/sim/SimPanelHost.qml          onde os TRES controllers se encontram
ui/qml/sim/SimPanel.qml              escolhe o RAMO: escalar ou vetorial
ui/qml/sim/Sim{FormulaField,BindingTable,ValueTable,Calculation,
                RunControls,RunResultView,Plot2d}.qml          ramo ESCALAR
ui/qml/sim/Sim{SystemAuthoring,ComponentEquation,SystemAccuracy,
                SystemResultView,PlotSystem}.qml               ramo VETORIAL
ui/qml/sim/Sim{AccuracyProvenance,DimensionsView}.qml   os DOIS, procedencia
                                                        do exato e as unidades
ui/qml/sim/SimFormat.qml             singleton: como um numero de simulacao
                                     aparece, num dono so'
    ↕ crates/kinein-protocol/src/{sim,sim_corrida}.rs
crates/kinein-core/src/handlers/sim.rs
    → crates/kinein-core/src/sim/{catalogo,entradas,entradas_sistema,formula,
        corrida,corrida_sistema,integrador,sistema,exata,invariante,
        persistencia}.rs
    → crates/kinein-core/src/sim/oraculo/{mod,programa,processo,portao}.rs
        ↕ python3 + SymPy, processo EXTERNO e opcional (a fronteira do GDB)
```

- **Nada é adivinhado** (`docs/arquitetura/34` §2.1). Campo começa vazio, a
  ligação variável→grandeza é escolha do autor, e a IDE nunca preenche nem
  corrige um número dele. Casar por nome é a dedução que erra calada.
- **O catálogo tem duas camadas:** o CONCEITO que o usuário conhece por cima, a
  FORMA matemática por baixo. Conceito novo é uma entrada em `entradas.rs` ou
  `entradas_sistema.rs`, não código novo.
- **Qual ramo a tela mostra vem do catálogo** (`concept.form`), nunca de olhar a
  fórmula. Um conceito `odeSystem` na tela escalar aceita fórmula algébrica e
  devolve número — foi o defeito de 2026-09-06, medido contra o binário real.
- **O simplético exige pareamento DECLARADO.** Sem ele o método não está
  definido, e a tela o recusa com o motivo em vez de integrar outra coisa.
- **Salvar ainda não vale para a forma vetorial:** o `SimSaved` carrega uma
  fórmula, e um sistema tem uma por componente. A tela diz isso.
- **As UNIDADES são checadas** nas formas que integram (2026-09-10), no mesmo
  processo do oráculo. Três camadas: argumento de transcendente, os termos entre
  si, e o LADO ESQUERDO. O limite vai na tela: unidade que fecha não quer dizer
  física certa — `E = m·v²` sem o meio passa.
- **A coluna `exato` tem PROCEDÊNCIA** (`SimAccuracySource`, 2026-09-10). Com o
  SymPy presente ela responde pela equação DIGITADA; sem ele, pela do conceito —
  e a tela diz qual, com o motivo. Sem esse campo ela mentia por 78.000x, medido.
  A dependência é **injetada** (`Core::set_oraculo`): descobrir o `python3` no
  teste faria a suíte depender do host.
- Testes: `crates/kinein-core/src/tests/sim{,_integrador,_oraculo,_persistencia,_sistema}.rs`
  e os harnesses
  `tst_sim_{controller,run_controller,layout,plot,plot_system,system_panel,provenance}.qml`.
  O do oráculo fala com um `python3` **falso** (`scripts/fake_sympy_oracle.py`)
  que grava o pedido — o que se mede é **o que foi perguntado**, não se a
  resposta chegou.


---

## Extraído de: docs/tooling/OPEN_COMPONENT_REGISTRY.json (exmex e SymPy)

```json
[
  {
    "id": "exmex",
    "name": "exmex",
    "role": "Expression parser and evaluator for the `sim` domain: parses the formula the user types, reports which variables it uses, and evaluates it per integration step",
    "adoptionMode": "A",
    "integration": "Rust crate linked only by kinein-core, behind the `sim` domain. `var_names()` feeds the concept check that runs on every keystroke; `eval()` runs the integration step. The evaluation vector is assembled from the user's explicit variable binding, never from the crate's positional order.",
    "upstream": {
      "repository": "https://github.com/bertiqwerty/exmex/",
      "documentation": "https://docs.rs/exmex"
    },
    "version": {
      "selected": "0.21.0",
      "pin": "Cargo constraint =0.21.0 plus Cargo.lock checksum e9a53dccfc387af263e69c1eb53d13a4e9b26a7134ffdd38398a208374cbf851",
      "reason": "Current release (2026-05-23) and the only MAINTAINED expression evaluator that passes this repository's deny.toml. MSRV 1.80.1 is below the workspace 1.85. The `partial` feature (partial derivatives) adds no dependency: it is `partial = []`."
    },
    "license": {
      "spdx": "MIT OR Apache-2.0",
      "accepted": true,
      "source": "Published crate manifest, read from the vendored source on 2026-09-05"
    },
    "maintenance": {
      "statusAtReview": "Active: latest release 2026-05-23. Compared against five alternatives; evalexpr relicensed MIT -> AGPL-3.0-only at v12.0.0 (2024-10-17) and is rejected, mexprp is rejected for LGPL-3.0-or-later reaching in through rug/gmp-mpfr-sys, and fasteval/meval/rsc pass the licence gate but last shipped in 2020/2018/2024.",
      "reviewedAt": "2026-09-05"
    },
    "runtimeSecurity": {
      "telemetry": false,
      "network": false,
      "shellExecution": false,
      "secretsAccess": false,
      "userContentUpload": false,
      "nativeCode": false,
      "notes": "Measured on 2026-09-05 by grepping the vendored src/: zero occurrences of std::net, std::process, std::fs, std::env, `unsafe`, or any HTTP client. It is pure computation over f64. Four behavioural traps were found by exercising it and are recorded in ADR-0006: alphabetical variable order (defused by the explicit binding in arquitetura/34 5.0), lowercase `pi` parsed as a free variable, division by zero yielding silent inf/NaN, and parser error text containing a raw pointer address, which the IDE never forwards to the screen."
    },
    "cost": {
      "addedCrates": 1,
      "notes": "Measured against this workspace, not an empty project: its six transitive dependencies (regex, regex-automata, regex-syntax, aho-corasick, memchr, smallvec) were already present, so only exmex itself is new. An isolated measurement reports +7 and does not apply here."
    },
    "verification": "cargo deny check (advisories, bans, licenses, sources) green on the real workspace with the crate in place, 2026-09-05",
    "adr": "docs/adr/ADR-0006-exmex-avaliador-de-expressao.md"
  },
  {
    "id": "sympy",
    "name": "SymPy",
    "role": "Exactness ORACLE for the `sim` domain: solves the differential equation the user actually typed, so the `exato` column stops answering for the concept's canonical equation",
    "adoptionMode": "A",
    "integration": "External process, ORCHESTRATED and never linked — the same boundary as GDB. The core spawns `python3 -c <program>` with a JSON request on stdin and reads one JSON line back; the program is embedded in `crates/kinein-core/src/sim/oraculo.rs`. ONE trip per FORMULA, never per point: SymPy returns the closed form and `exmex` evaluates the whole trail. Absent, it degrades in the open: the `exato` column keeps the concept's answer and the screen says so (`SimAccuracySource`).",
    "upstream": {
      "repository": "https://github.com/sympy/sympy",
      "documentation": "https://docs.sympy.org/"
    },
    "version": {
      "selected": "1.14.0 (as packaged by Fedora 44: python3-sympy 1.14.0-11.fc44)",
      "pin": "NOT pinned, and it must not be: this is a tool on the USER's machine, like clangd or GDB. The IDE detects it and says when it is missing; it never installs it.",
      "reason": "Optional dependency. Measured on 2026-09-06 and again on 2026-09-10: it is ABSENT on the author's machine, which is exactly why the degraded path is the one the product ships with by default."
    },
    "license": {
      "spdx": "BSD-3-Clause AND MIT",
      "accepted": true,
      "source": "Fedora 44 package metadata for python3-sympy, read on 2026-09-06 (roadmaps/31 §19.3.1). Not linked, so no license obligation propagates to the binary either way."
    },
    "maintenance": {
      "statusAtReview": "Active: 1.14.0 is the current release, packaged by every major distribution. No alternative was adopted — the CAS written in Rust (symbolica) is source-available with proprietary terms and is FORBIDDEN here (roadmaps/31 §12.3, the Pylance category); Maxima was measured and refused as a narrator (§13.1, upstream marked the step-by-step request won't fix)."
    },
    "runtimeSecurity": "Runs as a child process with stdin/stdout piped and stderr discarded, with a 5-second ceiling and a kill on expiry. It receives only the formula the user typed plus numeric parameters — never a file path, never the workspace. Its answer is NEVER trusted as code: it goes through a gate that requires `exmex` to parse it and its only free variable to be `t`, which is what stops the silent `pi` trap measured in §19.3.4.",
    "cost": "425-506 ms per run measured end to end on 2026-09-10, of which ~200 ms is `import sympy`. Paid once per explicit 'Integrar', never per keystroke.",
    "verification": "crates/kinein-core/src/tests/sim_oraculo.rs — 9 tests against a FAKE python3 (scripts/fake_sympy_oracle.py) that records the request it received. Five mutations prove the gates. Exercised against the real binary and real SymPy on 2026-09-10 (roadmaps/40 §7.3).",
    "adr": "docs/arquitetura/34-simulacao-por-conceito.md §13.10"
  }
]
```


---

## Extraído de: roadmaps/34-depois-do-mvp §6 (FRENTE D — simulação)

## 6. FRENTE D — a simulação física/matemática

> **DEIXOU DE SER ESTUDO em 2026-09-05, e o texto abaixo é de 2026-09-02.**
> As sete perguntas da §5 do [`31`](31-simulacao-fisica-matematica.md) estão
> respondidas, a arquitetura está em
> [`../arquitetura/34`](../arquitetura/34-simulacao-por-conceito.md), e o domínio
> `sim` existe em código: 11 métodos IPC, catálogo de 20 conceitos, integrador
> escalar **e** vetorial verificados por ordem de convergência, gráfico 2D nos
> dois modos e persistência em `.kinein/simulacoes/`.
>
> **A pergunta que o parágrafo abaixo trata como aberta foi respondida**, e a
> resposta manteve o invariante: a UI segue **100% 2D**. Quem desenharia em
> OpenGL é um processo `kinein-sim` separado, e ele **não existe ainda** — a
> medição de 2026-09-05 mostrou que o quadro comprimido cabe no JSON-RPC
> (1,42 MB/s a 30fps), então a memória compartilhada deixou de ser exigência do
> transporte.
>
> O que falta está na fila viva:
> [`40`](40-estado-e-continuidade.md) §4 — o oráculo SymPy, o motor de EDP (com
> o motor compilado como pré-requisito), a vista 3D e salvar um sistema.

**Era ESTUDO até 2026-09-05, e era de propósito.** Ele lista as perguntas que precisam de
resposta antes de existir arquitetura — e uma delas colide com um invariante já
travado no gate: `scripts/verificar-appimage.sh` verifica que a UI é **100% 2D**
(sem `ShaderEffect`, `QOpenGL`, `QRhi`, `QtQuick3D`), porque o AppImage força
renderer por software.

**A etapa, quando vier, é responder às perguntas do 31 e produzir arquitetura** —
não começar a implementar. E a primeira pergunta é essa: OpenGL na mesma janela
que hoje é garantidamente 2D, ou processo/janela separada?
