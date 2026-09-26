# Modelo semântico profundo do projeto — 0.4 em diante

> **Classe: ALVO / PLANO.** Escrito em 2026-09-25, a partir de pedido do autor e
> de medição do que já existe no core. Nada aqui está implementado. As decisões
> do autor registradas na §6 são **fechadas** e não se reabrem por conveniência
> de implementação.

## 1. O pedido, nas palavras do autor

> "toda a leitura automática do projeto, [...] a construção de um modelo
> semântico profundo do projeto, usando LSP + build system + package manager +
> toolchain + metadata da linguagem [...] ler o projeto inteiramente, entender
> onde cada parte se conecta, o porquê, as dependências [...] evitar que tenha
> quebra de ambiente por conta de atualizar apenas uma dependência e não outra,
> ter uma recomendação visual e implementação automatizada."

Este documento aceita o objetivo e **discorda de parte do meio**, com motivo
medido. O que muda está na §4.

## 2. O que já existe, medido em 2026-09-25

Não se começa do zero, e é importante dizer de onde se parte:

- `crates/kinein-core/src/index/context/` já carrega um contexto por workspace,
  recarregado quando o build muda e consultado **por arquivo**. Tem `cargo.rs` e
  `cdb.rs` (o `compile_commands.json`). O roadmap 47 já chama essa superfície de
  **C2 — "por que este arquivo compila assim"**;
- `crates/kinein-core/src/cargo.rs` roda `cargo metadata` e o interpreta;
- o `SyntaxTreeService` mantém a árvore Tree-sitter viva por documento;
- o LSP já responde `workspace/symbol` e `documentSymbol`.

**E o limite atual, por escrito no próprio tipo:** `CargoMetadataResult` diz
*"Workspace packages (no external dependencies)"*. O grafo de dependências
**ainda não existe** — só os membros do workspace. É daí que este trabalho
começa.

## 3. O princípio: a IDE lê e orquestra; ela nunca resolve

O projeto já tem a regra de que ferramentas do sistema rodam como processos e
não são reimplementadas (`ssh`, `rsync`, `gdb`, `openocd`). Aplicada aqui:

> **Quem resolve versão é o `cargo`, o `uv`/`poetry`, o `conan`/`vcpkg`, o
> `west`. A IDE roda a ferramenta do ecossistema e mostra o que ela decidiu.**

Reimplementar resolução significa reimplementar semver, unificação de features e
lockfile — e errar isso é pior que não ter. Uma sugestão errada sobre dependência
não é um botão que não funciona: é um build quebrado com a assinatura da IDE.

## 4. Deriva é decidível; "qual versão usar" não é

Aqui está a discordância, e ela é por ecossistema:

| ecossistema | o grafo existe? | o que realmente quebra | detectável sem compilar? |
| --- | --- | --- | --- |
| Cargo | sim (`cargo tree -i`, lockfile) | manifesto editado sem rodar `cargo` | **sim** |
| C/C++ CMake | não há gerenciador por padrão | **ABI**, não semver | **não** |
| Python | só com `uv`/`poetry`/`pdm` | drift do venv contra o manifesto | parcial (`pip check`) |
| Embarcado | manifesto do SDK (`west`, IDF) | toolchain ≠ SDK | sim, por versão |

Em Cargo, "atualizar a dependente" quase sempre é **desnecessário**: o lockfile
já impede a quebra. Em C++, é **indecidível** sem compilar — trocar uma lib não
muda manifesto nenhum, muda o ABI. Uma sugestão automática seria confiante onde
não há como estar.

**O que a IDE faz, então:**

```text
detectar DERIVA, e não propor resolução

  manifesto  ≠ lockfile            → "Cargo.toml mudou e Cargo.lock não"
  lockfile   ≠ o que está instalado → "o venv tem 3 pacotes fora do lock"
  toolchain  ≠ o que o SDK pede     → "ESP-IDF 5.2 pede GCC 13; o kit tem 12"
  cdb        ≠ a árvore de arquivos → "3 arquivos não estão em compile_commands"

  e então: o COMANDO que conserta, com o diff que ele faria — nunca a edição
  silenciosa de um manifesto.
```

Deriva é verificável: existem dois artefatos e eles divergem. "Qual versão você
deveria usar" exige o resolvedor, e o resolvedor tem dono.

**Aceite:** nenhuma sugestão é mostrada sem que a IDE possa exibir **os dois
lados da divergência** e o comando exato que a resolve. Uma recomendação sem os
dois lados vira palpite com cara de autoridade.

## 5. O grafo, que é leitura e por isso é seguro

A parte que se paga sozinha:

```text
alvo  →  depende de  →  alvo/pacote  →  vem de  →  origem (registro, sistema, vendored)
                                     →  por quê  →  a linha do manifesto que pediu
```

Fontes, todas já disponíveis: `cargo metadata` (com dependências externas, que
hoje são descartadas), `compile_commands.json`, a API de arquivo do CMake, o
manifesto do gerenciador Python quando houver, o manifesto do SDK embarcado.

Isso **estende o C2** em vez de criar superfície nova: hoje ele explica um
arquivo; passa a explicar um alvo e a cadeia acima dele.

## 6. Decisões do autor, registradas em 2026-09-25

### 6.1 NÃO haverá customização por Lua

**Fechado.** A IDE não embute interpretador de script para customização, nem
antes nem depois da 1.0, salvo decisão nova e explícita do autor com modelo de
permissão desenhado junto. Três motivos, e o terceiro é o que pesa:

1. **Colide com decisão já registrada.** O roadmap 47 §4 põe "stores/DI/registry
   universais e API pública de plugins" fora do compromisso. Customização por
   Lua **é** API pública de plugins com outro nome.
2. **Vira ABI que não se quebra mais.** No dia em que alguém escrever um `.lua`
   útil, renomear uma função interna passa a ser mudança incompatível — e o
   projeto perde a liberdade de refatorar que a catraca de arquitetura existe
   para preservar.
3. **É execução de código arbitrário no clone.** Em 2026-09-25 esta IDE passou
   uma fatia inteira impedindo que um `.md` de terceiros lesse arquivo arbitrário
   do disco (roadmap 40 §7.104). Um `.lua` no repositório que roda ao abrir o
   projeto é estritamente pior: não há política de recurso que o contenha, porque
   o ponto dele é justamente ter acesso.

**A alternativa, se a necessidade voltar:** customização **declarativa** — tema,
atalhos e layout em arquivo de dados, sem execução. Ela não tem nenhum dos três
problemas.

### 6.2 O alvo é Linux nativo

**Fechado.** O produto é Linux; o artefato é AppImage. Isso não é uma pendência
de portabilidade, é o escopo.

Consequência concreta e registrada para não voltar como ideia: a sugestão de
espelhar o *Object Explorer* do Windows ("list handle by process") **não se
aplica** — o equivalente Linux é `/proc/<pid>/fd`, que já é a forma nativa e não
precisa de ferramenta de terceiros. Handles do Windows ficam fora por
plataforma, e não por dificuldade.

### 6.3 Monitoramento entra, e separado por alvo

Aceito, com a divisão que o próprio pedido já fazia ("depende se é embarcado ou
roda em OS"):

**Processo local (Linux), barato e sem dependência nova:**

```text
/proc/<pid>/fd      descritores abertos — arquivo, socket, pipe, o alvo de cada um
/proc/<pid>/task    threads, nome e estado de cada uma
/proc/<pid>/status  memória, contexto, sinais
```

São leituras de arquivo. Cabem no core como qualquer outra leitura confinada, e
têm o mesmo dever dos outros painéis: dizer a **idade** da medida.

**Perfilamento:** `perf` entra **como processo**, igual a `gdb` e `openocd` —
nunca como biblioteca. E com um dever explícito: `perf_event_paranoid` costuma
bloquear em máquina de usuário, e a IDE tem de **dizer isso e mostrar o comando
que libera**, em vez de exibir painel vazio. Painel vazio sem motivo é a mentira
que este projeto persegue.

**Embarcado é outra superfície.** Em bare metal não há descritor nem thread do
SO. O equivalente é RTT, SWO, semihosting e o `openocd` que já está previsto.
Essa superfície pertence à versão que o autor já reservou só para embarcados
(roadmap 47 §10.1), e **não** deve ser espremida no painel de processo local —
um painel que mente em metade dos alvos é pior que dois painéis honestos.

## 7. Ordem proposta

```text
0.4   grafo de dependências legível + detecção de deriva + C2 crescido
0.5   monitoramento de processo local (/proc: fd, threads) e perf como processo
      (a superfície de embarcados fica na versão dedicada a ela)
```

Nada disso entra na 0.3: a §4 do roadmap 47 lista o obrigatório da 0.3.5, e
nenhum item deste documento está lá.

## 8. Critérios de prova

- **Grafo:** um projeto Cargo real, um CMake real e um Python real abrem e
  mostram a cadeia; um alvo sem manifesto mostra **por que** não tem grafo, e não
  uma tela vazia.
- **Deriva:** cada caso da §4 é provocado de propósito (editar manifesto sem
  rodar a ferramenta, instalar fora do lock, trocar o toolchain) e a IDE nomeia
  os dois lados e o comando.
- **Falso positivo é reprovação.** Uma deriva anunciada que não existe custa mais
  confiança do que uma deriva não detectada — este é o critério que decide
  quando cortar escopo.
- **Monitoramento:** processo com muitos descritores abertos; processo sem
  permissão de `perf`; processo que morre com o painel aberto.
