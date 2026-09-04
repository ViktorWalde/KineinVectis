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

H  BANCO E OBSERVABILIDADE     relacional + temporal (TimescaleDB) e Grafana,
   (ver §7)                    NATIVOS. Decisao do autor em 2026-09-03.
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

### 4.2 O catálogo — 13 entradas auditadas em 2026-09-03

Cada licença foi lida **na fonte**, não no campo automático:

| id | licença | versão pinada | release | sinal forte |
| --- | --- | --- | --- | --- |
| `fmt` | MIT | 12.2.0 | 2026-06-16 | virou `std::format` no C++20 |
| `boost` | BSL-1.0 | boost-1.92.0 | 2026-08-12 | revisão formal por pares |
| `spdlog` | MIT | v1.17.0 | 2026-01-04 | — |
| `nlohmann_json` | MIT | v3.12.0 | 2025-04-11 | — |
| `catch2` | BSL-1.0 | v3.16.0 | 2026-08-25 | — |
| `googletest` | BSD-3-Clause | v1.18.0 | 2026-08-10 | — |
| `cli11` | BSD-3-Clause | v2.7.2 | 2026-08-02 | — |
| `benchmark` | Apache-2.0 | v1.9.5 | 2026-01-21 | — |
| `asio` | BSL-1.0 | asio-1-38-2 | 2026-07-19 | — |
| `abseil` | Apache-2.0 | 20260817.0 | 2026-08-18 | — |
| `zlib` | Zlib | v1.3.2 | 2026-02-17 | — |
| `sqlite3` | domínio público | version-3.53.4 | 2026-07-24 | — |
| `eigen` | **MPL-2.0 (copyleft fraco)** | 5.0.1 | 2025-11-08 | — |

**LER A FONTE É O MÉTODO, e a auditoria provou quatro vezes.** O campo
`license` da API do GitHub devolveu `NOASSERTION` para **spdlog, CLI11, zlib e
asio** — as quatro têm licença permissiva; o detector é que falha. Copiar o
campo teria marcado quatro das treze como desconhecidas. Abrir o arquivo acha
MIT, BSD-3-Clause, Zlib e BSL-1.0.

**Duas entradas exigiram cuidado extra:**

- **SQLite não tem licença — está em domínio público.** Verificado em
  `sqlite.org/copyright.html`: *"All of the code and documentation in SQLite has
  been dedicated to the public domain by the authors."* Isso é *menos*
  restritivo que permissivo, não mais.
- **Eigen é MPL-2.0, classe diferente do resto.** Copyleft fraco, verificado no
  `COPYING.README`: *"Eigen is primarily licensed under the Mozilla Public
  License 2.0"* — e há dependências externas **opcionais** sob outras licenças,
  algumas GPL. Usar o Eigen básico não contamina; ligar uma dessas opcionais
  pode. O campo `license` diz isso na cara, em vez de esconder atrás de um
  identificador SPDX que o usuário teria de ir pesquisar.

**Um erro de memória que a verificação pegou:** eu ia registrar Eigen em 3.4.0
(2021). O atual é **5.0.1 (2025-11-08)** — quatro anos de diferença. Foi a
consulta ao GitLab que corrigiu, não a revisão do texto.

**O Boost entrou como bloco**, com `Boost::boost` (as partes header-only) como
alvo. A IDE **não chuta a lista de `COMPONENTS`**: quem precisa de uma parte
compilada (filesystem, thread) acrescenta o componente, porque linkar o que o
usuário não pediu é pior que pedir que ele complete.

### 4.3 O corte que a etapa 20 cobrou

`AppDomains.qml` cruzou 400 ao ganhar o domínio `library`. Ele é composition
root, e a `ARCHITECTURE.md` §4 regra 8 nomeia o caso: dividir a composição **por
área**, deixando a contagem de arquivos crescer — nunca subir o limite.

A costura já estava visível no arquivo: as primeiras 300 linhas **instanciam** os
donos; as últimas 110 **ligam** os donos ao IPC. Instanciar e ligar são coisas
diferentes.

```text
AppDomains.qml  416 -> 308   quem instancia os donos
AppRouters.qml  novo,  138   quem liga os donos ao IPC
```

**O `AppRouters` recebe o DONO, não 18 cópias.** A alternativa era declarar uma
property por controller e repassar cada uma — dezoito propriedades de passagem,
que é exatamente o que fez do `EditorController` uma fachada de 791 linhas.

### 4.4 Como o plano vira arquivo, sem o `library` escrever nada

O botão de cada passo **não escreve**. Ele entrega ao `configaction`:

```text
library.plan  ->  passo { actionId, params }
                        |
                        v
   ConfigActionController.openWith(actionId, params)
                        |
                        v
        o dialogo que ja existe: preview + consentimento
```

O passo carrega os `params` porque, sem eles, a UI saberia **nomear** a ação mas
não **executá-la** — e o botão teria de remontar os parâmetros, pondo o mesmo
conhecimento em dois lugares. O plano carrega o que ele decidiu.

O rótulo do botão é **"revisar…"**, não "aplicar". Aplicar sem mostrar o diff
seria a IDE mexendo no arquivo do usuário por conta própria, e o consentimento
já é contrato deste projeto desde as Configuration Actions (0.63.0).

**Um detalhe de ordem que custou um bug:** `openWith` não pode selecionar a ação
na hora — `select` procura no catálogo, e o catálogo só existe depois de
`configAction.list` responder. Por isso o pedido fica pendente e é consumido no
`handleListed`, com precedência sobre a seleção anterior: quem acabou de chamar
`openWith` disse o que quer ver.

### 4.5 O que NÃO é esta frente

Não é gerenciador de pacotes, não é resolver dependência transitiva, e não é
substituir vcpkg/Conan para quem já os usa (§2.2). É **reduzir o custo de
começar** um projeto C/C++ bem configurado.

## 5. FRENTE F — embarcados, e a decisão de que ela é PLUG AND PLAY

### 5.1 A decisão, registrada em 2026-09-03

**A IDE será plug and play no contexto de sistemas embarcados para C/C++.**
Decisão explícita do autor, registrada aqui porque é compromisso de produto e
porque muda o custo da frente inteira.

**O que "plug and play" significa, dito de forma verificável** — sem isto a
frase é marketing e não critério de aceite:

```text
1  a IDE DETECTA a sonda conectada (probe) sem o usuario configurar nada
2  DEDUZ o alvo a partir dela e oferece o kit correspondente, ja preenchido
3  o ciclo build -> flash -> debug roda sem editar arquivo na mao
4  quando NAO consegue deduzir, diz o que faltou e onde procurou —
   nunca falha em silencio nem pede "configure ai"
```

**O que plug and play NÃO promete, e dizer isso agora evita a promessa vazia:**

- não adivinha memory map, linker script nem clock de uma placa custom;
- não dispensa a auditoria de ferramenta (§2.3): plug and play é a experiência
  do **usuário**, não atalho no gate de adoção;
- não substitui `openocd.cfg`/`.gdbinit` de quem já tem um afinado — reconhece
  e usa, em vez de sobrescrever.

O item **4** é o que separa isto de uma demo. Uma IDE que acerta 80% dos casos e
falha calada nos outros 20% é pior que uma que pergunta sempre, porque o usuário
perde a confiança e passa a conferir tudo.

### 5.2 O que JÁ existe de fundação, medido em 2026-09-03

A etapa 14 (protocolo `0.67.0`) entregou mais do que parecia:

```text
kit por preset     um preset = executaveis + sysroot + triple do alvo
sysroot            -> -DCMAKE_SYSROOT
cross              -> --target no cargo, CMAKE_SYSTEM_NAME/PROCESSOR no cmake
toolchainFile      o campo do preset ja e lido e exposto
```

**Isso é exatamente o formato de um "kit" de embarcado.** Um alvo
`thumbv7em-none-eabihf` com sysroot do `arm-none-eabi` já cabe no modelo de hoje
sem entidade nova — que era a aposta da §2.3 e ela se confirmou.

### 5.3 Os dois obstáculos concretos, medidos e não supostos

**1. O adaptador de debug é CONSTANTE, não escolha.**

```rust
// crates/kinein-core/src/dap/session.rs
pub(super) const ADAPTER_BINARY: &str = "lldb-dap";
```

Embarcado não debuga com `lldb-dap`: debuga com `probe-rs dap-server`, com
OpenOCD mais `gdb`, ou com pyOCD. **Sem tornar o adaptador uma escolha do kit,
não há debug de embarcado nenhum** — e o `dap/` acabou de ser cortado em quatro
donos (etapa 15), então o encaixe está limpo: `session.rs` já isola o spawn.

**Medir:** `grep -n ADAPTER_BINARY crates/kinein-core/src/dap/session.rs`.

**2. O catálogo de toolchain não conhecia cross-compilador** — resolvido em
2026-09-03, e a medição corrigiu este próprio texto: veja a nota ao fim do item.



```text
papeis hoje:      CCompiler · CxxCompiler · Generator · Cmake · Cargo
candidatos hoje:  gcc · gxx · clang · clangxx · cmake · make · ninja · cargo
```

Não há `arm-none-eabi-gcc`, nem papel para **sonda** (probe) ou **gdbserver**. O
kit sabe guardar um sysroot mas não sabe que existe um gravador.

**Medir:** `grep -oE '"[a-z0-9-]+"' crates/kinein-core/src/toolchain/catalog.rs`.

> **Correção de 2026-09-03.** Este item dizia "papéis novos: cross-compilador,
> sonda, gdbserver". A medição mostrou que **cross-compilador não é papel**:
> `arm-none-eabi-gcc` escreve a mesma `CMAKE_C_COMPILER` que o `gcc`, e dois
> papéis na mesma variável seriam duplicação. Ele entrou como **candidato**.
> E **`sonda`/`gdbserver` foram descartados**: no caminho probe-rs — o escolhido
> pelo levantamento — o adaptador fala DAP direto, sem gdbserver, e a sonda é
> detecção em runtime (etapa 24), não papel de kit. O que faltava de verdade era
> um campo `chip`, que vai no `launch` do DAP.

### 5.4 O levantamento das ferramentas vem ANTES

Nada de código antes do levantamento: o que existe de open source, estável,
atualizado e não-proprietário, com **licença e manutenção verificadas na fonte**
— o mesmo tratamento do catálogo de bibliotecas, pelo mesmo motivo, e com a
mesma lição: em 2026-09-03 a API do GitHub errou a licença de 4 das 13 libs.

**Feito em 2026-09-03:**
[`integracoes/36-ferramentas-de-embarcados.md`](../integracoes/36-ferramentas-de-embarcados.md).

| ferramenta | licença | versão | protocolo |
| --- | --- | --- | --- |
| probe-rs | MIT **e** Apache-2.0 | v0.32.0 | **DAP nativo, stdin/stdout** |
| OpenOCD | GPL-2.0-or-later | v0.12.0 | GDB remote |
| pyOCD | Apache-2.0 | v0.45.1 | GDB remote |
| QEMU | GPL-2.0 | v11.1.1 | GDB remote |

**O achado que decide a etapa 22:** o `probe-rs dap-server` comunica por
stdin/stdout quando `--port` é omitido — *exatamente* a forma que o domínio
`dap/` já usa. Adotá-lo custa tornar `ADAPTER_BINARY` uma escolha do kit e mais
nada no transporte. As outras três falam GDB remote e exigiriam ponte.

As duas GPL não são eliminadas: `LEITURA_TECNICA` §4 fato 5 já diz que
ferramenta copyleft é **executada como processo, nunca linkada** — é como o
`gdb` (GPL-3) já entra.

**O QEMU merece nota:** ele é o que permite testar flash/debug **sem hardware**,
e sem isso a frente F não teria como ter gate — seria a única frente do projeto
verificada só na mão. A regra "gate nasce de falha silenciosa" não muda porque o
domínio tem fio.

### 5.5 A ordem dentro da frente F

```text
21  levantamento (licenca, manutencao, alvos, protocolo)   FEITO (2026-09-03)
22  adaptador DAP vira escolha do kit                      FEITO (2026-09-03)
23  cross-compilador e chip do alvo                        FEITO (2026-09-03)
24  deteccao da sonda                                      FEITO (2026-09-03)
25  ciclo build -> flash -> debug, com QEMU no gate        parcial: cross
                                                           EXERCITADO em
                                                           2026-09-03
```

O **21 não é cerimônia**: sem ele, escolher entre probe-rs e OpenOCD seria
palpite, e a escolha decide o desenho do 22 (probe-rs fala DAP nativo; OpenOCD
fala GDB remote, que exigiria uma ponte).

### 5.6 O que a exercitação de 2026-09-03 provou, e o que não provou

Exercitado contra ferramentas **reais** nesta máquina — `arm-none-eabi-gcc`
15.2.0, `probe-rs` 0.32.0 e `qemu-system-arm` 10.2.2 —, dirigindo o core por
JSON-RPC como o `arquitetura/04` §8 descreve.

**PROVADO:**

```text
tools.detect      arm-none-eabi-gcc e g++ detectados
toolchain.set     cross fixado no papel cCompiler
toolchain.setKit  sysroot + triple + chip gravados no .kinein/toolchain.json
cmake.configure   success, CDB gerada com --sysroot e o cross
build.run         ELF 32-bit LSB, ARM, EABI5 — compilou para o ALVO
probe.list        contra o probe-rs real: toolAvailable, saida limpa, dica certa
qemu-system-arm   carregou e executou o ELF
```

**DOIS BUGS ACHADOS, os dois meus, nenhum pego por teste de unidade:**

1. **Bare metal não configurava.** `CMAKE_SYSTEM_NAME=Generic` sozinho não
   basta; sem `CMAKE_TRY_COMPILE_TARGET_TYPE=STATIC_LIBRARY` o configure morre
   em `undefined reference to _exit`. Os testes verificavam a *montagem* do
   argumento — nenhum rodava um configure.
2. **Diagnóstico falso de udev.** O `probe-rs` imprime os avisos de permissão
   **sempre** no Linux, como orientação de setup. Meu `hint` os lia e acusava
   udev quando a ferramenta tinha dito *"No debug probes were found"* — mandando
   o usuário mexer em regra de sistema quando o cabo é que não estava plugado.
   Ele também **colore** a saída, e os escapes ANSI iam crus para a tela.

**NÃO PROVADO, e a fronteira importa:**

```text
flash e debug via DAP   precisa de sonda fisica, ou de um alvo QEMU que o
                        probe-rs suporte. O handshake com o `probe-rs
                        dap-server` NUNCA foi executado.
imagem bootavel         o ELF gerado nao tem vector table nem linker script,
                        e o QEMU trava em HardFault. Isso e' do PROJETO do
                        usuario, nao da IDE (§5.1: "nao adivinha memory map,
                        linker script nem clock").
```

**A distinção que a exercitação tornou nítida:**

```text
DA IDE       CMAKE_TRY_COMPILE_TARGET_TYPE — o teste de compilador e' do CMake,
             nao do usuario. Sem isso, projeto NENHUM configura.
DO PROJETO   --specs=nosys.specs, linker script, startup, vector table — sao
             do alvo de quem escreve, e adivinha-los seria a IDE decidindo
             sobre hardware que nao conhece.
```

## 6. FRENTE G — simulação

Continua sendo a etapa 18 do roadmap 34 e continua **ESTUDO**. As sete perguntas
de [31-simulacao-fisica-matematica.md](31-simulacao-fisica-matematica.md)
precisam de resposta antes de existir arquitetura.

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

## 7. FRENTE H — banco de dados e observabilidade

### 7.1 A decisão, registrada em 2026-09-03

**A IDE terá integração nativa com bancos relacionais e temporais
(TimescaleDB) e com Grafana**, tudo plug and play. Decisão do autor, e ela
estende o que já estava registrado: `LEITURA_TECNICA` §6 diz que **Docker e
banco são domínios NATIVOS, não plugins**, e a ordem L0–L10 põe banco em L5.5.

Levantamento em
[`integracoes/37-banco-e-observabilidade.md`](../integracoes/37-banco-e-observabilidade.md).

### 7.2 A licença do Grafana decide a FORMA da integração

Grafana é **AGPL-3.0**, copyleft com cláusula de rede. Isso não impede nada,
mas separa o que é legal do que não é:

```text
PODE   a IDE CONVERSA com um Grafana pela HTTP API dele
NAO    embutir o Grafana na Kinein ou distribui-lo no AppImage
```

É a mesma fronteira que o `gdb` (GPL-3) já respeita — ferramenta copyleft é
**executada como processo, nunca linkada** (`LEITURA_TECNICA` §4 fato 5).

**E TimescaleDB não é uma licença só:** Apache-2.0 fora de `tsl/`, e a
**Timescale License** dentro — que é *source-available*, **não** OSI. A IDE fala
protocolo Postgres com um servidor que o usuário instalou, então a escolha de
edição é dele; mas a ressalva fica escrita, porque a frente foi definida como
"open source, sem ser proprietário".

### 7.3 O risco que esta frente traz e as outras não trouxeram

**Credencial de banco.** Hoje o `.kinein/` guarda rascunho e toolchain em texto
puro. Senha de banco ali seria **regressão de segurança**, não feature — e o
projeto tem `docs/seguranca/23` (rede contra perda de dado) mas **não tem
cofre**. Nenhuma linha de conexão a banco entra antes dessa pergunta ter dono.

## 8. A fase de POLIMENTO, e por que ela é etapa e não sentimento

Decisão do autor em 2026-09-03: **quando as funcionalidades estiverem completas
e documentadas, começa a fase de polimento/pente-fino.**

Registrar isso agora tem uma razão prática: "polir" sem critério vira refatoração
infinita. A fase precisa de porta de entrada e de saída, e as duas já existem
neste repositório:

```text
ENTRA quando   as frentes E, F, G e H estiverem documentadas e medidas —
               nao "prontas na sensacao", mas com o comando que prova cada uma
SAI quando     o registro de saidas do dogfooding parar de crescer, e o debito
               da catraca parar de cobrar pedagio em fatia nova
```

**O que a fase de polimento NÃO é:** desculpa para reabrir decisão registrada.
As de `roadmaps/34` §8 e as deste documento continuam fechadas.

## 9. Ordem linear recomendada

```text
19  Dominio `library`: catalogo curado +   FEITA em 2026-09-03 (0.68.0). Sete
    disponibilidade + plano                bibliotecas auditadas na fonte,
                                           deteccao por config package e plano
                                           que delega a escrita. 8 testes.
                                           O criterio do grep volta ZERO.

20  UI do catalogo: escolher, entender     FEITA em 2026-09-03. LibraryPanel +
    e aplicar                              LibraryPlanView + controller e os
                                           dois roteadores. As duas acoes que
                                           o plano nomeava (findPackage e
                                           fetchContent) nasceram no
                                           configaction: 16 -> 18 acoes.
                                           AppDomains cruzou 400 e virou
                                           AppDomains + AppRouters (§4.3).
                                           FECHADA em 2026-09-03 com a entrada
                                           na paleta (Ctrl+Alt+L) e o botao que
                                           entrega o passo ao configaction.

21  Levantamento de embarcados             FEITA em 2026-09-03:
    (licenca, manutencao, alvos)           integracoes/36-ferramentas-de-
                                           embarcados.md. RESULTADO: probe-rs
                                           fala DAP NATIVO por stdin/stdout —
                                           a mesma forma que o dap/ ja usa. As
                                           outras tres falam GDB remote e
                                           exigiriam ponte. Recomendacao:
                                           comecar por probe-rs.

22  Adaptador DAP vira escolha do kit      FEITA em 2026-09-03 (0.69.0). O
                                           papel `debugAdapter` entrou no
                                           toolchain; lldb-dap segue padrao e
                                           probe-rs ganha `dap-server`. O
                                           catalogo diz QUAL binario; o dap/
                                           diz COMO invocar.

23  Cross-compilador e chip do alvo        FEITA em 2026-09-03 (0.70.0), e a
                                           MEDICAO CORRIGIU O ESBOCO: cross
                                           NAO e' papel novo — escreve a mesma
                                           CMAKE_C_COMPILER, entao e'
                                           CANDIDATO do papel existente. E o
                                           chip vai no `launch` do DAP, nao na
                                           linha de comando. Os papeis `sonda`
                                           e `gdbserver` foram DESCARTADOS:
                                           especulativos no caminho probe-rs,
                                           que fala DAP direto.

24  Deteccao da sonda                      FEITA em 2026-09-03 (0.71.0):
                                           dominio `probe`, `probe.list`. O
                                           parser e' TOLERANTE porque o
                                           formato veio da comunidade e nao
                                           do binario — a saida crua volta
                                           SEMPRE, e a `hint` cobre o caso de
                                           udev, que e' onde plug and play
                                           morre. FALTA: sugerir o kit a
                                           partir da sonda, que depende de
                                           mapear VID:PID -> chip.

25  Ciclo build -> flash -> debug,         PARCIAL, e o que esta provado esta
    com QEMU no gate                       na §5.6. O cross foi exercitado
                                           contra ferramentas REAIS em
                                           2026-09-03 e achou DOIS bugs. Falta
                                           o handshake DAP com o probe-rs, que
                                           precisa de sonda ou de alvo QEMU
                                           suportado.

26  Banco: dominio relacional + o cofre    §7.3. O cofre vem ANTES da
    de credencial                          primeira conexao, nao depois.

27  Temporal (TimescaleDB) e Grafana        §7.2. Grafana por HTTP API,
    por API                                nunca embutido.

28  Simulacao: calculo sem tela            roadmaps/31 §5.7 passo 1. A §5.1
    (o passo 1 da escada)                  ja foi respondida: kinein-sim
                                           separado, IDE pinta imagem 2D.
                                           Faltam as outras seis perguntas.

29  Fase de polimento / pente-fino          §8. Comeca quando E, F, G e H
                                           estiverem documentadas e medidas.
```

**O feedback de TR1 fura esta fila.** O autor disse em 2026-09-03 que passará a
usar a IDE e reportar; perda de dados, crash e bloqueio diário vêm antes de
qualquer item planejado (`GUIAIA.md` §2), e o registro é
`docs-privada/diario/19-registro-de-saidas.md`.
