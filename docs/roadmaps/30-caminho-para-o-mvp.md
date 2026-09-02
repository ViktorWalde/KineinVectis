# Caminho para o MVP: as etapas, em ordem linear

> **Classe: PLANO** (`docs/README.md`). Descreve o alvo e pode divergir da
> implementação — reconciliar ao retomar. O que é **medição** aqui está datado.
>
> **Decidido pelo autor em 2026-08-30.** Esta lista substitui "o que fazer
> depois?" espalhado por seis roadmaps. Ela é linear de propósito: cada etapa
> só entra depois que a anterior fecha, e a ordem é por **dependência**, não
> por preferência.

## 1. O que "inicialmente completo" quer dizer, medido

O alvo é o **MVP essencial** da §11.1 de
`docs/specs/KINEIN_VECTIS_FINALIZATION_MVP_ROADMAP_POLISH_CHECKLIST.md`. Ele
lista 21 itens. Medição de **2026-08-30**, conferindo item por item contra o
código:

```text
19 EXISTEM     workspace, deteccao de projeto, scan de toolchain, cmake
               configure/build, cargo metadata/check/build, editor, Tree-sitter,
               status de clangd/rust-analyzer, diagnosticos, terminal, wizard
               de projeto C++/Rust, settings, project health, jobs, eventos,
               storage, UI QML e core por JSON-RPC

 1 NULO        "AI Terminal externo simples" — a linha de IA na IDE foi
               CANCELADA em 2026-07-17 e o `aiBridge` saiu do codigo no 0.59.0.
               E' item morto na spec, nao pendencia. A spec §11.1 nao foi
               reescrita porque e' PLANO datado; esta linha e' a reconciliacao.

 1 FALTAVA     Configuration Actions — ZERO ocorrencias em `crates/` e `ui/`
               naquela data. ENTREGUE em 2026-09-02 (etapa 2 abaixo).
```

**Medição de 2026-09-02: os 21 itens do MVP essencial estão fechados** — 20
existem e 1 é item morto. O que resta desta lista é dívida de honestidade e de
hardening, não funcionalidade. É isso que a ordem abaixo reflete.

Duas correções à fila que circulava antes desta medição, ambas verificadas:

- **"AppImage: frente Distribuição intocada" era falso.** A frente tem quatro
  commits reais (`c75537d` estabiliza distribuição e self-hosting híbrido,
  `ebec631`, `948535d`, `e40b200`) e cinco scripts. O que falta é outra coisa:
  `appimage` aparece **0 vezes** no `verificar.sh`. É frente construída e **não
  verificada** — a mesma forma do `deny.toml` antes de 2026-08-30.
- **"reabrir documentos após configure toca o `EditorController`" era falso.**
  Ver `roadmaps/29` §5b: é fatia de core, e a razão está medida lá.

## 2. As etapas

### Etapa 1 — Verdade: nenhum `[x]` apontando para artefato ausente ✅ FEITA (2026-08-30)

Dois itens marcados `[x] validado e2e` citavam sondas que **nunca foram
commitadas**: `sonda_drafts.py` (`seguranca/23` §P2.6) e `sonda_terminal.py`
(`roadmaps/24`, D2.1/D2.2).

**Decisão do autor: reescrever, não rebaixar.** Ambas existem agora, rodam e
foram provadas por mutação. O que a etapa ensinou, e que vale mais que ela:
a **primeira** versão da `sonda_drafts.py` ficou **verde com o produto
quebrado** — ela olhava a resposta do `workspace.open`, e o `recover_drafts`
descarta sozinho o rascunho idêntico ao disco, escondendo a linha órfã. A
checagem certa é na store SQLite, no mesmo processo, antes de qualquer
reabertura.

*Veio primeiro porque era a mais barata e porque tudo abaixo lê esses
documentos como estado.*

### Etapa 2 — Configuration Actions ✅ FEITA (2026-09-02)

O único item de MVP que não existia, e o diferencial do produto. Escopo fechado
pela §12 da spec de MVP: **10 ações CMake** (habilitar `compile_commands.json`,
criar preset Debug, criar preset Release, adicionar executável, adicionar
biblioteca estática, adicionar fonte a um target, adicionar include dir,
adicionar `target_link_libraries`, inspecionar o cache, reparar build dir
obsoleto) e **6 Cargo** (add dependency, add dev-dependency, add feature, set
edition, cargo check, criar run config).

Entregue nas quatro camadas no protocolo `0.63.0`: `configaction.rs` no
protocolo, a pasta `configaction/` no core (catálogo, disponibilidade, plano e
um planejador por arquivo editado), `handlers/configaction.rs`, e na UI o
`ConfigActionController` + o diálogo de três painéis, acessível pela paleta
(`Ctrl+Alt+P`). Contrato em `docs/arquitetura/03-ipc-protocol.md`.

*Aceite cumprido:* `src/tests/configaction.rs` verifica **cada uma das 16
ações contra arquivo real** — abre um workspace de verdade, manda a requisição
pelo dispatch e lê o disco depois. Provado por mutação: desligar o filtro de
escopo, ignorar o snapshot do preview e mover a inserção do
`CMAKE_EXPORT_COMPILE_COMMANDS` para o fim do arquivo derrubam três testes
diferentes. Na UI, `scripts/qml-harness/tst_configaction.qml` cobre o
consentimento (o `expected` que volta ao core), e cai quando ele é omitido.

**Duas coisas que a etapa ensinou, e valem mais que ela:**

1. **Recusar é a funcionalidade, não a falta dela.** A IDE edita `Cargo.toml`
   e `CMakeLists.txt` por linha, sem parser universal (que é NÃO-MVP explícito,
   spec §11.2). Onde a forma do arquivo não é entendida — string multilinha,
   `dependencies` como tabela inline, `edition.workspace` herdada, target
   inexistente — ela **recusa com motivo** em vez de gravar por aproximação.
   Corromper o manifest do usuário não tem desfazer.
2. **O `commands.rs` virou pasta, e não por tamanho.** Uma entrada nova na
   paleta fez a catraca cobrar um arquivo de 696 linhas que fazia UMA coisa.
   O diagnóstico da §4 regra 8 é o mesmo do `AppDomains.qml`: aquele arquivo
   cresce com o NÚMERO DE DOMÍNIOS, não com a complexidade de nenhum. A saída
   foi dividir o catálogo por área (`commands/{ide,editor,build,run,git}.rs`),
   fazendo a contagem de arquivos crescer — nunca subir o baseline. A catraca
   caiu de 21 para 20 arquivos em débito **como efeito colateral**.

### Etapa 3 — Servidor LSP falso para os testes ✅ FEITA (2026-09-02)

Até 2026-09-02, **15 métodos IPC de `lsp.*` não tinham um único teste de
comportamento**: as 286 linhas de `src/tests/lsp.rs` eram todas "requires open
workspace" ou "reports unavailable", e nenhum teste deste repositório subia um
language server. Era a maior lacuna de evidência do projeto.

A forma prevista era a certa. A tabela de servidores saiu de `const` e virou
**estado injetável** (`ServerRegistry`): `Core::use_language_server_command`
aponta uma linguagem para outro executável, e `scripts/fake_lsp_server.py` entra
no lugar do rust-analyzer. O servidor falso é determinístico, instantâneo e —
o que importa — **grava tudo o que recebe**, uma mensagem por linha. Os testes
de `src/tests/lsp_server.rs` olham o WIRE, não a resposta do IPC: é a mesma
distinção que a etapa 1 ensinou quando uma sonda ficou verde com o produto
quebrado.

*Aceite cumprido*, cinco mutações, cada uma derrubando o teste certo:

```text
did_open nao envia didOpen              -> 5 testes caem
did_change nao sobe a versao            -> versao 1 onde tinha que ser 2
curto-circuito por hash desligado       -> 3 didChange onde tinham que ser 2
did_close nao envia didClose            -> 2 testes caem
fs.delete deixa de fechar o documento   -> os mesmos 2 (prova o CHAMADOR)
```

**`did_close` nasceu aqui, com chamador de verdade no mesmo commit.** Ele não
existia — `grep did_close` voltava vazio —, e a etapa 4 depende dele. Mecanismo
sem usuário é anti-padrão registrado (§8), então ele entrou pelo bug que já
existia em silêncio: apagar um arquivo aberto deixava o servidor com
diagnósticos de um arquivo que não existe mais. `fs.delete` agora fecha o
documento.

**O `lsp/manager.rs` saiu do débito (556 → 392) como efeito colateral.** A
catraca cobrou ao ver a tabela injetável entrar, e o diagnóstico foi o "e" no
próprio doc do módulo: *"mantém os servidores... **e** expõe as operações
interativas"*. Duas responsabilidades. Nasceu `lsp/session.rs` — qual
executável, subir, reiniciar, encerrar e o transporte — pelo mesmo corte que o
`sync.rs` recebeu em 2026-08-30. Catraca: 20 → 19 arquivos em débito.

*Novo requisito de ambiente:* `cargo test` passa a exigir `python3`, que já era
requisito de 4 das 13 verificações do gate. Faltar é FALHA, nunca teste pulado.

### Etapa 4 — Reabrir documentos após `cmake.configure` ✅ FEITA (2026-09-02)

O §5b do `roadmaps/29`: o clangd recarrega a `compile_commands.json` sozinho
desde a v12, mas o **documento já aberto fica com a compilação em cache**.
Reiniciar o servidor resolveria e jogaria o índice fora; a ação certa é reabrir
os documentos.

**O objeto compartilhado não foi preciso, e isso é o achado da etapa.** A
`arquitetura/04` §3 dava duas saídas — um `Arc<Atomic…>` entre o job e o manager,
ou uma requisição nova vinda da UI. Existe uma terceira, e ela já estava no
código: **o evento que o job emite volta ao dono do estado**. O loop principal
recebe `event.cmake.finished` como `LoopEvent::Notification` *antes* de escrevê-lo
no stdout, e é ele que possui o `Core`. Reagir ali fecha a fronteira sem um
segundo caminho para o mesmo fato, e sem dois donos. Nasceu
`Core::observe_notification` — o irmão do `handle_request`: aquele roteia
PEDIDO, este roteia FATO.

O core **fecha**, e não reabre. Reabrir do disco daria ao servidor o texto
gravado enquanto o editor tem um buffer sujo; fechado, o documento sai de
`versions` e o `didOpen` seguinte — disparado pela UI ao ver
`event.lsp.documentsClosed` — leva o **buffer real**. É a mesma re-sincronização
que a UI já fazia no `recovered()` e no `lspRestarted`.

*Aceite*, cinco mutações:

```text
o loop nao entrega o fato ao core       -> 2 testes caem
guarda de sucesso removida              -> configure que falhou passa a fechar
close_documents nao faz nada            -> 2 testes caem
evento para a UI suprimido              -> o arquivo ativo ficaria mudo
fecha no servidor mas nao esquece a      -> o "reabrir" nao seria real
  versao                                   (versao 2 onde tinha que ser 1)
```

**A primeira versão do teste ficou VERDE com a fiação do loop removida**, porque
chamava `observe_notification` direto. Foi preciso extrair
`runtime::drain_loop_events` e fazer o teste passar pelo loop de verdade — a
falha silenciosa da §8 (*"ligar/escutar no lugar errado não falha no build"*),
que já custou três fatias a este repositório, quase custou a quarta.

**`core_client_dispatch.cpp` encolheu de 751 para 640** pelo mesmo movimento de
2026-08-30: o domínio LSP foi para `core_client_dispatch_lsp.cpp` em vez de
fazer um arquivo em débito crescer por causa de um evento novo.

### Etapa 5 — Toolchain como entidade (B2 do TR2)

Compilador, gerador, sysroot e flags são implícitos hoje: o que estiver no
`PATH`. Não existe o "kit" que CLion e Qt Creator expõem, então trocar de
compilador ou cruzar-compilar é editar arquivo à mão. É o item caro da §5 do
`roadmaps/29`, e **etapa própria por decisão do autor**.

### Etapa 6 — Pagar o `EditorController.qml` (1070/400)

`LEITURA_TECNICA` §4: *"o maior débito bloqueia por área"*. Enquanto ele
estiver assim, qualquer feature ou polimento de editor entra travado, porque a
catraca reprova o arquivo que crescer. Ele já tem subcontrollers; é continuar
movendo por responsabilidade, com o teste de vocabulário da §4 regra 9 como
critério.

### Etapa 7 — soak / estresse ✅ FEITA (2026-09-02)

`scripts/sonda_soak.py` dirige o binário **release** por JSON-RPC em ciclos de
trabalho realista e mede as cinco coisas que teste curto não vê:

```text
memoria (RSS)   teto de crescimento + a FORMA da curva
descritores     pipe/PTY/arquivo que ninguem fecha
threads         sessao encerrada cuja thread continua viva
filhos          terminal ou job que nao morre (CRESCIMENTO, nao contagem)
latencia        p95 do fim contra a p95 do inicio
```

**A forma da curva é o que separa vazamento de cache**, e foi a lição da etapa:
o teto sozinho não basta. Um vazamento lento passa por qualquer orçamento num
soak curto — cache estabiliza, vazamento não. A sonda compara o crescimento do
último terço com o do terço do meio, e só julga a forma quando o crescimento já
saiu do ruído.

*Provada por mutação*, e as três tentativas ensinaram tanto quanto os acertos:

```text
terminal.close nao mata o shell    -> filhos 6->41, fds 16->51, threads 18->86
cache por VERSAO, sem teto         -> RSS 25 MB -> 96 MB, linear
cache por PATH, sem teto           -> NAO detectado, e esta certo: com um
                                      conjunto FINITO de arquivos, cache por
                                      caminho e limitado pelo projeto. O que
                                      vaza de verdade e o que cresce por
                                      REQUEST. A sonda diz o que prova.
```

**Duas descobertas medidas, ambas registradas no próprio script:**

1. **`syntaxTree.update` num arquivo de 400 linhas leva ~2.100 ms no debug e
   ~220 ms no release** — 10x. Soak que mede o debug mede outra coisa;
   "substituição diária" é afirmação sobre o binário que o usuário roda. A
   sonda prefere o release e avisa quando cai para o debug.
2. **Sonda contra binário velho prova o passado**, e a armadilha estava escrita
   (`arquitetura/04` §8) sem nada a checar. Durante esta etapa uma mutação que
   **não compilou** deixou o binário antigo no lugar e o soak passou verde
   "provando" um código que não existia. A sonda agora recusa rodar quando o
   binário é mais velho que as fontes — e essa guarda pegou a segunda tentativa
   de mutação minutos depois.

### Etapa 8 — Pôr o AppImage no gate ✅ FEITA (2026-09-02)

Os cinco scripts existiam e funcionavam; **nada os executava** (`appimage`
aparecia 0 vezes no `verificar.sh`). `scripts/verificar-appimage.sh` entrou como
a 14ª verificação.

**O que ele NÃO faz, e por quê:** não empacota. Gerar o AppImage exige Podman,
rede e uma compilação completa dentro do Debian 12 baseline — minutos, não
segundos. Gate que demora é gate que se desliga.

O que ele verifica são as invariantes que quebram **entre** dois
empacotamentos, e a primeira é a que mais importa:

```text
1. a UI continua 100% 2D          <- a garantia de abertura depende disso
2. as entradas do empacotamento existem (desktop, appdata, icone, licencas)
3. o hook grafico ainda forca 'software' por padrao
4. os pinos de terceiros tem URL + SHA256 (ADR-0003)
5. o testar-appimage.sh nao foi esvaziado
6. HA artefato em dist/? entao roda o smoke completo nele
```

**O cheque 1 é o coração.** O AppImage abre em qualquer máquina porque o hook
força `QT_QUICK_BACKEND=software`, e isso só funciona porque a UI não tem uma
linha de `ShaderEffect`/OpenGL. No dia em que alguém acrescentar aceleração,
essa garantia morre **em silêncio**: build passa, gate passa, e só o usuário com
driver ruim descobre. É exatamente a colisão que o
[31-simulacao-fisica-matematica.md](31-simulacao-fisica-matematica.md) §5.1
deixa em aberto — agora com um gate segurando a porta.

*Provado por mutação*, cinco vezes: um `ShaderEffect` na UI, o hook trocando
para `auto`, o smoke deixando de checar o renderer, um pino sem checksum, e uma
entrada de empacotamento removida. Todas reprovam.

**E foi validado de ponta a ponta**: um AppImage real foi gerado
(`Kinein-Vectis-0.1.0-x86_64.AppImage`, 35 MB) e o caminho oportunista do gate
rodou o smoke completo nele — estrutura, instalador, `core.ping` do binário
empacotado e o smoke offscreen, que confirma o renderer portátil e o primeiro
frame.

### Etapa 9 — Busca e substituição multi-linha

`handlers/fs.rs:76` recusa `\n` em `query` e `replacement`. O autor confirmou
que vai precisar. Fatia própria.

### Etapa 10 — `WorkspaceUiResetter`: remover `shellController` ✅ FEITA (2026-09-02)

Fiação morta no composition root: a propriedade era recebida e nunca usada.

**Medido antes de apagar**, porque a ausência podia ser um bug e não sujeira: o
que o `ShellController` guarda — painel aberto, aba de baixo, larguras, explorer
visível — é **preferência do usuário**, persistida em settings. Zerar isso a
cada troca de projeto seria perder a configuração de quem está trabalhando. Não
há estado por-workspace ali, então a fiação era mesmo morta.

O porquê ficou escrito no próprio arquivo, junto da definição do que é "por
workspace" — senão a próxima sessão re-adiciona a propriedade achando que
faltava.

## 3. Onde cortar, se a pergunta for "o mínimo para ser distribuível"

```text
FECHA O MVP        1, 2, 7, 8, 10  — TODAS FEITAS em 2026-09-02
SEPARA MVP DE      3 (feita), 4 (feita), 5, 6, 9
DAILY DRIVER
```

**O corte de distribuição fechou em 2026-09-02.** O que resta (5, 6, 9) é o que
transforma MVP em ferramenta de uso diário — o alvo seguinte, não este.

As etapas 1, 2, 7, 8 e 10 entregam o MVP com honestidade e algo que se
distribui. As demais são o que transforma MVP em ferramenta de uso diário — que
é o alvo **seguinte**, não este.

## 3.1 O que vem depois desta lista

Nada aqui, e é de propósito: esta lista termina no MVP mais o que o transforma
em ferramenta de uso diário. O horizonte seguinte — **simulação física/matemática
montada por layout, calculada pela IDE e exibida em OpenGL**, pedida pelo autor em
2026-09-01 — está registrado em
[31-simulacao-fisica-matematica.md](31-simulacao-fisica-matematica.md) como
**estudo, não fila**: ele lista as perguntas que precisam de resposta antes de
existir arquitetura, e não compete por prioridade com nenhuma etapa acima.

## 4. O que esta lista deliberadamente NÃO inclui

Nada aqui reabre decisão registrada. Em particular, seguem fora:

```text
IA na IDE        FORA DE ESCOPO desde 2026-07-17 (docs-legada/)
Python           adiado; foco em C/C++ e Rust (2026-08-30). A excecao do
                 Tree-sitter de Python NAO foi tomada
Pylance          PROIBIDO (licenca so para produtos Microsoft). Se Python
                 voltar: basedpyright (MIT)
Docker / banco   NATIVOS, nao plugins. L5/L5.5, depois de L2-L4
host de plugins  nunca houve; nao ha runtime de plugin de terceiro
```
