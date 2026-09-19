# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`DocsPublic/README.md`). Remedido em **2026-09-12** à noite,
> com o gate completo verde. O foco do produto, por decisão do autor no mesmo
> dia, são **dois contextos**: desenvolvimento de software (Python, C/C++,
> Rust, banco de dados) e **sistemas embarcados** (MCU bare metal e Linux
> embarcado) — a simulação física/matemática saiu do produto (§5). Se divergir
> do código, o código vence e este documento se corrige no mesmo gesto.
>
> **AVISO que a sessão de 2026-09-05 aprendeu na pele:** "gate verde" tem prazo
> de validade de uma atualização de sistema. Ao abrir a sessão o gate
> **reprovava**, e não era código — o Qt do sistema subiu de 6.11.1 para 6.11.2
> em 2026-09-04 22:21, e as duas árvores de build guardavam o caminho absoluto
> antigo. `cmake --preset dev-local` **e** `cmake --preset dev-local-release`,
> seguidos de rebuild, resolvem. **Cada árvore carrega o caminho por si; consertar
> uma não conserta a outra.** Detalhe na §7.7.
>
> **E esse remédio era INCOMPLETO — medido em 2026-09-11 (§7.7).** Reconfigurar
> troca o caminho da biblioteca e **não invalida objeto nenhum**: onze objetos
> compilados na tarde de 2026-09-04, contra os headers do 6.11.1, continuaram
> no link porque o rpm instala header com mtime de maio e o ninja compara mtime.
> O gate ficou verde e a IDE abortava ao abrir. O remédio inteiro para troca de
> Qt é reconfigurar **e** recompilar o que ficou velho — o 20º gate agora diz
> quais objetos são, e imprime o comando.
>
> **COMECE POR AQUI ao retomar.** (E, para o panorama do que falta em uma
> página, `DocsPrivate/Codex/HANDOFF-panorama.md`, escrito em 2026-09-17.)
> Ele substitui o
> [`38`](38-divida-restante-e-continuidade.md) nesse papel; o 38 vira registro
> de como a fila estava quando a dívida foi paga.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada seção carrega o comando.
>
> **Retomada de 2026-09-15:** a ordem vigente está na **§4.1**: backend e
> toolchains primeiro; UX/UI/HUD em etapa própria, aberta pelo autor.
> Ruff como segundo LSP já existe e foi validado nesta retomada (§7.36).
> **2026-09-16:** debugpy attach implementado e validado (§7.38).
> **2026-09-17:** a porta escolhida no Executar de MicroPython — FEITO e
> medido (§7.39, 0.110.0). À tarde, o stderr dos filhos DAP/LSP — FEITO e
> medido (§7.40, 0.111.0); os presets clang voltaram a compilar nesta máquina.
> Fim de tarde: **E5 identidade Espressif** — FEITO e provado com esptool
> falso (§7.41, 0.112.0). Noite: **E4 gravar como configuração de execução**
> — FEITO e provado pelo ciclo real proposta → save → run.start (§7.42,
> 0.113.0). **E2 permissão por canal** — FEITO e MEDIDO no ESP32 real desta
> máquina (§7.43, 0.114.0). **A5 ferramentas de embarcado no painel de
> instalação** — FEITO (§7.44); o bloco A do roadmap 41 está fechado.
> **Toolchains** — seletor de pasta nativo e SDK do Zephyr no `importKit` —
> FEITOS (§7.45). **P0** — preset no configure automático, Bear para Makefile
> puro, alvo do kit para o rust-analyzer — FEITO (§7.46, 0.115.0). Próximo:
> P4 MicroPython (firmware C5, arquivos no dispositivo C2, stubs C4).
> **2026-09-16:** compatibilidade dos verificadores Ubuntu/Qt 6.4 avançou
> (§7.37). Lógica QML verde; gate completo ainda tem impedimentos explícitos.

## 1. O estado, em números

```bash
bash scripts/verificar.sh                 # 24 verificacoes
cat scripts/arquitetura-baseline.txt      # a catraca
cargo test -q --workspace

# metodos IPC roteados. O `grep -v '^event.'` NAO e' cosmetico: sem ele a
# contagem pega dois nomes de EVENTO num `match` dentro de #[cfg(test)] em
# handlers/build.rs, e devolve 130 onde ha' 128.
grep -rhoE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"\s*(\||=>)' \
     crates/kinein-core/src/handlers/ crates/kinein-core/src/lib.rs \
  | grep -oE '"[a-z][a-zA-Z]*\.[a-zA-Z][a-zA-Z.]*"' | tr -d '"' \
  | grep -v '^event\.' | sort -u | wc -l

# eventos. Cinco nascem de format!("event.{domain}.*") em handlers/build.rs,
# com domain em {build, quality}: um grep de literal NAO OS VE e devolve 36
# onde ha' 41. Os cinco sao build.started/output/diagnostic e
# quality.started/output.
{ grep -rhoE '"event\.[a-zA-Z.]+"' crates/kinein-core/src/ | tr -d '"'
  for d in build quality; do
    for s in started output diagnostic finished; do echo "event.$d.$s"; done
  done
} | sort -u | wc -l
```

```text
protocolo   0.128.0
testes      840 Rust aprovados; 54 harnesses QML (medicao de 2026-09-19, §7.77)
metodos     161 IPC roteados, 55 eventos (run.stdin e event.run.* sairam em 0.125.0;
            datasource.discover/create e event.datasource.created
            em 2026-09-18 a noite; remote.open/sync/status e event.remote.synced,
            datasource.query e event.datasource.queried
            em 2026-09-18; serial.identify, runConfig.flashProposal,
            serial.access, serial.files, python.stubs, debug.scopes,
            debug.readMemory, debug.disassemble, coverage.run, coverage.lines,
            remote.list/save/remove/probe/deploy/command,
            event.lsp.log, event.coverage.finished, event.remote.probed,
            event.remote.deployed,
            event.serial.identified, event.serial.files e event.python.stubs
            entraram em 2026-09-17)
dominios    36, e os 36 documentados no arquitetura/03 (coverage.* e remote.* entraram em 2026-09-17)
catraca     1 arquivo em debito
gate        24 verificacoes (a 24a, 2026-09-18: fiacao IPC de ponta a ponta)
```

> **O par `metodos`/`eventos` foi CORRIGIDO DE NOVO em 2026-09-06, e desta vez
> o erro não estava no número — estava no COMANDO.** Ele dizia `130` e `36`;
> medidos com o comando certo, são **128 e 41**. As duas falhas têm a mesma
> causa: grep de literal não sabe o que o literal é. Uma contava dois nomes de
> evento como se fossem método porque eles aparecem num `match` de teste; a
> outra não via cinco eventos porque eles são montados com `format!`.
>
> **Isto é a segunda vez que este par mente** — em 2026-09-04 ele dizia `139` e
> `35`. Da primeira vez a culpa foi de um número escrito sem comando que o
> reproduzisse. Desta vez havia comando, ele rodava, e ainda assim mentia.
> **Comando que reproduz não é o mesmo que comando que mede certo**, e a defesa
> agora é o comentário dentro de cada um dizendo o que ele exclui e por quê.

> **O par `metodos` foi CORRIGIDO em 2026-09-04.** Ele dizia `139 IPC roteados,
> 35 eventos` e nenhum dos dois batia com o disco: medidos, sao 121 e 36. O
> numero entrou numa sessao anterior sem comando que o reproduzisse, e por isso
> envelheceu sozinho — exatamente o vicio que o `verificar-docs.sh` existe para
> pegar, e que ele nao pegou porque so' confere numeros que sabe medir. Os
> comandos acima ficam junto para o proximo que ler nao ter de confiar.

## 2. A dívida da catraca: 1 arquivo, e ele tem decisão do autor

```text
ui/qml/editor/EditorController.qml   791/400
```

**Não corte sem falar com o autor.** A decisão de mantê-lo congelado está em
[`../arquitetura/32`](../arquitetura/32-editor-por-responsabilidade.md) §8.4, e
as duas saídas estão medidas em [`39`](39-divida-tecnica-paga.md) §6: dissolver
a fachada (~190 pontos de chamada em 15 arquivos) ou corrigir a categoria.
Mexer em limite é decisão explícita dele, nunca do assistente.

## 3. O que esta sessão entregou (2026-09-03/04)

**Dívida paga: 8 arquivos → 1** — cada corte com a pergunta que o justifica em
[`39`](39-divida-tecnica-paga.md).

**Dois gates nasceram, os dois de relato de uso:**

```text
verificar-atalhos.sh       o atalho que a paleta ANUNCIA e' o que a IDE OBEDECE,
                           e todo item de menu tem tratamento no host
verificar-exercitacao.sh   o core contra as ferramentas REAIS desta maquina
```

**Frente de banco (etapa 26), completa até a introspecção:**

```text
26.1  perfil sem campo de senha, cofre decidido ANTES (../seguranca/40)
26.2  driver `postgres` 0.19.14, datasource.test como job
26.3  UI: painel, dialogo de senha por `secretRequired`, nunca por texto
26.4  introspeccao: esquemas, tabelas e colunas do information_schema
```

**Ambiente C/C++ e Rust:**

```text
6 acoes novas de REGIME de compilacao   rigor, sanitizers, OpenMP, .hex/.bin,
                                        alvo embarcado do Cargo
biblioteca com ciclo fechado            ativar E desativar, com a bolinha
                                        dizendo o que esta no PROJETO
campo que EXPLICA e SUGERE              valores reais lidos do projeto
toolchain automatico e VISIVEL          escolhe e mostra que escolheu
setup.list                              passo a passo OFICIAL por distro,
                                        com fonte e data
lista UNIFICADA "Ambiente do projeto"   biblioteca e acao lado a lado, com
                                        filtro por LINGUAGEM (nao por
                                        ferramenta) — o Cargo deixou de estar
                                        atras do C/C++
```

**A previa do `CMakeLists.txt` deixou de sumir** (roadmaps/35 §9.3.14). O autor
pediu polimento de tamanho; a medicao achou defeito: numa tela 1366x768, com uma
acao de seis campos, a caixa que mostra o arquivo media **10 pixels** — e o
botao "Ativar" continuava habilitado. A previa ganhou dono
(`ConfigActionDiffView`), o cabecalho ganhou teto de 40% com rolagem, o dialogo
cresceu de 1020x660 para 1100x820 e a caixa ganhou barra de rolagem.

```text
1920x1080, 3 campos    214px (~12 linhas)  ->  374px (~22 linhas)
1366x768,  6 campos     10px (NENHUMA)     ->  285px (~17 linhas)
```

**A observabilidade entrou: o Grafana pela API** (roadmaps/35 §9.6). Domínio
`grafana.*` com quatro métodos e um evento, cliente HTTP próprio (`ureq`: +5
crates, todas `MIT OR Apache-2.0`; o TLS foi ligado horas depois, quando o autor
decidiu a política de licença — §9.7.7), e o cruzamento que justifica o domínio
existir:

```text
dev  ->  kinein-dev      banco `kinein` em localhost:5432
```

Exercitado contra um **Grafana 13.0.2 de verdade**, incluindo os quatro modos de
falha (token inválido, sem token, porta errada, `https` — que na época era
recusado por falta de TLS e hoje conecta). Os dashboards
abrem no navegador — a licença AGPL decide a forma, e a IDE nunca embute.

**E a etapa 27 FECHOU, com o MongoDB** (roadmaps/35 §9.7), com as OITO
decisões do autor respondidas antes da primeira linha de código. O que ele traz
que nenhum motor anterior trazia:

```text
duas verdades    DECLARADO (validador `$jsonSchema`) ou INFERIDO de amostra,
                 e a tela NUNCA deixa os dois parecidos
o custo na tela  o `$sample` varre a colecao inteira quando N nao e' menor que
                 5% dela — a IDE conta ANTES e diz qual caminho vai acontecer
tetos de RAM     2.000 campos, 8 niveis, 10 elementos de array; quando um morde,
                 a colecao aparece com o aviso em vez de parecer completa
segunda forma    `DataSourceCollections` ao lado da arvore relacional, com
                 profundidade, tipo PLURAL e presenca em %
```

Exercitado contra um **MongoDB 8.2.12** real, e a exercitação achou o de sempre:
com o servidor em contêiner rootless, **`localhost` não conecta** (resolve para
IPv6 e o driver não cai para IPv4). A mensagem agora diz `tente 127.0.0.1`.

**Dois defeitos apareceram mexendo no código, não em gate:**

```text
Ctrl+O nao abria nada       `shellController` era usado pelo GlobalShortcuts e
                            NUNCA ligado pelo Main.qml; a chamada caia num
                            `null` em silencio. O menu funcionava, a tecla nao
requestFailed sem `code`    o cliente Qt descartava o codigo do erro, forcando
                            a UI a casar por TEXTO — exatamente o que o
                            comentario do `SecretRequired` existe para evitar
```

**E o harness passou a enxergar tela.** O `verificar-qml-logica.sh` monta um
espelho plano do modulo `KineinVectis` a partir das fontes; ate' 2026-09-04 so'
dava para testar componente que nao usa `Theme`, e geometria era zona sem
cobertura. O `tst_configaction_layout.qml` e o primeiro morador — provado por
mutacao: tirar o teto derruba a caixa a 70px, devolver o dialogo ao tamanho
antigo derruba a 249px, e cada mutacao acende um bit diferente.

## 4. O que está aberto

```text
27  bancos relacional/temporal/nao-     FECHADA em 2026-09-04. Postgres,
    relacional, e o Grafana               TimescaleDB por nome, SQLite, MongoDB
                                          (com a segunda forma de exibicao) e o
                                          Grafana pela HTTP API. Falta so' o que
                                          ficou registrado como fatia PROPRIA:
                                          executar consulta, escrever, e o TLS
                                          do `postgres` — ver roadmaps/35 §9.7.10
--  TLS do `postgres`                     a LICENCA ja' esta' decidida (o
                                          `deny.toml` aceita as duas de 2026-09-04
                                          e o `ureq` e o `mongodb` ja' cifram).
                                          Falta o conector, que muda a chamada de
                                          conexao — fatia propria
--  core_client.h em 500/500            FECHADO em 2026-09-10 (§7.5), por
                                         DECISAO DO AUTOR e pela saida (b): a
                                         categoria estava errada, nao o arquivo.
                                         Um header que so' DECLARA nao tem
                                         logica para esconder, e o limite dele
                                         passou a ser 2x o que ele declara — nao
                                         um numero escolhido. A saida (a) foi
                                         MEDIDA e descartada: 422 linhas de
                                         declaracao contra 63 de comentario, e
                                         mover comentario compraria ~33 linhas
                                         contra a convencao da linguagem
--  A IDE DO CHECKOUT NAO ABRE            FECHADO em 2026-09-11 (§7.7). Achado em
                                          2026-09-10 como prioridade 1: os dois
                                          builds abortavam (SIGABRT, display real
                                          e offscreen) com os 19 gates verdes,
                                          e o AppImage abria. A HIPOTESE do dia
                                          — "QML compilado em AOT contra o Qt
                                          6.11.2" — estava ERRADA na causa e
                                          certa no sintoma: o chamador era o
                                          AOT, mas o culpado eram 11 OBJETOS
                                          compilados antes da troca de Qt que o
                                          ninja nunca recompilou (rpm instala
                                          header com mtime de maio). Violacao de
                                          ODR no `copyAppend` inline. Nasceu o
                                          20o gate: `verificar-binario-abre.sh`
--  FECHADOS nesta passada, e ficam       a coluna `exato` que respondia por
    aqui so' como registro                OUTRA equacao (§19.0) — consertada em
                                          2026-09-10 pela PROCEDENCIA, §7.3; e o
                                          `03-ipc` sem cobertura de 5 dominios,
                                          escrito em 2026-09-06. Conferido em
                                          2026-09-10: dos 130 metodos roteados,
                                          ZERO estao ausentes do documento
25  handshake DAP com probe-rs           PARCIAL: precisa de sonda fisica ou
                                         alvo QEMU. O resto do ciclo de
                                         embarcado esta' provado no CORE; na
                                         TELA, nada dele chega (medido em
                                         2026-09-11: menu de toolchain sem
                                         `debugAdapter`, kit sem `chip`,
                                         `probe.list` sem consumidor). As DOZE
                                         decisoes do autor para fechar a frente
                                         estao no roadmaps/35 §5.7, e a ordem
                                         e' fio -> QEMU -> tela -> polimento
--  ESP32 NA MESA: conectividade         LEVANTADO em 2026-09-11 (§7.12,
    bare metal                           integracoes/38). O autor plugou um
                                         ESP32 na tarde do dia em que a §5.7 do
                                         35 dizia que nao podia. MEDIDO: e' um
                                         ESP32-D0WD-V3 CLASSICO (Xtensa) atras
                                         de CP2102 em /dev/ttyUSB0, dialout
                                         OK, auto-reset OK, flash 4 MB — sem
                                         USB-JTAG embutido. Gravar e monitorar
                                         sao exercitaveis HOJE; depurar exige
                                         ESP-Prog + openocd-esp32 + esp-gdb, e
                                         NAO esta' na mesa. O core nao sabe o
                                         que e' porta serial. Seis fatias
                                         propostas (E1 serial.list -> E3
                                         monitor -> E5 identidade Espressif ->
                                         E2 permissao por canal -> E4 gravar
                                         como JOB com motor por familia; E6
                                         debug bloqueado por hardware).
                                         DECIDIDO pelo autor na mesma noite
                                         (38 §6), com a regra "padrao de
                                         mercado, solucao pronta, nada
                                         escrito do zero": monitor = PROCESSO
                                         (`espflash monitor` / `tio`) no
                                         painel de terminal, MPL-2.0 NAO
                                         entra; gravar = CONFIGURACAO DE
                                         EXECUCAO, nao dominio novo; ordem
                                         E1 -> E3 -> E5 -> E4 -> E2; um C3/C6
                                         vai para a mesa (USB-JTAG embutido,
                                         probe-rs sem fork). E1 `serial.list`
                                         FEITA na mesma noite (§7.13, 0.91.0),
                                         exercitada contra o ESP32 real. E3
                                         (monitor como processo) FEITA em
                                         2026-09-12 (§7.15). O autor pediu uma
                                         ORGANIZACAO antes de seguir: a trilha
                                         PROFUNDA de embarcados e' o roadmaps/42
                                         — oito pilares, pronto POR FAMILIA. O
                                         PILAR 0 (o MODELO do projeto) teve
                                         quatro fatias em 2026-09-12: dominio
                                         `project` (9 frameworks com evidencia,
                                         SDKs, artefatos, alvo — §7.16), a
                                         receita e as particoes do ESP-IDF
                                         LIDAS, o build.size consumindo a
                                         particao app, e o INDICE DO PROJETO
                                         INTEIRO (§7.17) — este por exigencia
                                         nova do autor: "a IDE deve ler o
                                         projeto inteiro, todas as funcoes/
                                         arquivos/pastas, C/C++/Rust/Python".
                                         A quinta fatia (§7.18, 2026-09-12):
                                         o CONTEXTO DE COMPILADOR por arquivo
                                         — `index.context` (unidade da CDB,
                                         alvo do cargo, interpretador Python)
                                         e a CDB envelhecida por SUBPASTA
                                         detectada; e a sexta (§7.19): a
                                         GRAMATICA PYTHON na fundacao (realce,
                                         outline, indice); a setima (§7.20):
                                         o indice SEGUE O DISCO INTEIRO (as
                                         pastas caminhadas entram no watcher);
                                         a oitava (§7.21): o MODELO POR ALVO do
                                         CMake pelo file-api (0.96.0) — a "CDB
                                         em memoria" do 42 §8. O PROXIMO do P0
                                         (42 §3): o map file; o modelo por
                                         PRESET (kits x presets). Depois, o P1
                                         (setup). As tres decisoes do 42
                                         §7 foram
                                         TOMADAS em 2026-09-12: so' o ESP32
                                         classico na mesa (o resto fecha no
                                         gate e fica dito como nao exercitado);
                                         Raspberry Pi OS como alvo Linux; P0
                                         primeiro
--  O EFEITO JETBRAINS como criterio    MAPEADO em 2026-09-12 (42 §8), a pedido
    de pronto: zero-config, indexacao,   do autor. O que JA' EXISTE, medido: kit
    Alt+Enter proativo, project model    automatico + clangd com query-driver;
    antes do LSP, sysroot visual,        indice e contexto na barra; Alt+Return
    remote deploy & debug, SVD com       -> lsp.codeActions; a ORDEM model ->
    escrita, sondas visuais              indice -> LSP ja' e' a certa; kit com
                                         sysroot/remoteTarget/debugServer;
                                         attach por `target remote`; probe.list.
                                         O que FALTA, por pilar: o PRESET no
                                         configure automatico (que JA' EXISTE na
                                         UI — medido em 2026-09-12 depois de
                                         escrito o contrario; 42 §8 item 1
                                         corrigido) + CDB do file-api
                                         compileGroups + Bear para Makefile (P0);
                                         ".venv com uv" e "instalar" de um
                                         clique com o comando visivel (P1);
                                         lampada na margem + clang-tidy no
                                         clangd + ruff LSP (P5/editor);
                                         seletor de sysroot que LE a pasta,
                                         toolchain file gerado, importar kit
                                         Yocto (environment-setup) e Buildroot
                                         (P1/P6); "Remoto (SSH)" de um clique
                                         com gdbserver (P6); painel SVD com
                                         ESCRITA — svd-parser 0.14.10 MIT/Apache
                                         + DAP readMemory/writeMemory, que o gdb
                                         17.2 daqui implementa (P3); sonda
                                         escolhida na lista e OpenOCD deduzido
                                         do VID:PID (P2/P3). Nada muda na ordem
                                         do 42 §4
--  A CADEIA PYTHON (41 bloco B)         FECHADA em 2026-09-13: as cinco fatias
    FEITA (§7.24-§7.28); o que ficou     (§7.24 ambiente; §7.25 basedpyright+ruff;
    e' polimento, listado abaixo         §7.26 run+pytest; §7.27 debugpy; §7.28
                                         MicroPython+modulo nativo). O que a cadeia
                                         deixou de fora esta' em itens proprios:
                                         ruff como SERVIDOR, run.capabilities, a
                                         arvore do pytest, `-m pacote`/attach no
                                         debugpy, a porta escolhida no Executar de
                                         MicroPython, o resumo Python na barra.
                                         PROXIMO da fila: o provedor de download de
                                         toolchain (39 §5), que a cadeia adiou
--  o resumo Python nao chega a barra    FEITO em 2026-09-13 (§7.31): os resumos do
    de status                            projeto (indice, contexto, python) sairam
                                         para StatusBarProjectSummaries e a barra
                                         mostra "python: .venv · 3.14.7 · pybind11
                                         (scikit-build-core)"
--  Docker/Podman "nao esta' dando       MEDIDO ATE' ONDE ALCANCA em 2026-09-13
    certo" (relato)                      (§7.34, 0.108.0). O core respondeu certo
                                         o tempo todo; o painel REAL renderizado
                                         offscreen com a resposta REAL do Podman
                                         mostrou o defeito: "compose up" primario
                                         e ACESO sem projeto (botao primario
                                         desligado vestia o ambar), e aceso com
                                         projeto sem arquivo de compose (o job so'
                                         podia falhar). Corrigido nos dois lados:
                                         composeFile no status + recusa antes do
                                         job; KvButton/KvIconButton desligados
                                         apagam. Se o autor viu OUTRA coisa
                                         (icone, lista vazia, mensagem), e' um
                                         sintoma novo: dizer o que apareceu
--  a porta escolhida no Executar de     FEITO em 2026-09-17 (§7.39, 0.110.0): o chip
    MicroPython                          "Executar" por porta no painel de Embarcados
                                         (EmbeddedController.selectedPort, toggle, cai
                                         quando a porta some da lista ou o workspace
                                         troca) -> RuntimeController.serialDevice por
                                         binding -> `device` em run.script E em
                                         run.start (que nao o aceitava: o botao
                                         Executar sempre ia sem porta). Sem placa
                                         nesta maquina: provado com mpremote falso
--  debugpy: attach                      o `-m pacote` FEITO em 2026-09-13 (§7.33:
                                         DebugTarget::Module -> `module` no launch,
                                         provado pelo core real). Attach TCP FEITO
                                         em 2026-09-16 (§7.38): connect {host, port},
                                         campos na aba Debug, breakpoint/inspecao e
                                         desconexao preservando o processo externo
--  stderr dos processos filhos vai      FEITO em 2026-09-17 (§7.40, 0.111.0). Era a
    para /dev/null (adaptador DAP,       LACUNA vista na fatia 4 (2026-09-13): o gate
    servidor de debug, servidores LSP)   falhou UMA vez com "o adapter nao respondeu a
                                         `initialize`" sem causa. Agora `stderr_tail.rs`
                                         (uma thread por filho, cauda de 64 linhas) nos
                                         tres: adaptador -> event.debug.output
                                         `adapter` + cauda no erro do handshake;
                                         servidor de debug -> cauda no "saiu antes de
                                         abrir"; LSP -> event.lsp.log + cauda no
                                         status failed/exited; LSP que falha o
                                         initialize e' morto, nao fica orfao
--  debugpy no gate desta maquina        o ciclo real so' roda onde `python3`
                                         importa debugpy ou KINEIN_PYTHON_DEBUGPY
                                         aponta um venv — aqui foi provado com um
                                         venv temporario. Para o gate provar sempre:
                                         `sudo dnf install python3-debugpy` ou um
                                         venv fixo e a variavel no ambiente
--  `run.capabilities` (o que "Executar"  FEITO em 2026-09-13 (§7.33, 0.107.0): o
    aceita, publicado pelo core)         core publica runnable/debuggable, a UI
                                         consome e nao tem lista propria
--  Python free-threaded (3.14t+, PEP    PONTUACAO do autor (2026-09-13), sem
    703/779; 3.15 final em 2026-10-01,   prioridade: se for util, a IDE le se o
    PEP 790)                             interpretador do projeto e' free-threaded
                                         (`sysconfig.get_config_var("Py_GIL_
                                         DISABLED")` / `sys._is_gil_enabled()`) e
                                         mostra no python.status; o run/pytest/
                                         debugpy nao mudam — e' o MESMO binario,
                                         so' o nome (`python3.14t`) e o GIL. Nada
                                         a fazer ate' um projeto precisar; medir
                                         antes (nesta maquina ha' 3.14.7 padrao)
--  descoberta do pytest como arvore     FEITA em 2026-09-13 (§7.32, 0.106.0):
    (41 B6, o que faltou)                test.discover para pytest, cargo e ctest; a
                                         arvore no painel com "rodar so' este"
--  ruff como SERVIDOR LSP               FEITO, validado em 2026-09-15 (§7.36):
    (o Alt+Enter em Python)              `ruff server` ao lado do basedpyright,
                                         documentos sincronizados nos dois,
                                         diagnosticos fundidos e code actions na
                                         lista existente. Preview valida a versao
                                         no servidor que produziu a acao; aplica
                                         pela transacao existente. Seis testes de
                                         integracao e prova com servidores reais.
                                         Nao reimplementar; seguir para debugpy attach
--  TOOLCHAINS POR ALVO                  CONFERIDAS na fonte em 2026-09-12
    (integracoes/39): o catalogo,        (integracoes/39): o levantamento
    a busca alem do PATH, o sysroot,     recebido tinha 2 afirmacoes desatualizadas
    o provedor de instalacao             (Espressif por chip; LLVM Embedded) e 1
                                         errada (pasta do xpm), e faltavam Zephyr
                                         SDK, servidores de debug e o SYSROOT.
                                         FEITO (§7.23): 13 candidatos novos de
                                         compilador, 4 GDBs, 6 meta-ferramentas;
                                         busca em xPacks/espressif/IDE//opt;
                                         rustTargets e sysrootHint no
                                         toolchain.get (0.97.0). O PROVEDOR DE
                                         INSTALACAO FEITO em 2026-09-13 (§7.29,
                                         0.103.0): nove releases pinados com o
                                         SHA-256 lido na fonte, download em job,
                                         checksum antes de desempacotar, tar como
                                         processo, botao no painel. O GERENCIADOR
                                         QUE LE O DISCO FEITO em 2026-09-13 (§7.30,
                                         0.104.0): inspectSysroot com veredito;
                                         importKit de Yocto (environment-setup pelo
                                         sh), Buildroot (output/host) e pasta de
                                         toolchain; toolchainFile no kit. FALTA:
                                         um SDK Yocto/arvore Buildroot REAIS para
                                         medir (nao ha' nesta maquina); Zephyr SDK
                                         (setup.sh + ZEPHYR_SDK_INSTALL_DIR — e'
                                         do P6/Zephyr); um seletor de pasta nativo
                                         (hoje o caminho e' digitado); e um ciclo
                                         REAL de download no gate (baixar 155 MB
                                         no gate nao e' decisao deste repositorio)
--  A TRILHA PYTHON COMPLETA:            MAPEADA em 2026-09-12 (42 §9): MicroPython
    bare metal -> edge -> backend ->     no ESP32 (mpremote 1.29.0 aqui) ->
    banco                                Mosquitto como container (EPL/EDL) ->
                                         Pi com CPython/SQLite/Podman por SSH ->
                                         FastAPI + uv -> Postgres/Timescale/
                                         SQLite/Mongo (datasource ja' conecta e
                                         le) -> Grafana pela API. O que a IDE
                                         precisa de NOVO: MicroPython gravado e
                                         REPL (P2/P4), .venv de um clique (P1),
                                         a Pi como alvo (P6), a primeira RECEITA
                                         de container (Mosquitto), o projeto
                                         Python com CMake/cargo dentro
                                         (pybind11/nanobind/PyO3/maturin)
                                         reconhecido pelo project.model (P0),
                                         executar/escrever no banco (item 27)
--  O ECOSSISTEMA INTEIRO, em ordem      MAPEADO em 2026-09-11 (roadmaps/41), a
    linear: embarcados + Python +        pedido do autor: "nada deve ficar de
    MicroPython                          fora", com o VS Code (Python, C/C++,
                                         Rust, Cortex-Debug, probe-rs,
                                         PlatformIO, ESP-IDF, Pico) como
                                         referencia de FUNCIONALIDADE e a
                                         ferramenta aberta por tras como
                                         implementacao, licenca lida no
                                         arquivo. Seis blocos: A fecha o canal
                                         serial e o ciclo Espressif (E3->E5->
                                         E4->E2, + setup + saida do teste); B e'
                                         Python inteiro (Tree-sitter ->
                                         interpretador -> basedpyright -> ruff
                                         -> debugpy -> pytest); C MicroPython
                                         (mpremote, stubs, firmware); D
                                         profundidade (RTT, SVD, memoria,
                                         RTOS, clang-tidy, cobertura, Renode);
                                         E frameworks (ESP-IDF, pico-sdk,
                                         Zephyr, PlatformIO); F o grande
                                         (Jupyter, SSH, Docker). O PROXIMO
                                         continua sendo A1 = E3, o monitor
--  guias de instalacao para arch/suse   a fonte oficial dos tres projetos NAO
                                         cobre essas familias; entrar exige
                                         fonte de comunidade, marcada como tal
--  SSH REMOTO nativo                    DECIDIDO pelo autor em 2026-09-11,
                                         NAO arquitetado. Abrir e trabalhar num
                                         workspace REMOTO por SSH — editar,
                                         compilar, rodar e depurar na maquina do
                                         outro lado —, nativo como Docker e
                                         banco sao nativos (§5), nunca plugin.
                                         Antes de qualquer codigo, MEDIR: onde o
                                         core assume caminho LOCAL (fs, fswatch,
                                         run, terminal, build, dap, toolchain
                                         resolvem no disco desta maquina), o que
                                         vira canal remoto e o que continua
                                         local, e a licenca do transporte (o
                                         `ssh` do sistema como PROCESSO, ou uma
                                         crate — a decidir com fonte). E' fatia
                                         grande e propria; entra na fila sem
                                         data de execucao
--  o que a VARREDURA do §8 ainda acusa  REMEDIDO em 2026-09-13: event.quality.
    (calculado e nao mostrado)           started sem ouvinte QML; event.quality.
                                         output descartado no C++; os 3 sinais de
                                         environmentScan sem ouvinte (o progresso
                                         da varredura de ferramentas nao aparece);
                                         cmake.presets.list e job.list sem cliente;
                                         dataSourceTestAccepted/grafanaProbeAccepted
                                         sem ouvinte. Cada um e' fatia pequena; o
                                         quality.output e' o que mais esconde (a
                                         saida do clippy/ruff quando NAO ha'
                                         diagnostico parseavel)
--  SINCRONIZACAO DE DOCUMENTACAO         PEDIDO do autor em 2026-09-13 (fim de
    antes da etapa de UX/UI/HUD          tarde): "atualize/sincronize a documentacao
                                         do projeto para refletir o estado atual de
                                         tudo feito e para nao haver perda de
                                         contexto" — e ela vem ANTES da etapa de
                                         UX/UI/HUD (§5). Feita na mesma tarde
                                         (§7.35): manual sem simulacao e com
                                         Python/Embarcados/Containers, README com
                                         Python, 41/42/29/37 com estado, §8
                                         remedido, prompt de retomada novo. Vale
                                         como REGRA para as proximas: a passada de
                                         sincronizacao e' um item da fila, nao um
                                         gesto implicito
```

### 4.1 Ordem vigente — duas etapas (decisão do autor, 2026-09-15)

Esta seção consolida a sequência pedida pelo autor. As seções anteriores e
os roadmaps 41/42 detalham cada frente; referências antigas a "próximo" não
substituem esta ordem. Antes de cada item, conferir o código e reaproveitar os
serviços, contratos e testes existentes.

**Continuidade em 2026-09-16:** a instalação do notebook preparou a base de
desenvolvimento; as correções Qt/QML e de instrumentação (§7.37) são manutenção.
Elas não concluem os itens de toolchains nem alteram a sequência abaixo.
O attach do debugpy foi validado na §7.38, a porta do MicroPython na §7.39 e
o stderr dos filhos DAP/LSP na §7.40, o E5 na §7.41, o E4 na §7.42 e o E2
na §7.43 e o A5 na §7.44 — o bloco A do roadmap 41 fechou; Toolchains
(seletor de pasta nativo; SDK do Zephyr no `importKit`) na §7.45 e o P0
(preset no configure automático; Bear; alvo do kit para o rust-analyzer) na
§7.46, o P4 MicroPython (arquivos na placa, firmware oficial, stubs por
placa — provado no ESP32 real) na §7.47, e o bloco E (ESP-IDF, Zephyr,
pico-sdk e PlatformIO como motores de build/gravar/monitorar) na §7.48, e a
primeira fatia do P3 (o launch certo do probe-rs, SVD no kit, RTT, escopos,
memória e disassembly pelo DAP padrão) na §7.49, e o P5 (clang-tidy no
clangd e no `quality.run`, gtest/Catch2 na árvore, cobertura em LCOV na
calha, a lâmpada proativa) na §7.50, e a primeira fatia do P6 (o alvo Linux
por SSH como recurso do projeto: perfil sem senha, sonda, deploy, e
rodar/gdbserver/debugpy como configuração de execução e kit) na §7.51. O
próximo item é a segunda fatia do P6 (workspace remoto, LSP do outro lado)
ou o banco (consultas/escrita/TLS), à escolha do autor; a exercitação da
fatia 1 pede uma Pi real, que não há nesta máquina. Em 2026-09-18 o banco
fechou a sua fatia (§7.52: `datasource.query` com leitura imposta pelo
motor, escrita confirmada, TLS verify-full no PostgreSQL), e o P6 ganhou a
fatia 2 (§7.53: o workspace ESPELHADO por rsync — a pasta do alvo aberta
como workspace comum, salvar empurra, Puxar/Empurrar).

**Decisão do autor em 2026-09-18, ao fim da fila linear da Etapa 1:** a
fila fechou o que tinha de fechar (os restos de cada item estão na tabela
abaixo, e cada um diz o que precisa — hardware, servidor, ou uma fatia
pequena). O que vem agora é, nesta ordem: **(1) o PENTE-FINO da
arquitetura** — a varredura §8 remedida e ampliada (o "andar de cima" que
a §8.4 disse que faltava: evento que o core emite e nenhuma tela consome
vira gate), busca de qualquer defeito/falha no projeto (fiação, superfície
morta, testes intermitentes, `unwrap` fora de teste, avisos, o
`release-hardened`), com registro datado e correção do que for defeito; e
**(2) a Etapa 2 — HUD/UI/UX** (§4 abaixo), com o visual e o efeito
psicológico das IDEs JetBrains como referência de FLUXO, adaptados ao
contexto deste projeto — não uma cópia de tema.
Pendências independentes do gate ficam registradas; antecipar correções quando bloquearem comprovadamente
o item em curso ou repararem regressões da própria mudança. Continuar executando
as verificações exigidas e distinguindo falhas preexistentes; o gate completo
ainda não está verde.

**Etapa 1 — backend e toolchains impecáveis (agora).** Seguir linearmente:

| Frente | Trabalho restante e estado |
| --- | --- |
| Polimento Python | **CONCLUÍDO em 2026-09-17:** Ruff (§7.36), debugpy attach (§7.38), a porta escolhida no Executar de MicroPython (§7.39, 0.110.0) e o stderr dos filhos DAP/LSP (§7.40, 0.111.0). O que resta de Python é o bloco P4 (MicroPython) e o P6 (remoto). |
| Embarcados, bloco A | **E5 (§7.41, 0.112.0), E4 (§7.42, 0.113.0) e E2 permissão por canal (§7.43, 0.114.0: `serial.access` medido no ESP32 real — `uaccess` sem grupo, ModemManager candidato, regras de sonda da distro) FEITOS em 2026-09-17, e o A5 (§7.44: catálogo de embarcados no painel de instalação, fonte oficial ou índice da distro, com data) fechou o bloco A.** Resta a exercitação com a placa (E5 `flash-id` real, E4 gravação real, E2 o passo do ModemManager). |
| Toolchains | **Seletor de pasta nativo e SDK do Zephyr no `importKit` FEITOS em 2026-09-17 (§7.45).** Resta medir `importKit` com Yocto/Buildroot/Zephyr SDK reais — ainda sem exemplares locais. |
| P0 — modelo do projeto | **FEITO em 2026-09-17 (§7.46, 0.115.0):** preset no configure automático (kit > CMakeUserPresets > CMakePresets, dito e anotado); `kind: make` com `bear -- make`; `rust-analyzer.cargo.target` do kit. |
| P4 — MicroPython | **FEITO em 2026-09-17 (§7.47, 0.116.0):** arquivos na placa (`serial.files`, C2 — lido e reescrito no ESP32 do autor), firmware oficial no catálogo de instalação e gravado pela proposta do E4 (C5), stubs por placa no basedpyright (C4). Resta o C6 (CircuitPython: drive `CIRCUITPY` + `circup`), baixa prioridade. |
| P3 — depuração profunda | **Fatia 1 FEITA em 2026-09-17 (§7.49, 0.118.0):** o `launch` do probe-rs consertado (`coreConfigs`), `svdFile` no kit → escopo `Peripherals`, RTT/defmt como saída de debug (`rtt`), `debug.scopes`/`readMemory`/`disassemble` como passagem do DAP padrão, a vista de inspeção na aba Debug. Provado com adaptador falso — nenhuma sonda nesta máquina. **Resta:** D5 threads de RTOS (`rtos` do OpenOCD via `debugServer`), SVD com ESCRITA (`setVariable`/`writeMemory`, que os dois adaptadores anunciam), `rttChannelFormats` (defmt declarado), o caminho `cmsis-svd` pelo GDB. |
| P5 — qualidade | **FEITO em 2026-09-17 (§7.50, 0.119.0):** `--clang-tidy` no clangd e o clang-tidy do projeto pela CDB no `quality.run` (D6); gtest/Catch2 dentro dos binários do ctest na árvore, rodar um pelo filtro (D7); domínio `coverage.*` — cargo-llvm-cov e coverage.py em LCOV, a calha pinta (D8); a lâmpada 💡 na linha do cursor com diagnóstico. **Resta:** cppcheck como segundo motor; gcov/lcov para C/C++ (exige `--coverage` no build do usuário); doctest. |
| P6 — Linux embarcado | **Fatia 1 FEITA em 2026-09-17 (§7.51, 0.120.0):** domínio `remote.*` — perfil SSH sem senha, `remote.probe`, `remote.deploy`, `remote.command`; painel **Alvo remoto (SSH)**. **Fatia 2 FEITA em 2026-09-18 (§7.53, 0.122.0):** o workspace ESPELHADO — `remote.open` puxa a pasta do alvo por rsync para o cache e a IDE a abre como workspace comum (`workspace.open` responde `remote`), salvar empurra o arquivo, `remote.sync` pull/push sem `--delete`, `remote.status`. **Resta (42 §P6):** watcher do lado remoto, renomear/apagar propagados, LSP/interpretador do alvo, journalctl/dmesg, Yocto/Buildroot reconhecidos, `sshd` local no gate, exercitação numa Pi real. |
| Frameworks, bloco E | **FEITO em 2026-09-17 (§7.48, 0.117.0):** `build.run` compila pelo wrapper de cada um (`pio run`; `idf.py build` no ambiente ativado — export.sh ou EIM; `west build -d build -b <placa>`; CMake com `-DPICO_SDK_PATH`), Gravar ganhou `idf.py`/`west`/`platformio`, o monitor ganhou o IDF Monitor e o `pio device monitor`, `platformio.ini` é tipo de projeto. Provado com wrappers falsos — nenhum SDK real nesta máquina. Resta: E5 templates curados, E6 Unity/Ceedling. |
| Banco | **FEITO em 2026-09-18 (§7.52, 0.121.0):** `datasource.query` — leitura com teto imposto por fora e `READ ONLY` no motor, escrita só com `confirmWrite` (código `WRITE_CONFIRMATION_REQUIRED`), células em texto, Mongo `<coleção> <filtro>` só leitura; TLS `verify-full` no PostgreSQL (`tokio-postgres-rustls`, +11 crates, deny verde). **Resta:** escrever documento no Mongo; abas/histórico de consulta; exportar; cancelar consulta longa; PostgreSQL/Mongo reais e o TLS de ponta a ponta não provados no gate (só SQLite). |
| Varredura 40 §8 | `quality.output` descartado no C++; `environmentScan` sem ouvinte; presets sem tela. Os demais achados permanecem detalhados no §8. |
| Frentes grandes, bloco F | Jupyter; dev containers com contexto remoto; polimento Rust com nextest e llvm-cov. |

**Etapa 2 — UX/UI/HUD (o autor ABRIU em 2026-09-18, para depois do
pente-fino).** Banco com experiência à DataGrip adaptada ao Kinein Vectis;
Python como cidadão da tela; apresentação da IDE com frase e forma próprias
no lugar da enumeração. A referência declarada pelo autor é o **sucesso das
IDEs JetBrains** — o que nelas produz o efeito psicológico positivo (a
janela que parece "saber" do projeto: a barra de status que diz o que está
acontecendo, o painel de problemas que dá o próximo passo, a hierarquia
visual que não briga com o código, a densidade certa, a resposta imediata a
cada gesto) — **adaptado ao contexto deste projeto**: embarcados, Rust/C++/
Python, banco e remoto como cidadãos de primeira, e o desenho documental que
a IDE já tem (`arquitetura/32`, `iconografia/`). O que se estuda é o
comportamento observável; nenhum tema, ícone ou código da JetBrains entra
(licença e identidade). A etapa começa com um DESENHO (medido na IDE
abrindo: o que cada tela mostra hoje, contra o que a referência mostra), e
só depois código. **O desenho está no [`43`](43-etapa2-hud-ui-ux.md)
(2026-09-18):** a referência lida nas fontes, três fotos da IDE de hoje, e
as fatias F1–F8 com a medida de cada uma; a F0 (infra: `kinein-vectis
<pasta>`, `KINEIN_SCREENSHOT`, a status bar sem colisão, os gates sem
poluir os recentes) foi feita no mesmo dia (§7.55). **F1 a F8 foram feitas
no mesmo dia** (§7.56–7.65, uma fatia por commit); resta o fechamento da
etapa (§4.2.2) — o panorama consolidado está na §4.2.

### 4.2 Panorama consolidado — 2026-09-18 (noite), depois da F7: o que falta, e o que cada resto precisa

Escrito a pedido do autor antes da F8 ("documente tudo… qual a visão geral
do que ainda precisa ser feito?"). Cada linha diz **o que é**, **onde está
registrado**, e **o que precisa** para ser feito — porque a resposta
honesta à pergunta "o que falta" tem três classes distintas: o que é uma
fatia de código que qualquer sessão faz; o que é código mas só se PROVA
com hardware ou servidor que não há nesta máquina; e o que é só
exercitação do autor com a coisa real na mão. Misturá-las é o que faz uma
lista de pendências parecer maior ou menor do que é.

#### 4.2.1 O que está pronto (medido em 2026-09-18, noite)

```text
protocolo    0.128.0 · 161 metodos IPC · 55 eventos · 36 dominios (todos no arquitetura/03)
testes       840 Rust · 54 harnesses QML · 24 verificacoes no gate, todas verdes
binario      linux-clang-debug-strict abre em ~720-840 ms offscreen (debug);
             release-hardened abriu em 318 ms na medicao do pente-fino (§7.54)
catraca      1 arquivo em debito (core_client.h, decisao do autor §7.5); nenhum novo
commits hoje 4182769 F0 · 84b1e80 F1 · cf0de24 F2 · 5517538 F3 · 9d97bd1 F4 ·
             271d9c7 F6-a · 225564c F5 · 409f375 F6-b · 36f90fd F7
             (antes deles, no mesmo dia: aa7b891 banco · 2a3b168 P6 fatia 2 ·
             6edc340 pente-fino)
```

Etapa 1 (backend e toolchains), frente a frente: Polimento Python
(§7.36–7.40), bloco A de embarcados (§7.41–7.44), Toolchains (§7.45), P0
modelo do projeto (§7.46), P4 MicroPython (§7.47), bloco E frameworks
(§7.48), P3 fatia 1 (§7.49), P5 qualidade (§7.50), P6 fatias 1 e 2 (§7.51,
§7.53), Banco (§7.52), pente-fino com a varredura §8 fechada (§7.54) —
**todos feitos**. Etapa 2: F0–F7 (§7.55–7.63) **feitas**.

#### 4.2.2 Etapa 2 — o que resta para fechar

```text
F8  paineis de ambiente com a MESMA forma                 FEITA (§7.65, 2026-09-18)
    KvPanelFrame/KvPanelHeader/KvVerdict/KvDataGrid; os quatro paineis com a
    mesma primeira linha; o Embarcados cabe. Restos ditos: containers e portas
    como grade com acoes; Grafana/Setup/Biblioteca na moldura comum.

Fechamento da etapa                                       meia sessao
    43 §5 e §7 sincronizados; foto final das tres telas de 43 §2 (inicial,
    workspace, editor) lado a lado com as de manha; release-hardened REMEDIDO
    (primeiro frame e os tempos da tabela §7.62 — hoje so' o debug foi medido
    depois da F6); leitura-tecnica e README do DocsPublic com o estado; e a
    proposta da Etapa 3 ao autor (abaixo, §4.2.5).

Dividas de UX ditas em "nao feito" das fatias (pequenas, cabem no fechamento):
    - Ln:Col do cursor na status bar (F2/F3 disseram; ainda nao ha)
    - contagem na aba Testes ("12/14") como a de Problems (F5 disse)
    - a foto do gate a 1024 px alem de 1280 (F2 disse; a status bar tem
      regra de ceder, mas so' foi vista a 1280)
    - o trilho lateral em modo compacto/expandido (F1 fez o rotulo ao pairar,
      nao o modo expandido)
    - foco da StartScreen contra o TerminalPanel (`focus: true`) quando o
      painel de baixo esta' aberto sem workspace — caso raro (F7)
    - rename / codeActions / workspaceEdit ainda SINCRONOS no core (F6-a);
      raros, mas sao a ultima classe de pedido que pode segurar o laco

O que so' o autor mede (precisa de uma pessoa na frente da IDE):
    - clique -> primeiro feedback por acao, com mouse e teclado reais (a
      tabela §7.62 e' do core e do primeiro frame; offscreen nao clica)
    - Enter / setas na tela inicial; Ctrl+P; a sensacao de densidade das
      linhas do explorer (22 px) e das abas — o "efeito psicologico" que a
      etapa persegue so' se confirma com uso
```

#### 4.2.3 Etapa 1 — os restos, classificados pelo que precisam

**(a) Fatia de código que qualquer sessão faz, sem hardware** (em ordem
de valor para o uso diário, juízo do agente):

```text
Banco        abas/historico de consulta; exportar CSV/JSON; cancelar consulta
             longa (o job ja' e' cancelavel — falta o cancel chegar ao motor);
             escrever documento no Mongo (hoje so' leitura)           §7.52
P6 remoto    renomear/apagar propagados ao alvo (hoje so' fs.write empurra);
             watcher do lado remoto (inotifywait pelo ssh) para "mudou la'";
             journalctl/dmesg do alvo como aba; Yocto/Buildroot reconhecidos
             pelo P0 (kind do workspace)                            §7.53, 42 §P6
P5 qualidade cppcheck como segundo motor do quality.run; doctest na arvore;
             gcov/lcov para C/C++ (so' se o build do usuario tiver --coverage);
             a lampada por consulta previa de acoes                  §7.50
P3 debug     rttChannelFormats (defmt declarado por canal); SVD com ESCRITA
             (setVariable/writeMemory — provavel com adaptador falso); o
             caminho cmsis-svd pelo GDB                              §7.49
Bloco E      menuconfig/set-target do IDF no terminal; pio test/check; os
             runners de debug do west; E5 templates curados; E6 Unity/Ceedling
             (provaveis com wrappers falsos, como o resto do bloco)  §7.48
P4           C6 CircuitPython (drive CIRCUITPY + circup) — baixa prioridade §7.47
Gate         a classe "Connections no target errado" so' e' vista em runtime
             ate' o primeiro frame (§7.63); as telas que carregam depois
             (paineis de ambiente, dialogos) ficam fora — um harness que
             instancie cada painel e leia o stderr fecharia isso
```

**(b) Código pronto, prova pendente de hardware ou servidor** — nada a
escrever antes de ter a coisa; o que há de fazer é a exercitação com
registro datado:

```text
Pi / Linux embarcado   P6 fatias 1 e 2 inteiras (perfil, sonda, deploy,
                       espelho, Puxar/Empurrar) — provadas com ssh/rsync
                       FALSOS; falta uma Pi (ou qualquer Linux com sshd)
PostgreSQL / Mongo     datasource.query e o TLS verify-full — provados so'
                       com SQLite; precisa de um servidor (um container
                       local basta: o dominio container.* ja' sobe um)
Sonda JTAG             P3 fatia 2 (threads de RTOS, RTT real) — o ESP32
                       classico do autor NAO tem JTAG; precisa de ESP-Prog,
                       ou de um C3/C6 (USB-JTAG embutido), ou de uma Pico
                       com debugprobe
SDKs reais             ESP-IDF (export.sh), Zephyr (west), pico-sdk,
                       PlatformIO — os wrappers falsos provaram a forma; o
                       SDK real prova o conteudo (o importKit com Yocto/
                       Buildroot/Zephyr SDK idem)
Placa de teste         gravar firmware (E4/C5) — NUNCA na placa do autor
                       (apagaria o main.py dele); qualquer ESP32 vazio serve
```

**(c) Só o autor, com a placa na mão** (roteiro em
`DocsPrivate/Codex/2026-09-17-e2-permissao-por-canal.md`): o passo do
ModemManager (`ID_MM_DEVICE_IGNORE=1`, pede sudo — a IDE só mostra); o
botão "Identificar" da aba Serial — que até hoje NÃO chegava ao core (§7.63
consertou o fio); o painel de Embarcados inteiro clicado na IDE aberta.

#### 4.2.4 O que os gates NÃO cobrem, dito de uma vez

O gate prova core + ponte + controllers + a IDE abrindo sem aviso. Não
prova: o clique real (nenhum harness clica na janela); as telas que carregam
depois do primeiro frame (o stderr só é lido até ele); PostgreSQL/Mongo/TLS,
Pi, sonda, SDKs (item b); o release-hardened a cada commit (só o debug
abre no gate — o hardened é medido por sessão, §7.54). Quem lê "24
verificações verdes" deve ler junto esta lista.

#### 4.2.5 A ordem proposta a partir daqui (para o autor decidir)

1. **F8** e o **fechamento da Etapa 2** (§4.2.2) — uma sessão.
2. **Uma sessão do autor na IDE aberta**, com o roteiro de §4.2.2/§4.2.3(c):
   o que a Etapa 2 prometeu só se confirma com uso; os achados viram
   fatias pequenas.
3. **Etapa 3, candidatas** (não decidido): (i) os restos (a) de §4.2.3 em
   ordem de valor — banco e remoto primeiro, porque são os que o autor
   disse que usará; (ii) o **bloco F** (Jupyter; dev containers com contexto
   remoto; polimento Rust com nextest/llvm-cov); (iii) a **validação com
   hardware** (b), à medida que a coisa chegar à mesa; (iv) **release**:
   AppImage regenerado e testado (`testar-appimage.sh`) — só quando o autor
   pedir, como a regra diz.

**Depois da F8 (noite de 2026-09-18), a fila do teste do autor:** banco
descobre/cria (§7.66), âncoras do Git (§7.67), execução em aba de terminal
+ faixa de abas (§7.68–7.69), HUD do Git fatia 1 (§7.70). O contexto
linear dessa fila — commits, mapa de arquivos, o desenho da fatia 2 da HUD
do Git e a ordem depois dela — está no [`43`](43-etapa2-hud-ui-ux.md) §9.

#### 4.2.6 Como o autor roda a IDE e o que testar (pedido em 2026-09-18, ao fim da F8)

**Rodar** (da raiz do repositório — é de lá que a UI acha o core em
`target/debug/kinein-core`; `KINEIN_CORE_BIN=<caminho>` força outro):

```bash
cd /home/hugh/KineinVectis
cargo build -p kinein-core                                        # o core (debug)
cmake --build build/linux-clang-debug-strict --target kinein-vectis   # a UI que o gate usa
./build/linux-clang-debug-strict/ui/kinein-vectis /home/hugh/KineinVectis
```

Para sentir o tempo real (o debug abre em ~730 ms; o hardened em ~318 ms):

```bash
cargo build --release -p kinein-core
cmake --build --preset linux-clang-release-hardened
KINEIN_CORE_BIN=$PWD/target/release/kinein-core \
  ./build/linux-clang-release-hardened/ui/kinein-vectis /home/hugh/KineinVectis
```

Sem a pasta por argumento, a IDE abre na tela inicial (F7). `KINEIN_STARTUP_COMMANDS=
datasource.list` (ou `remote.list`, `probe.list`, `container.list`, `view.problems`,
`build.run`) abre um painel logo depois do workspace, como o gate faz.

**O roteiro do teste — o que só uma pessoa na frente da IDE mede** (cada
item vira uma fatia pequena se falhar; anote o que viu e em quanto tempo):

```text
tela inicial (F7)    Enter abre o recente em destaque; ↑↓ movem; "1 recente sem
                     caminho ocultado · Desfazer" traz de volta; "Ver" abre a
                     aba Ferramentas
barra (F1)           Projeto · Git · Executar: UMA configuracao, ▶ roda, o menu
                     "…" tem Build/Testes/Analise dos sistemas presentes
status bar (F2)      durante um build: o job com progresso e Cancelar no centro;
                     o LSP a direita muda de "subindo" para o nome
editor (F3/F6)       abrir mod.rs de kinein-core: nada trava enquanto o
                     rust-analyzer sobe; a aba ativa e a linha atual se veem;
                     digitar 2 s e parar: o ● some (autosave)
explorer (F4)        .git/target/build em cinza e no fim; o arquivo aberto
                     aparece selecionado ao trocar de aba
Problems (F5/F6-b)   um build que falha: cada linha com o proximo passo; o
                     mesmo erro NAO aparece duas vezes (build + LSP)
paineis (F8)         Ambiente > Banco / Alvo remoto / Embarcados / Containers:
                     a mesma primeira linha, a acao primaria a direita, o
                     veredito antes do formulario; Embarcados ROLA e nao vaza
resposta ao gesto    para cada clique acima: houve feedback em < 400 ms? Onde
                     nao houve, qual gesto e quanto demorou
placa (E2/E5)        com o ESP32 no USB: Embarcados > Portas seriais mostra
                     /dev/ttyUSB0 e o veredito de permissao; "Identificar" le
                     o flash-id (so' leitura — nunca gravar nesta placa)
```

## 5. As decisões registradas que NÃO se reabrem

```text
IA na IDE                    fora de escopo (2026-07-17)
symbolica (CAS Rust)         PROIBIDO — fonte visivel, uso proprietario, licenca
                             a adquirir. Categoria Pylance (2026-09-05)
Python                       ENTRA como vertical nativa — DECISAO DO AUTOR em
                             2026-09-11 (roadmaps/41), REVERTENDO o "adiado" de
                             2026-08-30. Sem anuncio parcial: a tela so' diz
                             "Python" quando a cadeia inteira funcionar (41 §5, B8)
Pylance                      PROIBIDO (licenca) — continua; o motor e' basedpyright
Docker e banco               NATIVOS, nao plugins. Docker/Podman IMPLEMENTADO
                             em 2026-09-12 (§7.14): dominio `container`
UX/UI/HUD: ETAPA PROPRIA,    DECISAO DO AUTOR em 2026-09-13: "a IDE focou muito
DEPOIS do backend            a UI para C/C++ e Rust; agora com o Python vai
                             precisar de uma formulacao melhor da UX/UI/HUD,
                             mas vamos deixar isso para uma etapa propria;
                             vamos fazer tudo referente a backend e integracao
                             de toolchains de forma impecavel antes de
                             arquitetar a reformulacao". Dois pedidos ja'
                             registrados para essa etapa: (1) o banco de dados
                             com o EFEITO PSICOLOGICO e a UI/UX/HUD do
                             JetBrains (DataGrip) ADAPTADOS ao Kinein Vectis —
                             nao copiados; (2) o Python como cidadao da tela,
                             nao um acrescimo; (3) a APRESENTACAO da IDE
                             (autor, 2026-09-13, fim de tarde): listar
                             "Python, C/C++, Rust e sistemas embarcados" na
                             StartScreen/About/.desktop/appdata "fica muita
                             coisa" — precisa de algo melhor para exibir o que
                             a IDE e' (uma frase, ou a forma visual, nao a
                             enumeracao). Ate' la': polimento de tela so'
                             quando e' DEFEITO (a promessa errada do botao,
                             §7.34), nunca reformulacao
simulacao: FORA DO PRODUTO   DECISAO DO AUTOR em 2026-09-12, em dois tempos.
                             Tarde: "esquece a parte de simulacao fisica/
                             matematica; vamos refinar ao maximo para sistemas
                             embarcados e desenvolvimento de software". Noite:
                             "vamos remover tambem tudo sobre a parte de
                             simulacao" — quem quiser simular escreve o proprio
                             codigo e a IDE o roda e mostra. O dominio `sim`,
                             o `kinein-sim`, os paineis, 7 harnesses, o oraculo
                             SymPy e o `exmex` SAIRAM do codigo; os documentos
                             (31, arquitetura/34, ADR-0006) e as decisoes de
                             simulacao que estavam aqui foram para
                             DocsPrivate/historico/simulacao/, integros. O foco
                             sao DOIS contextos: software (Python, C/C++, Rust,
                             banco) e sistemas embarcados. "Futuramente vejo
                             algo sobre simulacao" — reabrir e' do autor
documentacao: DUAS ARVORES   DECISAO DO AUTOR em 2026-09-12: `DocsPublic/` (toda a
                             documentacao do projeto, versionada; pastas com
                             nome explicito; documento = numero + nome
                             explicito) e `DocsPrivate/` (no .gitignore: log,
                             diario, prompts das sessoes com IA, historico do
                             que saiu, legado/). Sucede as tres arvores de
                             2026-08-29 (ADR-0005 anotado). Os numeros dos
                             documentos ficam: sao o sistema de citacao
instalar toolchain           REFINAMENTO de "comando de instalacao" (autor,
                             2026-09-12, integracoes/39 §5): instalar NO SISTEMA
                             continua sendo comando visivel com fonte, nunca
                             sudo; instalar NA PASTA DA IDE
                             (~/.local/share/kinein-vectis/toolchains) pode ser
                             download — DEPOIS de um clique, com URL, tamanho e
                             sha256 mostrados antes e verificados depois, de
                             fonte com checksum publicado e versao pinada.
                             Nunca calado, nunca "latest", nunca fora da pasta
foco do produto              DOIS contextos, por decisao do autor em 2026-09-12:
                             desenvolvimento de software (Python, C/C++, Rust,
                             banco de dados) e sistemas embarcados (MCU bare
                             metal e Linux embarcado), com TODO o ecossistema
                             aberto e credivel de toolchains dessas linguagens
                             a escolha do usuario. "Focar em fazer algo bem
                             feito do que tudo"
o efeito JetBrains           CRITERIO DE PRONTO, nao pilar novo (autor, 2026-09-12;
                             42 §8): zero-config = DETECTAR + UM CLIQUE com o
                             comando visivel, nunca download calado (a regra de
                             instalacao acima continua); indexacao visivel;
                             intention actions proativas; project model ANTES
                             do LSP; toolchain manager com sysroot; remote
                             deploy & debug; SVD com ESCRITA; sondas visuais
a trilha Python completa     bare metal -> edge -> backend -> banco, com C/C++/
                             Rust onde o Python nao cabe (autor, 2026-09-12; 42
                             §9). Ferramentas do PROJETO do usuario, que a IDE
                             reconhece, sobe, observa e depura — nao dependencias
                             da IDE. Python desta maquina: 3.14.7; 3.15.0 final
                             em 2026-10-01 (PEP 790); "3.16" e' outubro de 2027
A IDE LE O PROJETO INTEIRO   DECISAO DO AUTOR em 2026-09-12: todas as pastas,
                             arquivos, funcoes e tipos de C/C++/Rust/Python,
                             com integracao PROFUNDA do contexto de codigo e
                             compilador. Primeira forma: o dominio `index`
                             (§7.17). E' o "entender o projeto inteiro" da
                             especificacao do KSWE — reaberto pelo autor nesta
                             forma, sem adotar a especificacao inteira
EditorConfig                 auditado com resultado NEGATIVO (2026-07-16)
Grafana embutido             PROIBIDO (AGPL) — integracao por HTTP API
TLS na IDE                   ENTRA (autor, 2026-09-04): `subtle` (BSD-3-Clause) e
                             `webpki-roots` (CDLA-Permissive-2.0) estao na
                             allowlist, com justificativa datada no deny.toml
esquema de documento         DECLARADO quando ha' validador, INFERIDO de amostra
                             quando nao — e a tela nunca deixa os dois parecidos
custo de leitura             sempre VISIVEL: quantos documentos foram lidos e se
                             a leitura obrigou o servidor a varrer tudo
EditorController             congelado ate' decisao do autor (../arquitetura/32 §8.4)
senha em disco               PROIBIDA: a IDE guarda o PERFIL (../seguranca/40)
comando de instalacao        so' com FONTE OFICIAL citada e datada; sem fonte,
                             a IDE mostra o site e diz que nao tem passo a passo
dimensionamento da UI        AUTOMATICO pelo conteudo, com PISO e com o teto da
                             janela (autor, 2026-09-04). Altura fixa corta
                             conteudo; automatica sem piso faz a tela piscar
UI/UX moderna e minimalista  ETAPA A PARTE, depois do pente-fino (autor,
                             2026-09-04). Nao se antecipa em fatia de
                             funcionalidade, e nao reabre decisao registrada
embarcados: escopo           ARM Cortex-M via probe-rs nesta etapa; RISC-V e
                             ESP32 depois; AVR/Arduino NAO entra (2026-09-11)
embarcados: ponte GDB        ENTRA como `gdb -i dap`, 2o candidato do papel
                             debugAdapter — o GDB fala DAP desde a v14, e isso
                             derruba o "protocolo novo inteiro" do 36 §3
                             (2026-09-11)
embarcados: prova            QEMU no gate com fixture propria; sonda real so'
                             quando plugada, e ate' la' NAO PROVADO e dito
                             (2026-09-11)
embarcados: deducao          `probe-rs info` SUGERE, o usuario CONFIRMA; a IDE
                             nunca escolhe o chip calada (2026-09-11)
embarcados: templates        catalogo CURADO com fonte e licenca, nunca
                             linker script adivinhado (2026-09-11)
```

## 6. A lacuna que não é técnica, e continua sendo a mais cara

**O registro de saídas do dogfooding continua VAZIO**
(`DocsPrivate/diario/19-registro-de-saidas.md`).

E esta sessão deu a prova mais forte que existe de que ele importa: **cinco
defeitos reais** — a busca por arquivo quebrada pelo `fd` 10.4.2, o atalho da
biblioteca que formatava o arquivo, os dezoito alvos de link que não aceitam
link, o `CMAKE_CXX_STANDARD` escrito onde não faz efeito, e a prévia do
`CMakeLists.txt` com 10 pixels numa tela de notebook — **foram achados por
frases do autor usando a IDE**, não pelos dezoito gates.

O quinto ensina uma coisa a mais: **a frase apontou o lugar, a medição achou o
tamanho.** O autor disse *"a caixa ficou com um dimensionamento pequeno"* e
pediu polimento. Instanciar o diálogo real fora do app e medir mostrou que, na
tela dele com uma ação de seis campos, a caixa não era pequena — era
**inexistente**, com o botão de consentimento habilitado do lado. Relato de uso
diz **onde** olhar; medir diz **quanto**. Aceitar a frase como veredito teria
produzido um ajuste cosmético e deixado o defeito de pé.

Uma frase de uso vale mais que uma refatoração da lista.

## 7. O registro das entregas, por fatia

> Este capítulo é o registro datado de cada fatia entregue desde 2026-09-06.
> As seções §7.1–§7.4 e a introdução (a etapa 28, simulação: SISTEMA_EDO, a
> tela ligada, o oráculo SymPy, as unidades checadas) **saíram daqui em
> 2026-09-12** com a decisão do autor de tirar a simulação do produto; o texto
> está íntegro em `DocsPrivate/historico/simulacao/`. A numeração das seções
> seguintes foi mantida, porque o resto da documentação as cita.

### 7.5 O `core_client.h` saiu de 500/500 — a CATEGORIA estava errada, 2026-09-10

**Decisão do autor**, e ela vem depois da medição descartar a outra saída.

O arquivo bateu em **exatamente 500/500** e travaria a próxima assinatura IPC que
a UI consumisse — três dos itens abertos acrescentam método. As duas saídas
estavam escritas desde 2026-09-06: **(a)** mover comentário para junto da
implementação, ou **(b)** corrigir a categoria.

**A medição matou a (a).** O header tem **422 linhas de declaração pura contra 63
de comentário**; mover os `///` das declarações compraria ~33 linhas e custaria a
convenção da própria linguagem — corte por TAMANHO, que a `ARCHITECTURE` §4
regra 9 recusa em qualquer arquivo.

**O que ele é, medido:** 289 itens declarados (129 `Q_INVOKABLE`, 18
`Q_PROPERTY`, 142 `void`) em 423 linhas — **1,46 linha por item**. É o argumento
do composition root aplicado a C++: o tamanho é função do **contrato** que ele
espelha. E a saída padrão do composition root ("dividir a composição por área")
não existe aqui: um `QObject` é **uma** classe, e classe não se divide em dois
arquivos.

**O limite não virou um número escolhido**, e essa é a parte que importa. Fixar
700 porque 500 não coube seria levantar limite para caber. Ele é **2× o que o
arquivo declara**:

```text
so' cresce DECLARANDO        hoje 578 contra 500 — a folga de 78 linhas e' o
                             espaco entre a densidade medida (1,46) e o teto (2)
comentario sem declaracao    come a folga e REPROVA
logica que entra no header   derruba o arquivo para 500 NA HORA, porque ele
                             deixa de ser desta categoria
```

Provado por mutação nas três direções: um corpo de função reprova em 502/500;
noventa linhas de comentário reprovam em 590/578; trinta declarações novas
passam.

**A catraca continua com UM arquivo em débito** — o `EditorController.qml` em
791/400, que segue congelado por decisão registrada (`../arquitetura/32` §8.4).
Nenhum limite foi levantado nesta passada: corrigiu-se a categoria de um arquivo
que nunca teve lógica, que é a segunda vez que isso acontece (a primeira foi o
`ui/qml/app/`, que nunca foi QML visual).

### 7.6 O registro de saídas: o que foi feito, e o que continua faltando

**O registro continua VAZIO, e isso está certo** — não porque ninguém olhou,
mas porque a regra do arquivo é estreita de propósito:

```text
E' SAIDA        abandonei a Kinein e abri outra ferramenta para terminar
NAO E' SAIDA    bug que contornei DENTRO da Kinein — isso vai para a fila
                normal. "Misturar os dois esvazia o valor deste arquivo"
```

**Um assistente não pode preenchê-lo.** Uma saída é o autor largando a IDE no
meio de uma tarefa; escrever entrada sem isso ter acontecido é inventar o dado
que o arquivo existe para coletar. O que a sessão de 2026-09-10 pôde fazer foi o
oposto disso: **medir**, e mandar o achado para o lugar certo.

**E o achado é grande.** Ao tentar rodar a IDE para conferir uma tela, os
dois builds do checkout abortaram — com display real e offscreen —
enquanto o AppImage abre. Prioridade 1, anterior a esta sessão, e **com o gate
de dezenove verificações verde**. Está registrado na §4, com os dois contornos
que o isolam.

**A leitura honesta, e ela não mudou desde 2026-09-03:** o registro vazio não
significa que a IDE substituiu o VS Code. Significa que a semana de
desenvolvimento C/C++ e Rust dentro dela — o critério do TR1 — ainda não
aconteceu. E agora há um motivo medido para ela não ter acontecido: **quem
compila do checkout não consegue abrir a IDE nesta máquina.** *(Removido em
2026-09-11, §7.7: os três binários do checkout abrem, e o gate passou a
executá-los.)*

### 7.7 A IDE do checkout voltou a abrir — e a hipótese registrada estava errada, 2026-09-11

**A regra zero mandou medir a §4 antes de aceitá-la, e a primeira medição
confirmou o sintoma e derrubou a causa.** `exit=134` nos dois builds, offscreen
e com display; `exit=0` no AppImage. Até aí, igual ao registro. O que o gdb
acrescentou foi **um número que não batia**: a pilha apontava o assert em
`qarraydataops.h:286`, e no header em disco (6.11.2) a linha 286 é `private:` —
o `copyAppend` com seus quatro `Q_ASSERT` está na 301–304. O binário tinha sido
compilado contra um header que **não é o que está no disco**.

**A causa, medida em quatro passos:**

```text
1. onze objetos de 2026-09-04 14:23    main.cpp.o, clipboard, documentation, os
   nas TRES arvores de build           cinco editor_highlighter*, debug_flags,
                                       window_chrome_controller e um qrc — todos
                                       anteriores ao Qt subir para 6.11.2 as
                                       22:21 do mesmo dia
2. o ninja os da' como VALID           `ninja -t deps` compara MTIME; o rpm
                                       instala o header com o mtime de quando o
                                       PACOTE foi construido: 2026-05-11. Header
                                       "mais velho" que o objeto = nada a fazer
3. dois objetos definem o mesmo        `QGenericArrayOps<QVariant>::copyAppend`
   inline, em duas versoes             e' template inline (COMDAT). So' dois .o
                                       o instanciam: `editor_highlighter_
                                       folding.cpp.o` (VELHO, 6.11.1, assert na
                                       286) e o `GlobalShortcuts` compilado em
                                       AOT (NOVO, 6.11.2, assert na 301)
4. o linker dobra na versao velha      no 6.11.1 a classe HERDA de
                                       QArrayDataPointer e `this` e' o dado; no
                                       6.11.2 ela guarda `m_ptr`. O chamador novo
                                       passa um ponteiro para o ponteiro, o
                                       callee velho le como se fosse o dado, e
                                       `!this->isShared() || b == e` estoura na
                                       primeira `QVariantList{"Alt+F12","Ctrl+`"}`
                                       que o QML compilado monta
```

O `this->` no texto do assert é a prova final: a sintaxe do 6.11.2 é
`that()->`. **E os dois contornos registrados na §4 explicam-se pela mesma
causa:** `QV4_FORCE_INTERPRETER` e `QML_DISABLE_DISK_CACHE` desligam o AOT, que
era o **único chamador novo** daquele inline — a versão velha nunca recebia o
layout que não entende.

**Por que o stderr estava vazio, e isso custou uma sessão:** o Qt do Fedora é
compilado com journald. Sem tty, o assert vai para o journal, e quem roda por
`timeout` ou por pipe vê um SIGABRT mudo. `journalctl --user _COMM=kinein-vectis`
tinha a linha o tempo todo. `QT_FORCE_STDERR_LOGGING=1` a traz de volta — o
`atualizar-tudo.sh` já usava, o smoke novo também.

**O conserto, medido:** remover os onze objetos e relinkar — 9,5 s no
`dev-local`. Os três binários do checkout abrem; a cópia do anterior, guardada
antes do conserto, continua abortando:

```text
build/dev-local                       primeiro frame em 239 ms   exit 0
build/linux-clang-debug-strict        primeiro frame em 548 ms   exit 0
build/dev-local-release               primeiro frame em 235 ms   exit 0
o binario de 2026-09-10 (copia)       SIGABRT                    exit 134
```

**A lição que corrige o registro:** o remédio registrado em 2026-09-05 —
reconfigurar as duas árvores depois da troca de Qt — **resolvia o gate e não a
IDE**. Reconfigurar troca o caminho
da biblioteca no `build.ninja`; não toca em objeto. Só `--clean-first` (que o
`atualizar-tudo.sh` faz e o `verificar.sh` não) ou remover os objetos velhos
resolve. O ninja não tem modo por ctime; a defesa é gate.

**O gate que nasceu — o 20º, e a pergunta que ele faz é a que faltava:**

```text
verificar-binario-abre.sh   depois de CADA `cmake --build` do verificar.sh:
                            1. nenhum objeto da arvore e' mais velho (mtime) que
                               a chegada ao disco (ctime) de uma dependencia
                               FORA da arvore de build que o ninja registrou
                               para ele — o que esta' dentro e' do ninja, e a
                               primeira rodada do gate provou isso com um falso
                               positivo num gerado. Se for, diz qual header,
                               quando chegou, e imprime o comando que remove
                               exatamente aqueles objetos
                            2. o binario chega ao primeiro frame offscreen e
                               sai com 0 — o MESMO mecanismo do smoke do
                               AppImage (KINEIN_PERF_MARKER + KINEIN_PERF_EXIT),
                               que existia desde o M4.2 e nunca foi apontado
                               para o binario do checkout
                            3. (2026-09-18, §7.63) o stderr ate' o primeiro
                               frame nao tem aviso do motor QML de fiacao
                               quebrada ("no signal of the target matches",
                               "Binding loop", ReferenceError/TypeError) —
                               o qmllint nao ve, porque depende do tipo REAL
                               do target de uma Connections
```

O ctime é a hora em que o inode entrou **neste** disco; nenhum gerenciador de
pacote o falsifica, e é o que o ninja não olha. A primeira pergunta existe
porque a segunda, sozinha, dá um SIGABRT — e foi um SIGABRT sem explicação que
produziu a hipótese errada da véspera.

**O critério afinou duas vezes depois de nascer.** A primeira rodada excluiu
dependências DENTRO da árvore de build (o ninja as gerencia). A segunda
(2026-09-11, ao editar o próprio `CMakeLists.txt`) restringiu ao TRAP exato: a
dependência parece velha para o ninja (`mtime <= objeto`, então ele não
recompila) **e** chegou depois (`ctime > objeto`). Um arquivo do repo que o
autor acabou de editar tem `mtime > objeto` — o ninja o recompila sozinho, e
não é este gate que cuida disso. Sem essa segunda trava, editar um `.md` no
`CMakeLists` reprovava um `mocs_compilation` que o próximo build já conserta.

**Provado por mutação nas duas metades:** `touch -d` num objeto para antes da
instalação do Qt reprova em (1) e nomeia `QtQuick/qtquickglobal.h` chegando às
22:21:19; o comando impresso remove o objeto, o rebuild recompila, e o gate
volta a passar. O binário de 2026-09-10 no lugar do relinkado reprova em (2)
mostrando o assert — que agora aparece, porque o smoke força o stderr.

```text
gate    20 verificacoes (+1)
codigo  scripts/verificar-binario-abre.sh, scripts/verificar_binario_abre.py,
        dois passos no verificar.sh — nenhuma linha de core, protocolo ou QML
```

**O que NÃO mudou:** protocolo `0.88.0`, 666 testes, 31 harnesses, catraca com
um arquivo. O defeito era da árvore de build, não do fonte — e por isso nenhum
`git log` o explicava.

### 7.8 Frente F, fatia 1 — o fio: a tela passa a alcançar o que o core já tinha, 2026-09-11

**As doze decisões estão no [`35`](35-ambiente-cpp-e-embarcados.md)
§5.7; a ordem escolhida foi fio → QEMU → tela → polimento.** Esta é a primeira,
e ela não escreve motor nenhum: liga à tela três coisas que existiam no core
desde 2026-09-03 e que nenhuma tela pedia.

```text
papel `debugAdapter`   o menu de toolchain listava 5 papeis e omitia o sexto:
                       ninguem escolhia o probe-rs pela tela. Uma linha
`chip` do kit          o `toolchainSetKit` do C++ nao levava `chip`, e a
                       resposta nao o devolvia: so' a CLI gravava um. Agora
                       viaja nos dois sentidos (ponte, controller, roteadores)
`probe.list`           ganhou consumidor: `EmbeddedController` + painel
                       "Embarcados" em Ambiente do projeto (Ctrl+Alt+M, menu
                       e paleta), com a sonda reconhecida, a dica do core, a
                       saida CRUA quando nada foi reconhecido, e os tres campos
                       do kit (chip, alvo, sysroot) EDITAVEIS num gesto so' —
                       `toolchain.setKit` e' uma escrita, nao tres
```

**Exercitado contra o core real por stdio:** `probe.list` devolve
`probes`/`toolAvailable`/`rawOutput`/`hint` — os nomes que o C++ lê — com a
saída sem escapes ANSI; `toolchain.setKit { chip }` grava e `toolchain.get`
devolve `STM32F401CC`; o papel `debugAdapter` oferece `lldb-dap` e `probe-rs`
nesta máquina, com `lldb-dap` automático.

**Provado por mutação:** três no `EmbeddedController` (a linha da sonda esquece
a família; o erro de outro domínio acende este painel; trocar de workspace não
esquece a sonda) e uma no fio — tirar o `EmbeddedPanelHost` do
`ShellEnvironmentOverlays` reprova no 19º gate, que é exatamente o buraco que
esta fatia fecha.

```text
gate      20 verificacoes, 32 harnesses (+tst_embedded)
codigo    core: 1 descriptor de paleta (`probe.list`, Ctrl+Alt+M)
          C++:  core_client_probe.cpp (novo), `chip` na ponte da toolchain
          QML:  embedded/ (controller, painel, host, campo), 2 roteadores,
                `debugAdapter` no menu, `chip` no controller da toolchain
catraca   nenhum limite tocado; AppDomains em 390/400 — a proxima fatia que
          precisar de dominio novo la' corta por responsabilidade
```

**O que esta fatia NÃO faz, e está dito na tela:** não grava, não roda, não
sobe o depurador — isso é a fatia 3. E o `probe-rs` continua sem sonda no USB
desta máquina, então o painel mostra hoje o estado "nenhuma sonda conectada"
com a dica do core.

### 7.9 Frente F, fatia 2 — o QEMU e a ponte: o ciclo de embarcado provado sem placa, 2026-09-11

**A ordem da §5.7 do [`35`](35-ambiente-cpp-e-embarcados.md) era fio →
QEMU → tela → polimento.** A fatia 1 (§7.8) ligou a tela ao que o core tinha; a
2 dá ao core o que faltava para depurar o que **não está na máquina**, e prova
o ciclo inteiro no emulador.

**O achado que encurtou a fatia, medido antes de escrever:** o `integracoes/36`
§3 dizia que OpenOCD/QEMU exigiriam "uma ponte a mais" ou "protocolo novo
inteiro". **O GDB fala DAP nativamente desde a v14** (`/usr/share/doc/gdb/NEWS`,
*"Changes in GDB 14"*), por stdin/stdout, com `-i dap` — medido aqui no gdb 17.2
do Fedora, multiarch (`arm`, `riscv:rv32`). A ponte é um **segundo candidato do
papel `debugAdapter`**, não um cliente GDB-remote no core. O 36 §3 foi corrigido.

```text
protocolo 0.89.0   `remoteTarget` (host:porta, vai no `target remote`) e
                   `debugServer` (comando que a IDE sobe, `{program}` = ELF) no
                   toolchain.setKit e no ToolchainResult
dap/adapter.rs     saiu do session.rs: QUAL adaptador, com que argumentos, e que
                   PEDIDO — `attach` quando ha' remoteTarget, `launch` senao.
                   O `gdb` ganhou `-i dap` + dois `-iex` (medido: sem desligar o
                   debuginfod e o confirm, o attach TRAVA mudo perguntando)
dap/server.rs      o processo servidor: sobe com `exec` antes do adaptador,
                   ESPERA a porta abrir (QEMU em 0,05 s), morre com a sessao
gdb candidato      o papel debugAdapter agora oferece lldb-dap, probe-rs e gdb
escopo preferido   o GDB lista `Registers` PRIMEIRO; o core agora prefere
                   Globals/Locals — senao a tela de variaveis mostrava r0..pc
```

**O ciclo provado contra o core REAL, por stdio, no QEMU `lm3s6965evb`** (uma
fixture bare-metal NOSSA em `scripts/fixtures/embarcado/`, não do usuário):
`setKit(gdb, remoteTarget, debugServer)` → `setBreakpoints` → `debug.start`
sobe o QEMU e faz `attach` → `continue` para no breakpoint em `main.c:4` →
`evaluate contador` lê `0`, depois `1` → as variáveis **não** são registradores
→ `stop` → `finished(exitCode 0)`, e o QEMU morreu com a sessão. 19 ms do
`continue` ao `stop`.

**Provado por mutação, e uma mutação foi descartada por medição:**

```text
attach vira launch       o ciclo nunca para (sem servidor a que conectar)
escopo = primeiro barato  a tela mostra r0..pc no lugar de `contador`
Drop do servidor sem kill o teste de server.rs mede o TEMPO do drop: sem kill
                          ele espera 30 s o `sleep` sair sozinho
DESCARTADA: `sh -c` sem   nao muda nada — o shell exec-otimiza comando unico, e
  `exec`                  o QEMU ainda morre. O `exec` defende o caso de comando
                          COMPOSTO; o kill do Drop defende o OpenOCD, que NAO se
                          mata sozinho no pacote `k` do GDB (o QEMU se mata)
```

**O limite honesto, dito aqui e no código:** o QEMU se mata sozinho ao receber
o pacote `k` do GDB no `disconnect`, então o ciclo no QEMU **não** prova o
`kill` do servidor no `Drop` — quem prova é o teste de `server.rs`, que mede o
tempo do drop. O `kill` existe para o **OpenOCD**, que não se mata, e esse
caminho fica NÃO PROVADO até haver uma sonda. Sonda física e ESP32 continuam
fora desta passada (decisão do autor, §5.7).

```text
gate      21 verificacoes (+verificar-embarcado.sh), roda no gate so' onde
          arm-none-eabi-gcc, qemu-system-arm e gdb existem — ausente nao reprova
codigo    protocolo: remoteTarget/debugServer; core: dap/adapter.rs e
          dap/server.rs novos, gdb no catalogo, escopo preferido; fixture e
          driver stdio. Nenhuma linha de QML
testes    672 Rust (+6: adapter, server, escopo); mutacoes: 3 no ciclo de
          embarcado + 1 no Drop do servidor
```

### 7.10 Frente F, fatia 4.1 — o clangd enxerga o compilador cross, 2026-09-11

**Primeiro dos quatro itens de polimento da §5.7.** A fatia 3 (painel com
Gravar/Rodar/RTT via probe-rs) foi PAUSADA por decisão do autor até haver uma
sonda — ela não é exercitável no QEMU. O polimento, que é, segue.

**O que a medição achou, e derrubou meia premissa:** o clangd 22 do Fedora
acha `<stdint.h>` de um alvo `arm-none-eabi` sozinho (ele traz um
`-internal-isystem .../arm-none-eabi/include`). O `.c` de bare metal **não**
fica vermelho. Mas o `.cpp` fica: os cabeçalhos de `libstdc++` do GCC ARM
(`<array>`, `<cstdint>` em `/usr/lib/gcc/arm-none-eabi/15.2.0/.../c++`) o clangd
não encontra — **32 erros** medidos num arquivo de quatro linhas. Com
`--query-driver=<cross>` na allowlist, o clangd pergunta ao GCC seus `-isystem`
e os 32 somem.

```text
Toolchain::clangd_args   `--background-index` sempre; mais
                         `--query-driver=<caminho do cross>` quando o
                         compilador C ou C++ EFETIVO do kit nao e' um nativo
                         (clang/gcc/clangxx/gxx). Regra por EXCLUSAO: cada alvo
                         cross novo entra sem tocar aqui
fiacao                   configure_clangd_from_toolchain roda no workspace.open
                         e em toolchain.set/setKit; vale na PROXIMA subida do
                         servidor cpp (o server vivo nao e' trocado, mesmo
                         contrato do use_server_command)
seguranca                o clangd EXIGE a allowlist explicita — rodar driver
                         arbitrario e' risco. So' entra o compilador que o
                         usuario escolheu, nunca um glob aberto
```

**Provado em dois níveis:** três testes de unidade sobre `clangd_args` (nativo
sem driver; C++ cross com o caminho resolvido; C+C++ cross numa lista só sem
repetir), mutação-provados (contar o cross como nativo derruba os dois); e o
`verificar-clangd-cross.sh` (22º gate), que roda o **clangd de verdade** contra
o `arm-none-eabi-g++` real e mede 32 erros sem o driver, 0 com ele — e se
declara NÃO CONCLUSIVO onde o clangd achar tudo sozinho, em vez de passar em
falso. É a lição do `probe.rs`/`fd`: fixture não vê mudança de ferramenta.

```text
gate    22 verificacoes (+verificar-clangd-cross.sh)
testes  675 Rust (+3). Nenhuma linha de QML
```

**Faltam três itens de polimento** (§5.7), todos exercitáveis sem sonda:
tamanho de flash/RAM após o build (`arm-none-eabi-size`), estado do udev +
passo oficial, e o monitor serial UART.

### 7.11 Frente F, fatia 4.2 — o tamanho do ELF depois do build, 2026-09-11

**Segundo dos quatro itens de polimento.** Quanto o firmware ocupa de
não-volátil e de RAM — o número que decide se o próximo commit ainda cabe no
chip.

```text
build.size (0.90.0)   sincrono (o size le um arquivo em ms). `{ program? }`:
                      ausente, resolve o ELF como o debug.start. Roda
                      `<prefix>size -A` (prefixo do cross do kit) e devolve
                      secoes + regioes do linker script com a fracao usada
core/size.rs          parse do SysV e do bloco MEMORY do `.ld`; regiao = soma
                      das secoes ALOCADAS por endereco (`.comment`/
                      `.ARM.attributes`, addr 0, NAO contam — senao 35 B da
                      string de versao virariam "flash usada")
Toolchain::binutils_prefix  `arm-none-eabi-gcc` -> `arm-none-eabi-`; de onde
                      sai o size, o objcopy, o objdump do alvo
UI                    painel Embarcados: botao "Medir tamanho" e uma barra por
                      regiao (usado/capacidade), amarela e com aviso a partir
                      de 90%. Sem linker script legivel, mostra os totais por
                      secao
```

**Exercitado contra o `arm-none-eabi-size` real** (dobrado no gate de embarcado,
que já compila a fixture): FLASH 132/262144, SRAM 4/65536, ferramenta
`arm-none-eabi-size` escolhida pelo prefixo do kit. Provado por mutação: 5
testes de unidade sobre o parse; anular a exclusão de seção não-alocada faz a
FLASH medir 212 em vez de 132 e o teste cai (a mutação por dead-code foi
descartada — o compilador a pega antes do teste, a armadilha já registrada).

```text
protocolo 0.90.0 — BuildSizeParams, SizeReport
testes  680 Rust (+5). QML: painel Embarcados ganhou a seção de tamanho
```

**Faltam dois itens de polimento** (§5.7): estado do udev + passo oficial, e o
monitor serial UART.

### 7.12 O ESP32 chegou à mesa — o levantamento da conectividade bare metal, 2026-09-11

**Na tarde do mesmo dia em que a §5.7 do [`35`](35-ambiente-cpp-e-embarcados.md)
registrou *"ESP32-C3/C6/S3 existe mas NÃO pode ser plugado agora"*, o autor
plugou um ESP32** e pediu três coisas: o mapa de como um microcontrolador bare
metal se conecta, o que existe de open source para cada elo, e se o que vale
para o ESP32 vale para STM32 e Raspberry Pi. A resposta é o
[`integracoes/38`](../integracoes/38-conectividade-bare-metal.md) — documento,
não código, pela mesma regra que fez o 36 vir antes da etapa 22.

**Medido antes de ler qualquer fonte, e a medição mudou a pergunta:**

```text
o que esta' no USB   ESP32-D0WD-V3 rev v3.1 — o ESP32 CLASSICO (Xtensa LX6),
                     nao um S3/C3/C6. Ponte CP2102 (10c4:ea60) -> /dev/ttyUSB0
                     root:dialout 0660; o autor esta' em dialout; o esptool
                     5.3.1 conectou, leu chip e flash (4 MB) e resetou por RTS
                     sem ninguem apertar BOOT
o que isso implica   o classico NAO tem USB-JTAG embutido: so' tem o canal
                     SERIAL. Gravar (esptool/espflash) e monitorar (UART
                     115200) sao exercitaveis hoje; DEPURAR exige 4 GPIOs + um
                     adaptador FT2232H (ESP-Prog) + o fork openocd-esp32 + o
                     esp-gdb xtensa — o gdb do Fedora nao tem xtensa, e o
                     OpenOCD upstream desta maquina conhece o alvo mas NAO
                     grava nele (sem flash bank, sem program_esp)
quem mais olha       o ModemManager EXAMINOU a porta 4 s depois do plug
a porta              (journal: "couldn't check support ... not supported by
                     any plugin") — ID_MM_CANDIDATE=1 sem ID_MM_DEVICE_IGNORE.
                     brltty instalado, inativo, sem regra para 10c4
permissao USB        o 60-openocd.rules do Fedora e' o upstream SEM
                     GROUP="plugdev" (so' TAG+="uaccess"); o 69-probe-rs.rules
                     oficial nao esta' instalado e usa plugdev + uaccess;
                     plugdev nao existe nesta maquina. O passo oficial e o que
                     a distro ja' fez NAO sao o mesmo — a fatia 4.3 tem de
                     dizer qual vale
o core               NAO sabe o que e' porta serial: o unico ttyUSB0 nos
                     fontes e' um teste negativo do dap/server.rs
```

**O mapa que generaliza** (38 §2 e §5): todo bare metal chega por até três
canais — **A serial** (ROM bootloader + console + DTR/RTS), **B depuração**
(SWD/JTAG por sonda ou embutido) e **C massa/DFU** (UF2, `dfu-util`) — mais
uma camada de **identidade** (VID:PID → família do elo → chip lido **pelo
canal**). O que vale para as cinco colunas (ESP32 clássico, S3/C3/C6, STM32,
RP2040/RP2350, Pi) é o canal A inteiro, a identidade, o mecanismo de permissão
e o canal B, que já está feito. O que **não** vale é o protocolo do
bootloader — SLIP da Espressif, AN3155/DFU da ST, picoboot/UF2 da Raspberry
Pi — e a IDE não implementa nenhum: orquestra a ferramenta oficial como
processo. **A Raspberry Pi 4/5 como computador não é este domínio**: é o item
"SSH remoto" da §4.

**A licença decidiu uma forma, de novo:** o `espflash` (MIT OR Apache-2.0)
como **crate** não passa no `deny.toml` — a serial dele é o `serialport`
4.10.1, **MPL-2.0**, e a política é "nada de copyleft, nem LGPL". Admitir MPL
é decisão do autor; até lá, `esptool` (GPL-2.0) e `espflash` entram iguais:
processo filho, saída em evento, `NO_COLOR` injetado no `Command`. O monitor
serial abre o tty com `rustix` (já transitivo) — sem crate novo.

**O que entra na fila** (38 §6, proposta): E1 `serial.list` → E3 monitor UART
(a fatia 4.4) → E5 identidade Espressif (`chip-id`/`flash-id` como o
`probe-rs info`) → E2 permissão por canal (a fatia 4.3, redesenhada) → E4
gravar como JOB com motor por família. E6, o debug do clássico, fica
**bloqueado por hardware** ao lado da fatia 3. **Nenhuma das doze decisões da
§5.7 foi reaberta por este documento**; a de *escopo* ("RISC-V/ESP32 depois") é
a que o autor vai ter de rever, e a nota datada no 35 diz isso.

```text
gate      22 verificacoes, inalterado — nenhuma linha de codigo
docs      integracoes/38 (novo), indices em DocsPublic/README e integracoes/README,
          nota datada no 35 §5.7, esta secao e a entrada da §4
provado   nada gravado no chip: so' leituras (chip-id, flash-id). O 38 §7 diz
          o que NAO foi provado, item a item
```

### 7.13 E1 — `serial.list`: a IDE passa a ver a porta serial, 2026-09-11

**Primeira das fatias decididas na §7.12.** O core não sabia o que era uma
porta serial (o único `ttyUSB0` nos fontes era um teste negativo do
`dap/server.rs`); agora enumera as USB, e o painel Embarcados as mostra.

```text
serial.list (0.91.0)  /sys/class/tty/ttyUSB* e ttyACM*; sobe ate' o diretorio
                      USB com idVendor (ABI do kernel) para VID:PID, nomes,
                      serial e bInterfaceNumber; driver pelo link; by-id de
                      /dev/serial/by-id; ModemManager por `udevadm info` +
                      /proc/*/comm (null sem udevadm, nao false)
core/serial.rs        NUNCA abre a porta — abrir aciona DTR/RTS e reseta a
                      placa. Permissao MEDIDA com access(2) via `rustix`
                      (+0 crates: ja' era transitiva), porque access honra a
                      ACL do `uaccess`; stat sozinho mentiria
familia               VID:PID -> o que o ELO e' (ponte CP210x, USB-JTAG da
                      Espressif, VCP do ST-Link, Debug Probe...), nunca o chip
                      atras da ponte. Fontes no 38 §5
handler               NAO exige workspace: nao ha' ferramenta do kit aqui
UI                    EmbeddedSerialView (dono proprio; o painel estava em
                      260/300), controller com ports/portsHint/portsBusy, o
                      aviso do ModemManager so' com vivo + candidata + sem
                      IGNORE, e a dica de acesso sem `sudo` que a IDE rodasse
```

**Exercitado contra o core real com o ESP32 plugado:** a resposta bate campo
a campo com a medição manual da tarde — `/dev/ttyUSB0`, CP2102 `10c4:ea60`,
`cp210x`, `if00`, `crw-rw---- dialout` com `readableWritable: true`, e
`modemManager { candidate: true, ignored: false, running: true }`. O chip não
foi resetado. O `verificar-exercitacao.sh` agora pede `serial.list` ao core
real (sem placa: lista vazia com `ports` presente).

**Provado por mutação** (todas com o compilador calado): Rust — parar a subida
do sysfs na interface (reprova nos dois testes de sysfs); trocar `access(2)`
por "bits do grupo no modo" (reprova no caso `0o060`, que é dono sem bits);
trocar vid/pid na família (reprova). QML — o aviso do MM ignorar `running`;
`refresh()` não pedir as portas; trocar de workspace esquecer a sonda e não a
porta. **O limite honesto do teste de acesso:** o caso real do `uaccess` (nó
de root com ACL nomeada) não se monta sem root — medido: numa ACL o dono é
julgado pela entrada do dono; quem prova é a exercitação contra `/dev/ttyUSB0`.

```text
protocolo 0.91.0 — SerialPortInfo, SerialAccess, ModemManagerState
testes  685 Rust (+5, em tests/serial.rs), harness tst_embedded (+16 assercoes)
gate    22 verificacoes; exercitacao ganhou serial.list
```

### 7.14 Containers: Docker e Podman viraram domínio nativo de verdade, 2026-09-12

**A decisão era de 2026-07-17** ([`28`](28-plataforma-de-plugins-e-verticais.md)
§0: *"vão ser cidadãos nativos"*) e até ontem tinha **zero código**. O autor a
priorizou hoje, com um pedido a mais: *"deverá ter um ícone/atalho/visual para
auxiliar no uso, para ativar a ferramenta"* — para Docker **e** para o Grafana,
que já era nativo mas só se alcançava por menu, paleta e Ctrl+Alt+O.

**Medido antes de escrever, e mudou o desenho:** nesta máquina `docker` é o
shim **`podman-docker`** (imprime *"Emulate Docker CLI using podman"* em
stderr); não há Docker Engine; há **Podman 5.8.4 rootless**, socket do usuário
ativo, `podman-compose` 1.6.0 e `podman compose` delegando. Exatamente o
"Podman é alternativa a auditar" do 28 §4 — então o domínio nasceu com **motor**
(docker | podman) atrás da mesma CLI, e a detecção **pergunta ao binário** em
vez de confiar no nome.

```text
protocolo 0.92.0   container.status/list/images (sincronos), action e compose
                   (JOBS, event.container.finished), open (logs|shell numa ABA
                   DE TERMINAL pelo open_command que existia sem chamador)
core/container/    mod.rs: deteccao, status (versao, rootless, socket,
                   responde, compose, passo oficial), comandos; parse.rs: as
                   DUAS formas de JSON (array do Podman, objeto-por-linha do
                   Docker) numa lista so', saida crua sempre
tools.rs           docker, podman, podman-compose no catalogo (tools.detect)
UI                 ContainerController/Panel/ListView/PanelHost + 2 roteadores;
                   AppDomains cortado por RESPONSABILIDADE: os donos do
                   "Ambiente do projeto" foram para AppEnvironmentDomains (ele
                   estava em 390/400); RuntimeController: aba com TITULO
                   opcional (o comando que roda nela)
icone/atalho       SideRail ganhou dois botoes — `container` e `observability`
                   (glifos proprios em KvIconGlyphs.js; nenhum e' marca
                   registrada) — que abrem os paineis SEM projeto; menu
                   Ambiente > Containers...; paleta `container.list`;
                   Ctrl+Alt+W (C e D ja' tinham dono na UI)
invariantes do 28  a UI nunca chama docker (so' o core); acao = job cancelavel;
                   permissao visivel (status e' a tela de ATIVAR: instalar,
                   grupo docker, daemon, podman.socket — impresso, nunca sudo);
                   rm SEM -f; compose up e' -d
```

**Exercitado contra o Podman real:** `status` bate com a medição manual
(podman 5.8.4, emulated, rootless, socket, compose); `list` devolve os 5
containers do autor com nomes, estado, status humano e portas; `images` os 12
(o Podman em `--format json` **não** traz `repository`/`tag` — vem de `Names`,
e o parser aprendeu isso na exercitação, não no teste); o ciclo **start →
logs em aba → rm** num container descartável (`podman create … true`) rodou
por jobs com `event.container.finished { ok: true }` e o container sumiu. O
`verificar-exercitacao.sh` agora pede `container.status` e `container.list`.

**Provado por mutação** (compilador calado): Rust — o shim tratado como Docker
(reprova na detecção); `rm -f` (reprova); `host_ip` ignorado nas portas
(reprova). QML — job falho sem motivo na tela; `isRunning` pelo status humano;
trocar de workspace esquecer o motor. **O que NÃO foi provado:** um Docker
Engine real (a forma `{{json .}}` é da documentação); `compose up/down` de
verdade (só a montagem do comando); o shell dentro do container (só o comando).

```text
testes  693 Rust (+8 em tests/container.rs), 33 harnesses (+tst_container)
docs    03-ipc (dominio + indice), 02 (container/), 28 §4 (nota), 41 (F3)
```

**O que fica para a fatia seguinte deste domínio, e é o que o 28 §4 chama de
contexto remoto:** *dev containers* de verdade — abrir o workspace **dentro**
do container (path mapping, `devcontainer.json` como formato de entrada, build
e LSP do outro lado). É o mesmo contrato do SSH remoto da §4, e os dois devem
nascer da mesma abstração, não de duas.

### 7.15 E3 — o monitor serial como processo numa aba de terminal, 2026-09-12

**A decisão era de 2026-09-11** (38 §6: processo pronto, nunca código serial
nosso — a forma da extensão oficial da Espressif para o VS Code) e a aba com
título que o domínio `container` acabara de dar ao `RuntimeController` era
exatamente o que faltava.

```text
papel `serialMonitor` (kit)   candidatos tio, picocom, minicom, espflash — a
                              ordem e' a preferencia automatica; a UI lista o
                              papel no menu de toolchain desde o nascimento
serial.monitor (0.92.0)       { device, baud? } -> aba de terminal com o
                              comando no titulo. Chip Espressif no kit E
                              espflash detectado -> `espflash monitor --elf`;
                              escolha FIXADA vence; sem monitor nenhum,
                              TOOL_NOT_FOUND com o que instalar
linhas de comando             lidas na fonte: espflash e' `--monitor-baud` —
                              o `--baud` dele e' o de GRAVACAO (pegadinha que
                              a fonte mostrou); tio/picocom `-b`; minicom `-D -b`
UI                            botao de monitor por porta no painel Embarcados
                              (so' com acesso a porta); a aba abre e o painel
                              fecha
```

**Exercitado no ESP32 real:** `serial.monitor { device: /dev/ttyUSB0 }` →
`/usr/bin/picocom -b 115200 /dev/ttyUSB0` numa aba (`picocom` e `minicom` são
os detectados nesta máquina; `tio` e `espflash` não estão), e o render da aba
mostrou o banner do picocom com a porta aberta. Nenhuma linha de código serial
foi escrita. **Provado por mutação:** `--baud` no lugar de `--monitor-baud`;
espflash para qualquer chip; ignorar a escolha fixada — as três reprovam.

```text
testes  695 Rust (+2), tst_embedded (+2 assercoes); exercitacao pede serial.monitor
```

### 7.16 Pilar 0, primeira fatia — o MODELO do projeto embarcado, 2026-09-12

**A resposta em código à pergunta do autor** ("a IDE já lê todo o projeto?",
[`42`](42-trilha-profunda-embarcados.md) §1): até hoje o `workspace` olhava
marcadores só na raiz e não conhecia framework de embarcado nenhum. Nasceu o
domínio `project` (protocolo 0.93.0).

```text
project.model / event.project.changed   o que o projeto E', com evidencia
detect.rs     9 frameworks — ESP-IDF, Zephyr, pico-sdk, PlatformIO, STM32Cube,
              Rust embarcado, MicroPython, Yocto, Buildroot — reconhecidos ate'
              3 niveis abaixo, LENDO o marcador (project.cmake, find_package
              (Zephyr), pico_sdk_init, `import machine`...), ignorando pastas
              de saida, com o detalhe que o arquivo diz (IDF_TARGET, PICO_BOARD,
              DeviceId, triple, MACHINE, ambientes do PlatformIO)
sdk.rs        o que cada framework exige e se esta' aqui — variavel, pasta
              padrao ou binario; ~/.espressif/tools e' vasculhado para a
              toolchain que so' o export.sh poe no PATH; o passo oficial em
              cada falta; nenhum processo roda
artifacts.rs  ELF/bin/hex/uf2/map dos build dirs de cada framework, do mais
              novo ao mais velho; flasher_args.json; tabela de particoes;
              memory.x; linker scripts da fonte; o ELF sem extensao do cargo
              pelo magico
alvo          kit > framework; familia; motores por familia — e o ESP32
              classico recebe "sem depurador: ESP-Prog" em vez de um probe-rs
              que nao funcionaria
evento        emitido em activate_workspace (open e createProject) e ao fim de
              event.cmake.finished / event.build.finished: a tela SEGUE o modelo
tela          EmbeddedProjectView no painel Embarcados: framework · evidencia ·
              detalhe; alvo com as linhas de evidencia; ✓/✗ por SDK com o
              passo; as dicas
fixtures      scripts/fixtures/projetos/<framework>/ — reais e minimas, as
              que os testes leem
```

**Exercitado contra o core real** com a fixture ESP-IDF como workspace: o
evento chega na abertura com `espIdf · CMakeLists.txt · IDF_TARGET esp32c3`,
o alvo `esp32c3/espressif` com `esptool/espflash/probe-rs`, `esptool` achado
em `~/.local/bin`, ESP-IDF e a toolchain riscv **faltando** com o passo — que
é exatamente o estado desta máquina. **Provado por mutação** (compilador
calado): descer em `build/` (a cópia gerada venceria); o framework vencer o
kit; pasta "achada" sem existir; e no QML: trocar workspace sem esquecer o
modelo, `refresh` sem pedir, resumo do alvo sem o depurador.

**Segunda fatia, no mesmo dia:** `flasher_args.json` e `partitions.csv`
passaram de localizados a **lidos** (`project/esp.rs`), com a forma tirada da
fonte do ESP-IDF (`flasher_args.json.in` + `project_include.cmake`; guia
*Partition Tables*): a receita com caminhos absolutos, nome e cifragem por
imagem, e a tabela com os offsets em branco resolvidos como o
`gen_esp32part.py` (a primeira em tabela+0x1000, `app` a 64 KB). Linha que
não se lê vai em `unreadable`. Provado por mutação: caminho não resolvido,
linha ilegível sumindo, `app` alinhada a 4 KB — as três reprovam. A vista do
projeto mostra "receita: N imagens, flash 4MB dio · N partições, app 1 MB em
0x10000".

**Terceira fatia, no mesmo dia — o modelo passa a ser CONSUMIDO:** o
`build.size` de um projeto ESP-IDF ganha a região `factory (particao app
@0x10000)`, com a **imagem** (`.bin` que a receita aponta como `app`) como
usado e a partição como capacidade — a partição que casa é a que **começa**
no offset da receita, não "a primeira app" (numa tabela OTA seria outra).
É o que o `idf.py size` chama de *total image size*; até aqui o ESP32 não
tinha flash nenhuma no painel porque o `.ld` do IDF não a declara. Provado
por mutação: "a primeira app" e "usado = capacidade" reprovam; o teste de
despacho monta um workspace ESP-IDF falso com receita, tabela e uma imagem
de 123.456 bytes.

**O que falta do P0** ([`42`](42-trilha-profunda-embarcados.md) §3): o modelo
por **alvo/preset** (hoje é um por workspace), o map file interpretado, e a
detecção de SDK que hoje o modelo declara "não medido" (alvo rustup).

```text
protocolo 0.93.0 — project.*, event.project.changed, FlashRecipe, PartitionTable
testes  705 Rust (+10, tests/project.rs e size.rs), tst_embedded (+9 assercoes)
gate    exercitacao pede project.model
```

### 7.17 Pilar 0, quarta fatia — o índice do projeto INTEIRO, 2026-09-12

**A exigência do autor, textual:** *"a IDE deve ler o projeto inteiro que
for aberto, ter integração profunda de leitura do contexto do
código/compilador, deve ser lido todas funções/arquivos/pastas, etc. Tudo que
for de acordo com python/c/c++ e rust."* Até aqui a IDE lia de forma
preguiçosa e delegada ([`42`](42-trilha-profunda-embarcados.md) §1). Nasceu
o domínio `index` (protocolo 0.94.0).

```text
index/mod.rs      ao abrir o workspace, um JOB caminha a arvore inteira (a
                  MESMA lista de pastas ignoradas do watcher), conta todo
                  arquivo, le os de fonte, extrai as declaracoes de C/C++/Rust
                  com as gramaticas Tree-sitter do editor (lang/extract.rs:
                  a query `tags` oficial, sem cache); Python contado e medido
                  ate' a gramatica entrar (entrou a tarde, §7.19); >4 MiB ou
                  ilegivel = contado e DITO
                  em `skipped`; cancelavel; progresso a cada 200 arquivos
index.status      totais: pastas, arquivos, fonte, linhas, bytes, simbolos,
                  funcoes, tipos, por linguagem, elapsed, estado
index.symbols     exato > prefixo > substring, sem caixa, filtro por kind,
                  total antes do limite
incremento        os caminhos de event.fs.changed sao reindexados no loop
                  principal e os totais reemitidos
UI                barra de status: "indice: 1.010 arquivos · 70.030 linhas ·
                  3.720 simbolos" (e o progresso enquanto constroi);
                  `#nome` no Search Everywhere pede ao indice E ao LSP — o
                  indice responde primeiro e SEM arquivo aberto (antes,
                  `#nome` recusava sem editor com LSP); o LSP substitui
sem duplicata     a `tags` do Rust captura o fn de impl como function E
                  method (medido aqui: handle_request em dobro); fica a mais
                  especifica por (linha, nome). `struct` vem como `class` da
                  gramatica e o indice NAO renomeia
```

**Medido neste repositório pelo core real:** 1.010 arquivos, 146 pastas,
307 de fonte, 70.030 linhas em 1,97 s no build de depuração; `escolher` e
`handle_request` achados sem LSP. Um projeto de 1 M de linhas leva ~30 s neste
ritmo, em job com progresso. **CORRIGIDO à tarde (§7.19):** o número de
declarações aqui dizia 4.658 (3.802 funções, 524 tipos) — medido ANTES de o
dedup do `fn` em `impl` entrar no mesmo commit. O mesmo código, remedido, dá
**3.720**; com a gramática Python, 3.939.

**Provado por mutação** (compilador calado): Rust — entrar em pastas de saída
(os números mentiriam); substring antes de prefixo; arquivo apagado ficando
no índice; dedup por kind em vez de nome. QML — o índice voltar por cima do
LSP; caminho relativo não virar absoluto (o clique não abriria); fechar o
workspace deixando o resumo do anterior. O teste `test_run_starts_a_job` foi
corrigido para filtrar os eventos de job pelo id — ele assumia um job só, e
agora o `workspace.open` sobe o do índice.

**Limite honesto, e é a próxima fatia:** o watcher observa só as pastas que
a UI expandiu (ADR-0001); mudança fora delas entra no próximo `workspace.open`
(fechado à tarde — §7.20). E o índice é **estrutural**: o contexto de
compilador por arquivo (flags da CDB por TU, crate/target do `cargo
metadata`, interpretador/ambiente do Python) — a segunda metade da exigência
— é o que vem em seguida no P0 (§7.18).

```text
protocolo 0.94.0 — index.status, index.symbols, event.index.progress/finished
testes  712 Rust (+7, tests/index.rs), 34 harnesses (+tst_index)
gate    exercitacao pede index.status e index.symbols (main sem LSP)
```

### 7.18 Pilar 0, quinta fatia — o CONTEXTO DE COMPILADOR por arquivo, 2026-09-12

A segunda metade da exigência do §7.17: *"integração profunda de leitura do
contexto do código/compilador"*. O índice dizia **o que há** em cada arquivo;
agora diz **com que** cada arquivo é compilado ou executado (protocolo
0.95.0, `index/context.rs`, `index.context { path }`).

```text
C/C++     a unidade da compile_commands.json que o cdb::status acha, nas DUAS
          formas do padrao (arguments[] e command dividido como shell); file
          relativo resolvido contra directory e chave pelo caminho REAL
          (canonicalize: a CDB do Ninja escreve ../src/a.cpp); -I/-isystem/
          -iquote colados ou separados -> includes absolutos; -D -> defines;
          -std= -> standard; output vence -o; o resto inteiro em arguments.
          Cabecalho: sem unidade, com a dica (o clangd deduz pela unidade que
          o inclui). Fonte fora da CDB: "nenhum alvo o compila" — ou QUAL
          arquivo envelheceu a CDB
Rust      cargo metadata --no-deps com o cargo do kit EFETIVO (so' se ha'
          Cargo.toml na raiz e o cargo existe; PATH vazio nos testes = nada
          roda). src_path EXATO > diretorio de src_path mais longo > lib em
          empate (src/x.rs e' do lib E do bin; o rust-analyzer prefere o lib).
          build.rs (custom-build) so' casa exato: por prefixo seria dono de
          tudo
Python    a precedencia do roadmaps/29 §4.1: VIRTUAL_ENV (se o interpretador
          existir) > .venv/ > venv/ > env/ > poetry.lock + `poetry env info
          -p` > python3 do PATH COM AVISO; version = --version
recarga   event.cmake.finished (o configure reescreve a CDB) e Cargo.toml em
          event.fs.changed recarregam SO' o contexto, em job curto; os
          arquivos ficam; event.index.finished sai com o context novo e a UI
          pergunta de novo pelo arquivo ativo
injecao   o que roda processo (cargo, poetry, python --version) vem por
          `Ferramentas`, injetado pelo Core: teste passa None e nada da
          maquina entra (a regra do `unsafe`/env var, aplicada a processo)
UI        barra de status: "contexto: c++ · gnu++23 · 12 -I · 9 -D",
          "cargo · kinein-core (lib, 2024)", "python · sistema · 3.14.7 ⚠";
          o detalhe (diretorio, origem, dica) ao pairar; resposta atrasada de
          outro arquivo descartada; trocar de arquivo limpa ate' chegar
```

**A falha silenciosa que este gesto achou — e fechou.** Medido neste
repositório pelo core real: a CDB de `.kinein/build` era de **04/09** e não
tinha `core_client_probe.cpp` nem `core_client_index.cpp` (cinco fontes que o
`ui/CMakeLists.txt` de **12/09** acrescentou), e o `cdb::status` dizia
`stale: false` — ele só compara a CDB com os arquivos de build da **raiz**.
O contexto, com as unidades em mãos, sobe de cada diretório de fonte até a
raiz procurando um `CMakeLists.txt` mais novo que a CDB: `cdbStale: true,
cdbStaleBecause: "ui/CMakeLists.txt"`, e a unidade velha vem com a dica
"reconfigure". Reconfigurado, 288 unidades e `core_client_index.cpp` com
`c++`, `gnu++23`, 12 `-I`, 9 `-D`. A subida **para na raiz do workspace** (um
`CMakeLists.txt` alheio acima dela não vale — testado).

**Medido neste repositório:** 288 unidades; 4 pacotes, 6 alvos do cargo
(`context.rs` → `kinein-core`/`kinein_core` lib 2024; `main.rs` → bin);
Python do sistema `/usr/bin/python3` 3.14.7 com o aviso. `cargo metadata
--no-deps` leva 32 ms aqui.

**Provado por mutação** (compilador calado, 24 em Rust + 9 em QML): lib
perdendo o empate; sem `src_path` exato; sem "diretório mais longo" (precisou
do alvo `src/bin/tool/main.rs` dentro de `src/` para morrer); `build.rs` dono
de tudo; envelhecer ao contrário; `staleBecause` absoluto; subir além da
raiz; `venv` antes de `.venv`; `VIRTUAL_ENV` fantasma valendo; `-o` vencendo
`output`; forma separada `-I x` perdida; chave sem canonicalizar; `stale`
sempre falso; sistema sem aviso; versão invertida; features sem ordem
(precisou de três features fora de ordem: com duas, o inverso da inserção
era o ordenado); `kind` errado; Python virando `other` no handler; gatilho
`Cargo.lock`; recarregar quando NÃO é `Cargo.toml`; C sem contexto; `.h` não
cabeçalho; sem `-iquote`; nunca achar a CDB. QML — resposta atrasada aceita;
não perguntar de novo ao terminar; mesmo caminho perguntando de novo;
perguntar sem workspace; CDB velha sem aviso; sistema sem aviso; só-dica
sumindo; trocar de arquivo mantendo o contexto velho; detalhe sem a dica.

**Um teste que falhava ao acaso, e por quê:** dois testes escrevem scripts
executáveis e os executam; quando correm em threads paralelas, o `exec` de um
acha o script do outro ainda aberto para escrita (`ETXTBSY`: o filho herda o
descritor entre o fork e o exec). Um `Mutex` estático serializa os dois — o
mesmo risco existe entre módulos de teste que fazem o mesmo (container,
toolchain, serial), e fica anotado.

**O que ainda falta no P0** ([`42`](42-trilha-profunda-embarcados.md) §3):
modelo por alvo/preset; map file; `rustup target list --installed` medido. (A
gramática Python e o índice seguindo o disco inteiro entraram logo depois —
§7.19 e §7.20.)

```text
protocolo 0.95.0 — index.context; IndexStats.context (ContextSummary)
testes  719 Rust (+7, tests/index_context.rs), 34 harnesses (tst_index +12 assercoes)
gate    exercitacao escreve uma CDB a mao e pede index.context (standard c++20)
        e ve cdbEntries no status — e' o que prova o JOB carregando o contexto
```

### 7.19 Python entra na fundação sintática — a quarta gramática, 2026-09-12

O B1 do [`41`](41-ecossistema-embarcados-e-python.md) e o item "gramática
Python" do P0 ([`42`](42-trilha-profunda-embarcados.md) §3): `tree-sitter-python`
0.25.0 (MIT, LICENSE lido no repositório; publicado em 2025-09-11) entrou no
registro ao lado de C/C++/Rust, com a `highlights` e a `tags` oficiais e um
`locals` mínimo escrito aqui (função é escopo; parâmetro e atribuição definem;
identificador referencia). Zero dependência externa — é o único item do
bloco B que não precisa de nada instalado.

```text
o que muda        .py/.pyi/.pyw sao linguagem `python` na fundacao: realce,
                  outline (classe > metodo), folding e locals no editor pelo
                  MESMO syntaxTree.update; o indice extrai as declaracoes (a
                  tags oficial da' function e class; o metodo vem como
                  function com container); index.context ja' classificava
                  Python — agora pela gramatica, nao pela extensao
medido            neste repositorio: 18 .py, 3.577 linhas, 217 declaracoes
                  (scripts/medir-core.py, verificar_qml_propriedades.py, ...);
                  o indice inteiro em 2,1 s
o numero errado   o §7.17 dizia 4.658 declaracoes — medido ANTES de o dedup
                  entrar no mesmo commit. Remedido: 3.720 sem Python, 3.939
                  com. Corrigido no §7.17, no 03-ipc, no 42 e no harness
```

**Provado:** teste de `syntaxTree.update` com Python (linguagem, sem erro,
outline com a classe/o método filho/a função de módulo nas linhas certas,
escopos `keyword`/`function`/`comment`/`string` no realce, folding, escopo
local); o teste do índice passa a exigir a declaração Python; a exercitação
escreve um `tools/gera.py` e pede `index.symbols` (`"language":"python"`).

```text
protocolo 0.95.0 (sem mudanca de fio: a linguagem ja' era um campo)
testes  720 Rust (+1), 34 harnesses; deps: +tree-sitter-python 0.25.0 (MIT)
gate    exercitacao pede index.symbols de um .py
```

### 7.20 O índice segue o disco INTEIRO — as pastas caminhadas entram no watcher, 2026-09-12

O "limite honesto" do §7.17: o watcher só observava a raiz e as pastas que a
UI expandiu (ADR-0001, `NonRecursive` por decisão), então um arquivo criado
pelo terminal — ou por um `git checkout` — numa pasta fechada ficava fora do
índice até o próximo `workspace.open`, **em silêncio**. Era a "indexação
agressiva" mentindo por omissão.

```text
o que muda        ProjectIndex guarda `folder_paths` (as pastas que caminhou,
                  a raiz inclusa); a cada event.index.finished o Core
                  registra TODAS no watcher, uma a uma, NonRecursive, com a
                  mesma lista de pastas ignoradas — o ADR-0001 fica de pe':
                  nada de recursivo na raiz (que arrastaria target/, build/,
                  .git/); idempotente (o watcher ignora pasta ja' registrada);
                  para no PRIMEIRO erro e o relata uma vez (o limite de
                  inotify e' o caso real)
pasta nova        reindex_paths caminha a subarvore inteira (o mesmo
                  `caminhar` do build), ela entra em folder_paths e, pelo
                  event.index.finished que sai dai, no watcher
pasta apagada     leva os arquivos dela e as subpastas; pasta ignorada que
                  nasce (src/target/) nao entra
medido            148 inotify watches neste repositorio (= 148 pastas), contra
                  186.243 de fs.inotify.max_user_watches nesta maquina
```

**Provado:** unidade (pasta nova caminhada com dois níveis, idempotência,
pasta ignorada fora, pasta apagada limpando 2 arquivos + 2 pastas); **ponta a
ponta com o inotify real** (`enable_lsp`, `workspace.open`, um arquivo em
`src/net/` que ninguém listou, uma pasta nova `src/hal/` e um segundo arquivo
nela, a pasta apagada — tudo pelo `event.fs.changed` → `index.symbols`); a
exercitação cria `src/tarde/tarde.c` depois do índice pronto e o acha. Seis
mutações mortas com o compilador calado (registrar só a primeira pasta; nunca
registrar ao terminar; pasta ignorada entrando; só a pasta de partida na
lista; pasta nova nunca caminhada; pasta apagada deixando os arquivos;
subpasta apagada ficando). Uma mutação **sobreviveu e mudou o código**: o
registro explícito das pastas novas no incremento era redundante com o
registro no `event.index.finished` — saiu, e ficou um caminho só.

```text
protocolo 0.95.0 (sem mudanca de fio)
testes  722 Rust (+2), 34 harnesses
gate    exercitacao: arquivo nascido em pasta nova depois do indice pronto
```

### 7.21 O MODELO POR ALVO do CMake — a "CDB em memória" do file-api, 2026-09-12

O item "modelo por alvo" do P0 e o item 4 do "efeito JetBrains"
([`42`](42-trilha-profunda-embarcados.md) §8): o backend extrai flags,
includes e targets do sistema de build e alimenta o LSP e a tela — na forma
que o `CMake` oferece de verdade, o **file-api**, nunca parseando
`CMakeLists.txt`. Protocolo 0.96.0; `cmake.rs` virou `cmake/mod.rs` +
`cmake/model.rs`.

```text
cmake/model.rs    CmakeModel::load(build_dir): o codemodel-v2 -> por target:
                  id, nome, tipo cru, artefatos ABSOLUTOS ao build dir, pasta
                  de fonte, fontes (geradas marcadas; indice do grupo),
                  grupos de compilacao (linguagem, languageStandard, includes,
                  defines, compileCommandFragments, sysroot), dependencias
                  com os ids RESOLVIDOS para nome (alvo importado, fora do
                  codemodel, nao entra), linguagem do link; a toolchains-v1
                  (CMake >= 3.20; a query passou a pedi-la) -> compilador por
                  linguagem com id, versao e includes implicitos.
                  targets_for(arquivo) e compile_group_for(arquivo): o INVERSO
cmake.targets.list cada target com artifacts, sources/generatedSources,
                  languages, standard, includes/defines distintos, sysroot,
                  dependencies, sourceDir — utilitarios continuam fora
index.context     `targets` em todo arquivo C/C++ (cabecalho incluso: o target
                  o LISTA); sem compile_commands.json, a unidade vem do grupo
                  de compilacao + compilador da toolchains-v1 ("(CXX do kit)"
                  quando ela nao existe — dito, nao inventado); com a CDB, a
                  CDB vence e os targets ficam. O cabecalho ganhou a dica
                  certa em qualquer caso (antes, sem CDB, dizia "configure")
UI                o target dono do arquivo no detalhe do chip de contexto
```

**Medido neste repositório pelo core real:** 19 targets no modelo (1
linkável: `kinein-vectis`, 301 fontes + 521 geradas, `CXX` 23, 12 includes,
8 defines, artefato `.kinein/build/ui/kinein-vectis`); `core_client.h` →
`targets: ["kinein-vectis"]` sem unidade; reconfigure com cache em 1,3 s (o
configure do zero levou 47 s). **E uma armadilha do gate:** com
`CMAKE_CXX_STANDARD 20` o GCC 16.2 desta máquina já é C++20 por padrão e o
CMake **não escreve flag nenhuma** — a CDB sai sem `-std=`; o projeto de
exercitação passou a pedir 23 para a flag existir e o gate não mentir.

**Provado:** fixture fiel à forma do cmake-file-api(7) (dois targets reais,
um utilitário, dependência importada, dois grupos com defines repetidos,
`toolchains-v1` opcional); quatro testes (o modelo; `cmake.targets.list` com
todos os campos e o utilitário fora; `index.context` sem CDB → unidade do
file-api com `-std=` do fragmento e o compilador da linguagem certa, cabeçalho
com targets e sem unidade, arquivo solto com a dica de configurar, e a CDB
vencendo depois do `event.cmake.finished`; sem `toolchains-v1` o compilador é
dito como do kit). Exercitação com o **cmake real**: configure em
`.kinein/build` com a query, `index.context` com a unidade da CDB real e o
target, `cmake.targets.list` com fontes, padrão e artefato. Onze mutações
mortas com o compilador calado (dependência não resolvida ficando como id;
gerada invertida; cabeçalho ganhando grupo; `.` não normalizado; artefato
relativo à fonte; `languageStandard` vencendo o fragmento; sempre o
compilador de C++; unidade do file-api nunca; utilitário listado; gerada
contada como fonte; includes/defines sem dedup; linguagem repetida). Duas
equivalentes ficaram ditas: include relativo ao build (o file-api escreve
includes absolutos) e targets pela chave não canônica (raiz canônica nos
testes).

```text
protocolo 0.96.0 — CmakeTargetInfo com o modelo; FileContext.targets;
        ContextSummary.cmakeTargets; a query pede toolchains-v1
testes  726 Rust (+4), 34 harnesses (tst_index +2 assercoes)
gate    exercitacao configura com o cmake real e le o modelo por alvo
```


### 7.22 A documentação reorganizada em duas árvores — e a simulação fora dela, 2026-09-12

Pedido do autor: *"organiza a documentação do projeto, para tudo ficar
explícito e de fácil entendimento… uma pasta DocsPrivate no .gitignore com
dados de conversa com IA etc., e DocPublic para toda a documentação, com
pastas de nomes explícitos e o mesmo sobre os .md; remover tudo sobre
simulação."* As três perguntas foram feitas e respondidas: código da
simulação sai junto (§7.21 antecede; o commit `remove(sim)` faz isso);
número + nome explícito nos `.md` (o número é o sistema de citação);
`DocsPublic/` + `DocsPrivate/`.

```text
docs/            -> DocsPublic/            (git mv; 198 arquivos com referencia reescrita)
docs-privada/    -> DocsPrivate/           (.gitignore; saiu do indice do git)
docs-legada/     -> DocsPrivate/legado/    (estava rastreada apesar do .gitignore)
PONTO_ATUAL.md   -> DocsPrivate/historico/ (registro do porque, nao estado)
MANUAL.md        -> DocsPublic/manual.md   (o AppImage continua a embarca-lo)
Tutorial.md      -> DocsPublic/tutorial.md (o dist/ continua a entrega-lo como Tutorial.md)
COMO_EXECUTAR.md -> DocsPublic/build/como-executar.md
specs/           -> especificacoes/        (17 KINEIN_VECTIS_*.md em kebab-case
                                            explicito, .svg ao lado renomeados junto)
adr/             -> decisoes-adr/
tooling/*.json   -> integracoes/registro-de-componentes-abertos.json
LEITURA_TECNICA  -> leitura-tecnica.md; CONTRIBUINDO -> contribuindo.md;
                    COMANDOS_BUILD_VERIFICACAO -> comandos-de-build-e-verificacao.md;
                    os tres roadmaps em caixa alta -> nomes explicitos
README por pasta arquitetura/, roadmaps/, especificacoes/, build/, seguranca/,
                    decisoes-adr/ (integracoes/ e iconografia/ ja' tinham)
```

O que **não** mudou: `ARCHITECTURE.md` e `README.md` (convenção), os números
dos documentos, a `iconografia/` por dentro (assets com checksum). O gate de
links (`verificar-links-docs.sh`) foi o que segurou a mudança: 222 links
relativos, nenhum morto ao fim; o de veracidade e o do AppImage (que embarca o
manual e entrega o tutorial) continuam verdes.


### 7.23 Toolchains por alvo — o catálogo credível entra no detector e no kit, 2026-09-12

O autor trouxe um levantamento (de outra IA) sobre toolchains por target
triple e pediu para conferir a credibilidade, implementar o que fosse real e
achar o que faltava. O resultado está no
[`integracoes/39`](../integracoes/39-toolchains-por-alvo.md): linha a linha na
fonte, com data. Duas afirmações desatualizadas (a Espressif unificou as
toolchains em `xtensa-esp-elf`/`riscv32-esp-elf` desde o IDF 5; o LLVM
Embedded Toolchain for Arm parou em 19.1.5 e o sucessor é o Arm Toolchain for
Embedded 23.1.0, de 2026-09-10), uma errada (a pasta do xpm é
`~/.local/xPacks`), e três ausências (o Zephyr SDK como bundle oficial, os
servidores de debug, e o **sysroot** — o problema real do Linux embarcado).

```text
tools/known.rs        a tabela de ferramentas saiu do tools.rs (tools/mod.rs):
                      +13 compiladores cross (riscv-none-elf, riscv32-esp-elf,
                      xtensa-esp-elf, aarch64-linux-gnu, arm-linux-gnueabihf,
                      riscv64-linux-gnu, C e C++, com a grafia do OUTRO
                      distribuidor como binario alternativo), +4 GDBs de alvo,
                      +6 meta-ferramentas (west, pio, picotool, dfu-util,
                      openocd, espup)
toolchain/catalog.rs  os mesmos como candidatos dos papeis cCompiler,
                      cxxCompiler e debugAdapter; dap/adapter.rs reconhece
                      QUALQUER GDB pelo nome (gdb, gdb-multiarch, <triple>-gdb)
                      e lhe da' o `-i dap` — nao um adaptador que so' termina
                      parecido
tools/search_dirs.rs  onde procurar ALEM do PATH: a pasta da IDE, o store do
                      xpm (XPACKS_STORE_FOLDER), o ~/.espressif/tools
                      (IDF_TOOLS_PATH), ~/.cargo/bin, ~/.local/bin, /opt/*/bin
                      — so' o que existe, deterministico, puro; o PATH primeiro
toolchain.get         rustTargets (rustup target list --installed, do rustup
(0.97.0)              DETECTADO; ausente sem rustup) e sysrootHint: compilador
                      cross `*-linux-gnu*` sem sysroot no kit e sem usr/include
                      no sysroot que ele mesmo declara (-print-sysroot) ganha a
                      dica com as tres saidas (rsync da placa, Bootlin, SDK)
```

**Medido nesta máquina:** o detector passou a ver `aarch64-linux-gnu-gcc` e
`arm-linux-gnu-gcc` (Fedora 16.1.1) como candidatos; `rustTargets:
["x86_64-unknown-linux-gnu"]`; fixando o `aarch64-linux-gnu-gcc` no kit, a
dica veio com o sysroot real (`/usr/aarch64-linux-gnu/sys-root` sem
`usr/include`) — o pacote da distro compila e não linka programa de usuário
nenhum, e agora a IDE diz isso em vez de deixar o link falhar.

**Provado:** os diretórios extras com uma `$HOME` falsa (ordem exata, só o
que existe, as duas variáveis mandando); os alvos Rust por um `rustup` falso
que só responde a lista com `--installed`; a dica de sysroot por um
`aarch64-linux-gnu-gcc` falso (automático em bare metal sem dica; fixado com
dica; `usr/include` presente sem dica; sysroot no kit sem dica); todo GDB
recebendo `-i dap` e `mygdb` não. Onze mutações mortas com o compilador
calado; uma equivalente dita (o dedup dos diretórios).

**A decisão registrada em §5:** instalar na pasta da IDE pode ser download
depois de um clique, com checksum — o provedor é a próxima fatia (39 §5).

```text
protocolo 0.97.0 — ToolchainResult.rustTargets, sysrootHint
testes  645 Rust (+4), 27 harnesses; ferramentas conhecidas: 52 (+23)
```


### 7.24 A cadeia Python, fatia 1 — o ambiente do projeto num clique, 2026-09-12

O autor perguntou *"como está para desenvolver em Python na IDE, o que falta"*
e a resposta medida foi: fundação pronta (projeto reconhecido, editor, índice,
interpretador resolvido), **cadeia por fazer** (LSP, ruff, run, pytest,
debugpy, ambiente). Ele mandou seguir a cadeia recomendada, fatia 1: as
toolchains Python no detector, os guias oficiais no painel de instalação, e
**criar o `.venv` num clique**. Domínio `python` (0.98.0).

```text
python/env.rs        o resolvedor do interpretador (29 §4.1) saiu do index/context
                     para um lugar so' — index.context e python.status leem dele;
                     PythonTools injeta poetry, python3, VIRTUAL_ENV, medir versao
python/mod.rs        status(): interpretador, hasEnvironment (origem != sistema),
                     environmentTool (uv > venv; uv SEM python3 basta — ele baixa
                     um Python), arquivos de projeto, a dica com o remedio;
                     create_environment_command(): `uv venv .venv` ou `python3 -m
                     venv .venv`, como a fonte oficial escreve
handlers/python.rs   python.status; python.createEnvironment como JOB: `$ comando`
                     na saida, success SO' com .venv/bin/python existindo,
                     event.python.finished; o observe_notification recarrega o
                     contexto — o interpretador do projeto passa a ser o novo
tools/known.rs       +9: python3, uv, pipx, ruff, basedpyright(-langserver),
                     pytest, mypy, poetry, mpremote (61 ferramentas)
setup/catalog.rs     +4 ferramentas (pipx, uv, ruff, basedpyright) e 7 guias com
                     fonte e data: pipx por distro (Fedora, Ubuntu), uv/ruff/
                     basedpyright agnosticos (familia `any`, que a familia da
                     maquina vence — ruff no Arch e no openSUSE)
UI                   PythonController (so' em workspace com `python` nos build
                     systems); a faixa de saude: "o projeto usa o Python do
                     SISTEMA: crie um ambiente" + botao "Criar .venv com uv";
                     ShellWorkspaceHost passou a emitir UM sinal
                     (healthActionRequested) e o Main.qml despacha — o host
                     estava a 398/400
```

**Medido nesta máquina:** python3 3.14.7, pipx, ruff, mypy, poetry 2.4.1,
mpremote 1.29.0 detectados; uv, basedpyright, pytest ausentes; a exercitação
criou o `.venv` do projeto de exercitação com `python3 -m venv .venv` (real)
e o `python.status` passou de `sistema` para `.venv`.

**Provado:** os três estados do status (sem Python; sistema com uv/venv;
ambiente próprio), o uv sem python3, as recusas com motivo, o job com um `uv`
falso que grava o pedido (`venv .venv`), o contexto seguindo o ambiente novo
(`index.context` → `.venv`, versão do interpretador novo), a ferramenta que
sai com 0 sem criar o interpretador = `success: false` e job `failed`; a
precedência família > `any` no setup; o harness `tst_python` (resumo, botão,
criar uma vez, falha, esquecer ao trocar de workspace) e a faixa em
`tst_project_health`. Onze mutações mortas com o compilador calado (Rust 6,
QML 5); uma sobreviveu e virou caso de teste (uv sem python3).

**Uma armadilha de QML:** dentro de `onWorkspaceBuildSystemsChanged` o
binding de uma propriedade derivada (`isPython`) pode ainda não ter sido
reavaliado — o handler lia `false` e não perguntava o status. Os handlers
leem a função, não a propriedade.

```text
protocolo 0.98.0 — python.status, python.createEnvironment, event.python.finished
testes  649 Rust (+4), 28 harnesses (+tst_python); 134 metodos, 46 eventos,
        34 dominios; ferramentas conhecidas 61
gate    exercitacao: pyproject.toml no projeto; python.status; createEnvironment
        com a ferramenta REAL; status de novo -> .venv
proximo fatia 2: basedpyright com o interpretador + ruff como servidor
```

### 7.25 A cadeia Python, fatia 2 — o basedpyright com o interpretador do projeto, e o ruff, 2026-09-13

Sem esta fatia o basedpyright subiria com o Python do `PATH` e o completar
mentiria sobre os pacotes do `.venv`; e um `.py` não tinha formatar nem lint.
Três contratos existentes ganharam Python, sem método novo (0.99.0).

```text
lsp/server.rs        ServerSpec.settings (JSON). Com settings, o core envia
                     workspace/didChangeConfiguration LOGO APOS o initialized e
                     responde ao workspace/configuration do servidor (LSP 3.17):
                     um valor por item, `python.analysis` navegado por pontos,
                     secao inexistente = null, item sem secao = tudo. Spec de
                     Python: basedpyright-langserver --stdio, languageId python
lsp/session.rs       use_server_settings, is_running
handlers/python.rs   configure_python_lsp(root): settings a partir do
                     interpretador que python::env resolve (o MESMO do
                     python.status e do index.context); comando = o
                     basedpyright DETECTADO (~/.local/bin) ou o nome nu; sem
                     interpretador nao se empurra nada. on_python_environment_
                     finished(success): reconfigura e, se o servidor esta' vivo,
                     reinicia (event.lsp.restarted) — so' no sucesso
handlers/workspace   activate_workspace chama configure_python_lsp depois do
                     clangd
format.rs            FormatterKind::Ruff (.py/.pyi): `ruff format
                     --stdin-filename <arquivo>` (sem `-`: com --stdin-filename
                     e sem caminhos o ruff le stdin — medido no 0.16.4);
                     formatter_command recebe o PROGRAMA — o handler passa o
                     binario detectado, e so' cai no nome nu sem deteccao
build/mod.rs         run_quality(.., ruff: Option<&Path>, ..): braco Python =
                     `ruff check --output-format concise --no-fix [--select] .`
                     no root; ruff_profile_args: o perfil de rigor SO' quando o
                     projeto nao declara (ruff.toml, .ruff.toml, [tool.ruff*]);
                     BuildError::ToolMissing{tool,hint} — o erro nomeia o ruff e
                     o passo do painel de instalacao
build/parse.rs       parse_ruff_concise_line: `arquivo:l:c: CODIGO [*] msg`;
                     E9xx/SyntaxError = erro, resto aviso, `[*]` = "(corrigivel:
                     ruff check --fix)"; resumo e ruido nao viram diagnostico
handlers/build.rs    quality.run aceita RustCargo | Python; passa o ruff detectado
scripts/fake_lsp_    depois do initialized pergunta workspace/configuration (id
server.py            9001) com 5 itens: python, python.analysis, inexistente,
                     sem secao, secao vazia
```

**Medido:** ruff 0.16.4 em `~/.local/bin` (pipx); basedpyright ausente nesta
máquina — o wire foi provado com o servidor falso, e o binário detectado com
um `basedpyright-langserver` falso na pasta de busca do detector que grava os
argumentos (`--stdio`) e encaminha para o servidor falso. A exercitação do
gate formata um buffer com o **ruff real** (`changed: true`) e o
`quality.run` de Python emite o `F401` do `tools/gera.py` como
`event.quality.diagnostic`.

**Provado (26 mutações mortas com o compilador calado; 9 sobreviveram à
primeira rodada e cada uma virou caso de teste):** no wire, `initialized` antes do `didChangeConfiguration`,
`pythonPath` = `.venv/bin/python` do workspace, `diagnosticMode:
openFilesOnly` nas duas seções, a resposta ao `workspace/configuration`
(objeto, sub-seção, `null`, tudo ×2), `didOpen` com `languageId: python`, o
`event.python.finished` falho NÃO reinicia (nenhum `event.lsp.restarted` em
300 ms) e o bem-sucedido reinicia; sem interpretador nenhuma configuração sai
(nem para o Rust). No formatar, um `ruff` falso grava argumentos, cwd e stdin:
`format --stdin-filename <root>/pacote/app.py`, cwd = root, o buffer por
stdin — e o ruff real devolve `def f(a, b):`. Na qualidade, sem ruff o erro
diz `ruff` e `pipx`; com um `ruff` falso: `check --output-format concise
--no-fix --select E,F,W,I,UP,B,N .` (perfil `strict` em
`.kinein/settings.json`, projeto sem regras), cwd = root, dois diagnósticos
(`F401` aviso corrigível em 1:8; `E999` erro), `success: false`. O parser:
sete formas de ruído (resumo, `All checks passed!`, aviso de configuração,
linha sem número, código minúsculo, código sem dígito, sem arquivo) e
`ruff_profile_args` com `pyproject` sem e com `[tool.ruff.lint]`, `ruff.toml`
e `.ruff.toml` sozinhos. As sobreviventes que viraram teste: código sem
dígito; `--select` do perfil e cwd do ruff no despacho; item sem seção e seção
vazia do `workspace/configuration`; `diagnosticMode`; `--stdio`; falha do
ambiente que não reinicia; `--stdin-filename` com o arquivo (não o root).

**Uma descoberta do mutante:** `command.arg("-")` no ruff era redundante — o
ruff lê stdin sempre que há `--stdin-filename` e nenhum caminho. Saiu.

**Dívida:** o ruff como *servidor* (code actions no Alt+Enter) precisa de dois
servidores por linguagem em `lsp/session.rs`. Registrada no §4.

```text
protocolo 0.99.0 — settings no ServerSpec; ruff no format.capabilities; Python no quality.run
testes  658 Rust (+9), 28 harnesses; 134 metodos, 46 eventos, 34 dominios
gate    exercitacao: format.text de um .py com o ruff REAL; quality.run de Python
        com o F401 do gera.py como diagnostico
proximo fatia 3: run (python arquivo / -m / uv run) e pytest com a saida no painel
```

### 7.26 A cadeia Python, fatia 3 — executar e testar com o interpretador do projeto, e a saída dos testes na tela, 2026-09-13

"Executar" num `.py` e o botão Executar não existiam para Python; o `test.run`
recusava o tipo; e a saída bruta de qualquer runner de testes ia para um sinal
sem ouvinte (41 A6). Protocolo 0.100.0.

```text
python/run.rs        PythonLauncher: Uv(uv) quando o projeto tem uv.lock E o uv
                     foi detectado, senao Interpreter(o de python::env) — com
                     program()/display()/shell_command()/script_display().
                     entry_point(root) por EVIDENCIA: main.py > app.py >
                     __main__.py na raiz; UM pacote com __main__.py (raiz, depois
                     src/) -> `-m pacote`, dois = ambiguidade = None; script de
                     [project.scripts] SO' se instalado em .venv/bin. default_
                     command() erra dizendo o que procurou
handlers/run.rs      python_launcher(root) (sem medir versao — custo do status);
                     run.script de .py -> start_program(interpretador, [arquivo])
                     sem shell, cwd no root; run.start sem comando em projeto
                     Python -> python::run::default_command
test.rs              run_tests(.., python: Option<&PythonLauncher>, ..): braco
                     Python = `python -m pytest -v [-k filtro]` no root;
                     parse_pytest_case (nome antes do estado, com `::`; o estado
                     e' a ULTIMA palavra-chave; resumo curto nao conta);
                     TestError::ToolMissing{tool,hint}: sem interpretador, e sem
                     o modulo pytest naquele ambiente ("No module named pytest"
                     do proprio Python) — o passo para instalar NELE
handlers/build.rs    test.run aceita RustCargo | Cmake | Python; o lancador
                     resolvido fora do job
core_client          testFinished ganhou `error` (o emit_run_error do core)
JobsController       testOutputModel (teto 2000), handleTestStarted/Output;
                     handleTestFinished com `error` = resumo "nao rodou: …"
JobsEventRouter      onTestStarted/onTestOutput ligados (o A6)
TestsPanel           a metade de baixo e' a saida bruta quando ha' linhas
ProjectTree/Explorer .py e' executavel ("Executar" no menu e o icone da linha)
```

**Medido:** python3 3.14.7; a exercitação do gate roda `tools/gera.py` com o
`.venv` recém-criado (`.venv/bin/python 'tools/gera.py'`, a saída do programa
por `event.run.output`) e pede `test.run` de Python — sem pytest no `.venv`
novo, o `event.test.finished` traz o `error` que nomeia o pytest e o `uv add
--dev pytest`: a string `No module named pytest` que a detecção espera é a que
o Python 3.14 real escreve. uv não está nesta máquina — o caminho `uv run`
foi provado com um `uv` falso que grava os argumentos.

**Provado (20 mutações mortas com o compilador calado, 16 Rust + 4 QML; 4
sobreviveram à primeira rodada e viraram teste ou código a menos):** o
`uv.lock` como condição do uv; pasta oculta não é pacote; dois pacotes =
`None`; raiz antes de `src/`; só `[project.scripts]` (não outra tabela) e só
instalado; `-v`, `-k`, cwd do pytest; XPASS = passou; o estado é o último
(`test_p[caso FAILED antes] PASSED`); o `::` obrigatório (o print de um
programa sob teste com `-s` não é caso); executar não mede versão (o falso
registra cada chamada — depois de esperar o `event.index.finished`, que mede);
o `.py` antes do `script_interpreter`; o lançador chega ao `test.run`. Uma
sobrevivente revelou código redundante: a checagem de fronteira depois do
estado do pytest saiu. No QML: `return` do `error`, limpar a saída ao começar,
o `$ ` do comando, o `.py` da lista.

**Uma armadilha de teste:** três testes que compartilhavam a pasta do
workspace (`{pid}-run-python`) corriam em paralelo e um apagava o `.venv` do
outro — cada teste tem a sua pasta agora. E o índice do `workspace.open` mede
a versão do Python em thread própria: a asserção "executar não mede versão" só
vale depois do `event.index.finished`.

```text
protocolo 0.100.0 — .py no run.script; Python no run.start e no test.run; error no event.test.finished
testes  669 Rust (+11), 29 harnesses (+tst_tests_output); 134 metodos, 46 eventos, 34 dominios
gate    exercitacao: run.script de um .py com o .venv REAL (saida do programa);
        test.run de Python (pytest ausente no ambiente -> o passo para instalar nele)
proximo fatia 4: debugpy como adaptador DAP (`python -m debugpy.adapter`), dap/target.rs
        para Python
```

### 7.27 A cadeia Python, fatia 4 — depurar com o debugpy do interpretador do projeto, 2026-09-13

O esboço do 41 dizia "debugpy como candidato do papel `debugAdapter`, como o
gdb entrou". A medição corrigiu: o debugpy **não é um binário** — é um
módulo do interpretador, e o adaptador é `<interpretador> -m debugpy.adapter`.
Um candidato do kit apontaria para um executável que não existe; o que existe
é o Python do projeto, o mesmo do status, do índice, do basedpyright, do run e
do pytest. Protocolo 0.101.0.

```text
dap/adapter.rs       DEBUGPY: argumentos `-m debugpy.adapter`; o launch e' o do
                     desktop (program, cwd) — o `console` ficou de FORA: sem
                     supportsRunInTerminalRequest no initialize o debugpy cai no
                     internalConsole sozinho (a mutacao que tirou o campo
                     sobreviveu contra o debugpy real; campo redundante saiu)
dap/target.rs        Python -> o ponto de entrada do Executar como ARQUIVO
                     (main.py/app.py/__main__.py, script de [project.scripts]
                     instalado); pacote (-m) nao e' arquivo: o erro aponta o
                     __main__.py
python/debug.rs      debugpy_available(interp): `-I -c "import debugpy"` com
                     prazo de 10 s — e' o que o adaptador vai ver; o erro nomeia
                     o interpretador e o passo (uv add --dev debugpy /
                     .venv/bin/python -m pip install debugpy)
handlers/debug.rs    alvo .py (qualquer workspace) -> interpretador do projeto
                     (sem medir versao) + sonda + AdapterChoice{debugpy, path};
                     DebugError::MissingAdapterModule -> TOOL_NOT_FOUND com a
                     mensagem inteira (a de MissingAdapter fala de lldb-dap)
UI                   "Depurar" no menu da arvore para .py (ProjectTreeController.
                     isDebuggableScript = runnable && .py — o mesmo `kind` de
                     sempre, a catraca de duplicacao pegou a copia);
                     DebugController.startDebugProgram(program)
gate                 scripts/verificar-python-debug.sh (+ verificar_python_debug.py):
                     o ciclo pelo core real, com KINEIN_PYTHON_DEBUGPY ou o
                     python3 que importa debugpy; sem nenhum, "nao provado"
```

**Medido (debugpy 1.8.21 num venv temporário, Python 3.14.7):** o adaptador
manda `initialized` só depois do `launch` (a ordem que a sessão já esperava
do GDB); telemetria vem como `output` de categoria `telemetry` (o leitor já a
ignorava); `stopped { reason: breakpoint }`, `stackTrace` com `soma` no topo,
`scopes` Locals/Globals, `variables` a=2 b=3, `evaluate a + b` = 5, a saída
do programa por `output` stdout, `exited { exitCode: 0 }`, `terminated`; e o
adaptador **não sai** no `disconnect` — o `Drop` da sessão o mata. Ciclo
inteiro pelo core: 1,4 s. O pytest real (9.1.1) no mesmo venv confirmou as
formas que o parser da fatia 3 espera (PASSED/SKIPPED (motivo)/XFAIL/FAILED
e o resumo `FAILED arquivo::caso - …`).

**Provado (11 mutações mortas, 8 Rust + 3 QML; 2 contra o debugpy REAL):**
trocar o id do adaptador ou o interpretador → "o adapter nao respondeu a
`initialize`" no ciclo real; `-I` da sonda; o script instalado em `.venv/bin`
e o arquivo sob o root; `.py` em qualquer caixa e só ele (`.pyi`, `.pyc`, `.rs`
não); no QML, a guarda de workspace, fechar o menu, e o programa que chega
inteiro ao `startRequested`. Duas sobreviveram: o `console` (redundante,
saiu) e uma mutação equivalente (`.and(Ok(()))`, sem mudança de
comportamento). Os desvios que precedem o adaptador (sem interpretador; sem o
módulo; a sonda não sobe o adaptador) são teste de despacho com um python
falso.

**Uma armadilha de symlink:** o `python` de um venv é um symlink para o do
sistema; `Path.resolve()` no driver do gate jogou o venv fora (e o debugpy
com ele) — `absolute()` mantém o link, e o `$VIRTUAL_ENV` que o driver passa
ao core é lido, nunca escrito, pelo core.

```text
protocolo 0.101.0 — alvo .py no debug.start -> debugpy do interpretador; MissingAdapterModule
testes  675 Rust (+6), 30 harnesses (+tst_debug_python); gate 23 verificacoes
        (+verificar-python-debug.sh)
proximo fatia 5: mpremote (MicroPython) no terminal e o modulo nativo (pybind11/
        nanobind/PyO3/maturin) no project.model
```

### 7.28 A cadeia Python, fatia 5 — MicroPython pelo mpremote e o módulo nativo, 2026-09-13

A última fatia da cadeia. Duas coisas que a fundação já sabia reconhecer e
não sabia USAR: um projeto MicroPython (o `project.model` o via desde
2026-09-12) e um projeto Python com extensão em C++/Rust. Protocolo 0.102.0.

```text
serial/monitor.rs    escolher(toolchain, micropython): projeto MicroPython + mpremote
                     detectado -> `mpremote connect <dev> repl` (o REPL E' o monitor
                     de um firmware MicroPython); fixado pelo autor vence; fora de
                     MicroPython o mpremote e' o ULTIMO candidato e nunca vence so'
toolchain/catalog    mpremote como 5o candidato de serialMonitor
project/mod.rs       e_micropython(root): a MESMA evidencia do project.model
python/run.rs        PythonLauncher::Mpremote{program, device}: `[connect <porta>]
                     run` como prefixo; display "mpremote connect /dev/ttyUSB0 run";
                     default_command recusa `-m pacote` na placa
handlers/run.rs      python_launcher(root, device): MicroPython -> mpremote ou o erro
                     "pipx install mpremote" (NUNCA o Python do desktop — nao ha'
                     pinos para `import machine`); python_host_launcher para o
                     pytest; run.start sem comando roda main.py na placa mesmo sem
                     pyproject (tipo Unknown)
protocolo            RunScriptParams.device?; PythonStatus.nativeModule?
                     (PythonNativeModule{kind, tool, evidence[], buildHint})
python/native.rs     detect(root): pyproject [build-system] requires (maturin |
                     scikit-build-core | setuptools-rust | pybind11 | nanobind),
                     [tool.maturin], Cargo.toml pyo3, CMakeLists pybind11/nanobind,
                     setup.py; buildHint = `maturin develop` / `pip install -e .`
UI                   PythonController.summary() ganha " · pybind11 (scikit-build-
                     core)"; nativeModuleLine()/nativeModuleBuildHint();
                     CoreClient.runScript(path, device = "") — campo ausente, nao vazio
```

**Medido:** mpremote 1.29.0 (`~/.local/bin`): `connect device next_command`,
`run [--follow] path`, `repl`; sem `connect` o mpremote usa a primeira porta
que acha (`--help`). O ESP32 da mesa não tem firmware MicroPython gravado
(41 P4/C5), então o `run` real na placa fica para quando ele tiver — o que
está provado é o comando que o core monta e o processo que ele sobe (um
`mpremote` falso grava os argumentos).

**Provado (11 mutações, 8 mortas com o compilador calado; 3 sobreviveram e
viraram código a menos):** `[tool.maturin]` como evidência; setuptools-rust como Rust; o `!fixado` do mpremote; os dois
`e_micropython` do run (script e botão); o pytest que fica no host num
projeto MicroPython (a mutação que trocava o lançador morreu por um teste
novo). As três sobreviventes eram os dois `break` do leitor de `requires`
(o próximo `[` já encerra a seção) e o braço `"Rust" => maturin` (Rust só
existe COM ferramenta) — saíram.

```text
protocolo 0.102.0 — device no run.script; nativeModule no python.status; mpremote no serialMonitor
testes  683 Rust (+8), 30 harnesses; 134 metodos, 46 eventos, 34 dominios
proximo o provedor de download de toolchain (integracoes/39 §5) — a fila do §4
```

### 7.29 O provedor de instalação de toolchain — nove releases pinados, o checksum antes do tar, 2026-09-13

O item de cima da fila depois da cadeia Python (`integracoes/39` §5): *"se o
usuário abrir um projeto de STM32 e a máquina estiver limpa, a IDE oferece um
botão"*. Com a regra que a decisão do autor exige — zero-config é DETECTAR +
UM CLIQUE com o comando visível, NUNCA download calado. Protocolo 0.103.0.

```text
toolchain/install/catalog.rs   nove Entradas PINADAS: Arm GNU 15.2.rel1 (arm-none-eabi,
                               aarch64-none-linux-gnu, arm-none-linux-gnueabihf), xPack
                               arm-none-eabi-gcc 15.2.1-1.1 e riscv-none-elf-gcc 15.2.0-1,
                               ATfE 23.1.0, Bootlin glibc stable 2026.08-1 (aarch64,
                               armv7-eabihf, riscv64-lp64d) — URL, tamanho (HEAD),
                               SHA-256 lido no .sha256asc/.sha/.sha256 da fonte em
                               2026-09-13, licenca, fonte com data
toolchain/install/mod.rs       install(): <id>/<versao>.part -> download por ureq (TLS
                               rustls; prazo de 60 s ENTRE bytes) com SHA-256 no caminho
                               -> digest conferido ANTES do tar -> `tar -xf
                               --strip-components=1 -C` (processo) -> exige bin/ ->
                               rename para <versao>; qualquer falha deixa so' o .part,
                               que a proxima tentativa apaga
tools/search_dirs.rs           install_root(home) e installed_bin_dirs(root): a pasta
                               da IDE e' enumerada A CADA busca (nao na construcao do
                               detector); ToolDetector.with_install_root para os testes
handlers/toolchain_install.rs  toolchain.installable (nao exige workspace; com um, a
                               familia do project.model marca `recommended`) e
                               toolchain.install (recusas ANTES da rede: id fora do
                               catalogo, ja' instalada, tar ausente; job com progresso
                               por ponto percentual; event.toolchain.installed)
lib.rs                         event.toolchain.installed com sucesso refaz o registro
                               de ferramentas: o toolchain.get seguinte ja' lista o
                               compilador novo como candidato
UI                             EmbeddedInstallView (catalogo recolhido por padrao;
                               label, versao, MiB, URL, sha256, licenca; recomendadas
                               primeiro; um botao por linha, UMA instalacao por vez);
                               EmbeddedAdapterView saiu do EmbeddedPanel para abrir
                               espaco (293/300); ToolchainController.install/
                               handleInstalled; core_client_toolchain.cpp
protocolo                      InstallableToolchain, ToolchainInstallableResult,
                               ToolchainInstallParams, ToolchainInstalledEvent
Cargo                          sha2 0.11 (ja' no lock via mongodb/postgres; +0 crates)
```

**Medido em 2026-09-13:** os nove arquivos de checksum baixados e lidos; a
Bootlin publica o `.sha256` ao LADO do tarball e não com o sufixo `.tar.xz.
sha256` (o primeiro chute deu 404 — a listagem do diretório mostrou o nome);
tamanhos de 91 a 433 MB. Não entram: Espressif (`idf_tools.py` é o instalador
oficial; a IDE já lê `~/.espressif/tools`), Zephyr SDK (`setup.sh` — importar
kit) e "latest".

**Provado (15 mutações, 13 mortas com o compilador calado; 2 sobreviveram e
viraram teste ou código a menos):** o download REAL pelo ureq contra um
servidor HTTP local, o SHA-256 REAL, o `tar` REAL com `--strip-components=1`
e o `bin/` no lugar que o detector procura; checksum errado = nada
desempacotado e os dois digests no erro; HTTP 404, cancelamento e `tar`
ausente sem pasta final nem `.part`; tarball sem `bin/` recusado (a
sobrevivente que virou teste); o `.part` apagado; a recusa de reinstalar;
`recommended` só pela família; `installed` pela pasta; o registro refeito
só no sucesso; a pasta da IDE lida a cada busca e depois do `PATH`. No QML:
uma instalação por vez, instalada não pede, recomendadas primeiro, o
catálogo pedido de novo no desfecho. A outra sobrevivente era o
`remove_file` do tarball dentro do `.part` que já cai inteiro — saiu.

```text
protocolo 0.103.0 — toolchain.installable, toolchain.install, event.toolchain.installed
testes  692 Rust (+9), 31 harnesses (+tst_toolchain_install); 136 metodos, 47 eventos
gate    exercitacao: toolchain.installable com o sha256 da Arm GNU 15.2.rel1
proximo seletor de pasta do sysroot; importar kit Yocto/Buildroot/Zephyr; ou o
        polimento da cadeia Python (40 §4) — a fila decide
```

### 7.30 O gerenciador que lê o disco — o sysroot com veredito, o kit importado do SDK, 2026-09-13

Os itens (b) e (d) do 42 §8: *"LER o sysroot e mostrar o que há"* e
*"IMPORTAR kit de SDK: Yocto pelo environment-setup, Buildroot pelo
output/host"*. Protocolo 0.104.0.

```text
toolchain/sysroot.rs       inspect(path): usr/include, usr/lib, lib, os usr/lib/<triple>
                           e lib/<triple> do multiarch, os .pc onde o pkg-config
                           procura, a libc (glibc por features.h, musl pela libc.so)
                           e um VEREDITO: utilizavel (com/sem .pc), so' headers, so'
                           bibliotecas, vazia (o sysroot de distro do Fedora) — com
                           o remedio
toolchain/import.rs        import(path): yocto (o environment-setup carregado pelo sh;
                           CC/CXX/GDB resolvidos por command -v no PATH que ele monta;
                           SDKTARGETSYSROOT; OEToolchainConfig.cmake no
                           OECORE_NATIVE_SYSROOT; TARGET_PREFIX), buildroot (a marca e'
                           share/buildroot/ — os tarballs da Bootlin entram por aqui),
                           toolchain-dir (bin/<triple>-gcc; sysroot por -print-sysroot
                           quando o gcc roda e a pasta existe, senao <triple>/libc ou
                           <triple>/sysroot). Propoe; nao grava
store/mod/handlers         Kit.toolchain_file; KitUpdate; setKit { toolchainFile } ("" limpa);
                           ToolchainResult.toolchainFile; cmake_arguments emite
                           -DCMAKE_TOOLCHAIN_FILE so' sem toolchainFile no PRESET (o do
                           preset vence)
handlers/toolchain_import  toolchain.inspectSysroot e toolchain.importKit (caminho
                           absoluto; nada reconhecido = INVALID_REQUEST que diz o que
                           procurou)
UI                         EmbeddedKitImportView: campo de caminho, "Ler sysroot",
                           "Importar kit", a proposta em linhas, "Aplicar proposta ao
                           kit" (um setKit; o chip fica); ToolchainController.
                           toolchainFile chega por sinal proprio (toolchainKitFileResolved)
                           e o applyKit do painel o PRESERVA; core_client_toolchain.cpp
```

**Não medido** contra um SDK Yocto ou uma árvore Buildroot reais — não há
nenhum nesta máquina (42 §8 item 5 já o dizia). O que está provado é o
contrato documentado: um `environment-setup` que exporta as variáveis do
sdk-manual e uma `output/host` com a forma do manual do Buildroot, ambos
como fixtures; e a exercitação do gate lê uma pasta real como sysroot
"vazia". Quando um SDK real aparecer, a fixture vira exercitação.

**Provado (23 mutações, 20 mortas com o compilador calado; 3 sobreviveram e
viraram teste ou código a menos):** o `toolchainFile` do preset vencendo o
do kit; `usr/include` + `lib` (sem `usr/lib`) ainda utilizável; `usr/lib/
python3.11` não é um triple; só `.pc` conta; `share/buildroot` como a marca
do Buildroot; dois `environment-setup` na pasta = recusa; o `-print-sysroot`
que aponta pasta inexistente cai na convenção da Arm; `TARGET_PREFIX` sem o
`-` final; no QML, o arquivo preservado pelo `applyKit` do painel, o chip
mantido pela proposta, a recusa do `importKit` como erro do painel. As
sobreviventes: o `count('-') >= 3` do `gcc_em` (redundante: `gcc` não termina
em `-gcc`) saiu; as outras duas viraram os testes acima.

**Duas correções de desenho no caminho:** `SysrootReport` com quatro `bool`
reprovou no clippy (`struct_excessive_bools`) — os três de pasta viraram
`folders { usrInclude, usrLib, lib }`, que também lê melhor no wire. E o
`toolchainFile` levou `toolchain/mod.rs` a 512 linhas: a catraca mandou
olhar, e o corte foi por responsabilidade — a escolha EXPRESSA em argumentos
(`cmake_arguments`, `clangd_args`, `binutils_prefix`, o sistema e o
processador do triple) saiu para `toolchain/arguments.rs`; resolver e
expressar são coisas diferentes.

```text
protocolo 0.104.0 — toolchain.inspectSysroot, toolchain.importKit, toolchainFile no kit
testes  700 Rust (+8), 32 harnesses (+tst_toolchain_import); 138 metodos, 47 eventos
gate    exercitacao: inspectSysroot de uma pasta real ("vazia para o compilador")
proximo o polimento da cadeia Python (40 §4) ou o P0 que falta — a fila decide
```

### 7.31 O Python aparece na IDE, o rail do autor, e o Docker sem projeto, 2026-09-13

O autor redirecionou a fila: *"vamos pela fila/polimento de Python por hora
[…] falta ter o reconhecimento de Python na IDE, e na UI/UX ter a parte do
Python ser incluído também"* — mais três pontos: o Docker *"aparenta não
estar dando certo"*, o banco de dados com ícone no rail acima de Containers,
e "Ferramentas" por último. Protocolo 0.105.0.

```text
Novo projeto     template `python` (workspace/create.rs): PEP 621, pytest em
                 [dev], ruff, main.py, o pacote NA RAIZ (layout plano: roda e
                 testa sem `pip install -e .`), tests/, .gitignore, README —
                 geracao interna, nada executado; chip "Python" no painel de
                 criar, com a previa do que nasce
barra de status  StatusBarProjectSummaries (indice, contexto, python) saiu da
                 WorkspaceStatusBar (298/300); "python: .venv · 3.14.7 ·
                 pybind11 (scikit-build-core)", em ambar quando ha' ⚠
arvore           .py/.pyi/.pyw com icone proprio (tree-file-python: o `>>>` do
                 REPL e o cursor — nao o logotipo, marca da PSF), no mesmo
                 desenho de documento dos icones C/C++/Rust
menu Build       "Testar com pytest" quando o projeto e' Python E outro sistema
                 (senao "Testes" ja' vai para o pytest); "Analise (ruff)" no
                 lugar de "Analise Cargo" num projeto sem Cargo
rail             ordem do autor: Projeto, Busca, Git, Build, Debug, BANCO DE
                 DADOS (glifo novo: o cilindro), Containers, Observabilidade,
                 Ferramentas por ultimo; o banco abre o DataSourceController
containers       Logs/Shell sem projeto aberto: o botao desabilita e diz que a
                 aba de terminal e' do projeto (o core recusava depois do clique)
C++              o kind `python` entra nos buildSystems de tolerancia (core
                 antigo) como cargo/cmake ja' entravam
```

**Medido:** o projeto Python gerado roda (`Ola, mundo!`), passa no pytest
(1 passed) e no `ruff check`/`ruff format --check` com um interpretador e um
pytest reais — e um primeiro rascunho do template tinha a indentação do
código-fonte Rust vazando para os arquivos gerados (uma string literal
multilinha); ficou visível ao ler o `pyproject.toml` gerado e ao rodá-lo, não
no teste, que só procurava substrings — a medição com a ferramenta real é o
que pegou. O Docker: o core responde `container.status/list/images` e um `action`
start/stop contra o Podman 5.8.4 daqui (o `postgres-dev` subiu e desceu pelo
core); `container.open` sem workspace recusava com "nenhum workspace
aberto" — é o único defeito encontrado sem o relato do sintoma.

```text
protocolo 0.105.0 — template `python` no workspace.createProject
testes  701 Rust (+1), 32 harnesses (tst_container ganhou o caso sem projeto)
proximo o polimento da cadeia Python (40 §4): ruff servidor, arvore do pytest,
        `-m`/attach no debugpy, run.capabilities — e o relato do Docker
```

### 7.32 A árvore de testes antes do primeiro run — e rodar só um, 2026-09-13

O que faltou do 41 B6: *"a árvore de testes que o JetBrains mostra antes do
primeiro run"*. Hoje os casos só apareciam quando rodavam. Protocolo 0.106.0.

```text
test/discover.rs       parse_pytest_collect_line / parse_cargo_list_line /
                       parse_ctest_list_line — cada linha do modo de listar do
                       runner vira TestCaseInfo { id, name, file? } com o id EXATO
                       que o mesmo runner aceita para rodar um so'
test/mod.rs            discover_tests(): `pytest --collect-only -q` (so' o stdout
                       conta; exit 5 = sem testes = lista vazia), `cargo test --
                       --list`, `ctest -N`; Selection { All | Filter | Exact }
                       substitui o `filter: Option<&str>` — Exact e' posicional no
                       pytest, `<nome> -- --exact` no cargo, `-R ^nome$` escapado
                       no ctest (regex_literal); os comandos viraram funcoes puras
                       (cargo_command/ctest_command) para o teste ler os
                       argumentos sem compilar; test.rs virou test/{mod,runners,
                       parse,discover}.rs — a catraca pegou o mod.rs em 585 linhas
                       e o corte foi por responsabilidade (rodar, ler, listar)
handlers/test_discover test.discover como JOB (o cargo compila para listar) ->
                       event.test.discovered; test.run ganhou testId (vence filter)
UI                     JobsController.discoveredModel/discoverTests/runOneTest;
                       handleTestCase pinta a linha da arvore quando o id bate
                       (senao caso solto); um run novo zera os status e mantem a
                       arvore; TestsPanel recebe o JobsController inteiro (o
                       BottomPanelHost e o ShellWorkspaceHost ENCOLHERAM: -3
                       escalares) e mostra "Listar testes", a arvore com ▶ por linha
```

**Medido:** pytest 9.1.1 (`--collect-only -q`: um node id por linha, o
resumo no fim, avisos no stderr; rodar um node id posicional roda só ele);
libtest (`nome: test`); ctest 4.x (`  Test #N: Nome`, `Total Tests: N`) — e
o ctest REAL no teste de unidade: `Core` exato não arrasta `CoreParsing` (a
regex é ancorada), `Broken.Case` roda só ele (o `.` escapado). A exercitação
lista o projeto CMake do gate pelo ctest real (zero testes declarados = lista
vazia com sucesso) e pede a árvore ao `.venv` novo, que não tem pytest e
responde o passo.

**Provado (14 mutações, 11 mortas com o compilador calado; 3 sobreviveram e
viraram teste):** o `--exact` do cargo e a âncora do ctest (viraram o teste
dos comandos puros), só o stdout conta e o exit 5 do pytest (viraram o teste
com o python falso), o `testId` vencendo o `filter`, o `: test` do libtest;
no QML, a linha da árvore pintada pelo id, os status zerados no run novo, a
listagem pedida uma vez. Uma sobrevivente era redundante (uma cláusula do
parser do pytest para uma linha que não existe) e saiu.

```text
protocolo 0.106.0 — test.discover, event.test.discovered, testId no test.run
testes  708 Rust (+7), 32 harnesses; 139 metodos, 48 eventos
gate    exercitacao: test.discover pelo ctest real e pelo .venv real
proximo o polimento da cadeia Python (40 §4): ruff servidor, `-m`/attach no debugpy,
        run.capabilities, a porta no Executar de MicroPython
```

### 7.33 O módulo como alvo de debug, e o catálogo do Executar publicado pelo core, 2026-09-13

Dois itens pequenos do polimento Python, fechados juntos. Protocolo 0.107.0.

```text
dap/target.rs        DebugTarget { Program(PathBuf) | Module(String) }; resolve_program
                     devolve o alvo: um pacote com __main__.py e' Module (antes era
                     recusa "aponte o __main__.py"); program_path() para quem precisa
                     de um ELF (servidor de debug, build.size, espflash)
dap/adapter.rs       start_request: Program -> `program`, Module -> `module` (o -m do
                     debugpy, medido no 1.8.21); o attach continua com `program`
dap/session.rs       um servidor de debug do kit com um Module e' recusa com motivo
handlers/debug.rs    e_um_alvo_python: .py OU modulo; DebugStartResult.program mostra
                     `-m pacote`
run.rs               SHELL_SCRIPTS e PYTHON_SCRIPTS como FONTE UNICA de script_interpreter,
                     do braco Python do run.script e de capabilities()
run.capabilities     { runnable: [sh, bash, zsh, py], debuggable: [py] }; sem workspace
UI                   ProjectTreeController.applyRunCapabilities (antes do catalogo nada
                     e' executavel); ProjectExplorer le runnableExtensions do controller
                     (a lista duplicada saiu); Main pede uma vez por conexao, como o
                     format.capabilities
gate                 verificar-python-debug.sh ganhou o 7o passo: sem main.py, o pacote
                     e' lancado como `-m pacote` — breakpoint em pacote/__init__.py:2,
                     x=21, `dobro 42`, exitCode 0 — contra o debugpy REAL
```

**Provado (9 mutações, 8 mortas com o compilador calado; 1 sobreviveu e
virou teste):** `module` em vez de `program`; o `py` no catálogo e no
braço do `run.script`; `debuggable` vazio; a extensão sem caixa no QML (a
sobrevivente: `APP.PY` virou caso); sem catálogo nada é executável (o
harness funcional do shell agora começa sem catálogo e o aplica no meio).
Uma falsa "compilador pegou" no meio: um `use` que faltava no módulo de
testes do `run.rs` mascarou três mutações como erro de compilação até o
import ser corrigido — a classificação só vale com a suíte compilando.

```text
protocolo 0.107.0 — run.capabilities; DebugTarget::Module no debug.start
testes  709 Rust (+1), 32 harnesses; 140 metodos, 48 eventos
gate    python-debug: o 7o passo (-m pacote) contra o debugpy real
proximo o polimento da cadeia Python (40 §4): ruff servidor, attach no debugpy,
        a porta no Executar de MicroPython
```

### 7.34 O "sintoma do Docker": o core estava certo, a tela prometia errado, 2026-09-13

O autor pediu *"prossiga para o sintoma do docker"* sem descrever o sintoma. A
regra zero, então, foi **reproduzir o que ele viu** sem ter o clique dele: o
core real (`target/release/kinein-core`) respondeu `container.status/list/
images` pelo stdio contra o Podman 5.8.4 (motor, 5 containers, 12 imagens,
cada comando medido em <0,2 s), e o **`ContainerPanelHost` real** foi
instanciado offscreen com essas respostas reais e fotografado
(`grabToImage`). A foto mostrou o defeito que nenhum gate via:

```text
o que a foto mostrou          "compose up" — o UNICO botao primario do painel — ACESO
                              em ambar sem projeto aberto. Estava `enabled: false`;
                              o KvButton desligado so' perdia 28% de opacidade e
                              continuava vestindo o acento. Clique = nada. A leitura
                              humana e' "o Docker nao funciona"
o que a foto nao mostrou      com projeto aberto o botao acendia SEM arquivo de
mas o codigo dizia            compose: o job `podman compose up -d` so' podia falhar
                              (medido: exit 255, "no compose.yaml, docker-compose.yml
                              or container-compose.yml file found")
a mesma classe, na fileira    KvIconButton: `iconColor` sobrescrevia o `disabled` do
                              KvIcon — "Logs"/"Shell" sem projeto (desligados desde
                              §7.31) pareciam tao clicaveis quanto "Iniciar"
```

```text
container/mod.rs     COMPOSE_FILES na ordem do podman-compose 1.6.0 (COMPOSE_DEFAULT_LS,
                     lido do fonte; os *.override.* fora); compose_file_in(root);
                     status(engine, workspace_root) preenche composeFile
handlers/container   container.compose sem `file` num projeto sem arquivo recusa com
                     INVALID_PARAMS ANTES do job, dizendo o que criar
protocolo 0.108.0    ContainerStatus.composeFile?
KvButton             `accented = primary && enabled`: desligado e' um botao comum e
KvIconButton         apagado (fundo surface1, borda, texto e icone textDisabled);
                     nem o perigoso veste o vermelho desligado
ContainerController  composeTool/composeFile/canCompose/composeSummary — a linha do
                     motor diz "abra um projeto" ou "o projeto nao tem compose.yaml"
ContainerPanel       compose up/down acendem so' com canCompose
harness              tst_kvbutton_states (o estado visual dos dois botoes) e
                     tst_container_panel (o PAINEL real sobre o controller real — o
                     painel "burro" nunca era instanciado, e foi nele que o defeito
                     morava); tst_container ganhou os 4 estados do compose;
                     verificar-qml-logica copia os .js do modulo para o espelho
                     (KvIcon importa KvIconGlyphs.js — sem isso nenhum harness podia
                     instanciar um botao com icone)
```

**Provado (16 mutações, 14 mortas com o compilador calado; 2 sobreviventes):**
Rust — `is_file`→`exists`, a ordem `compose.yaml`/`compose.yml`, cada par de
nomes removido (o `[&str; 8]` virou `&[&str]` para o compilador se calar),
a recusa invertida, `composeFile` sempre `None`. QML — cada `accented` de
volta a `primary`, `canCompose` sem o arquivo e sem `reachable`, a frase do
"abra um projeto", o `enabled` de cada botão de compose, a `composeSummary`
trocada por texto fixo. Os dois sobreviventes: um virou teste (o ícone do
primário desligado — o harness não tinha botão com ícone) e um revelou
redundância (`accented` no segundo ramo do `iconColor` do KvIconButton, já
coberto pelo `!enabled` antes dele — voltou a `primary`).

**O que continua sem medida:** o clique do autor. Se o que ele viu foi outra
coisa — o ícone do rail, a lista vazia, uma mensagem — é sintoma novo, e a
entrada do §4 pede a descrição.

```text
protocolo 0.108.0 — ContainerStatus.composeFile; container.compose recusa sem arquivo
testes  711 Rust (+2), 34 harnesses (+2); 140 metodos, 48 eventos
decisao UX/UI/HUD e' etapa PROPRIA depois do backend impecavel (§5)
proximo o polimento da cadeia Python (40 §4): ruff servidor, attach no debugpy,
        a porta no Executar de MicroPython; depois o bloco A de embarcados
```

### 7.35 A passada de sincronização de documentação, 2026-09-13

Pedido do autor, no fim da tarde: *"atualize/sincronize a documentação do
projeto para refletir o estado atual de tudo feito e para não haver perda de
contexto"* — e ele a colocou **antes** da etapa de UX/UI/HUD. A regra zero
valeu para os documentos: cada afirmação de estado foi conferida no código
antes de ser reescrita.

```text
manual.md            ainda tinha a §10 "Simulacao fisica e matematica" (FORA do
                     produto desde 2026-09-12) e a frase "IDE para ... e
                     simulacao"; nao tinha Embarcados, Containers nem Python.
                     Agora: §9 com Embarcados (Ctrl+Alt+M) e Containers
                     (Ctrl+Alt+W), §10 Python (ambiente, linguagem, executar,
                     testar, depurar, modulo nativo, MicroPython), a arvore de
                     testes na §4, debugpy/gdb -i dap na §4.1, o rail na ordem
                     do autor, os 7 atalhos de Ambiente na tabela
README.md (raiz)     "para C, C++ e Rust" -> "C, C++, Rust e Python — no desktop
                     e em sistemas embarcados"; bullets de Python e toolchains
roadmaps/29          cabecalho: a coluna Python da §1 FECHOU (o que cada linha
                     virou); a tabela fica como a medicao de 2026-08-29
integracoes/37       cabecalho: as candidatas foram ADOTADAS em 2026-09-04; o
                     que falta; a experiencia "a DataGrip" e' a etapa de UX
roadmaps/41 §5       B7 FEITO (setup Python ja' estava no catalogo desde
                     0.98.0 — a fila mentia); B8 e F3 com o estado de hoje
roadmaps/42 §8       item 1 (b)(c) FEITOS (o .venv de um clique; o comando no
                     terminal); item 5: (0)(b)(c)(d) FEITOS em 0.103/0.104, o
                     "falta" reescrito (pasta nativa, Zephyr SDK, medir Yocto/
                     Buildroot reais, toolchain file gerado)
40 §8                REMEDIDO: test.output/test.started chegam a tela desde
                     §7.26; probe.list tem consumidor desde §7.8; o resto
                     continua e virou entrada do §4
40 §4                + a varredura restante; + esta passada como ITEM DA FILA
leitura-tecnica      gate 23 (era "dezenove" de 2026-09-10); container e python
                     com 0.105–0.108
GUIAIA §5.9c         KvButton/KvIconButton, StatusBarProjectSummaries,
                     TestsPanel/discover, run.capabilities, os harnesses novos
build/comandos       o passo verificar-python-debug.sh (e como ele e' provado)
prompt de retomada   RETOMADA_2026-09-13-noite.md, consolidado: as DUAS etapas
                     na ordem, o que sobrou, a mesa e a maquina (inclusive o
                     AppImage de 2026-09-03 no menu), as decisoes
```

**O que a passada revelou de método:** a fila do 41 dizia B7 pendente e ele
estava feito há um dia; o manual anunciava um painel que não existe mais. Os
dois são a mesma falha — documento atualizado *por domínio tocado*, nunca
*por leitura completa*. Por isso a sincronização entrou no §4 como item da
fila, a ser feito antes de cada etapa nova, e não como gesto implícito de
cada commit (que continua valendo para o que o commit toca).

### 7.36 Ruff como segundo LSP — retomada e prova, 2026-09-15

A implementação já estava no checkout. A retomada preservou o trabalho
existente: `LspManager`, sincronização, aba Problemas, lista do Alt+Enter e
transação de workspace edit continuam sendo os mesmos caminhos.

- `lsp/registry.rs` guarda os servidores principais e companheiros;
  `handlers/python.rs` registra `ruff server` quando o detector encontra Ruff.
- `lsp/sync.rs` envia documentos aos dois servidores e mantém suas versões
  separadas; `diagnostics_merge.rs` combina os diagnósticos por arquivo.
- `lsp/manager.rs` guarda a origem de cada ação. O preview em
  `handlers/lsp/edicao.rs` valida `documentChanges` contra a versão desse
  servidor, inclusive quando hover/completion avançaram só a do principal.
- `runtime/services.rs` concentra a habilitação dos serviços antes presente
  em `lib.rs`; o transporte continua em `lsp/server.rs`.

**Prova em 2026-09-15:** seis testes de integração `lsp_companion` aprovados;
720 testes Rust do workspace aprovados, Clippy estrito e auditoria de
dependências aprovados. Com **Ruff 0.16.7 e basedpyright 1.40.1 reais** em
ambiente temporário, o core publicou juntos `reportAssignmentType`, F401 e
I001, retornou as ações dos dois servidores e aplicou a remoção de import
inutilizado após duas consultas de hover. O preview não escreveu no disco;
`lsp.workspaceEdit.apply` realizou a correção.

O protocolo continua `0.108.0`: não houve método nem campo IPC novo. O
aceite visual do gesto no editor não foi realizado nesta prova por stdio.

**Gate desta retomada (2026-09-15): não está verde.** A UI debug compilou
com `cmake --build --preset debug-strict --parallel 2` (Clang 18.1.3,
Qt 6.4.2). O gate parou em `verificar-cpp.sh`: o Clang-Tidy apontou
`NewDeleteLeaks` na chamada a `QMetaObject::invokeMethod` de
`ui/src/typing_perf_harness.cpp:157` e `NewDelete` na atribuição de
`QPointer` de `ui/src/window_chrome_controller.cpp:52`, com os diagnósticos
emitidos nos headers do Qt. Esses arquivos não foram alterados nesta retomada;
a causa dos achados ainda precisa de investigação, sem supressão dos checks.

A verificação QML separada também não passou: o script não encontrou o
response file de qmllint esperado, mesmo após esse build. O build release e
os checks posteriores à falha C++ não foram concluídos pelo gate. Em
separado, arquitetura, veracidade da documentação e links passaram; nove
links históricos ausentes no índice foram corrigidos (229 links relativos
válidos). Logs locais: `/tmp/kinein-codex-ruff-gate.log`,
`/tmp/kinein-codex-ruff-ui-build.log` e `/tmp/kinein-codex-ruff-qml.log`.

Referências consultadas em 2026-09-15: [configuração do Ruff](https://docs.astral.sh/ruff/editors/setup/),
[ações do servidor](https://docs.astral.sh/ruff/editors/features/) e
[integração Python do Zed, revisão `main` consultada nessa data](https://github.com/zed-industries/zed/blob/main/crates/languages/src/python.rs).
As lições são separar análise de tipos e lint, manter a origem das ações e
reutilizar o cliente LSP. Nenhum código dessas referências foi transplantado.

**Próximo:** attach do debugpy, seguido pelos demais itens da §4.1.

## 8. A VARREDURA de 2026-09-10 — o que está entregue e não chega à tela

Feita a pedido do autor, antes do pente-fino. **Ela procura uma classe só, e é a
que já bateu duas vezes nesta etapa:** algo que o core calcula, a ponte
transporta, e nenhuma tela mostra. Foi assim que o motor vetorial ficou um dia
sem porta, e foi assim que a coluna `exato` respondeu por outra equação.

**Os comandos que a reproduzem** — nenhuma linha abaixo pede confiança:

```bash
# metodos IPC roteados no core que o C++ nunca pede
# sinais do CoreClient que ninguem escuta (descontando NOTIFY de Q_PROPERTY)
# Q_INVOKABLE que o QML nunca chama
# sinais QML declarados sem tratador
```

> **Remedido em 2026-09-13 (fim de tarde).** Do que a varredura acusou:
> `event.test.output` e `event.test.started` **chegam à tela desde
> 2026-09-13** (§7.26, o A6 do 41: o painel Testes mostra o comando, a saída
> bruta e o `error` do runner); `probe.list` **tem consumidor desde
> 2026-09-11** (§7.8, o `EmbeddedRequestRouter`). Continuam como a varredura
> os deixou: `event.quality.started` é emitido pelo C++ e nenhum QML o
> escuta; `event.quality.output` continua descartado no próprio C++
> (`core_client_notifications.cpp`, `return true`); os três sinais de
> `environmentScan` sem ouvinte; `cmake.presets.list` e `job.list` sem
> cliente; os dois `*Accepted` do banco/Grafana sem ouvinte; os dois sinais
> QML do §8.3. O texto abaixo é o achado de 2026-09-10, mantido como
> registro.

> **Remedido e FECHADO em 2026-09-18 (pente-fino, §7.54).** As quatro
> medições viraram gate — `scripts/verificar-fiacao-ipc.sh`, a 24ª
> verificação, o "andar de cima" que a §8.4 pedia — e o que sobrava ganhou
> dono: `event.quality.output` e `qualityStarted` vão para o painel de Build
> (a análise escreve onde o build escreve); `environmentTool` faz a lista de
> ferramentas crescer enquanto o scan roda; `cmake.presets.list` alimenta o
> **seletor de preset** no kit (chips no painel de Embarcados — escolher um
> preset é ativar o kit dele); os dois sinais QML da §8.3 saíram; os cinco
> `*Accepted` e os dois `environmentScan*` ficam como exceções DITAS no gate
> (o desfecho chega por evento; a tela usa a Q_PROPERTY). `job.list` fica
> para a CLI/recuperação. O texto abaixo é o achado de 2026-09-10, mantido
> como registro.

### 8.1 O que é calculado e nunca aparece — prioridade real

```text
event.test.output      EMITIDO e DESCARTADO. O `JobsEventRouter` roteia
                       `onBuildOutput` e NAO roteia `onTestOutput`; o
                       `JobsController` tem `handleBuildOutput` e nao tem o par.
                       O `BuildPanel` tem `outputModel`; o `TestsPanel` tem
                       `casesModel` e `summary`, e nao tem saida nenhuma.

                       O ESTRAGO: num teste que falha, a razao esta' na SAIDA —
                       a mensagem do panico, a diferenca da assercao, o erro do
                       compilador. A tela mostra "falhou" e engole o porque.

                       Conferido: o core emite esta saida SO' em
                       `event.test.output` (handlers/build.rs:170); ela nao
                       chega por `event.job.output` nem por outro caminho.

event.test.started     idem, e o build mostra o dele: o comando que rodou nao
                       aparece nos testes

event.quality.started  idem, na analise

event.quality.output   pior: descartado no PROPRIO C++, com `return true` e sem
                       emitir sinal nenhum. Ele nao chega nem a ter dono

environmentScan        `Started`, `Tool` e `Finished` nao tem ouvinte. O painel
  (tres sinais)        de Ferramentas recebe a lista PRONTA; o progresso da
                       varredura, que existe, nao aparece
```

### 8.2 Superfície morta — o protocolo afirma o que ninguém consome

```text
cmake.presets.list    roteados no core e nao pedidos nem pela UI nem pela CLI
job.list
probe.list

dataSourceTestAccepted  o `jobId` do teste de conexao e da sonda do Grafana e'
grafanaProbeAccepted    emitido e ninguem escuta
```

**Não confundir com superfície morta:** `core.shutdown` e `workspace.status` não
são pedidos pelo C++ **porque quem os usa é a `kinein-cli`**. Ela é cliente do
protocolo tanto quanto a UI, e a varredura que olhasse só a UI os acusaria por
engano.

### 8.3 Menor

```text
GitPanel.branchMenuDismissRequested        sinal QML declarado sem tratador
TerminalScrollController.sessionChanged
```

### 8.4 O que a varredura NÃO cobre, e vale dizer

Ela mede **fiação**: o que existe e não está ligado. Ela não mede se a tela que
está ligada mostra a coisa certa, nem se o que aparece é legível — isso é o
pente-fino, e ele precisa da IDE **abrindo** (§4, prioridade 1).

**E a classe da §8.1 não tem gate.** O 19º pega componente QML entregue e não
instanciado; o 20º (2026-09-11) pega binário que compila e não abre; falta o
andar de cima — **evento que o core emite e nenhuma tela consome**. É o mesmo critério, uma camada acima, e os cinco achados de hoje
teriam saído dele automaticamente.


### 7.37 Verificadores no Ubuntu/Qt 6.4 — estado medido, 2026-09-16

Antes de avançar no attach, a retomada confirmou os impedimentos do ambiente
registrados na §7.36. **Ruff/rustls e o protocolo foram preservados**; esta
fatia corrigiu ferramentas de verificação e instrumentação existentes.

**Feito e medido:**

- `verificar-qml.sh` aceita builds sem `.rsp`: `verificar_qml.py` usa o alvo
  `kinein-vectis_qmllint_json` gerado pelo Qt, com os mesmos fontes/imports.
  Avisos, JSON vazio/inválido, relatório antigo e falha do build reprovam.
  Nove provas do runner passaram, incluindo o caminho `.rsp` com `-W 0`.
- `qml-qt6` instalado e incorporado ao bootstrap apt. O runner de lógica
  encontra `/usr/lib/qt6/bin/qml`; antes escolhia o wrapper Qt 5 sem runtime.
  **34 harnesses QML passaram** depois da correção.
- Dependências QML QtQuick/QtQuick.Window explícitas e ajustes equivalentes
  de URL/animação eliminaram cinco de sete warnings do qmllint 6.4.
- Harness de digitação: amostra vai à GUI por sinal tipado queued; o tempo
  continua capturado na render thread. Passou no Clang-Tidy. A localização
  do editor usa `domains.editorController`, a composição real do Main.qml.
  Erros antes de `app.exec()` agora encerram com exit 1, em vez de esperar
  o timeout externo. Prova de uma tecla inseriu um caractere e registrou uma
  amostra; o caso sem configuração encerrou com exit 1.
- WindowChrome: limpeza redundante de QPointer em `destroyed` removida;
  troca, destruição e notificações passaram sob ASan/UBSan.
- Debug e release recompilados; abertura offscreen validada. ShellCheck,
  fiação/propriedades/duplicação QML e catraca de arquitetura passaram.

**Não confundir esses resultados com gate completo verde. Restam:**

1. Clang-Tidy 18: `NewDelete` no header Qt, a partir da atribuição de
   `m_window` em `window_chrome_controller.cpp:47`. O teste de runtime não
   prova sozinho que o diagnóstico está errado; investigar a reprodução
   mínima e versões das ferramentas antes de qualquer exceção.
2. qmllint 6.4: dois `Unqualified access` em `StandardKey.Open/Save`, no
   `GlobalShortcuts.qml:25/31`. Exemplo mínimo executa no runtime Qt 6.4 e
   reprova no lint da mesma instalação. Nenhum warning foi desativado.
3. A medição de cinco teclas expira após a primeira amostra. A prova de uma
   tecla passou; a sequência completa continua pendente, inclusive o ciclo
   de espera por quietude entre amostras. Não publicar benchmark de digitação.
4. QEMU/GDB 15.1: escopo com registradores em vez de `contador`, conforme a
   medição anterior do notebook; não alterado nem revalidado nesta fatia.

Logs e provas reproduzíveis: `DocsPrivate/Codex/2026-09-16-compatibilidade-qt64.md`.
O ambiente de build atualizado está em `build/14-development-environment.md`.
A próxima funcionalidade permanece **debugpy attach** (§4.1), usando o DAP
existente; não abrir a etapa UX/UI/HUD.


### 7.38 Debugpy attach por TCP — 2026-09-16, protocolo 0.109.0

O próximo item da §4.1 foi implementado no fluxo DAP existente. `debug.start`
aceita `connect: { host, port }`, exclusivo com `program`. Conecta ao adaptador
já aberto por `debugpy --listen`/`debugpy.listen`; não inicia outro adaptador
nem exige interpretador local para attach. `dap/transport.rs` concentra a posse
de filho stdio/socket; sessão, framing, breakpoints e inspeção são compartilhados.

A aba Debug recebe host/porta e **Conectar ao Python**. A mesma intenção de
início cobre automático, arquivo e attach, com bloqueio durante o pedido e
retry após falha. `attached` no resultado/evento permite mostrar **Desconectar**.
O disconnect do attach usa `terminateDebuggee: false`. Fechar/trocar workspace
agora solta a sessão e limpa breakpoints nos donos centrais da transição.

**Medido:** 723 testes Rust e 35 harnesses QML passaram; Clippy, formatação C++,
Clang-Tidy dos dois arquivos C++ alterados e verificações de arquitetura,
fiação, propriedades, duplicação, alcance e transição de workspace passaram.
O gate real Python (debugpy 1.8.0, Python 3.12.3) preservou launch de arquivo e
módulo; attach provou breakpoint, stack, locais, evaluate e duas reconexões.
Desconectar pausado, desconectar rodando e fechar workspace mantiveram o mesmo
processo vivo, com heartbeat avançando. Build debug e primeiro frame passaram
(3151 ms offscreen, medida pontual). O lint QML continua com apenas os dois
avisos StandardKey preexistentes; nenhum aviso novo nos componentes do attach.

A prova real adicional (`scripts/verificar_python_attach.py`) reutiliza o
cliente Core do gate Python existente. Não há SSH, attach por PID ou
pathMappings nesta entrega; P6 continua pendente. O gate completo mantém as
falhas da §7.37; isso não foi declarado verde. Core/UI release recompilados;
primeiro frame release em 988 ms offscreen (uma amostra), e launcher validado
com os binários novos. Nenhum AppImage gerado/publicado nesta fatia.

**Próximo (na época):** porta selecionada no Executar de MicroPython — feita
na §7.39. Registro detalhado e prompt de continuidade: `DocsPrivate/Codex/`.

### 7.39 A porta escolhida no Executar de MicroPython — 2026-09-17, protocolo 0.110.0

O item seguinte da §4.1, fechado de ponta a ponta: **seleção → estado →
roteador → `device`**. O que a leitura do código mostrou antes de escrever —
e que o resumo anterior não dizia — é que **`run.start` não aceitava
`device`**: o botão Executar (toolbar, atalho, paleta), que num projeto
MicroPython roda o `main.py` na placa, iria sem porta mesmo depois de a tela
passá-la ao `run.script`. O contrato ganhou `run.start { device? }`
(0.110.0), exclusivo com `command` (`INVALID_PARAMS` nos dois juntos: a
porta é do lançador padrão; um comando digitado roda como foi escrito). Uma
configuração de execução ativa continua vencendo o lançador e não recebe a
porta. `device` presente e vazio/só espaço/com controle é recusado nos dois
métodos — antes `run.script { device: "" }` virava `mpremote connect '' run`.

Na tela: `EmbeddedController.selectedPort` (toggle; só porta da lista atual;
cai quando a lista nova não a tem — placa desplugada — e ao trocar de
workspace), um chip **Executar** por porta no `EmbeddedSerialView` (desligado
sem acesso R/W, como o botão do monitor) e a linha que diz o comando que vai
valer. `RuntimeController.serialDevice` recebe a escolha por binding em
`AppDomains` e a repassa em `runStartRequested(command, device)` /
`runScriptRequested(path, device)` — só com `command` vazio no primeiro.
`CoreClient::runStart(command, device)` espelha a regra. De quebra, a recusa
do core ao `run.script` (mpremote ausente, porta inválida) agora aparece na
sessão de execução que o gesto abriu — `RuntimeEventRouter` só repassava
`run.start|stdin|stop`, e a aba ficava muda.

**Medido em 2026-09-17:** 725 testes Rust (14 CLI, 1 config, 639 core, 71
protocol) e **36 harnesses QML** passaram — `tst_run_device.qml` é novo e
`tst_embedded.qml` ganhou o bloco da escolha; mutação `runScriptRequested(path,
"")` reprovou o harness (bitmask 10). Clippy, `cargo fmt`, `git diff --check`,
fiação, propriedades, alcance, duplicação, arquitetura (catraca intacta),
veracidade dos .md e links passaram. Prova contra o **core real**
(`scripts/verificar_micropython_porta.py`, agora na primeira metade do
`verificar-python-debug.sh`): `run.start {}` → `mpremote run main.py`;
`run.start { device }` e `run.script { device }` → `connect <porta> run
<arquivo>` nos argv que o mpremote falso ecoou; troca de porta entre
execuções; contradição e vazio recusados sem nenhum `event.run.*`. O ciclo
debugpy (launch, `-m`, três attaches) continuou verde na mesma rodada.
UI compilada com `dev-local` (g++ 15, zero avisos), **qmllint estrito
limpo** (os dois avisos StandardKey eram do Qt 6.4; com o Qt 6.10.2 desta
máquina não existem) e o binário abre (primeiro frame 248 ms offscreen, uma
amostra).

**O que NÃO foi provado, dito:** não há placa nesta máquina — a execução na
placa fica provada pelos argv, não pelo LED. O `mpremote` do PATH está
quebrado pela atualização do sistema (shim do pipx para um Python que não
existe mais; `pipx reinstall mpremote`). **Os presets clang não configuram
no Ubuntu 26.04 desta máquina**: `clang++` 21 escolhe a GCC 16
(`libgcc-16-dev` presente) e não há `libstdc++-16-dev` — `ld: cannot find
-lstdc++` no try-compile. Remédio: `sudo apt-get install libstdc++-16-dev` e
reconfigurar `linux-clang-debug-strict`/`release-hardened`. Até lá,
Clang-Tidy e o lint QML pelo alvo desse preset ficam sem prova, e o gate
completo continua não verde por impedimento de ambiente, não de código.

**Próximo (na época):** stderr dos processos filhos DAP/LSP — feito na §7.40.

### 7.40 O stderr dos filhos DAP/LSP — 2026-09-17 (tarde), protocolo 0.111.0

A lacuna da §4 fechada nos três pontos que a nomeavam. `stderr_tail.rs` é o
dono novo: uma thread por filho lê o stderr linha a linha (`read_until`, UTF-8
com perda, corte em 4 KiB com marcador), guarda as últimas 64 numa cauda e,
quando o dono quer, entrega cada linha a um coletor. Adaptador DAP
(`dap/transport.rs`): `Transport::Process { child, stderr }`, cada linha vira
`event.debug.output { category: "adapter" }` e a cauda entra no
`DebugError::Adapter` do handshake sob `--- stderr do adaptador ---` (o attach
TCP não tem: o processo é de outro dono). Servidor de debug (`dap/server.rs`):
só cauda, nas duas mensagens de `wait_for_port`. LSP (`lsp/server.rs`): o
handshake saiu para `handshake()`, cada linha vira **`event.lsp.log {
language, line }`** (evento 49), a cauda entra no `status: failed` (dentro do
erro) e no `status: exited` (`message`) — e um servidor que falha o
`initialize` agora é morto, não fica órfão, vivo e mudo. UI: `event.lsp.log`
e o `exited` com motivo vão para a aba IDE (`appendLog`); `adapter` veste
`stderr` no `DebugController`.

**Medido em 2026-09-17:** 734 testes Rust (+9: 4 do `stderr_tail`, 1 do
servidor de debug morto com `could not load kernel` na mensagem, 1 do
adaptador que morre com `ImportError` no erro E como eventos `adapter`, 3 do
LSP com o `fake_lsp_server.py --stderr N [--morre]`: log ao vivo com
`running`, cauda no `failed`, cauda no `exited` após `lsp.restart`); 36
harnesses QML (`tst_debug_python` cobre a categoria). Clippy, fmt,
`diff --check`, fiação, propriedades, alcance, duplicação, arquitetura, docs,
links e shell passaram. Verbosidade dos servidores REAIS desta máquina, 30 s
após abrir um arquivo deste repositório: rust-analyzer 4 linhas, clangd 66
(pico 18/s durante o índice), basedpyright+ruff 3 — sem lote, por número.
O gate Python real continuou verde (attach ×3, launch, `-m`).

**Os presets clang voltaram:** com `libstdc++-16-dev` instalado pelo autor,
`linux-clang-debug-strict` e `release-hardened` configuraram e compilaram
(Clang 21 + Qt 6.10.2) — depois de UMA correção: o moc do Qt 6.10 monta os
`QtMocHelpers` por CTAD e `-Wctad-maybe-unsupported -Werror` reprovava o
`.moc` que o `typing_perf_harness.cpp` inclui inline (o `mocs_compilation.cpp`
já tinha `-w`); a isenção ficou presa ao `#include` gerado, sob `#ifdef
__clang__`. Clang-Tidy dos arquivos alterados: limpo (o inteiro só apontou
os dois `#if defined` da própria correção, trocados por `#ifdef`). qmllint
estrito pelo alvo nativo: limpo — e `verificar-qml.sh` passou a exigir que o
`qmllint` candidato responda `--version` (o do PATH no Ubuntu 26.04 é um
wrapper Qt 5 quebrado). Ambos os binários abrem: 623 ms debug (sanitizers),
275 ms release; launcher vivo no smoke de 8 s.

**Não provado:** nenhum adaptador/servidor real foi posto a falhar de
propósito nesta máquina além dos falsos; o `adapter` no lldb-dap/probe-rs
reais fica para a exercitação com placa. **Próximo (na época):** E5 — feito
na §7.41.

### 7.41 E5 — a identidade Espressif pelo canal — 2026-09-17 (fim de tarde), protocolo 0.112.0

O A2 do bloco A do roadmap 41, na forma decidida em `integracoes/38` §6:
**`serial.identify { device, tool? }`** roda `esptool --port <device> --chip
auto --before default-reset --after hard-reset flash-id` como **job**
(`JobRisk::Medium`: reseta a placa, não escreve nela; cancelável — o cancel
mata o esptool) e emite **`event.serial.identified`** com `identity` (chip na
chave do IDF, descrição, features, cristal, USB mode, MAC, fabricante/ID e
tamanho da flash em texto e bytes), `target` (o kit que o chip SUGERE pelas
mesmas tabelas de família e motores do `project.model` — `project::alvo_do_chip`
reutiliza `familia`/`motores`, sem segunda tabela), `raw` sempre, `error`
quando não. `serial/identify.rs` é o dono: linha de comando, parser
TOLERANTE (o formato veio do código do esptool 5.4.0 instalado —
`__init__.py` e `cmds.py` —, não de uma placa), chave do chip
(`ESP8685/8686 → esp32c3`, `ESP8684 → esp32c2`) e a queda para o `flash_id`
da v4 quando a ferramenta diz "invalid choice"/"No such command". Recusas
ANTES de abrir a porta: nó inexistente (`INVALID_PARAMS`), sem R/W
(`INVALID_REQUEST` com a `hint` do `serial.list` — `serial::acesso_de` mede
com `access(2)`), sem esptool (`TOOL_NOT_FOUND`, `pipx install esptool`).
Prazo de 30 s com "não respondeu" distinto de "cancelada".

Na tela, sem propriedade de repasse nova: `EmbeddedIdentityController.qml` é
FILHO do `EmbeddedController` (`embeddedController.identity`; o pai estava em
335/400 e ganhou 9 linhas), com um pedido por vez, desfecho de outra porta
ignorado, resumo sem `undefined`, recusa como erro, `applyToKit` emitindo só
o chip; a fiação até o `ToolchainController.applyKit(sysroot, targetTriple,
chip)` mora no `AppEnvironmentDomains` (composição de dois donos). Um botão
"Identificar" (lupa) por porta com acesso no `EmbeddedSerialView` — o tooltip
diz que reseta a placa — e o resultado num `EmbeddedIdentityView.qml` novo:
chip · flash · MAC, as features, "sugere: chip … · gravar/monitor/debug" com
**Usar chip no kit**, o erro, e a saída crua só quando nenhum chip foi lido.
Ponte C++: `serialIdentify(device)`, `serialIdentifyStarted(jobId, command)`,
`serialIdentified(map)`.

**Medido em 2026-09-17:** 745 testes Rust (+11: 6 do parser/linha de comando,
5 do handler com esptool FALSO — v5 com a fixture, v4 com a queda, recusas sem
job nascer, falha com as últimas linhas, cancel matando um `sleep 30`); **37
harnesses QML** (`tst_embedded_identity` novo); clippy, fmt, `diff --check`,
fiação, propriedades (230 componentes), alcance (229), duplicação,
arquitetura (o pai em 335/400), docs e links passaram; qmllint estrito
limpo; Clang-Tidy dos dois .cpp alterados limpo; `debug-strict` compila e
abre (773 ms, sanitizers); 141 métodos e 50 eventos pelos comandos da §1.

**Não provado, dito:** nenhuma placa nesta máquina — a fixture é a forma do
código do esptool, não uma saída real; a primeira placa deve substituí-la e
registrar a diferença. `esptool v5.4.0` do PATH funciona (`pipx
reinstall-all` do autor) mas só foi visto pelo `--help`. `chip-id` não é
usado: o `flash-id` já imprime tudo o que o `chip-id` imprime mais a flash.
**Próximo (na época):** E4 — feito na §7.42.

### 7.42 E4 — gravar como CONFIGURAÇÃO DE EXECUÇÃO — 2026-09-17 (noite), protocolo 0.113.0

O A3 do bloco A, na forma que o autor decidiu em 2026-09-11 (`integracoes/38`
§6): **não é domínio novo.** `flash.rs` é um compositor PURO —
`runConfig.flashProposal { device?, engine?, flashSizeBytes? }` devolve
`{ name, command, engine, source, warnings }` a partir do que o
`project.model` já leu: a receita `flasher_args.json` (offsets, imagens,
mode/size/freq, before/after, stub, chip), o ELF/UF2/BIN mais novo, o
`target` — e da porta escolhida no painel (0.110.0). Motores: `esptool
write-flash` (a receita inteira; porta obrigatória; `--no-stub` quando a
receita diz; v5 — a linha salva é editável para a v4), `probe-rs download
--chip`, `picotool load -f -x`, `dfu-util -a 0 -s 0x08000000:leave -D` só
para STM32 (outra família: sem tabela, sem comando). Sem motor no modelo,
uma receita do IDF no build basta para sugerir o esptool. Cada peça tem uma
linha de evidência; os avisos são as imagens marcadas como cifradas e a
flash lida pelo E5 menor que a da receita. Rodar e salvar são os donos que
já existem: **Gravar agora** = `run.start { command }`; **Salvar como
"Gravar (esptool)"** = `runConfig.save` (vira a ativa, o botão Executar
grava). Na tela, `EmbeddedFlashController` FILHO do `EmbeddedController`
(`embeddedController.flash`, como o `identity`) e `EmbeddedFlashView`:
chips de motor (solto = o modelo decide), **Prévia**, a linha em mono, as
evidências, os avisos, os dois botões; a fiação até `RuntimeController.startRun`
e `RunConfigController.saveRunConfigRequested` mora no `AppDomains`.

**Medido em 2026-09-17:** 752 testes Rust (+7: 4 do compositor — as quatro
linhas, recusas e variantes, aspas em caminho com espaço; 3 pelo despacho
real com um workspace ESP-IDF, a receita na forma do template do IDF e um
esptool FALSO: proposta → `runConfig.save` → `run.start {}` entrega ao
"esptool" EXATAMENTE os argv da receita, `hello world.bin` inteiro; recusas
com o código certo e nada salvo; a flash menor vira aviso); **38 harnesses
QML** (`tst_embedded_flash` novo); clippy, fmt, `diff --check`, fiação,
propriedades (232), alcance (231), duplicação, arquitetura, docs, links,
shell; qmllint estrito limpo; Clang-Tidy dos dois .cpp alterados limpo;
`debug-strict` compila e abre (767 ms); 142 métodos, 50 eventos.

**Não provado, dito:** nenhuma placa — nenhum firmware foi escrito; o que
está provado é a LINHA que o motor recebe. `esptool` real só pelo `--help`;
`probe-rs`/`picotool` não estão nesta máquina (o `dfu-util 0.11` está).
**Próximo (na época):** E2 — feito na §7.43.

### 7.43 E2 — permissão por canal — 2026-09-17, protocolo 0.114.0

O A4 do bloco A, a fatia 4.3 redesenhada em `integracoes/38` §6: para CADA
canal, o que falta — MEDIDO — e o passo OFICIAL, datado. `serial/access.rs`:
**`serial.access { device? }`** → um canal `serial` por porta (o `access(2)`
do `serial.list`; o grupo dono × os grupos do processo, lidos de
`/proc/self/status`; a tag `uaccess` nas propriedades udev credita a ACL
quando o acesso vem sem grupo), um `modemManager` por porta quando o
`udevadm` respondeu (rodando ∧ candidata ∧ sem `ID_MM_DEVICE_IGNORE` = passo),
e um `probe` da máquina (regras `*probe-rs|openocd|stlink|jlink|cmsis*.rules`
em `/etc`, `/usr/lib` e `/lib`, pastas canônicas iguais uma vez; fora de
`/etc` é "a distro já instalou"). Passos: `sudo usermod -a -G <grupo> $USER` +
re-login (ESP-IDF *Establish Serial Connection*, conferida em 2026-09-17;
o Arch Wiki não respondeu ao fetch), regra `TAG+="uaccess"` quando o grupo
não é de usuários, `77-mm-kinein-<vid>-<pid>.rules` com a forma das regras
que o próprio ModemManager instala (numerada antes do `80-mm-candidate.rules`;
a página do freedesktop devolveu 403, a fonte é o pacote), e os três passos
do probe.rs *Probe Setup* (baixar `69-probe-rs.rules`, `udevadm control
--reload`, `udevadm trigger`, citados). Cada `fix` leva `sourceUrl` e
`checkedOn`. Na tela: botão **Permissões** no cabeçalho das portas,
`EmbeddedAccessController` filho (`embeddedController.access`),
`EmbeddedAccessView` com ✓/✗ por canal, o problema, cada passo com
**Escrever no terminal** — o comando vai para o terminal da IDE pelo mesmo
`submitShellInput` do painel de instalação (fiação no
`ShellEnvironmentOverlays`); a IDE nunca roda `sudo`.

**MEDIDO NO HARDWARE REAL (manhã de 2026-09-17, ESP32 com CP2102 em
`/dev/ttyUSB0`, antes de o autor desplugar):** `serial` **ok** — o usuário
NÃO está em `dialout`, e o acesso existe pela ACL `user:hugh:rw-` que o udev
pôs pela tag `uaccess` (o `stat` sozinho diria "sem acesso"; o `access(2)`
diz a verdade); `modemManager` **não ok** — ModemManager 1.25.95 rodando,
`ID_MM_CANDIDATE=1`, sem regra: o passo gerado é a regra para `10c4:ea60`;
`probe` **ok** — a distro já instalou `49-stlinkv*.rules` e
`60-openocd.rules` (a primeira rodada listou `/lib` em dobro por usrmerge;
corrigido). Log em `DocsPrivate/Codex/evidencias-2026-09-17-e2-permissao/`.

**Medido no gate:** 755 testes Rust (+3, `tests/serial.rs`: grupo/ACL/
usermod, a regra do ModemManager com o VID:PID e o canal ausente sem udevadm,
as regras de sonda em qualquer pasta com crédito à distro e o link `/lib`
uma vez); **39 harnesses QML** (`tst_embedded_access` novo, com o caso real
como fixture); clippy, fmt, `diff --check`, fiação, propriedades (234),
alcance (233), duplicação, arquitetura, docs, links, shell; qmllint estrito
limpo; Clang-Tidy do `.cpp` alterado limpo; `debug-strict` compila e abre
(831 ms); 143 métodos, 50 eventos.

**Não provado, dito:** o passo do ModemManager não foi executado (é do
autor, com sudo) — a validação "escrever a regra e o `ID_MM_DEVICE_IGNORE`
aparecer no `udevadm info`" fica para a noite, com a placa; o canal `probe`
com uma sonda real (ST-Link/USB-JTAG) idem. QEMU não substitui: o que se
mede é o nó do host. **Próximo (na época):** A5 — feito na §7.44.

### 7.44 A5 — ferramentas de embarcado no painel de instalação — 2026-09-17

O último item do bloco A. `setup/catalog_embedded.rs` (arquivo próprio: o
`catalog.rs` estava em 392/500) traz 11 ferramentas e 26 guias, e o
`setup.list` une os dois catálogos (`all_tools`/`all_guides` em
`setup/mod.rs`) — sem mudança de contrato. A regra do `catalog.rs` vale
inteira: **nenhum comando escrito por quem programou a IDE.** Duas classes de
fonte, ditas no arquivo: a página da ferramenta, verbatim (esptool: `pip
install esptool` num venv, como a página recomenda; mpremote: `pipx install
mpremote`; espflash: `cargo install espflash --locked`; probe-rs: o
instalador oficial e as dependências Debian; picotool: os cinco passos do
`BUILDING.md`, inclusive `cmake --install` — o README é explícito que copiar
o binário não basta ao pico-sdk — e a regra `60-picotool.rules`), e o índice
de pacotes da distro (a página do pacote prova o nome; o comando é a forma
padrão do gerenciador) para openocd, dfu-util, picocom, tio,
gcc-arm-none-eabi/gdb-multiarch (Fedora: `arm-none-eabi-gcc-cs` + newlib +
binutils; Arch: `arm-none-eabi-gcc` + `gdb`), QEMU (`qemu-system-arm` +
`misc` no Debian, `riscv` no Fedora/Arch). **Onde o índice não tem o
pacote, não há guia:** `tio` e `picotool` no Arch oficial (AUR não é
fonte), `picotool` e `espflash` no Fedora, e `espflash` no Ubuntu 26.04 —
o Debian trixie o tem, mas a família `debian` cobre os dois e um guia que
falha em metade dela não entra (o `any` do cargo vale). O
`install_command` do espflash em `tools/known.rs` passou a `--locked`, como
o README.

**Medido em 2026-09-17:** `setup.list` real nesta máquina — família
`debian`, 18 ferramentas, guias certos por família (esptool pelo pacote
`esptool 4.7.0` do Ubuntu, probe-rs pelas dependências + instalador,
picotool pelo build), 7 das 11 de embarcado já instaladas (esptool,
mpremote, dfu-util, picocom, arm-none-eabi, qemu, openocd) e 4 não
(espflash, probe-rs, picotool, tio) — coincide com o inventário feito à mão
no início do dia. 756 testes Rust (+1: ids únicos entre os catálogos, toda
ferramenta de embarcado com site e ≥ 1 guia, as famílias SEM guia são as
conferidas, nenhum guia traduz comando de outra família); 39 harnesses QML
(a tela não mudou: o painel já é uma lista rolável); clippy, fmt, `diff
--check`, arquitetura, docs, links; launcher vivo. Fontes: `WebFetch` nas
páginas das ferramentas e `curl` (HTTP 200) em cada página de pacote citada.

**Não provado, dito:** nenhum dos 26 guias foi executado nesta máquina. O
bloco A do roadmap 41 (E1 → E3 → E5 → E4 → E2 → A5) está fechado; o que
resta é a exercitação com a placa. **Próximo (na época):** Toolchains — feito
na §7.45.

### 7.45 Toolchains — seletor de pasta nativo e o SDK do Zephyr no `importKit` — 2026-09-17

Duas fatias, sem contrato novo. **(1) O seletor de pasta nativo.** O campo
"Pasta/SDK" do gerenciador de toolchain era só texto; agora tem
**Escolher pasta…**, e o que abre é o `FolderPicker` que a Start Screen já
tinha — o mesmo navegador de pastas do core (`fs.browse`), o mesmo visual —
com um PROPÓSITO: `FolderPickerController.purpose` (`"workspace"` abre o
projeto, como sempre; `"kitPath"` devolve o caminho por `folderPicked` e não
abre nada; o botão vira "Escolher"). O caminho vai
`ToolchainController.pickImportPath()` → `folderPickRequested("kitPath",
atual)` → `AppEnvironmentDomains` abre o picker → `Main.qml onFolderPicked` →
`ToolchainController.handlePickedPath` → `importPath`, que o campo lê. Nenhum
diálogo do sistema, nenhum componente novo de seleção; a digitação continua
valendo. **(2) O SDK do Zephyr.** `toolchain.importKit` reconhece a raiz de
um Zephyr SDK do sdk-ng pela dupla `sdk_version` + `cmake/Zephyr-sdkConfig.cmake`
e propõe um kit `zephyr-sdk` (03: `importKit`): as toolchains em
`gnu/<toolchain>/` (v1.x) ou na raiz (0.16/0.17 e os links de
bisectability), a `arm-zephyr-eabi` proposta quando há várias — dita na
evidência com as outras listadas —, `sysroot` pelo `-print-sysroot` (a libc
da toolchain), `gdb` da toolchain, sem `toolchainFile` e sem board (o `hint`
diz que o Zephyr compila pelo `west build -b` e acha o SDK por
`ZEPHYR_SDK_INSTALL_DIR`/registro CMake). O layout foi lido no
`scripts/template_setup_posix` e no `cmake/` do sdk-ng em 2026-09-17 (a
página de docs do Zephyr não detalha a estrutura interna; o script é a
fonte).

**Medido em 2026-09-17:** 757 testes Rust (+1: o SDK falso com duas
toolchains e o link de bisectability contado uma vez, a proposta ARM, a
riscv sozinha, o SDK vazio com a dica do `setup.sh`, e o erro geral que
agora nomeia o Zephyr); **40 harnesses QML** (`tst_folder_picker_purpose`
novo; `tst_toolchain_import` com o pedido/retorno do picker); clippy, fmt,
`diff --check`, fiação, propriedades (234), alcance (233), duplicação,
arquitetura, docs, links, shell; qmllint estrito limpo; `debug-strict`
compila e abre (598 ms). **Não provado:** nenhum Zephyr SDK real nesta
máquina — a fixture é a forma do `setup.sh`; o clique real no picker foi
provado pelo harness da lógica, não por uma sessão gráfica. **Próximo (na
época):** P0 — feito na §7.46.

### 7.46 P0 — o modelo do projeto: preset, Bear e o alvo no rust-analyzer — 2026-09-17, protocolo 0.115.0

Três fatias, na ordem do §4.1. **(1) Preset no configure automático.** O
`cmake.configure` sem `preset` — o caso do configure automático ao abrir e
do botão Configurar — escolhe agora: o preset do **kit ativo** (a ponte C++
guarda o último `toolchain.get` que a tela pediu e o manda; `presetSource:
"kit"`; as escolhas de ferramenta são as desse kit) ou o **padrão do
projeto** (`cmake::default_preset`: o primeiro `configurePresets` não
`hidden` de `CMakeUserPresets.json` — a escolha do usuário para esta máquina
vence —, senão de `CMakePresets.json`, pulando o que uma `condition` exclui
no Linux; as escolhas são as do kit padrão). Antes, um projeto com presets
era configurado SEM preset. `event.cmake.started` diz `preset` e
`presetSource`; no sucesso o preset fica anotado em `<buildDir>/.kinein-preset`
(o `CMakeCache.txt` não o guarda) e o `cmake.status { preset? }` o devolve —
o `ProjectHealthController.cmakePreset` o carrega; a aba IDE registra "preset
de: kit|CMakeUserPresets.json|CMakePresets.json". **(2) Bear para Makefile
puro.** `ProjectKind::Make` / `BuildSystem::Make` (marcadores `Makefile` e
`GNUmakefile`, atrás do CMake — um Makefile ao lado de um `CMakeLists.txt`
continua `cmake`); `build/make.rs`: `bear -- <make>` na raiz quando o `bear`
existe (Bear 3, README `bear -- <your-build-command>`, conferido em
2026-09-17; a CDB sai onde o clangd a acha, e o `cdb` a vê como `"."`),
senão `make` a seco com a linha que diz o passo — nunca um make "diferente"
para fingir CDB. `bear` entrou em `tools/known.rs` e no catálogo de
instalação (páginas de pacote Debian/Fedora/Arch conferidas; Ubuntu 26.04:
`bear 3.1.6`); o Project Health de um `make` sem bear nomeia o bear como
ferramenta ausente, com o gesto "Ferramentas"; o rótulo do projeto é
"Make". **(3) O alvo do kit no rust-analyzer.** `handlers/lsp/toolchain.rs`
(arquivo novo: o `handlers/workspace.rs` passou de 500 com isto — o kit
chegando aos servidores é uma responsabilidade): `rust-analyzer.cargo.target
= <triple>` pela secção `rust-analyzer`, como o basedpyright recebe a dele;
servidor vivo reiniciado ao mudar o kit; sem triple, sem configuração.

**Medido em 2026-09-17:** 761 testes Rust (+4: o preset padrão com
`condition`, o ciclo `cmake.configure` real com um `cmake` FALSO fixado no
kit — padrão do usuário → `--preset meu-local` nos argv, kit → `projeto`, sem
presets → sem `--preset`, e o `status.preset` em cada caso; o Makefile com e
sem bear, com a CDB gravada e a precedência do CMake; o rust-analyzer
recebendo `cargo.target` após `setKit` e reiniciando, e subindo sem
configuração ao limpar); 40 harnesses QML (`tst_project_health` com o
`cmakePreset` e o `make` sem/com bear); clippy, fmt, `diff --check`,
clang-format, Clang-Tidy dos 3 .cpp tocados, fiação, propriedades, alcance,
duplicação, arquitetura (dois arquivos passaram de 500 no caminho —
`build/mod.rs` e `handlers/workspace.rs` — e foram DIVIDIDOS por
responsabilidade: `build/make.rs`, `handlers/lsp/toolchain.rs`; catraca
intacta), docs, links, shell; qmllint estrito limpo; `debug-strict` compila e
abre (612 ms). `schemas/workspace.schema.json` ganhou `make`.

**Não provado, dito:** nenhum projeto Makefile real nem `bear` real (o
`bear` desta máquina não está instalado — o guia está no painel); o
rust-analyzer real com um alvo bare metal não foi observado além do
`fake_lsp_server`. **Próximo:** P4 MicroPython — firmware oficial gravado
pela IDE (C5, motor do E4 com o `.bin` do micropython.org), arquivos no
dispositivo (`mpremote fs`, C2), stubs por placa (C4).

### 7.47 P4 — MicroPython: arquivos na placa, firmware oficial, stubs por placa — 2026-09-17 (noite), protocolo 0.116.0

Três fatias do bloco C do `roadmaps/41`, na ordem do prompt, **com o ESP32
do autor plugado** (CP2102, `/dev/ttyUSB0`) — a primeira fatia de embarcados
provada no hardware no mesmo dia em que nasceu. **(1) C2, arquivos na
placa.** `serial.files { device, action: list|get|put|rm|mkdir, path?,
local? }` como job sobre `mpremote connect <dev> fs …`, desfecho em
`event.serial.files`. O formato do `ls` (`{tamanho:12} {nome}[/]`, uma
linha verbosa antes) e a linha `mpremote: cp: x: No such file or directory.`
do erro foram MEDIDOS na placa e conferidos no código do mpremote 1.29.0.
Duas descobertas que só a placa daria: a primeira conexão morreu com
`could not enter raw repl` porque o firmware do autor inunda a UART (1,3 MB
de binário em segundos) — o job repete uma vez e as duas saídas viajam em
`raw`; e o `put` do mesmo conteúdo não regrava (o mpremote confere o hash:
`Up to date`). `serial/job.rs` nasceu com o "rodar na porta com prazo e
cancelamento" que o `identify` já tinha (agora compartilhado, sem cópia);
`handlers/serial.rs` ganhou `porta_pronta` (as recusas antes de tocar a
porta) para os dois. Na tela: `EmbeddedFilesController` (filho do
`EmbeddedController`, como identity/flash/access) e `EmbeddedFilesView` —
pasta ao lado da porta, lista, entrar/subir, Baixar (para `placa/<caminho>`
sob o workspace, o espelho da placa, e abre no editor), Enviar o arquivo
aberto (o editor diz qual NO CLIQUE, pelo AppDomains), Apagar; **todo gesto
que escreve pede um segundo clique** (`pending`). **(2) C5, firmware
oficial.** O catálogo do provedor de instalação ganhou `kind:
toolchain|firmware` e `firmware { board, engine, offset?, chip?, file }`:
cinco releases v1.29.0 (2026-08-24) do micropython.org — ESP32_GENERIC
(offset `0x1000`), ESP32_GENERIC_C3 e _S3 (`0x0`), RPI_PICO e RPI_PICO_W
(`.uf2`) — com o offset lido na página de cada placa. **A fonte não publica
checksum**: o SHA-256 pinado foi medido no download desta sessão (os cinco
arquivos, 0,7–1,8 MB) e o `source` confessa. O provedor baixa e guarda o
arquivo inteiro (sem `tar`, sem `bin/`); `runConfig.flashProposal
{ firmware }` compõe a linha da página (`flash/firmware.rs`; o `flash.rs`
virou `flash/mod.rs` para não passar de 500) com os avisos: `erase-flash`
na primeira instalação (a linha pronta no aviso, nunca na de gravar), chip
do kit ≠ chip da página, flash identificada menor que o arquivo. Na tela: o
catálogo marca "firmware" e diz Baixar/Baixado; o Gravar oferece os
firmwares baixados como chips no lugar do build. **(3) C4, stubs por
placa.** `python.stubs { port?, board? }` — `uv pip install -U
--link-mode=copy --target <root>/typings micropython-<port>[-<board>]-stubs`
(a forma da fonte, `--target ./typings`; sem uv, o `pip` do interpretador do
projeto); o modelo do projeto sugere o pacote pelo chip do kit/identidade
pela lista publicada (`esp32c3` → `micropython-esp32-esp32_generic_c3-stubs`);
`python.status` diz `stubsPath`/`stubsSuggested`; ao terminar, o
basedpyright recebe `basedpyright.analysis.stubPath` e
`reportMissingModuleSource: none` (as chaves do manual do basedpyright e do
`pyproject` de exemplo dos micropython-stubs, conferidas) e reinicia. Na
tela: faixa informativa de saúde com **Instalar stubs**.

**O que a placa consertou.** (a) `serial.identify` devolvia `chip:
esp32d0wdv3` para o `ESP32-D0WD-V3` do autor — o encapsulamento do clássico
virava série; agora só `s2/s3/c2/c3/c5/c6/h2/p4` entram na chave e o resto é
`esp32`. A `FIXTURE_V5` de `serial/identify.rs` passou a ser a saída REAL
(cabeçalho `Serial port`/`Detecting chip type`, linhas do stub flasher,
`Flash voltage set by a strapping pin`); a antiga, escrita do código com
um C3, ficou como segundo formato. (b) O `EmbeddedEventRouter` chamava
`handleIdentifyStarted`/`handleIdentified` no PAI (onde não existem — moram
no filho `identity`) e `flashProposalResolved`/`serialAccessResolved` não
tinham consumidor QML: o core respondia E5/E4/E2 e a tela nunca recebia — o
"sintoma do Docker" (§7.34) de novo, invisível a todos os gates porque um
sinal C++ sem ouvinte QML não é erro para ninguém. Corrigido: cada desfecho
vai ao filho dono, e `requestFailed` chega a todos os filhos.

**Medido em 2026-09-17:** 782 testes Rust (+21: `serial/files` — linha,
parser sobre a saída real, erro, validação; `serial.files` por despacho —
list/get/put/rm/mkdir com o risco por ação, a repetição do raw REPL, a
linha do erro, as recusas, o cancelamento; catálogo de firmware pinado;
download de firmware servido localmente com checksum e sem `tar`; a
proposta esptool/picotool com firmware; o despacho `flashProposal
{ firmware }` até o `run.start` com o esptool falso; `python/stubs` —
pacote, sugestão, linhas; `python.stubs` por despacho com o uv falso e o
status antes/depois; o `stubPath` chegando ao servidor falso no
`didChangeConfiguration` após `event.python.stubs`); 41 harnesses QML
(`tst_embedded_files` novo; `tst_embedded_flash`, `tst_python`,
`tst_project_health` estendidos); 145 métodos, 52 eventos; clippy, fmt,
clang-format, Clang-Tidy dos 4 .cpp tocados, fiação, propriedades, alcance,
duplicação (a derivação `kind === "firmware"` ficou com um dono só, o
`ToolchainController`), arquitetura, docs, links, shell; qmllint estrito
limpo; os dois presets clang compilam e abrem. **Na placa real:** `ls`,
`get main.py` (13588 B, SHA-256 igual ao `sha256sum` da placa), o erro do
mpremote, o `put` do mesmo `main.py` (SHA-256 igual antes e depois),
`flash-id` → `esp32`, 4MB, MAC — registros em
`DocsPrivate/Codex/evidencias-2026-09-17-p4/`.

**Não provado, dito:** o firmware não foi GRAVADO na placa do autor (apagaria
o `main.py` dele; a linha foi provada com o esptool falso e roda pelo
`run.start` real); nenhum Pico (o `picotool` não está nesta máquina); os
stubs reais foram instalados pelo `uv` desta máquina (2 pacotes em 7 ms)
fora do core — no core, com o uv falso; o passo do ModemManager (E2) e a
gravação do E4 continuam sendo do autor. C6 (CircuitPython) não foi feito.
**Próximo:** bloco E — ESP-IDF, pico-sdk, Zephyr e PlatformIO como motores
de build/flash/monitor.

### 7.48 Bloco E — os frameworks como MOTORES de build, gravar e monitorar — 2026-09-17 (noite), protocolo 0.117.0

O `project.model` reconhecia os quatro frameworks e dizia o que faltava; o
build continuava sendo cargo/cmake/make. Agora `build/engine.rs` decide, pelo
modelo e sem tocar o disco de novo, com que motor o `build.run` roda —
**sem campo novo no contrato**: PlatformIO → `pio run` (o `platformio.ini`
virou `kind: platformIo`, atrás do CMake e na frente do Makefile; um
`CMakeLists.txt` do ESP-IDF ao lado não tira o `pio` do comando); ESP-IDF →
`idf.py build` DENTRO do ambiente ativado — `bash -c '. "$1" >/dev/null ||
…; shift; exec idf.py "$@"' idf <ativação> build`, a ativação sendo o
`export.sh` de `IDF_PATH`/`~/esp/esp-idf` ou o `activate_idf_<versão>.sh`
mais novo de `~/.espressif/tools` (as duas formas do "Get Started" v6, lidas
em 2026-09-17: EIM é o caminho recomendado; o banner vai para /dev/null, o
erro não); Zephyr → `west build -d build [-b <placa>]`, a placa do
`CACHED_BOARD` do `build/CMakeCache.txt` ou do `west config build.board` (a
forma da doc do west — nenhum campo de placa inventado no kit); pico-sdk →
o CMake de sempre com `-DPICO_SDK_PATH=<sdk>`, no `build.run` e no
`cmake.configure` automático. Framework reconhecido e ferramenta ausente é
`TOOL_NOT_FOUND` antes do job, com o passo que o modelo já dava. O
`handlers/build.rs` passou de 500 no caminho e o `build.size` foi para
`handlers/build_size.rs` (medir o ELF é outra responsabilidade). **Gravar**
(`flash/frameworks.rs`): `idf.py` (`-p <porta> flash`, ativado; porta
exigida), `west` (`west flash -d build`, o padrão do Zephyr — antes o
`target.flashEngine: west` do modelo era motor DESCONHECIDO para o E4),
`platformio` (`pio run -t upload [--upload-port]`, o padrão do PlatformIO —
idem). **Monitor**: o IDF Monitor e o `pio device monitor`, antes do
catálogo quando o papel não está fixado. E um conserto de base: o modelo do
projeto lia os binários pelo PATH do processo, não pelo detector do core —
agora é o detector (o mesmo do build e do kit), e o `Core::with_home` dos
testes fixa o `$HOME` das pastas padrão dos SDKs sem escrever no ambiente.

**Medido em 2026-09-17:** 791 testes Rust (+9: o motor por framework e o que
falta; a ativação do IDF (export.sh vs. EIM mais novo); a placa do Zephyr
(cache, depois `west config`); as linhas de gravar dos três wrappers; e por
despacho, com wrappers FALSOS: ESP-IDF build+flash+run.start pelo `export.sh`
falso que põe o `idf.py` falso no PATH, Zephyr build com a placa do `west
config` e a proposta `west flash`, PlatformIO como tipo de projeto com `pio
run`/upload e a precedência sobre o CMakeLists, pico-sdk com o `-D` no
`build.run` e no `cmake.configure`); 41 harnesses QML (`tst_embedded_flash`
com os motores por framework); clippy, fmt, fiação, propriedades, alcance,
duplicação, arquitetura (o split do `build.size`), docs, links, qmllint;
`debug-strict` compila.

**Não provado, dito:** nenhum ESP-IDF, Zephyr, pico-sdk ou PlatformIO REAL
nesta máquina (`west` e `pio` estão no PATH, mas sem workspace/projeto
deles) — os wrappers foram exercitados por falsos que ecoam argv; o
`export.sh` real do IDF nunca foi sourced por `bash -c` aqui. E5 (templates
curados) e E6 (Unity/Ceedling) não entraram. **Próximo:** P3 — depuração
profunda (SVD, RTT/defmt, memória/disassembly, threads de RTOS).

### 7.49 P3, fatia 1 — o que o depurador de embarcado MOSTRA, pelo DAP padrão — 2026-09-17 (noite), protocolo 0.118.0

A pergunta do autor foi se dava para "pegar uma solução pronta do VS Code"
para JTAG. A resposta que virou código: a extensão (Cortex-Debug, a do
probe-rs) não se embute num Qt, mas o que ela consome — o **protocolo** — já
é o nosso. Medido no `initialize` desta máquina em 2026-09-17: o `probe-rs
dap-server` 0.32.0 e o `gdb -i dap` 17.1 anunciam `supportsReadMemoryRequest`,
`supportsDisassembleRequest`, `supportsSetVariable`,
`supportsWriteMemoryRequest`. Então o P3 é **consumir**, não reimplementar.

**O que a medição consertou primeiro.** O `launch` que a IDE mandava ao
probe-rs (`program` + `chip` no topo, desde 0.70.0) **falha**: `Serialization
error "missing field coreConfigs"` (medido com o dap-server real, sem sonda —
o erro vem antes de procurar a sonda). A forma certa, lida em
`server/configuration.rs` do 0.32.0, é `{ cwd, chip?, coreConfigs: [{
coreIndex: 0, programBinary, svdFile?, rttEnabled }] }`. Ou seja: o
adaptador `probe-rs` nunca teria depurado nada até aqui — e nenhum gate pegou,
porque a prova do ciclo de embarcado é o `gdb -i dap` com QEMU. Corrigido, com
teste da forma. **Depois, as quatro peças de D1–D4:** `svdFile` no kit
(`toolchain.setKit`, `ToolchainResult`, campo **SVD** no painel — o painel
bateu em 300 e os campos do kit saíram para `EmbeddedKitView`), que vai em
`coreConfigs[0].svdFile`; **RTT/defmt**: `rttEnabled: true` no launch, os
eventos `probe-rs-rtt-channel-config`/`probe-rs-rtt-data` (nomes lidos em
`debug_adapter/dap/adapter.rs`) viram `event.debug.output { category: "rtt",
channel, channelName? }`, e o core responde o request `rttWindowOpened` que o
probe-rs exige para começar a ler ("will delay polling RTT channels until the
data window has opened"); `probe-rs-show-message` vira console.
**`debug.scopes { frameId }`** devolve os escopos inteiros (Locals, Registers,
Peripherals com `expensive`) — o D4 era "mostrar o escopo que hoje se
esconde", e o `debug.variables { frameId }` escondia ao escolher o primeiro;
**`debug.readMemory`** e **`debug.disassemble`** são o DAP verbatim. Na UI:
`DebugInspectController` (filho `inspect` do `DebugController`, que estava
em 398/400 — os quatro `ListModel` viraram uma linha cada) e
`DebugInspectView` entre as variáveis e os watches: Listar escopos → chips →
variáveis do escopo; endereço → Memória (hex + ASCII, 16 por linha, bytes
ilegíveis ditos) / Desmontar (endereço, bytes, instrução, símbolo).

**Medido em 2026-09-17:** 794 testes Rust (+3: a forma do launch do probe-rs
com e sem SVD; por despacho, contra um adaptador falso em TCP que fala como o
probe-rs — o canal RTT anunciado e os dados viram `rtt`, o aviso vira
`console`, a parada enriquecida, os três métodos verbatim com os argumentos
conferidos no que o adaptador recebeu, e o `rttWindowOpened` mandado; as
recusas sem sessão e de parâmetro); 42 harnesses QML (`tst_debug_inspect`
novo; `tst_toolchain_import` com o `svdFile`); 148 métodos; clippy, fmt,
clang-format, Clang-Tidy dos 3 .cpp tocados, fiação, propriedades, alcance,
duplicação, arquitetura, docs, links, qmllint; `debug-strict` compila.

**Não provado, dito:** nenhuma sonda nem chip com USB-JTAG nesta máquina — o
ESP32 clássico do autor não tem JTAG embutido; o probe-rs real só foi levado
até o `launch` (onde a forma velha falhava e a nova passa a procurar a
sonda). D5 (threads de RTOS) não entrou; `setVariable`/`writeMemory` (SVD com
escrita) e `rttChannelFormats` (defmt declarado por canal) ficam para a fatia
2, quando houver placa. **Próximo:** P5 — qualidade (clang-tidy no clangd,
lâmpada proativa, gtest/catch2, cobertura).

### 7.50 P5 — qualidade: clang-tidy, gtest/Catch2, cobertura e a lâmpada — 2026-09-17 (noite), protocolo 0.119.0

Quatro peças, todas provadas com ferramentas falsas onde a real não cabe num
teste. **(1) clang-tidy em dois lugares.** O clangd sobe com `--clang-tidy`
(o clangd 21 desta máquina lista a flag "Enable clang-tidy diagnostics"; ele
lê o `.clang-tidy` do projeto sozinho) — os avisos do tidy entram no canal
dos diagnósticos do arquivo aberto, sem job. E o `quality.run` de um
`CMake`/`Makefile` roda o projeto inteiro pela CDB (`build/tidy.rs`):
`run-clang-tidy -p <cdb> -quiet` quando o script do LLVM existe, senão
`clang-tidy -p <cdb> <arquivos da CDB>` — os arquivos vêm da própria
`compile_commands.json`, nunca de um glob; o parser gcc-like casa e o nome
do check fica na mensagem. Sem CDB, "configure/compile com o bear"; sem
`clang-tidy`, a ferramenta. O botão de análise da barra aparece em C/C++ e
Python (antes só Cargo). **(2) gtest e Catch2 na árvore** (`test/frameworks.rs`):
o `ctest --show-only=json-v1` (ctest 4.2.3, medido) dá o comando de cada
teste; cada binário é perguntado (`--gtest_list_tests`, depois `--list-tests
--verbosity quiet`) e os casos entram como `teste::caso` depois das linhas do
ctest; `test.run { testId }` com esse id roda o binário direto
(`--gtest_filter=`, ou o nome no Catch2, com `-r compact` e o exit code como
desfecho). Um binário mudo continua sendo só a linha do ctest. **(3)
Cobertura** — domínio `coverage.*`, o 35º: `coverage.run` (job) → Rust pelo
`cargo llvm-cov --lcov --output-path .kinein/coverage.lcov` (o `cargo` do
kit; cargo-llvm-cov 0.9.1 aqui), Python pelo `coverage run -m pytest` +
`coverage lcov` do interpretador do projeto; C/C++ recusado antes do job
(exige `--coverage` no build — decisão do usuário). O LCOV é relido:
resumo por arquivo no `event.coverage.finished`, e `coverage.lines { file }`
dá as linhas de um arquivo. Na UI: `CoverageController` (resumo + o mapa
linha→covered|missed do arquivo ativo, pedido a cada troca de aba),
comando **Cobertura dos testes** (paleta e menu Build, pelo `JobsController`
como a análise), e a calha do editor com uma barra verde/vermelha ao lado da
do diff. **(4) A lâmpada proativa:** na linha do cursor com diagnóstico a
calha mostra 💡 antes do Alt+Enter — o clique pede as ações (o mesmo
`lsp.codeActions`). A linha do cursor vem do `cursorRectangle` pelo índice
visível (folding). No caminho, `EditorGutter` e `EditorTextSurface` bateram
em 300: as marcas da faixa esquerda (diff, cobertura, dobra, breakpoint)
saíram para `EditorGutterLineMarks.qml`, e o `blameColumnWidth` duplicado
do surface caiu.

**Medido em 2026-09-17:** 803 testes Rust (+9: os arquivos da CDB; o
`quality.run` C/C++ sem CDB, sem tidy, com tidy falso avisando e com o
`run-clang-tidy` falso vencendo; o `json-v1`, as listagens gtest/Catch2 e o
`[ OK ]` por caso; os binários falsos perguntados e um caso rodado por
cada framework; o LCOV lido e resumido; a cobertura Rust por despacho com o
cargo falso fixado no kit e o `coverage.lines`; a Python pelo interpretador
falso e a recusa do C/C++); 43 harnesses QML (`tst_coverage` novo); 150
métodos, 53 eventos, 35 domínios; clippy, fmt, clang-format, Clang-Tidy do
.cpp novo, fiação, propriedades, alcance, duplicação, arquitetura (o split
da calha), docs, links, atalhos (o comando `coverage.run` tratado no host),
qmllint; `debug-strict` compila e abre.

**Não provado, dito:** o clang-tidy real do projeto inteiro não rodou aqui
(a IDE não tem projeto C/C++ com CDB aberto no gate — o `verificar-cpp.sh`
roda o tidy da própria IDE, o que prova a ferramenta, não o caminho pelo
`quality.run`); nenhum gtest/Catch2 real (só binários falsos que respondem
como eles); a cobertura Rust real deste repositório pelo `cargo llvm-cov`
não foi executada no gate (leva minutos; o cargo-llvm-cov está instalado e o
autor pode rodar pelo menu). cppcheck, doctest e gcov/lcov não entraram.
**Próximo:** P6 — Linux embarcado (SSH remoto: deploy, `gdbserver`, debugpy
attach remoto), que precisa de um desenho antes de código; ou o banco
(consultas/escrita/TLS).

### 7.51 P6 fatia 1 — o alvo Linux por SSH como recurso do projeto — 2026-09-17 (noite), protocolo 0.120.0

O desenho foi escrito antes do código (`roadmaps/42` §P6, "Desenho da fatia
1") e o código o segue. **O que é:** a Raspberry Pi, a placa com imagem
própria — um Linux alcançável por SSH — no molde do `datasource.*`: um
perfil **sem senha em disco** (`RemoteTarget { name, host, user?, port?,
identityFile?, deployDir? }`, `deny_unknown_fields`; um teste reprova
qualquer chave que pareça segredo no `.kinein/remotes.json`), um **probe** que
MEDE o alvo (`ssh -o BatchMode=yes -o ConnectTimeout=5 [-p] [-i] [user@]host
'uname -m; uname -sr; command -v gdbserver python3 rsync'` — uma linha por
fato; a chave recusada vira "copie a sua com `ssh-copy-id user@host`", o
host não resolvido e o sem rota em 5 s têm cada um a sua frase), um
**deploy** como job (`rsync -az --delete -e 'ssh …' <origem>
[user@]host:<dest>/`, ou `scp -r` sem `rsync`; origem padrão `build/`,
destino `deployDir` ou `~/kinein/<projeto>`; origem inexistente é recusa
síncrona) e o `remote.command` **puro** que compõe a linha `ssh -tt …
'<comando>'` para `run` (configuração "Rodar em pi"), `debugServer` (o kit
ganha `debugServer` + `remoteTarget = host:2345` — a ponte tinha o
`toolchain.setKit` sem esses dois campos; ganhou `toolchainSetKitRemote`),
`debugpy` (`--listen 0.0.0.0:5678 --wait-for-client`, attach em host:5678) e
`shell` (para o terminal da IDE). O `-tt` é decisão medida no desenho: o
`debugServer` do kit roda por `sh -c` sem terminal, e sem pty forçado matar
o `ssh` local deixaria o `gdbserver` órfão na placa. Transporte é o
`ssh`/`rsync`/`scp` do sistema como processo (OpenSSH BSD, rsync GPL-3 —
nunca crate). UI: painel **Alvo remoto (SSH)** no menu Ambiente (`remote.list`
na paleta, sem atalho): lista, formulário sem campo de senha (a nota diz o
`ssh-copy-id`), Sondar primário, Enviar, e os quatro botões que levam o
resultado ao dono certo (run configs, kit, terminal) — `RemoteController` +
`Remote{Event,Request}Router`, `core_client_remote.cpp`.

**Medido em 2026-09-17:** 813 testes Rust (+10: o store ida e volta, lixo e
schema desconhecido, o arquivo sem chave de segredo; as linhas de ssh/rsync/
scp, o script do probe, o parser linha a linha e as três frases de falha; os
comandos e a validação; por despacho — o catálogo sem segredo e ordenado, o
`password` recusado pelo contrato, o probe com `ssh` falso respondendo como
uma Pi e outro recusando a chave, o deploy por `rsync` falso e a queda para
`scp`, o `remote.command` nos quatro tipos); 44 harnesses QML (`tst_remote`
novo); 156 métodos, 55 eventos, 36 domínios (150/53/35 + os desta fatia);
clippy, fmt, clang-format, Clang-Tidy do .cpp novo, fiação, propriedades,
alcance, duplicação, arquitetura, docs, links, shell, atalhos (o comando
`remote.list` tratado no host e no menu), qmllint; `debug-strict` compila.

**Não provado, dito:** nenhum alvo real — não há Pi nesta máquina; o `ssh`
real não foi exercitado (só os falsos ecoando argv) e o `sshd` local no gate
fica para quando houver chave de teste. Workspace remoto, LSP do outro lado,
mapeamento de caminhos, journalctl/dmesg, Yocto/Buildroot: fatia 2 (42 §P6).
**Próximo:** fatia 2 do P6, ou o banco (consultas/escrita/TLS).

### 7.52 Banco — consultas, escrita e TLS — 2026-09-18, protocolo 0.121.0

O desenho foi escrito antes do código (`roadmaps/35` §7.4) e o código o
segue. **`datasource.query { name, password?, sql, maxRows?, confirmWrite? }`**
como job → `event.datasource.queried { columns[], rows[[texto | null]],
rowCount, affected?, truncated, elapsedMs, message?, secretRequired }`. A
primeira palavra classifica a instrução (`SELECT`/`WITH`/`VALUES`/`TABLE`/
`SHOW`/`EXPLAIN` = leitura) e o **motor impõe**: `BEGIN READ ONLY` no
PostgreSQL, `SQLITE_OPEN_READ_ONLY` no SQLite — o teste mostra um `WITH …
INSERT` recusado pelo próprio SQLite ("attempt to write a readonly
database"). O teto vem de fora do texto (`SELECT * FROM (<sql>) AS kinein_q
LIMIT n+1`; o `step` até n+1 no SQLite) e `truncated` diz quando cortou.
Uma instrução que não é leitura sem `confirmWrite: true` é recusada **antes
do job** com o código novo `WRITE_CONFIRMATION_REQUIRED`; a UI mostra o
botão "ESCREVE — executar mesmo assim" por esse código, nunca lendo texto, e
reenvia. Células em texto: o protocolo simples do PostgreSQL devolve toda
coluna assim (sem mapa de tipos); `ValueRef` do SQLite; no Mongo a consulta
é `<coleção> <filtro JSON>` → `find` com `limit`, só leitura, colunas = chaves
de primeiro nível. **TLS:** `DataSourceProfile.tls: disable | require` +
`caFile` — `require` é o `verify-full` do libpq (cadeia e host) pelo
`tokio-postgres-rustls` 0.14 sobre o `rustls` que o `mongodb` já trazia; +11
crates medidos, `cargo deny` verde, nenhuma licença nova; não existe "cifra
sem conferir". UI: `DataSourceQuery.qml` (editor com Ctrl+Enter, confirmação,
grade de texto com `null` em itálico) dentro do painel, chips de TLS e o
campo do PEM no formulário; a ponte ganhou `dataSourceQuery` e o evento.

**Medido em 2026-09-18:** 819 testes Rust (+6: classificação e wrapper puros;
SQLite real — leitura com teto/`truncated`/`NULL`/`BLOB`, a leitura que
escreve recusada pelo motor, escrita com `affected`, erro do motor; tabela
de documentos do Mongo e as duas recusas de texto; por despacho — vazio,
perfil desconhecido, leitura, `WRITE_CONFIRMATION_REQUIRED`, escrita
confirmada, erro; TLS salvo/normalizado/só PostgreSQL/valor inválido); 45
harnesses (`tst_datasource_query` novo; `tst_datasource` ajustado para os 10
campos do perfil); 157 métodos, 56 eventos, 36 domínios; clippy, fmt,
clang-format, `cargo deny`, fiação, propriedades, alcance, duplicação (a
derivação `engine === "mongo"` que nasceu duplicada foi devolvida ao
controller), arquitetura, docs, links, shell, atalhos, qmllint;
`debug-strict` compila.

**Não provado, dito:** PostgreSQL e MongoDB reais (o gate só tem o SQLite);
o TLS de ponta a ponta — nenhum servidor com certificado nesta máquina;
o `READ ONLY` do PostgreSQL só por leitura do manual (`SET TRANSACTION READ
ONLY` recusa `INSERT`/`UPDATE`/`DELETE`/DDL — documentado, não exercitado).
**Falta:** escrever documento no Mongo; abas e histórico de consulta;
exportar; cancelar consulta longa (driver síncrono — o job não é
cancelável). **Próximo:** varredura 40 §8 ou P6 fatia 2.

### 7.53 P6 fatia 2 — o workspace ESPELHADO — 2026-09-18, protocolo 0.122.0

O desenho foi escrito antes do código (`roadmaps/42` §P6, "Desenho da fatia
2"). A pergunta era como abrir a pasta da Pi. O VS Code (Remote-SSH,
proprietário) sobe um servidor Node no alvo — pesado numa Pi e, para nós,
reescrever `fs.*`, índice, git e LSP para um segundo filesystem. A resposta
é a outra escola (o "deployment" do PyCharm, o fluxo Zephyr/Yocto): **a
pasta remota vira um espelho local por `rsync`, e a IDE abre o espelho como
workspace comum**. `remote.open { name, path }` → job `rsync -az -i
--exclude .kinein -e 'ssh … -o ControlMaster=auto -o ControlPath=<cache>/
ssh-%C -o ControlPersist=60' [user@]host:<path>/ <espelho>/` para
`~/.cache/kinein-vectis/remote/<alvo>/<hash>/<basename>`, grava o marcador
`.kinein/remote-mirror.json` e copia o alvo para o catálogo do espelho
(autossuficiente; o `.kinein` nunca sincroniza) → `event.remote.synced {
direction: pull, changed[] }` → a UI abre o espelho pelo `workspace.open`
de sempre, que responde `remote: RemoteMirror`. **Salvar empurra**: o
`fs.write` num espelho dispara o job `Empurrar <arquivo> para <alvo>` só com
aquele caminho. `remote.sync { pull | push, paths? }` move a árvore ou
caminhos relativos (sem `..`, nunca `.kinein`), **nunca `--delete`**;
`remote.status` diz se o workspace é espelho. `changed` é a saída `-i` do
rsync, lida linha a linha. UI: `RemoteMirrorView` no painel Remoto ("Abrir
espelho"; a faixa "este workspace é um espelho de pi:/…" com Puxar /
Empurrar tudo); o espelho aberto seleciona o alvo dele. No caminho, um
defeito que o teste revelou antes da Pi: o espelho aberto não tinha o alvo
(usuário/porta/chave ficavam no workspace de origem) — por isso o `remote.
open` copia o alvo para o catálogo do espelho.

**Medido em 2026-09-18:** 825 testes Rust (+6: o espelho nomeado pelo
basename sob o cache; o marcador ida-e-volta e o schema estranho; as linhas
rsync com excludes/transporte/sentido; a saída itemizada; `safe_relative`;
por despacho com um `rsync` FALSO que copia entre o espelho e uma "Pi"
local — open sem rsync/alvo/pasta recusados, o pull com a linha certa e o
marcador, o `workspace.open` com `remote`, `remote.status`, o push
automático de um `fs.write` só com aquele arquivo, `remote.sync` com paths
inválidos recusados, pull/push, a falha do rsync no evento); 45 harnesses
(`tst_remote` estendido); 160 métodos, 57 eventos, 36 domínios; clippy,
fmt, clang-format, Clang-Tidy, fiação, propriedades, alcance, duplicação,
arquitetura, docs, links, shell, atalhos, qmllint; `debug-strict` compila e
abre.

**Não provado, dito:** nenhum alvo real (sem Pi; o `rsync` real e o
`ControlMaster` não exercitados — só o falso ecoando argv). **Falta:**
watcher remoto (o que muda no alvo só aparece ao Puxar); renomear/apagar
propagados (`--delete` explícito); LSP/interpretador do alvo;
journalctl/dmesg; Yocto/Buildroot no P0; `sshd` local no gate.
**Próximo:** varredura 40 §8, bloco F.

### 7.54 Pente-fino da arquitetura — 2026-09-18 (a Etapa 1 fechou; antes da Etapa 2)

Pedido do autor: "buscar qualquer tipo de problema/falha no projeto" antes de
abrir a Etapa 2. O que se mediu e o que se achou:

```text
fiacao IPC (§8, as 4 medicoes + evento descartado)   6 achados -> gate novo + 5 fixos + 1 aceito
cargo test x5 seguidas                                 827/827 nas cinco: nenhum intermitente
unwrap/expect fora de teste                            0 (medido por script sobre o codigo antes do cfg(test))
TODO/FIXME/HACK                                        1 (o "TODO GDB" do adapter.rs — virou o gdb_pick)
verificar-embarcado (QEMU, gdb -i dap)                 FALHAVA nesta maquina: attach mudo 30 s
verificar-appimage                                     FALHAVA no artefato: MANUAL.md ausente
release-hardened                                       compila e abre (318 ms)
binario debug (ASan/UBSan) sem workspace               0 avisos QML/sanitizer no stderr
clippy pedantic, fmt, deny, shellcheck, qmllint, 24 gates   verdes
docs: afirmacao morta                                  1 ("credencial nao tem onde morar", de 2026-09-03)
```

**Defeitos corrigidos.** (1) **O `gdb` nativo não fala ARM e a IDE
esperava para sempre**: com o kit em "gdb" e um ELF Cortex-M, o `attach`
ficava mudo (o gate do QEMU parava em "event.debug.stopped não chegou em
30 s"; registrado como "falhou" desde 2026-09-15 e nunca atacado). Agora
`dap/gdb_pick.rs` lê o `e_machine` do ELF antes de subir o adaptador: ELF
de outra máquina com o `gdb` nu → o GDB de alvo detectado entra
(`arm-none-eabi-gdb`, `gdb-multiarch`, `xtensa-esp-elf-gdb`,
`riscv32-esp-elf-gdb`, na ordem do catálogo) e o console de debug diz;
sem nenhum, a recusa diz o pacote. O gate do QEMU voltou a passar aqui (22
ms do continue ao stop). (2) **O AppImage instalava `manual.md` e o smoke
exigia `MANUAL.md`** — `install(FILES … RENAME MANUAL.md)`; conferido com
`cmake --install` local (o artefato em `dist/` é de 2026-09-16 e fica
antigo até o autor pedir um novo). (3) `event.quality.output` **descartado
no C++** desde a varredura: a saída da análise (o clang-tidy dizendo que
não achou a CDB) não chegava a tela nenhuma — vai para o painel de Build,
com o `$ comando`. (4) `cmake.presets.list` sem cliente desde 0.25.0 →
seletor de preset no kit. (5) `environmentTool` sem ouvinte → as
ferramentas aparecem uma a uma durante o scan. (6) Dois sinais QML mortos
(`GitPanel.branchMenuDismissRequested`, `TerminalScrollController.
sessionChanged` — este só o harness ouvia) saíram. (7) A afirmação morta
da leitura-técnica.

**O gate novo.** `verificar-fiacao-ipc.sh` (24ª): método roteado sem
cliente, evento do core que o C++ não trata ou trata e descarta, sinal do
`CoreClient` sem ouvinte, sinal QML sem tratador — com a lista de exceções
DITAS (cada uma com motivo) dentro do script. Ele mede 160 métodos e 57
eventos, os mesmos números que o 03 afirma — pela primeira vez o número
documentado sai de um comando do gate. (Desde a noite do mesmo dia, §7.64,
a quinta pergunta: nenhum elo `dispatch*Result`/`handle*Notification` da
ponte C++ sem chamador.)

**Medido em 2026-09-18:** 827 testes Rust (+2: o `e_machine` nas duas
ordens de byte; a troca do gdb nu por um GDB de alvo, só detectado, e a
recusa com o pacote), 5 rodadas seguidas sem intermitente; 45 harnesses
(`tst_toolchain` com o seletor de preset; `tst_terminal_scroll` lê o
`renderedSessionId`); as 24 verificações passam nesta máquina — inclusive
QEMU, debugpy real, clangd cross e exercitação, que não entravam nos
registros diários.

**O que o pente-fino NÃO cobre, dito:** ele mede o que está ligado, não se
a tela ligada mostra a coisa certa nem se é legível — isso é a Etapa 2 com
a IDE aberta, e é onde os testes práticos do autor entram. Sem GUI não se
prova o clique; os harnesses provam o controller.

### 7.55 Etapa 2, F0 — o desenho medido e a infra de medição — 2026-09-18

O autor abriu a Etapa 2 pedindo pesquisa de métodos de UI/UX/HUD e o
"visual JetBrains-like adaptado". O que se fez: (1) a referência lida nas
FONTES (New UI e Islands da JetBrains, o critério dos widgets da status
bar no SDK, Doherty/Nielsen para o tempo de resposta, progressive
disclosure/Sweller, o orçamento de frame do Zed) — está na §1 do `43`;
(2) **a IDE fotografada de verdade**: `kinein-vectis <pasta>` abre o
projeto direto (o argumento que faltava para o gate e para um lançador),
`KINEIN_SCREENSHOT=<png>` + `KINEIN_SCREENSHOT_DELAY_MS` grava a janela
headless — três fotos em `imagens/prints/2026-09-18-etapa2/` e a leitura
delas na §2 do `43` (12 controles na barra superior; 10 ícones sem rótulo;
"Salvar" amarelo permanente; aba ativa pouco distinta; `.git`/`build`/
`target` no mesmo peso que `crates`); (3) dois defeitos que a foto mostrou,
corrigidos: **a barra de status colidia** ("3 alterações" por cima de
"IDE" a 1280 px — a faixa esquerda agora para antes da direita, o caminho
do workspace elide no meio e o git vem antes dos números do índice, que
cedem) e **três `/tmp/…` "caminho ausente" nos recentes do autor** — os
gates abriam pastas temporárias no config real; `verificar_embarcado`,
`verificar_python_debug`, `verificar_micropython_porta` e
`verificar-exercitacao` isolam `XDG_CONFIG_HOME`, e as entradas de `/tmp`
foram tiradas do arquivo do autor. (4) As fatias F1–F8 do `43` §4, cada
uma com a sua medida, esperando a aprovação do autor (§5) — duas decisões
são dele: autosave e o destino do botão duplo Cargo/CMake.

**Medido:** 827 testes, 45 harnesses, 24 gates verdes; as três fotos.
**Não feito:** nenhuma fatia de UI além da status bar — por regra, o
desenho vem antes.

### 7.56 Etapa 2, F1 — a barra principal vira três widgets — 2026-09-18

Decisões do autor ("prossiga"): ordem F1–F8 como no `43`; autosave sim
(entra na F3/F6); um widget Executar com menu. **A barra** (foto 04 do
`43`): `HeaderProjectWidget` (nome + "Cargo + CMake" em cinza + o ponto do
core; o clique abre o menu de projeto — abrir, recentes, fechar — pela
MESMA lista que começa o menu Arquivo: `AppMenuItems.openItems()`, um dono),
`HeaderGitWidget` (branch, ↑↓, o contador de alterações com fundo; o clique
abre o painel Git; some fora de repositório), `HeaderRunWidget` (a
configuração ativa ▾, ▶ Rodar/■, 🐞 Depurar/■, e o **⋯** com o menu Build
inteiro — Configurar CMake, Compilar/Testar por sistema com rótulo,
Análise, Cobertura; um ponto pulsa enquanto build/teste/análise roda, com
o nome no tooltip). Saíram: "Target: host local", os dois pares de botões
Cargo/CMake sem rótulo, o triângulo/✓/⚠ soltos e o "● Cargo + CMake" da
direita — **12 controles → 3 widgets (7 controles)**, a medida da F1.
`ShellController.tabActive(tab)` nasceu como dono da derivação "aba de
baixo visível" (a catraca de duplicação pegou a segunda cópia na hora).

**Medido:** 46 harnesses (`tst_app_menu_items` novo: o menu de projeto e o
de build, por sistema presente); qmllint, fiação, propriedades, alcance,
duplicação, arquitetura, atalhos (52 itens de menu tratados), fiação IPC;
o binário abre; foto antes/depois no mesmo tamanho.
**Não feito, dito:** o modo compacto do trilho (F1 o cita; fica para quando
a largura pedir); o teste prático do autor na tela.

### 7.57 Etapa 2, F2 — a barra de status diz o que está acontecendo — 2026-09-18

`ActiveJobController` (jobs/, NOVO) deriva do `jobsModel` do JobsController
o job vivo mais recente — título, progresso, última linha, cancelável — e a
contagem; `StatusBarJobWidget` (shell/, NOVO) o desenha no centro da barra:
título (clique → aba Jobs), barra de progresso determinada ou o traço que
anda, a última linha em mono, ✕ cancelar (`job.cancel` pelo id — a ponte
expôs `cancelJob`). Sem job, os resumos do projeto voltam. **Os servidores
de linguagem ganharam tela**: `event.lsp.status` só ia para o log (o
pente-fino não pegou porque o C++ o tratava); agora `lspStatusChanged` →
`LspStatusController` (workspace/, NOVO: mapa linguagem → estado) → o chip
`LSP ● 2` / `LSP … cpp` / `LSP ✗ python` (vermelho, o motivo ao pairar). O
git saiu da status bar (mora no widget da barra principal, F1); os quatro
"compilando…/testando…" separados viraram o job único. Foto 05: "Indexar
/home/hugh/KineinVectis" com a barra andando e o ✕, aos 4,5 s da abertura.
No caminho: `AppDomains` bateu em 400 e os tratadores de uma linha viraram
setas; a catraca de duplicação pegou `status === "running"` em dois
controllers (são fatos diferentes — job vivo, servidor rodando — e cada um
ganhou a sua lista de estados em vez do literal).

**Medida da F2:** durante build/teste/índice/configure/deploy a barra mostra
o job e o progresso; nada colide (a faixa esquerda para antes da direita).
**Medido:** 47 harnesses (`tst_status_bar_state` novo); todos os gates QML,
fiação IPC (164 sinais, todos com dono), arquitetura, qmllint; o binário
abre. **Não feito, dito:** a foto a 1024 px; Ln/Col do cursor (entra na F3
com o editor).

### 7.58 Etapa 2, F3 — aba ativa, linha atual, explorer que segue, autosave — 2026-09-18, protocolo 0.123.0

**Aba ativa óbvia** (`EditorTabsBar` reescrita): fundo do editor + borda de
acento em cima + texto primário; a inativa no fundo do painel; o arquivo
modificado mostra ● no lugar do ✕ até o mouse chegar. **O botão "Salvar"
amarelo permanente saiu** (foto 03 do `43` o mediu como o controle mais
chamativo da tela) — Ctrl+S continua, e o **autosave** entrou: decisão do
autor, `SettingsValues.autoSave` (0.123.0, ausente = ligado, toggle em
Configurações), no `EditorPersistenceController`: 2 s de pausa depois da
edição, `flushAutoSave()` ao trocar de aba (antes do `selectTab`, com o
buffer ainda na superfície) e quando o editor perde o foco; só com buffer
sujo; pelo MESMO caminho do Ctrl+S (`fs.write` com `expectedContent` — o
disco mudado por fora é recusado como sempre); o rascunho do
`seguranca/23` continua aos 1,5 s. **A linha atual** no número da calha em
texto primário. **O explorer segue o arquivo ativo**
(`ProjectTreeRevealController`, filho do `ProjectTreeController` que bateu
em 400): abre uma pasta por vez até o arquivo (`fs.list` por pasta, sem
pedir duas vezes a mesma — duas listagens recolheriam o que a primeira
abriu) e o seleciona quando aparece; fora do workspace, nada.

**O que a foto mediu além da F3 — o defeito que manda na F6.** Ao
fotografar, o explorer parava em `crates/kinein-core` sem abrir mais nada.
Instrumentado: a UI PEDIU `fs.list` (id 33) e o core não respondeu por
~20 s; a resposta anterior tinha sido `lsp.semanticTokens`, e o core
**bloqueia o laço inteiro** em cada pedido LSP (`REQUEST_TIMEOUT` 4 s;
`INITIALIZE_TIMEOUT` 15 s) — com o rust-analyzer indexando este
repositório, nada responde enquanto ele sobe: explorer, salvar, git, tudo.
Aos 30 s o explorer completou a cadeia (foto 06b). Não é da F3; é o
primeiro item da F6 (resposta assíncrona do LSP), e passa na frente de
medir cliques.

**Também no caminho:** a pasta `crates/kinein-core/192.168.0.42:` — lixo do
`rsync` falso do teste do espelho (o alvo sem usuário não era traduzido) —
tinha entrado no commit 2a3b168; saiu, e o falso traduz as duas formas.
**Medido:** 827 testes Rust; 49 harnesses (`tst_editor_autosave`,
`tst_project_tree_reveal` novos); gates QML, fiação IPC, arquitetura,
atalhos verdes; `debug-strict` abre. **Não feito, dito:** Ln/Col na status
bar; a foto sem o freeze do LSP.

### 7.59 Etapa 2, F4 — o explorer com o peso certo — 2026-09-18

Pedido direto do autor ("otimiza essa árvore de projeto, o dimensionamento
e etc.; esse ícone de Cargo + CMake reposicionado para um lugar melhor").
O chip amarelo "Cargo + CMake" do cabeçalho do explorer saiu — o widget de
projeto da barra principal (F1) já diz isso, e o chip era o segundo lugar a
dizer o mesmo; o cabeçalho ficou nome + quatro ações. Linhas de 24 → 22 px,
ícone 20 → 16, recuo 14 → 12 por nível. `ProjectTreeRules` (NOVO, puro):
a lista do que é da máquina (`.git .idea .kinein .vscode .cargo
.ruff_cache .mypy_cache .pytest_cache .venv __pycache__ node_modules build
target dist`) e a ordem — pastas do autor, pastas da máquina, arquivos —
que o `ProjectTreeController` aplica ao inserir (`machine` na linha) e o
explorer pinta em cinza (seta, ícone e nome). Foto 07. Harness
`tst_project_tree_reveal` cobre a ordem e o `machine`.

### 7.60 Etapa 2, F6-a — o core não para mais pelo servidor de linguagem — 2026-09-18

O defeito que a F3 mediu (§7.58): o laço do core bloqueava até 4 s por
pedido LSP e 15 s no `initialize` — a IDE inteira muda enquanto o
rust-analyzer sobe. Três correções, todas medidas:

**(1) Respostas ADIADAS.** `RequestOutcome::Deferred` + `Core::
enable_deferred_responses(tx)` + `LoopEvent::Response`: o handler de
`lsp.hover/definition/completion/references/documentSymbols/
workspaceSymbols/semanticTokens` sincroniza o documento e escreve o pedido
(`LspManager::begin_query` → `LspBegun::Pending { reply, finish }`), devolve
o marcador, e uma thread (`Core::defer_lsp`) espera (`LspReply::wait`, 4 s)
e manda a resposta pelo canal — o laço escreve como escreve um evento. Sem
o canal (testes, `run_json_lines`) a espera é inline como sempre: os 74
testes de LSP passaram sem mudar. A contagem de timeouts para o
auto-restart virou compartilhada (`Arc<Mutex>`), e a decisão de reiniciar
fica no laço, no pedido seguinte. **Prova:** `tests/lsp_deferred.rs` — o
servidor falso demora 3 s no hover (`FAKE_LSP_HOVER_DELAY_MS`), o
`lsp.hover` volta `Deferred` na hora, um `fs.list` pedido logo depois
responde em < 100 ms, e a resposta real chega pelo canal com o `id` certo.
**(2) Handshake fora do laço.** `spawn_server` escreve o `initialize` e
volta com o handle `starting`; `lsp/handshake.rs` (NOVO) espera a resposta
numa thread, manda `initialized` + configuração, sobe a thread leitora,
marca `ready` e emite `running` — ou `failed` com o stderr e mata o filho.
O laço espera no máximo 300 ms (`HANDSHAKE_GRACE`) por um servidor recém-
subido; pedidos e `didOpen` antes do `ready` voltam `LspError::Starting`
sem bloquear, e a UI reenvia o buffer ao ver `status: running`
(`EditorEventRouter`). **(3) `syntaxTree.update` de ~0,9 s para 0,05 s.**
No caminho, a instrumentação SEND/RESP mostrou cada `syntaxTree.update`
segurando o laço ~1 s: `utf16_position` varria o prefixo inteiro do arquivo
a cada realce (milhares por atualização). `lang/positions.rs::LineIndex`
(início das linhas, uma vez por atualização; linha por busca binária,
coluna só na própria linha), com o teste que o compara à varredura em todo
offset. Medido pelo core: 0,9 s → 0,175 s (primeiro parse) / 0,05 s
(incremental). **Também:** `RequestOutcome`/`CoreError` saíram para
`outcome.rs` (o `lib.rs` bateu em 500).

**Medido na IDE (mod.rs aberto pela sessão, rust-analyzer real):** antes,
o `fs.list` id 33 esperou ~20 s e a cadeia do explorer completou aos 30 s;
depois, todos os pedidos respondem em ≤ 300 ms durante a abertura e a
cadeia completa aos 6 s (foto 08). 829 testes Rust (+2), 49 harnesses, os
24 gates verdes (QEMU, debugpy, exercitação, clangd cross incluídos).

**Não feito, dito:** rename/codeActions/workspaceEdit continuam síncronos
(raros; precisam do resultado no laço); o `workspace.open` leva ~1,4 s
(medido; a investigar na F6-b junto com os tempos de clique → feedback);
`didOpen` enfileirado durante o handshake não existe — a UI reenvia ao
`running`.

### 7.61 Etapa 2, F5 — Problems com o próximo passo — 2026-09-18

`ProblemNextStep` (jobs/, regra PURA, com harness): dado o problema, o
rótulo e o destino do botão — fonte `lsp` → **Ações** (abre o arquivo na
linha e pede as code actions, o Alt+Enter); mensagem que fala de
`compile_commands`/CDB/configurar → **Configurar CMake** (a mesma ação da
faixa de saúde); "não encontrado"/"not found"/"instale"/"ausente" →
**Ferramentas**; o resto não ganha botão (inventar um passo é pior que não
ter). `ProblemsPanel` desenha o botão à direita, sempre visível quando
existe; o caminho é `nextStepRequested` → `BottomPanelHost` →
`ShellWorkspaceHost` → `Main` (code actions pelo editor; o resto pelo
`healthActionRequested` que a faixa já usa). **Infra de medição:**
`KINEIN_STARTUP_COMMANDS=build.run` (ids da paleta) executa comandos depois
de o workspace abrir — foi assim que a foto 09 saiu: um projeto Rust com
`let x: i32 = "texto"`, `kinein-vectis <pasta>` + build automático →
Problemas (3) com dois **Ações** (os do rust-analyzer) e o erro do cargo
sem botão. A foto também mostrou o que fica para depois: o mesmo erro
aparece duas vezes (build + LSP) e a barra de status diz "toolchain:
Clang++ · Ninja" num projeto Rust.

**Medido:** 50 harnesses (`tst_problem_next_step`); gates QML, duplicação
(as derivações `kind === "directory"` e `source === "lsp"` ganharam dono),
arquitetura, fiação IPC verdes. **Não feito, dito:** a contagem de testes
na aba (Testes 12/14) e a deduplicação build/LSP.

### 7.62 Etapa 2, F6-b — a resposta ao gesto, medida — 2026-09-18

A tabela que a F6 pedia, medida pelo core (build debug, esta máquina) e
pela IDE headless (`KINEIN_PERF`, `KINEIN_STARTUP_COMMANDS`):

```text
gesto / pedido                              antes            depois         como
workspace.open (qualquer projeto)           1,4 s            25 ms          detected_tools() de registro vazio
                                                                            fazia 62 `--version` SINCRONOS;
                                                                            agora so' presenca (path), e o
                                                                            scan/detect traz a versao
tools.detect (a UI pede na abertura)        1,4 s PARANDO    adiado, 1,6 s  defer_work: thread + canal;
                                            tudo atras dele  sem parar nada os pedidos seguintes nao esperam
syntaxTree.update (470 linhas, a cada       0,9 s            0,05 s         LineIndex (F6-a)
  pausa de digitacao)
lsp.* durante o rust-analyzer subir         ate' 20 s mudo   <= 300 ms      respostas adiadas + handshake
                                                                            em thread (F6-a)
explorer segue o arquivo (cadeia fs.list)   30 s             6 s            consequencia dos dois acima
primeiro frame com workspace por argumento  —                718-817 ms     3 amostras, offscreen, debug
```

O que passa de 100 ms ainda: o primeiro frame (build debug; o release abre
em ~300 ms) e a primeira sincronia com o servidor de linguagem — que agora
acontece sem parar o resto.

**Também na fatia, vindos da foto 09:** o mesmo erro aparecia duas vezes
em Problems (o do `cargo build` e o do rust-analyzer) — `ProblemRules.
isDuplicate` (mesmo arquivo e linha, uma mensagem começando com a outra)
descarta o segundo; e a barra de status dizia "toolchain: Clang++ · Ninja"
num projeto só de Cargo — `ToolchainController.summary(buildSystems)` resume
os papéis dos sistemas que o projeto TEM ("Cargo · automática"; num híbrido,
"Cargo · G++ · Ninja"). Foto 10. `ProblemNextStep` virou `ProblemRules`
(as duas regras puras dos problemas).

**Medido:** 829 testes Rust; 50 harnesses (`tst_problem_rules`,
`tst_toolchain` estendidos); gates QML, arquitetura, fiação IPC verdes.
**Não feito, dito:** medir clique → feedback com um usuário real (offscreen
não clica); o release-hardened não foi remedido nesta fatia.

### 7.63 Etapa 2, F7 — a tela inicial: começar em 1 clique, ambiente em 1 linha — 2026-09-18

Foto 11a (antes) e 11 (depois), com uma lista de recentes montada para a
foto (`XDG_CONFIG_HOME` isolado: dois projetos reais, um fixado, um
caminho ausente). O que mudou:

- **O último aberto em destaque, Enter abre.** `RecentWorkspacesController`
  ganhou `visibleWorkspaces`, `highlightedIndex` (o de maior `lastOpenedAt`
  entre os visíveis — não o primeiro da lista, porque os fixados vêm
  antes), `moveHighlight(±1)`, `openHighlighted()`. A `StartScreen` toma o
  foco quando não há workspace e trata ↑ ↓ Enter; a linha em destaque diz
  "Enter abre".
- **"Caminho ausente" some da lista, com desfazer.** Os recentes cujo
  caminho não existe ficam ocultos (não são apagados do disco — um disco
  externo pode voltar); um rodapé "1 recente sem caminho foi ocultado ·
  Desfazer" (`restoreMissing`) os traz de volta, com o × de sempre para
  remover de vez.
- **O ambiente numa linha.** "Ambiente: 37 de 62 ferramentas detectadas ·
  Ver · Redetectar" — a grade de dez ferramentas saiu da primeira dobra;
  "Ver" abre a aba Ferramentas, que já é a lista inteira. Na foto 11a o
  cartão de ambiente cortava abaixo da dobra a 1280×800; na 11 a tela
  inteira cabe com folga.
- **A `StartScreen` fala com o controller**, como os hosts fazem: a
  `RecentWorkspacesCard` recebe `controller` em vez de cinco sinais
  reencaminhados (o `ShellWorkspaceHost` perdeu dez linhas; 384/400).

**Defeito achado pela foto, fora do desenho:** o stderr da IDE avisava
`Connections: Detected function "onIdentifyRequested" … no signal of the
target matches`. O sinal `identifyRequested` é do controller filho
`EmbeddedController.identity` (desde a divisão da E5), e o
`EmbeddedRequestRouter` o escutava no pai — o `serial.identify` nunca saía
pelo botão "Identificar" da aba Serial. Corrigido (uma `Connections` no
filho) e **o gate `verificar-binario-abre.sh` ganhou a terceira pergunta**:
o stderr até o primeiro frame não pode ter aviso do motor QML de fiação
quebrada. Provado por mutação (a `Connections` errada de volta reprova e
cita `EmbeddedRequestRouter.qml:40`). O gate de fiação IPC não vê essa
classe: ele confere que `onX` existe em algum lugar, não em QUE target.

**Medido:** 50 harnesses (`tst_recent_workspaces` estendido: oculto,
destaque, Enter, desfazer, lista vazia); gates QML, lógica QML, fiação
IPC, arquitetura, binário-abre (sem aviso) verdes. **Não feito, dito:**
Enter/↑/↓ conferidos pela regra pura, não por tecla real (offscreen não
digita na janela); a identidade Espressif não foi reexercitada na placa
(a regra do autor: nunca gravar; identificar é leitura, fica para ele).

### 7.64 Oito domínios sem resposta na tela por seis dias — o elo solto do despacho C++ — 2026-09-18

Achado ao fotografar os quatro painéis de ambiente para o desenho da F8
(`KINEIN_STARTUP_COMMANDS=container.list`, `probe.list`…): Containers dizia
"procurando o motor…" e Embarcados "lendo o projeto… / procurando…" aos
15 s, enquanto o core respondia `container.status` em 2,4 s e `container.
list` em 10 ms (medido com o core na mão e com um `kinein-core` envolvido
por um script que registra o IPC nos dois sentidos — `KINEIN_CORE_BIN`;
o registro está em `DocsPrivate/Codex/evidencias-2026-09-18-etapa2-f8/
ipc-container-sem-resposta-na-tela.log`: as três respostas chegam aos
4,2 s e a tela não muda).

**A causa.** A ponte C++ despacha respostas por uma CADEIA de funções
(`dispatchResult` → workspace → cmake → configAction → toolchain → library
→ dataSource → grafana → **probe → buildSize → serial → container → index →
python → coverage → remote**). Quando a simulação saiu do produto
(2026-09-12, commit 6733c20), o fim de `dispatchGrafanaResult` — que
chamava `dispatchSimResult`, que chamava `dispatchProbeResult` — virou
`return false;`. A partir dali, toda resposta dos oito domínios em negrito
chegava ao `CoreClient`, era registrada no log da IDE, e **morria antes do
sinal**: nenhum `*Resolved` era emitido, nenhum controller mudava de
estado. Seis dias, doze fatias e 24 gates verdes depois.

**Por que nenhum gate viu.** O gate de fiação IPC (§7.54) confere que cada
método tem cliente, cada evento tem tratador, cada sinal C++ tem ouvinte e
cada sinal QML tem `onX` — e tudo isso continuava verdade: o sinal
`containersResolved` existia e era ouvido; só que ninguém o emitia mais. Os
testes de despacho do core provam o core; os harnesses provam os
controllers com dados injetados; o binário-abre prova o primeiro frame,
antes de qualquer painel. A classe "a resposta chega e o elo do meio não a
passa adiante" não tinha pergunta.

**O que entrou.** `dispatchGrafanaResult` volta a terminar em
`dispatchProbeResult` (uma linha, com o comentário do porquê); e o gate de
fiação ganhou a **quinta pergunta**: todo `bool CoreClient::dispatch*Result(`
e `handle*Notification(` definido tem de ser chamado em algum lugar de
`ui/src` — sem lista de exceções, porque um elo sem chamador é resposta
perdida por definição. Provado por mutação: o `return false` de volta
reprova com `elo do despacho C++ sem chamador: dispatchProbeResult`.

**O que isto muda no que foi dito antes.** As fatias que passaram por esses
domínios desde 2026-09-12 — E1/E3/E5 serial, P4 arquivos na placa, P6
remoto (fatias 1 e 2), P5 cobertura, o índice, a cadeia Python — foram
provadas **no core e nos controllers** (gates e harnesses), e os registros
dizem isso; mas a afirmação implícita "e a tela mostra" era falsa desde
então para respostas diretas (os EVENTOS `event.*` seguiam outro caminho,
`handleNotification`, e continuavam chegando — por isso a status bar
mostrava o índice e o Jobs mostrava os jobs). O C2 "lido e reescrito no
ESP32 real" da §7.47 foi pelo core e pelo gate, não pelo painel. A
§4.2.3(c) — "só o autor, com a placa na mão" — ganha peso: a exercitação
dos painéis na IDE aberta é o que teria pego isto no dia.

**Medido:** fiação IPC verde com a quinta pergunta; foto dos quatro painéis
com as respostas chegando (a F8 parte delas); Clang-Tidy limpo
(`cpp-fio-despacho.log`).

### 7.65 Etapa 2, F8 — os painéis de ambiente com a mesma forma — 2026-09-18

Fotos 13a–13d (depois) contra 12a–12d (antes), nas mesmas condições
(`KINEIN_STARTUP_COMMANDS`, este repositório, config isolada, 8 s). O
desenho é o do 43 §8; o que entrou:

- **Três componentes comuns** em `ui/qml/components/`: `KvPanelFrame`
  (a moldura: centralizada, dispensa por clique fora, e o conteúdo ROLA
  quando não cabe — dois modos: o painel preenche e rola por dentro, ou
  pede a altura e a moldura rola), `KvPanelHeader` (título, subtítulo de
  UMA linha com elide, a ação primária à direita, o ×), `KvVerdict` (a
  faixa "medindo…" / verde / vermelha / cinza-neutra com o que o core
  mediu, em mono). Mais `KvDataGrid` + `GridRules` (a grade: cabeçalho
  fixo, largura por conteúdo com piso 56 / teto 320, a sobra distribuída
  quando cabe, as largas encolhem até 120 antes de rolar, `null` em
  itálico; a regra é pura e tem harness, `tst_grid_rules`).
- **Os quatro painéis** (banco, remoto, embarcados, containers) começam
  com a mesma primeira linha e a ação primária de cada um no mesmo lugar:
  Testar · Sondar · Procurar sonda e portas · Atualizar. Fechar saiu dos
  rodapés (é o ×). O veredito vem ANTES do formulário no banco e no
  remoto (`DataSourceVerdict` 77 → 56 linhas e `RemoteVerdict` 106 → 85,
  os dois sobre o `KvVerdict`); o motor dos containers e a busca de sonda
  dos embarcados viraram a mesma faixa.
- **O Embarcados cabe:** o painel declara `implicitHeight` e a moldura
  rola — na foto 12c ele vazava por cima do editor e da barra de status;
  na 13c, a 800 px, a moldura vai de 28 a 760 e o resto rola.
- **A grade** substitui a caseira do `DataSourceQuery` (212 → 138
  linhas) e lista as imagens de container (repositório · tag · tamanho ·
  criada; o tag de 64 hex encolhe até 120 px e as quatro colunas cabem —
  foto 13d). Os hosts: `DataSourcePanelHost` 78 → 51, `RemotePanelHost`
  80 → 56, `EmbeddedPanelHost` 64 → 27, `ContainerPanelHost` 51 → 25.

**Não feito, dito:** a lista de containers (cinco ações por linha) e as
portas seriais (monitor/identificar por porta) NÃO viraram grade — a
grade comum não tem coluna de ações, e "selecionar a linha e agir na
barra" é outra interação, a decidir com o autor; o Grafana
(`GrafanaPanelHost`) e o Setup/Biblioteca ficaram na moldura antiga (cabem
na mesma forma numa fatia de meia hora); o `container.status` de 2,4 s
síncrono (§7.64) fica como resto; nenhum painel foi exercitado com
servidor/placa reais (§4.2.3-b).

**Medido:** 51 harnesses (+`tst_grid_rules`; `tst_container` passa a
provar `imageRows`); gates QML (lint, fiação, propriedades, duplicação,
alcance, lógica), fiação IPC (com a quinta pergunta), arquitetura,
atalhos, docs, links, binário-abre (732 ms, sem aviso) verdes; sem
mudança em Rust; o C++ mudou só na §7.64 (Clang-Tidy limpo lá).

**Com a F8, a Etapa 2 fecha o que o 43 §4 desenhou (F0–F8).** O que
resta dela é o fechamento da §4.2.2 (dívidas pequenas + o que só o autor
mede), e a Etapa 3 é decisão do autor (§4.2.5).

### 7.66 Banco — descobrir e criar; o chip que acendia dois — 2026-09-18 (noite), protocolo 0.124.0

**O teste do autor** (primeiro uso real da IDE depois da Etapa 2) disse
três coisas do banco: clicar em MongoDB acendia também PostgreSQL; não dá
para criar um banco nem descobrir um; "a maior parte aparentou ser apenas
visual". A primeira era uma linha (`active: !arquivo` é verdadeiro para o
Mongo — um motor, um chip). As outras duas são esta fatia.

**Contrato (0.124.0, `arquitetura/03`).** `datasource.discover` — o que
responde NESTA máquina, medido e nunca deduzido: o socket do PostgreSQL
de distro (`/var/run/postgresql/.s.PGSQL.5432`) ou a porta 5432/27017 no
loopback (200 ms), os containers com imagem de banco pelo motor que o
painel de Containers já usa (parados também, ditos como parados; a porta
publicada vira a do perfil), os `.sqlite/.db` do workspace com o cabeçalho
real (`.kinein/kinein.db`, que é da IDE, fica de fora). Cada achado traz o
perfil pronto. Roda adiado (`defer_work`): bater em porta não segura o
laço. `datasource.create`: `sqliteFile` cria `data/<nome>.sqlite` com
cabeçalho (recusa se existe) e salva o perfil; `containerServer` sobe
`postgres:16`/`mongo:7` em `127.0.0.1:<porta>` como job `High` — o comando
volta na resposta e aparece na tela antes e depois; `trust` só no loopback,
porque a IDE não guarda senha. Banco DENTRO de um PostgreSQL: `CREATE
DATABASE` pelo `datasource.query` confirmado; ao responder, o perfil clonado
com o banco novo é salvo (`<perfil>-<banco>`). MongoDB cria na primeira
escrita — a tela diz, não finge.

**Tela.** A coluna da esquerda virou "Nesta máquina" (descobertos, com ↻)
+ "Salvos"; abrir o painel descobre, como Containers. Clicar num descoberto
põe o perfil no formulário (salvar é do autor). **Novo banco** abre a caixa
(`DataSourceCreateBox`): SQLite (arquivo) · PostgreSQL em container ·
MongoDB em container · Banco no servidor, com o comando exato em mono
antes do clique. `DataSourceDiscoveryController` é filho do
`DataSourceController` (dono próprio; a `Connections` do roteador aponta
para o filho — a lição da §7.63).

**Provado:** 4 testes de despacho (`tests/datasource_discover.rs`: um
`podman` falso que lista um container de PostgreSQL parado com porta
publicada e registra o `run`; SQLite real no workspace e lixo em `target/`;
criar/recusar/rever no discover; o comando pinado e o perfil salvo no
evento; sem motor → `TOOL_NOT_FOUND`) + 8 unitários; harness
`tst_datasource_discovery` (adotar sem salvar, SQLite pronto, job com
comando, `CREATE DATABASE` → clone salvo, falhas no dono certo). Foto 14:
o painel num workspace com `data/app.sqlite` — descoberto com 8 KB.

**Medido:** 840 testes Rust (+11); 52 harnesses (+1); 162 métodos, 58
eventos; gates verdes (duplicação: `engine===sqlite` virou "sem host é
arquivo"; `defaultPort` por mapa). **Não feito, dito:** PostgreSQL/Mongo
reais não subiram no gate (o `run` é o falso — o real é um clique do autor
com Podman, e baixa ~150 MB); abas/histórico/exportar da consulta seguem
como resto; `container.status` de 2,4 s idem.

### 7.67 O painel Git sem nomes de arquivo por quinze dias — seis âncoras perdidas, e a regra — 2026-09-18 (noite)

O autor pediu uma HUD de Git "à JetBrains". Antes de desenhar, a foto do
painel de hoje (15a/15b): em **Mudanças**, três checkboxes **sem nome de
arquivo**; em **Histórico**, o hash desenhado **por cima** de "há 56 min".
Não era desenho — era defeito: o refactor `dbdafa0` (2026-09-03, "painel e
controller cortados por responsabilidade") apagou seis linhas
`anchors.left/right: parent.*` e deixou as `anchors.*Margin` órfãs. Uma
margem sem a âncora é um no-op silencioso: o item fica em x=0, e o que se
ancora nele (o texto do caminho entre o checkbox e o chip "diff") ganha
largura negativa e some. `qmllint` não vê; o binário abre; nenhum harness
instancia o delegate. Quinze dias.

Consertadas as seis (`GitChangesList` ×2, `GitHistoryList` ×2,
`GitBranchMenu`, `DebugInspector`), e **o gate `verificar-qml-propriedades`
ganhou a regra**: `anchors.<lado>Margin` num bloco sem `anchors.<lado>`
(nem `fill`/`centerIn`/`*Center`) reprova — provado por mutação
(`GitHistoryList.qml:95`). Foto 15d: os caminhos de volta.

Lição para a lista da §4.2.4: "o que os gates não cobrem" tinha "telas
que carregam depois do primeiro frame" — este é o caso concreto; a regra
estática fecha ESTA forma (margem órfã), não todas.

### 7.68 A execução é uma aba de terminal; a "Execução" saiu — 2026-09-18 (noite), protocolo 0.125.0

**O autor, no teste:** "a aba 'Execução' dentro de 'Terminal' não faz
sentido: já temos o terminal integrado, não precisamos de mais nada para
executar, e já temos os botões de atalho no canto superior direito."
Estava certo pelos dois lados: a "Execução" era um executor por pipes,
sem TTY (o próprio `run.rs` dizia "this is NOT a full terminal"), com um
campo de stdin próprio e um "limpar" próprio — um segundo terminal pior,
ao lado do bom.

**Core (0.125.0).** `run.start`/`run.script` seguem resolvendo O QUE rodar
(configuração ativa, lançador padrão por tipo, Python/MicroPython com a
porta) e abrem o comando numa sessão de terminal (`sh -lc`, ou argv
direto), como `container.open`/`serial.monitor` já faziam; a resposta traz
`terminalId`. A saída é `event.terminal.render`; o fim,
`event.terminal.closed { exitCode }`. `run.stop` fecha a última execução.
Saíram `RunManager`, `run.stdin`, `event.run.*` (−1 método, −3 eventos).
Fechar o workspace fecha tudo. Os testes que liam `event.run.output` (run,
flash, frameworks, MicroPython) passaram a ler o render da sessão por um
helper comum (`tests/mod.rs::terminal_run_until_closed`, que desenrola
linhas mais largas que as 80 colunas); os gates `verificar-exercitacao` e
`verificar_micropython_porta` idem.

**UI.** O ▶ abre uma aba `▶ cargo run` (o nome é o comando; bolinha verde
enquanto roda); ao terminar **a aba fica**, com `✓` ou `✗ 101` no nome,
para o autor ler — fechá-la depois é local. Saíram `RunPanel`, o chip
"Execução", o "limpar", `run.stdin` da ponte e do menu. De quebra, dois
defeitos vistos na foto: um gesto abria **dois** terminais (o painel e o
controller pediam cada um o seu — agora o dono é só o `RuntimeController`,
e clicar na aba Terminal passa por ele); e a limpeza de abas por
`terminalActive=false` (feita para o crash do core) apagava a aba da
execução recém-terminada — agora só as sessões vivas saem.

**Medido:** 837 testes Rust (−3: os do `RunManager`); 52 harnesses
(`tst_run_device` reescrito: aba, título, desfecho, fechar local); 161
métodos, 55 eventos; gates QML/fiação/arquitetura/atalhos/docs verdes;
python-debug e exercitação verdes com o contrato novo; foto 15e (a aba
`▶ cargo run ✗ 101` com o erro real do cargo em vermelho). Manual §5
reescrito. **Não feito, dito:** a HUD do painel de baixo (abas com
contagem, densidade) é a fatia seguinte; o Run widget da barra continua
igual.

### 7.69 O painel de baixo: a faixa de abas na linguagem das F1–F8 — 2026-09-18 (noite)

Na mesma passada da §7.68 (o autor: "essa parte da IDE deve ter a UI/HUD
reformulada"). Antes (foto 15c): dez chips iguais, cada um com borda,
texto 10 px em negrito, só Problemas com contagem, "Execução" e "limpar"
soltos abaixo. Depois (foto 15f): chips **sem borda**, a ativa como
pílula (`surfaceSelected`, texto primário DemiBold), as outras em texto
secundário; a **contagem de cada painel** ao lado do nome, só quando há o
que contar — Problemas `N` (vermelho), Testes `passou/total` (verde ou
vermelho; `JobsController.testsPassed/testsFailed/testsBadge`, provado no
`tst_tests_output`), Jobs `n` em curso (`ActiveJobController.
runningCount`), a bolinha do Terminal enquanto roda; a **ordem do uso
diário** (Terminal · Build · Problemas · Testes · Jobs · Debug · Git ·
Busca · Ferramentas · IDE); o **×** à direita esconde o painel
(`hideRequested` → `showBottomPanel=false`). A regra do que cada aba
mostra mora numa função só (`BottomTabBar.badgeFor`). 24 px de altura.

**Não feito, dito:** a primeira linha de cada painel (o `KvPanelHeader`
dos painéis de ambiente) não entrou nos painéis de baixo — são densos e
a linha custaria altura; fica para quando o autor vir a faixa nova. O Git
é a fatia seguinte (HUD própria).

### 7.70 A HUD do Git — primeira fatia (save point) — 2026-09-18 (noite), protocolo 0.126.0

**O autor:** "melhorar essa parte visual do git de versionamento,
histórico e afins; bem provável de ser necessário criar uma HUD única para
o Git com base no funcionamento visual das IDEs JetBrains." E, com a
sessão longa: "faz um save point — finaliza a parte atual e faz um
commit/documentação antes de prosseguir". Este é o save point: a HUD
funciona de ponta a ponta; o que falta está dito abaixo.

**Antes (fotos 15a/15b/15d):** uma lista de mudanças plana, o diff num
diálogo, o histórico como lista de linhas, o commit numa linha, nada de
refs nem de grafo, o painel de 220 px.

**Contrato (0.126.0).** `git.log` traz `parents` e `refs` por commit;
`git.commit { amend? }`. Testes: `parse_log` com merge e refs; amend de
mensagem sem nada staged (legítimo) e commit novo sem staged (recusado).

**Tela (fotos 16a/16b).** O painel Git é DUAS colunas: à esquerda a lista
(Mudanças **agrupadas por pasta**, com o nome do arquivo; ou o Histórico
com o **grafo** — raias pela regra pura `GitRules.lanes`, merge como anel
— e os **refs como chips**: `main` no HEAD em âmbar, `origin/*` em cinza,
tags em roxo) e o commit embaixo (mensagem, **Amend** com o aviso
"reescreve o último commit", **Commit e Push**, Commit); à direita o
**inspetor** (`GitInspectorController`, filho do `GitController`): um
clique numa mudança mostra o diff dela (`GitPatchView`, linhas coloridas
com fundo) — dois cliques abrem o arquivo; um clique num commit mostra
sha, autor · idade, os refs, os **arquivos com +/−** na grade comum
(`KvDataGrid`) e o patch. O diálogo de diff ficou só para o pedido do
editor. O painel de baixo sobe para 340 px quando a aba Git está ativa.
`GitPanel` fala com o controller (o `BottomPanelHost` perdeu 11
propriedades e 13 sinais de passagem; o `ShellWorkspaceHost` 41 linhas).

**Provado:** `tst_git_rules` (raias em linha reta, merge que abre e fecha,
sem pais, arquivos do patch com +/−, classe da linha, pasta, refs, o
delegate antes do modelData); gates QML/fiação/arquitetura/atalhos verdes;
fotos headless com este repositório (24 mudanças em 4 pastas; 11 commits
com `main`, `origin/main`, `origin/HEAD`).

**Não feito, dito (a fatia seguinte da HUD):** stage por pasta (o
checkbox da seção); busca/filtro no histórico (por branch, autor, texto);
o grafo desenha as raias e os pontos, mas ainda **não as curvas** de
merge/ramo; o branch no widget da barra (F1) segue abrindo o menu antigo;
"Commit e Push" faz o push quando o status do commit volta — sem
confirmação extra; cherry-pick/revert/reset não existem; nada disto foi
clicado por uma pessoa (offscreen). O `GitController` está em 398/400.

### 7.71 HUD do Git, fatia 2 (a), (b) e (c) — stage por pasta; filtro por texto e por branch; as curvas do grafo — 2026-09-18 (noite), protocolo 0.127.0

Os dois primeiros itens do desenho do `43` §9.4, um commit cada:

- **(a) Stage por pasta** (`0dfe956`): a seção de cada pasta na lista de
  mudanças ganhou o checkbox — cheio quando todas as mudanças da pasta
  estão staged, meio quando algumas, vazio quando nenhuma; o clique faz
  stage (ou unstage) de todos os caminhos da pasta pelo `git.stage`/
  `git.unstage` que já aceitavam vários. `GitRules.folderState(model,
  folder)` é a regra (harness); `GitController.stageFolder` são três
  linhas. Sem contrato.
- **(b) Filtro do Histórico** (este commit): um campo acima da lista
  filtra **localmente** por mensagem, autor ou prefixo de sha
  (`GitRules.matchesFilter`; o grafo é calculado sobre todos e só a
  exibição filtra); o chip **HEAD** abre a lista de branches e pede ao
  core `git.log { ref }` (0.127.0) — o histórico de outro branch sem
  trocar de branch. O handler valida o `ref` (nome de branch/tag; `--all`,
  `a..b`, espaço → `INVALID_PARAMS`); inexistente é erro do git. Teste de
  despacho com dois branches; `tst_git_history` (raias, refs, filtro
  local sem pedido, troca de ref com pedido, clear). `GitHistoryController`
  guarda `allEntries/logRef/filterText` e refaz o modelo.

- **(c) As curvas do grafo** (mesmo commit): `GitRules.lanes` devolve
  também as **arestas** de cada linha para a de baixo (`edges: [{from,
  to}]` — as raias que passam direto e as que o commit liga aos pais); a
  vista desenha num `Canvas` por linha: reta quando a coluna é a mesma,
  Bézier quando muda; a raia que sai do commit em âmbar, as que só passam
  em cinza. O `git log` passou a `--topo-order` (filhos sempre antes dos
  pais — a ordem por data intercalava commits do mesmo segundo e punha um
  pai acima do filho). Foto 16c: um repositório temporário com um merge —
  a curva do `feature`, o anel do merge, os chips `main`/`feature`/`v0.1`.

**Medido:** 839 testes; 54 harnesses; gates verdes; fotos 16c/16d.
**Não feito:** (d) o branch pela barra, (e) as confirmações — `43` §9.4.

### 7.72 Mais âncoras perdidas — a lista de variáveis do depurador sem largura nem altura; e a regra que pega a impressão digital — 2026-09-18 (noite)

Ao mexer no `GitBranchMenu` para a HUD do Git, a mesma marca da §7.67:
linhas de `anchors` **mais recuadas que as irmãs** — o rastro de um
refactor que apagou a linha de cima e deixou a seguinte torta. Uma
varredura por essa marca achou: no `GitBranchMenu`, a `ListView` de
branches e a linha de criar branch **sem `anchors.left`** (largura zero —
o menu abria vazio); no `DebugInspector` (split de 2026-09-03, `527a728`),
a lista de **frames sem `anchors.bottom`** e a de **variáveis sem
`anchors.right` nem `anchors.bottom`** — o painel de variáveis do
depurador nunca teve tamanho desde então. Nenhuma das quatro dispara a
regra da §7.67 (não são margens órfãs). Consertadas, e o gate
`verificar-qml-propriedades` ganhou a segunda regra: **um binding mais
recuado que o irmão completo de cima reprova** (provado por mutação no
`GitBranchMenu`); varredura da árvore inteira: zero achados depois dos
consertos. A lição fica dita: um refactor que "só move código" pode
apagar âncoras sem que nada reclame — a foto e a marca de indentação são
as duas redes que existem.

### 7.73 HUD do Git, fatia 2 (d) e (e) — o branch pela barra; as confirmações — 2026-09-18 (noite)

Fecham o desenho do `43` §9.4:

- **(d)** No widget de Git da barra (F1), o **nome do branch** virou um
  alvo próprio (sublinha ao pairar): o clique abre a aba Git com o menu
  de branches (trocar, criar) — `HeaderGitWidget.branchMenuRequested` →
  `TopHeaderBar` → `ShellHeaderHost` (`showTab("git")` +
  `gitController.openBranchMenu()`). O resto do widget segue abrindo/
  fechando a aba. Foto 16e: o menu, que até a §7.72 abria sem largura.
- **(e)** "Commit e Push" pede o **segundo clique**: o primeiro arma
  ("vai enviar para origin/<branch> — clique de novo", o botão vira
  primário "Confirmar Commit e Push"); editar a mensagem ou mexer no
  Amend desarma. Amend com o HEAD já enviado (`aheadCount === 0`) troca o
  aviso para vermelho: "reescreve um commit JÁ ENVIADO — vai exigir push
  forçado". Sem contrato; só a `GitCommitBox` e duas propriedades no
  `GitPanel`.

**A fatia 2 da HUD do Git está completa como desenhada** (a–e). Fica dito
o que não entrou em nenhuma das duas fatias: cherry-pick/revert/reset;
stash com lista; o clique real do autor.

### 7.74 Fechamento da Etapa 2, item 1 — Ln:Col na status bar — 2026-09-19

A dívida dita na F2 e na F3: a posição do cursor. `EditorSurfaceBridge.
cursorSummary` ("30:2"), recalculada 80 ms depois de o cursor parar
(contar quebras até o cursor é O(n); não vale a cada tecla), exposta por
alias no `EditorController` e mostrada à esquerda do LSP na
`WorkspaceStatusBar` (mono, muted; vazia sem arquivo). Foto pela
`KINEIN_PERF_TYPING` (o harness de digitação abre um arquivo e digita na
linha 30): `30:2`. **Nota de método:** para desfazer as cinco teclas que
o harness digitou no `GitRules.qml` usei `git checkout -- <arquivo>` — a
regra proíbe; o arquivo não tinha diff meu pendente (conferido no
`status`), nada se perdeu, e fica registrado como o atalho errado.

### 7.75 Fechamento da Etapa 2, itens 2 e 3 — rename/code actions fora do laço (a continuação); `container.status` adiado — 2026-09-19

As duas últimas classes de pedido que ainda seguravam o laço:

- **A continuação** (`Core::defer_then`, `LoopEvent::Continue`): rename e
  code actions precisam do `Core` DEPOIS da resposta do servidor (a
  transação do `workspaceEdit` valida versões e guarda o plano; as code
  actions juntam vários servidores e guardam as cruas para o `apply`). A
  espera vai para uma thread; o fecho volta ao laço como uma closure com
  `&mut Core`, pela mesma ponte das respostas adiadas, e a resposta sai na
  ordem em que o fecho rodou. O `LspManager` ganhou `begin_rename/
  rename_plan` e `begin_code_actions/finish_code_actions`; o servidor falso
  ganhou um `rename` lento (`FAKE_LSP_RENAME_DELAY_MS`); o teste prova:
  Deferred na hora, `fs.list` em < 500 ms, a continuação chega em ~3 s e,
  rodada com o Core, devolve a prévia com `transactionId` e o novo nome.
  Sem o canal (os outros testes), tudo inline como antes — os 74 testes
  de LSP não mudaram.
- **`container.status`** (2,4 s de `podman info` + `version`, §7.64) vai
  por `defer_work`, como o `tools.detect`.

`arquitetura/04` §6 descreve as três formas (adiar a resposta, adiar o
trabalho, a continuação). **Medido:** 840 testes; gates verdes. Fica
síncrono só o que é escrita local (`workspaceEdit.apply/cancel`).

### 7.76 Fechamento da Etapa 2, item 4 — Grafana, Instalar ferramentas e Bibliotecas na moldura comum — 2026-09-19

Os três painéis que a F8 deixou de fora entram no `KvPanelFrame`: os hosts
perderam a moldura escrita à mão (`LibraryPanelHost` 61 → 34 linhas,
`SetupPanelHost` 48 → 27, `GrafanaPanelHost` 61 → 27; o Grafana mantém a
altura pelo que há para mostrar). Sem mudança nos painéis em si; fotos
headless dos três com o mesmo chrome dos quatro da F8. Fica dito: o
`KvPanelHeader` (a primeira linha comum) não entrou nestes três — cada um
tem o próprio cabeçalho; unificar é meia hora quando o autor os vir.

### 7.77 Fechamento da Etapa 2, item 5 — o trilho lateral compacto/expandido — 2026-09-19, protocolo 0.128.0

O que a F1 prometeu e não fez: o trilho com os **rótulos ao lado dos
ícones**. `SideRail.expanded` (52 → 168 px; o rótulo é o tooltip até o
primeiro parêntese, ou um `label` próprio — "Grafana"); o chevron do pé
alterna ("›" / "‹ recolher"); no modo expandido o tooltip cala. A escolha
persiste em `SettingsValues.railExpanded` (0.128.0), lida pelo
`ShellController.applySettings` **independente** do resto do layout salvo
(o `hasPersistedLayout` não a gaba); `toggleRail` persiste pelo mesmo
`layoutSaveRequested`. Harness `tst_shell_functional` estendido. Foto 17
(`imagens/prints/2026-09-19-fechamento/`): o trilho aberto com dez rótulos.

### 7.78 Fechamento da Etapa 2, item 6 — a foto a 1024 px — 2026-09-19

`KINEIN_SCREENSHOT_SIZE=<largura>x<altura>` redimensiona a janela antes da
foto (mínimo 640×400; sem a env, nada muda). A 1024×700 durante um `cargo
build` (foto 18): a status bar cede como a F2 desenhou — o caminho do
workspace elide, o job "Cargo Build" com a barra fica no centro, `1:1`,
IDE e core à direita; a faixa de abas do painel de baixo **batia no ×**
("IDE" por cima do botão) — agora a faixa pára antes das ações da direita
e rola se não couber (a última aba, IDE, fica sob rolagem a 1024). O que
ainda cede feio: a mensagem do job ("Compil…") — o `StatusBarJobWidget`
prefere a barra ao texto; fica dito.
