# 35 — Crescer sem god object

> **Classe: ESTUDO** (`../README.md`). É análise e opções **com custo medido**,
> não plano aprovado. Nada aqui foi implementado, e nada aqui deve ser
> implementado sem decisão explícita do autor — a §5 tem trade-offs reais e uma
> recomendação que **contraria** duas das ferramentas pesquisadas.
>
> **Medido em 2026-09-10**, a pedido do autor, com o gate completo verde.
>
> **A regra zero desta página é a §1.1 da [`ARCHITECTURE.md`](ARCHITECTURE.md):**
> antes de propor arquitetura, MEDIR — porque duas vezes seguidas, neste
> repositório, a resposta certa foi *"o projeto já tem arquitetura, é boa, e não
> era aplicada"*. Esta análise começa medindo justamente para não ser a terceira.

## 1. O que está SAUDÁVEL — e a lista importa tanto quanto os problemas

Medido antes de qualquer proposta, porque metade do valor de uma análise é
impedir que alguém conserte o que não está quebrado.

```text
                    arquivos  linhas  mediana   p90   maior
crates/kinein-core       156  38.811     217    483   800  (lsp/parse.rs)
crates/kinein-protocol    32   6.371     167    393   575  (lsp.rs)
ui/src (C++)              38   5.182     103    333   501  (core_client.h)
ui/qml                   229  34.132     126    283   792  (EditorController.qml)
```

**A mediana é o número que importa, e ela é boa nas quatro camadas.** A catraca
funciona: **1 arquivo em débito entre 455**.

```text
ciclos de dependencia entre os 32 dominios do core     NENHUM
dominios que dependem de outro dominio                 11 de 32
o protocolo importa o core?                            NAO
o protocolo conhece Qt?                                NAO
rebuild incremental tocando 1 arquivo do core          2,08 s
rebuild incremental tocando o protocolo inteiro        3,18 s
```

E duas escolhas que **já são** o que a indústria recomenda, e que seria fácil
"melhorar" por engano:

- **Layout plano de crates** (`crates/<nome>`), que é exatamente a recomendação
  do matklad para projetos de 10 mil a 1 milhão de linhas — *"even comparatively
  large lists are easier to understand at a glance than even small trees"*.
- **`QML_ELEMENT`, não `setContextProperty`.** A documentação do Qt desaconselha
  context property porque *"their lookup is slow, QML compilers cannot reason
  about them"* e porque ela injeta estado sem declaração. A ponte deste projeto
  já é um tipo registrado.

## 2. O achado: o split é FÍSICO, não LÓGICO

A catraca mede **arquivo**. Um god object sobrevive a ela intacto, bastando ser
espalhado por arquivos pequenos — e é o que acontece nas duas pontes:

```text
Core          15 campos, ~176 metodos, `impl Core` em 30 arquivos
CoreClient    289 declaracoes (129 Q_INVOKABLE, 119 sinais, 18 Q_PROPERTY),
              uma classe QObject espalhada por 20 .cpp
```

Nenhum dos dois reprova em nada hoje. Os arquivos são pequenos; **o tipo é que é
grande.**

### 2.1 E a medição diz que o god object é EVITÁVEL, não necessário

Esta é a medição que muda a conversa. Quantos campos do `Core` cada handler
realmente toca:

```text
workspace       9   workspace, fswatch, syntax, lsp, workspace_edits, ...
cmake           4   workspace, events, lsp, jobs
tools, git      3
run, fs, debug, cargo, build      2
terminal, syntax, sim, setup, jobs, grafana, draft, datasource,
  configaction                    1
toolchain, settings, runconfig, probe, library, format     0
```

**Quinze dos vinte e quatro handlers tocam ZERO ou UM campo.** Seis não tocam
nenhum: são funções puras vestidas de método de um objeto que carrega quinze
campos. O acoplamento real está concentrado em dois campos — `jobs` (8 handlers)
e `workspace` (7) —, que são exatamente as duas coisas transversais de verdade.

**A leitura honesta:** o `Core` não é grande porque o domínio exige; é grande
porque `impl Core` é o único idioma que o projeto usa para chegar ao estado.

## 3. O segundo achado: os tipos do protocolo são os tipos internos do core

```text
arquivos de dominio do core que importam kinein_protocol   75 de 122
campos de struct do core cujo TIPO vem do protocolo        77
```

`GitEntryKind`, `DataSourceProfile`, `SimConcept` — os tipos que carregam
`#[derive(Serialize, Deserialize)]` para atravessar o JSON-RPC são os mesmos que
o core usa como estrutura interna.

**É a única prática deste repositório que a arquitetura do rust-analyzer
desaconselha por escrito.** Lá, apenas o crate de fronteira conhece LSP e
serialização, e *"rather than deriving serialization on core types, clients
convert between internal representations and IPC formats"* — para que
compatibilidade retroativa do fio não vire pressão sobre o modelo interno.

**O custo hoje é zero e o risco é datado:** enquanto UI e core sobem juntos, o
fio pode mudar de forma livremente. O dia em que houver uma versão do protocolo
a manter — CLI de terceiro, plugin, sessão remota — cada mudança interna passa a
ser mudança de contrato.

## 4. O que a pesquisa externa de fato diz

Quatro fontes, e o que cada uma acrescenta a **este** projeto:

**rust-analyzer — invariantes de arquitetura escritos por fronteira.** O
mecanismo central deles não é uma ferramenta: é uma lista de frases da forma
*"o crate `syntax` é completamente independente do resto; ele não sabe nada
sobre salsa nem LSP"*. Eles também nomeiam **quais** crates são fronteira de API
(`syntax`, `hir`, `ide`) e declaram os outros explicitamente internos, onde
código acoplado é permitido. **É o que este projeto tem para arquivo e não tem
para dependência.**

**matklad, *Large Rust Workspaces*.** Layout plano (já feito), manifesto raiz
virtual, `version = "0.0.0"` para crates internos, pasta separada para o que for
publicável, e automação em Rust via `cargo xtask` em vez de scripts espalhados.
Ele **não** trata de forçar aciclicidade — não há bala de prata ali.

**Fitness functions / testes de arquitetura.** A ideia é a que este repositório
já pratica sob outro nome: *"qualquer checagem automatizada que protege uma
propriedade arquitetural — não o que o código faz, mas como ele está
organizado"*. A recomendação prática é começar pequeno: um teste de dependência
entre camadas.

**cargo-pup (Datadog) — ArchUnit para Rust.** Permite escrever asserções sobre a
arquitetura em Rust e rodá-las no CI. **E aqui vai a ressalva que decide:** ele
se enfia no `rustc` como o clippy faz e **exige nightly**. Este repositório fixa
o toolchain em `rust-toolchain.toml` e proíbe `unsafe`; trazer nightly para
dentro do gate é um preço alto por uma capacidade que 60 linhas de Python já
dão, no mesmo estilo dos dezenove gates existentes.

**Qt/QML.** A recomendação é substituir um objeto de contexto único por
singletons por domínio. Metade já está feita aqui (`QML_ELEMENT`, roteadores
`<X>EventRouter` por domínio); o que resta é o `CoreClient` ser **um** tipo.

## 5. As três saídas, com custo medido

### 5.1 O que NÃO fazer, e por que — medido

```text
quebrar o core em crates      Nao ha' dor para aliviar. O argumento classico e'
"para compilar mais rapido"   tempo de compilacao, e ele foi MEDIDO: 2,08 s
                              tocando um arquivo do core, 3,18 s tocando o
                              protocolo inteiro. Dividir por um numero que ja'
                              e' bom e' pagar complexidade por nada

adotar o cargo-pup            exige NIGHTLY. O repositorio fixa o toolchain e o
                              gate ja' faz este tipo de checagem em Python, sem
                              dependencia nova

trocar o layout de crates     ja' e' o recomendado (plano)
```

### 5.2 O que custa pouco e paga logo

**(a) Invariantes de dependência, escritos e verificados.** O grafo entre os 32
domínios do core já é acíclico — hoje, por acidente feliz. Congelá-lo é o mesmo
gesto do `verificar-qml-duplicacao.sh`: uma baseline que só pode diminuir. Custa
o mesmo que aquele gate custou, e passa a proteger a propriedade que a análise
achou intacta antes que ela deixe de estar.

**(b) O detector de god object que falta.** A regra sai direto da §2.1 e é
mecânica: **um handler que toca zero ou um campo do `Core` não precisa ser
`impl Core`.** Hoje ela pegaria quinze de vinte e quatro — e é justamente por
isso que ela não deve nascer como reprovação, e sim como **catraca**: congela os
quinze, e o décimo sexto reprova.

**(c) Nomear as fronteiras, como o rust-analyzer faz.** Três frases, uma por
fronteira, na `ARCHITECTURE.md`: quem pode conhecer quem. Custa uma tarde e é o
que impede a próxima integração de atravessar duas camadas porque foi mais
rápido.

### 5.3 O que é decisão de produto, não de engenharia

**Separar os tipos do fio dos tipos internos** (§3). Não é refatoração cosmética:
são 77 campos e 75 arquivos. **Só vale quando existir uma versão do protocolo a
manter** — e essa é a pergunta do autor, não minha. Até lá, o acoplamento é uma
troca consciente, e o que falta é ela estar **escrita** como troca consciente, em
vez de parecer descuido para quem chegar.

## 6. Como conferir tudo que está escrito aqui

```bash
# medianas e maiores arquivos por camada
find crates ui -name '*.rs' -o -name '*.qml' -o -name '*.cpp' -o -name '*.h'

# quantos campos do Core cada handler toca (a medicao da §2.1)
for f in crates/kinein-core/src/handlers/*.rs; do
  echo "$(basename "$f") $(grep -oE 'self\.(detector|tool_registry|workspace|fswatch|syntax|events|lsp|workspace_edits|run|debug|terminal|jobs|drafts|global_storage|oraculo)\b' "$f" | sort -u | wc -l)"
done

# o custo real de recompilar (o argumento que NAO se sustenta)
touch crates/kinein-core/src/sim/catalogo.rs && time cargo build -p kinein-core
```

## 7. Fontes

- matklad, *Large Rust Workspaces* — <https://matklad.github.io/2021/08/22/large-rust-workspaces.html>
- rust-analyzer, *Architecture* (invariantes e fronteiras de API) —
  <https://rust-analyzer.github.io/book/contributing/architecture.html>
- DataDog, *cargo-pup* (ArchUnit para Rust; exige nightly) —
  <https://github.com/DataDog/cargo-pup>
- Qt, *Context properties* (por que evitá-las) —
  <https://doc.qt.io/qt-6/qmllint-warnings-and-errors-context-properties.html>
- Qt, *Singletons in QML* — <https://doc.qt.io/qt-6/qml-singleton.html>
- KDAB, *Cleaner QML Controller Wiring with Singleton Instances* —
  <https://www.kdab.com/singleton-controllers-in-times-of-declarative-qml/>
