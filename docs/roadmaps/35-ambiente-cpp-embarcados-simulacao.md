# 35 — Ambiente C/C++, embarcados e simulação

> **Classe: PLANO.** Diverge da implementação por natureza. Os números da §3
> foram medidos em **2026-09-03** e cada um diz como remedi-lo.
>
> Sucessor parcial de [34-depois-do-mvp.md](34-depois-do-mvp.md), cuja ordem
> linear (§7) **fechou em 2026-09-03** — 11, 11.1, 12, 13, 14, 15, 16 e 17
> feitas; só a 18 (simulação) ficou, e ela reaparece aqui como frente G.

## 0. Como ler isto

**Regra zero: fila é hipótese, não estado.** Antes de aceitar qualquer item
abaixo como pendente, MEÇA. Cada item carrega o comando que decide se ele ainda
existe.

Este documento nasceu de uma conversa de escopo em 2026-09-03, e a primeira
coisa que a medição fez foi **encolher a frente principal**: o que parecia
"construir configuração visual de CMake do zero" tem 936 linhas de UI e 16 ações
já escrevendo `CMakeLists.txt`. Ver §3.

## 1. As três frentes novas

```text
E  AMBIENTE C/C++ FACILITADO   a PRINCIPAL, decidida pelo autor em 2026-09-03.
   (TR1)                       Importar/usar bibliotecas solidas, explicar o
                               que cada uma faz, e tornar o CMakeLists
                               configuravel sem decorar sintaxe.

F  EMBARCADOS                  ferramentas open source, estaveis, nao
   (reordenada, ver §5)        proprietarias. Sobe de L6 por decisao do autor.

G  SIMULACAO FISICA/MATEMATICA  roadmaps/31 — continua ESTUDO. A etapa 18 do
   (ver §6)                    roadmap 34 e' esta.
```

A frente **E vem primeiro**, e não por gosto: é a única das três que não
reordena nenhuma decisão registrada, é a que tem fundação medida, e é TR1 — o
autor disse em 2026-09-03 que passará a dar feedback de uso diário, o que
finalmente alimenta o registro de saídas (`docs-privada/diario/19`), vazio desde
que nasceu.

## 2. As decisões registradas em 2026-09-03

Três perguntas foram feitas ao autor antes de qualquer código, porque as três
mudam o desenho e duas contrariam algo já escrito.

### 2.1 O catálogo de bibliotecas ENDOSSA — não apenas relata

**Decisão: curado e auditado.** Quando a IDE oferece uma biblioteca, ela está
afirmando *"esta é sólida"*, e a afirmação precisa de quem a sustente. Cada
entrada do catálogo carrega:

```text
licenca permissiva VERIFICADA      (nao "parece MIT")
manutencao ativa                   (ultimo release, com data)
versao PINADA                      (tag ou commit, nunca "latest")
o que ela FAZ                      em uma frase, para quem nao conhece
link para a doc oficial
```

É o mesmo tratamento que `docs/integracoes/` já dá às ferramentas que a **Kinein
adota** — a diferença é que aqui o alvo é o projeto do **usuário**. O contrato
antigo não cobria esse caso; agora cobre.

**A alternativa recusada** foi listar o que `find_package` acha instalado sem
opinião. Ela é muito mais barata e perde exatamente o que foi pedido: quem não
sabe qual biblioteca usar não é ajudado por um inventário.

**O risco aceito, dito de frente:** catálogo é dívida por ENTRADA, não por
feature. Ele envelhece sozinho — uma lib abandonada continua no catálogo até
alguém medir. Por isso a auditoria não é fatia futura: entra junto, ou a IDE
afirma "sólida" sem ter verificado, que é a forma exata do `deny.toml` antes de
2026-08-30 e do AppImage antes do gate — **frente construída e não verificada**.
A primeira execução do `cargo-deny` achou uma dependência abandonada desde 2017.

### 2.2 O mecanismo é CMake puro: `find_package` + `FetchContent` pinado

**Decisão: sem gerenciador de pacote de terceiro.** `find_package` para o que
está no sistema; `FetchContent` com tag/commit pinado para o que não está.

```cmake
find_package(fmt CONFIG REQUIRED)
target_link_libraries(app PRIVATE fmt::fmt)
```

**vcpkg e Conan foram considerados e recusados por ora**, e nenhum dos dois por
licença — os dois são MIT e open source. O motivo é outro: cada um vira um
processo externo no caminho do `configure`, entra pelo gate de adoção (modos
A–D) e passa a ser mais uma coisa que pode falhar entre o usuário e o build. O
`find_package` é o caminho que o próprio CMake documenta e não adiciona nada
para auditar.

**O custo dessa escolha, medido e não escondido:** nem toda biblioteca tem
config package ou find module, e para essas o `FetchContent` compila do fonte —
mais lento na primeira vez. Se isso doer no uso real, o registro de saídas vai
dizer, e aí a decisão se revisita **com dado**.

O encaixe do `toolchainFile` de preset, exposto na etapa 14 (protocolo
`0.67.0`), continua valendo: quem já usa vcpkg/Conan por fora aponta o
`toolchainFile` no preset e a IDE mostra que ele existe.

### 2.3 Embarcados sobe de L6 — decisão explícita do autor

A ordem registrada em `docs/integracoes/README.md` é *"capacidade antes de
ferramenta"*, com embarcados em **L6**, depois de L5 (RemoteContext/SSH) e L5.5
(banco). **O autor decidiu em 2026-09-03 que embarcados sobe**, e esta linha é o
registro — contrariar decisão registrada exige registro novo, que é como este
repositório muda de regra (mesmo mecanismo das três árvores de documentação em
2026-08-29).

**O que a regra L0–L10 continua exigindo, e não muda:** o gate de promoção de
nível é *"ao menos uma integração vertical real, testes de falha/cancelamento,
orçamento medido, configuração reversível e nenhum processo ou handle órfão"*.
Subir na fila não dispensa o gate — só muda a ordem.

## 3. O que JÁ existe, medido em 2026-09-03

Esta seção existe porque a medição encolheu a frente E antes de ela começar.

```text
16 Configuration Actions que ESCREVEM o CMakeLists.txt:
   cmake.addExecutable · addStaticLibrary · addSourceToTarget
   cmake.addIncludeDirectory · addTargetLinkLibraries
   cmake.createDebugPreset · createReleasePreset · enableCompileCommands
   cmake.inspectCache · repairBuildDir
   cargo.addDependency · addDevDependency · addFeature · setEdition

com PREVIEW e CONSENTIMENTO antes de escrever, e cada acao ja carrega
LINK PARA A DOC OFICIAL (`"CMake: cmake-presets(7)"`).

UI:  ConfigActionController 244 · Dialog 233 · List 172 · Preview 287 = 936
Core: crates/kinein-core/src/configaction/ (9 arquivos)
```

**Medir:** `ls crates/kinein-core/src/configaction/` e
`grep -c 'id: "' crates/kinein-core/src/configaction/catalog.rs`.

**O que NÃO existe:**

```text
catalogo de bibliotecas   grep -rli "find_package\|FetchContent\|vcpkg\|conan"
                          crates/ ui/  -> ZERO no core e na UI
```

**A leitura correta disso:** a lacuna não é o maquinário de editar CMake, que
está construído e testado. É o **domínio de bibliotecas** que alimentaria as
ações que já existem. `cmake.addTargetLinkLibraries` já sabe escrever o
`target_link_libraries`; ninguém sabe dizer *qual* biblioteca linkar e por quê.

## 4. FRENTE E — o ambiente C/C++ facilitado

### 4.1 A primeira fatia: o domínio `library`

Um domínio de core novo, no molde do `toolchain/` e do `configaction/`:

```text
library/catalog.rs      a tabela curada: id, nome, o que FAZ, licenca,
                        versao pinada, doc oficial, find_package vs FetchContent
library/availability.rs  o que ESTA instalado nesta maquina (mede, nao adivinha)
library/plan.rs          o que sera escrito no CMakeLists — reusa configaction
```

**A regra que separa este domínio do `configaction`:** o `library` sabe *o que
existe no mundo e o que existe nesta máquina*; o `configaction` sabe *como
escrever no arquivo*. Um plano de biblioteca vira um plano de configuration
action — não um segundo escritor de `CMakeLists.txt`. Dois escritores do mesmo
arquivo é a forma de eles divergirem em silêncio.

**Critério de aceite:** `grep -c "CMakeLists" crates/kinein-core/src/library/`
tem que voltar **0**. Se o domínio de bibliotecas souber escrever no arquivo, o
corte falhou.

### 4.2 O catálogo inicial

Começa pequeno e honesto — 10 a 15 bibliotecas que cobrem o comum de C/C++,
cada uma auditada de verdade. Candidatas a avaliar (a auditoria decide quais
entram; esta lista é ponto de partida, **não** o catálogo):

```text
fmt · spdlog · Catch2 · GoogleTest · nlohmann_json · Eigen · SQLite ·
zlib · CLI11 · Abseil · Boost (subconjunto) · asio · benchmark
```

**Nenhuma entra sem:** licença verificada, último release com data, versão
pinada e a frase que explica o que ela faz.

### 4.3 O que NÃO é esta frente

Não é gerenciador de pacotes, não é resolver dependência transitiva, e não é
substituir vcpkg/Conan para quem já os usa (§2.2). É **reduzir o custo de
começar** um projeto C/C++ bem configurado.

## 5. FRENTE F — embarcados

Reordenada em 2026-09-03 (§2.3). **Antes de qualquer código, o levantamento:**
o que existe de open source, estável, atualizado e não-proprietário, com licença
e estado de manutenção verificados — o mesmo tratamento do catálogo de
bibliotecas, pelo mesmo motivo.

Candidatas a auditar (ponto de partida, não adoção):

```text
probe-rs   · flash e debug, forte em ARM/RISC-V, ecossistema Rust
OpenOCD    · o veterano; JTAG/SWD, cobertura ampla de alvos
pyOCD      · CMSIS-DAP, ecossistema Python
QEMU       · emulacao, permite testar sem placa
```

`docs/integracoes/README.md` §"Checklist para adicionar uma integração nova"
governa cada uma. A frente F **não abre** antes desse levantamento estar
escrito.

## 6. FRENTE G — simulação

Continua sendo a etapa 18 do roadmap 34 e continua **ESTUDO**. As sete perguntas
de [31-simulacao-fisica-matematica.md](31-simulacao-fisica-matematica.md)
precisam de resposta antes de existir arquitetura.

**A primeira pergunta colide com um invariante travado no gate:**
`scripts/verificar-appimage.sh` reprova `ShaderEffect|QOpenGL|QRhi|QtQuick3D`
porque o AppImage força renderer por software, e é isso que faz a IDE abrir em
qualquer máquina. OpenGL na mesma janela quebra essa garantia **em silêncio** —
o build passa. Ou a simulação roda em processo/janela separada, ou o invariante
muda por decisão registrada e o AppImage passa a exigir GPU.

## 7. Ordem linear recomendada

```text
19  Dominio `library`: catalogo curado +   §4.1. A fatia principal. Reusa o
    disponibilidade + plano                configaction; nao escreve arquivo.

20  UI do catalogo: escolher, entender     §4. O "explicando o que cada uma
    e aplicar                              faz" mora aqui.

21  Levantamento de embarcados             §5. Documento, nao codigo. Sem ele
    (licenca, manutencao, alvos)           a frente F nao abre.

22  Responder as perguntas do roadmaps/31  §6 + roadmap 34 etapa 18.
```

**O feedback de TR1 fura esta fila.** O autor disse em 2026-09-03 que passará a
usar a IDE e reportar; perda de dados, crash e bloqueio diário vêm antes de
qualquer item planejado (`GUIAIA.md` §2), e o registro é
`docs-privada/diario/19-registro-de-saidas.md`.
