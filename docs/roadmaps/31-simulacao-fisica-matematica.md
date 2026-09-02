# Simulação física/matemática por layout, com exibição em OpenGL

> **Classe: PLANO** (`docs/README.md`) — e a parte mais fraca dela: isto é
> **estudo registrado, não arquitetura decidida**. Nada aqui está em fila de
> execução, nenhuma linha de código foi escrita, e nenhuma das perguntas da §5
> tem resposta hoje.
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

### 5.1 Quem calcula, e onde o resultado é desenhado

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

### 5.2 O que é, exatamente, um "conceito matemático/físico selecionado"

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

### 5.3 Como a equação do usuário vira cálculo

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

### 5.4 Onde a simulação mora no projeto do usuário

Uma simulação montada por layout é **conteúdo do usuário**, não configuração da
IDE: precisa ser versionável, diffável e sobreviver a uma troca de máquina. O
precedente existe (`.kinein/runconfigs.json` e `.kinein/settings.json`, ambos
com `schemaVersion`, ambos tratando arquivo inválido como vazio em vez de
quebrar), mas nenhum deles guarda conteúdo autoral.

Perguntas abertas: formato texto (diffável) ou binário? Dentro de `.kinein/`
(que hoje também guarda build dir e o SQLite de rascunhos) ou em pasta própria
do projeto? Um arquivo por simulação ou um catálogo?

### 5.5 Determinismo, unidades e precisão

Uma simulação que não diz seu passo de integração, suas unidades e sua precisão
não é resultado — é animação. Antes da primeira linha: o método de integração é
escolha do usuário ou da IDE? Unidades são declaradas ou implícitas? Dois runs
com a mesma entrada produzem o mesmo resultado?

Isso não é rigor acadêmico: é a mesma regra que já vale no resto do projeto —
número sem procedência mente (`docs/README.md`), e ferramenta que mente é pior
que ferramenta ausente.

### 5.6 O risco de virar uma IDE dentro da IDE

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

### 5.7 A ordem interna das três responsabilidades

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
