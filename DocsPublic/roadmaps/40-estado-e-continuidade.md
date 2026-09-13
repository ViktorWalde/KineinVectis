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
> **COMECE POR AQUI ao retomar.** Ele substitui o
> [`38`](38-divida-restante-e-continuidade.md) nesse papel; o 38 vira registro
> de como a fila estava quando a dívida foi paga.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada seção carrega o comando.

## 1. O estado, em números

```bash
bash scripts/verificar.sh                 # 23 verificacoes
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
protocolo   0.105.0
testes      701 Rust + 32 harnesses QML   (2026-09-13; 2026-09-12 noite: a simulacao saiu)
metodos     138 IPC roteados, 47 eventos
dominios    34, e os 34 documentados no arquitetura/03
catraca     1 arquivo em debito
gate        23 verificacoes
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
--  Docker/Podman "nao esta' dando       RELATO do autor em 2026-09-13, ainda sem o
    certo" (relato)                      sintoma: o core responde status/list/
                                         images/action/open contra o Podman 5.8.4
                                         daqui (medido: start e stop do postgres-dev
                                         pelo core, ok). O que se achou e corrigiu:
                                         Logs/Shell sem projeto aberto recusavam
                                         DEPOIS do clique ("nenhum workspace
                                         aberto") — agora o botao diz antes. Falta
                                         o autor dizer o que viu (icone, lista,
                                         botao, mensagem) para medir o resto
--  a porta escolhida no Executar de     run.script ja' aceita `device` (0.102.0);
    MicroPython                          a tela nao tem "porta atual" — o monitor e'
                                         por linha da lista. Uma escolha persistida
                                         (EmbeddedController.selectedPort) e o
                                         RuntimeRequestRouter a passa
--  debugpy: `-m pacote` e attach        o que a fatia 4 deixou: um ponto de entrada
                                         que e' pacote so' se depura apontando o
                                         __main__.py (o launch por `module` do
                                         debugpy resolveria); e o attach a um
                                         processo/porta (`debugpy --listen`) para
                                         servicos. Ambos pequenos; medir antes
--  stderr dos processos filhos vai      LACUNA vista na fatia 4 (2026-09-13): o
    para /dev/null (adaptador DAP,       gate falhou UMA vez com "o adapter nao
    servidor de debug, servidores LSP)   respondeu a `initialize`" e nao havia como
                                         saber por que — a sessao nula o stderr do
                                         adaptador (dap/session.rs:116, dap/server.rs,
                                         lsp/server.rs). Nao reproduziu em 5 rodadas
                                         seguintes. Fatia pequena: guardar as ultimas
                                         linhas do stderr e po-las no erro de subida
                                         (e em event.debug.output como `console`)
--  debugpy no gate desta maquina        o ciclo real so' roda onde `python3`
                                         importa debugpy ou KINEIN_PYTHON_DEBUGPY
                                         aponta um venv — aqui foi provado com um
                                         venv temporario. Para o gate provar sempre:
                                         `sudo dnf install python3-debugpy` ou um
                                         venv fixo e a variavel no ambiente
--  `run.capabilities` (o que "Executar"  DIVIDA anotada na fatia 3 (2026-09-13): a
    aceita, publicado pelo core)         lista de extensoes executaveis (.sh/.bash/
                                         .zsh/.py) vive em DOIS QML (ProjectTree-
                                         Controller e ProjectExplorer) e no core
                                         (run.script). E' o mesmo defeito que o
                                         format.capabilities (0.61.0) corrigiu para
                                         formatar: duas fontes divergem por
                                         construcao. Fatia pequena: o core publica,
                                         a UI consome
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
--  descoberta do pytest como arvore     `pytest --collect-only -q` antes de rodar
    (41 B6, o que faltou)                (a arvore de testes que o JetBrains mostra
                                         antes do primeiro run); hoje os casos so'
                                         aparecem quando rodam
--  ruff como SERVIDOR LSP               DIVIDA da fatia 2 (2026-09-13): as code
    (o Alt+Enter em Python)              actions do ruff ("organizar imports",
                                         "corrigir F401") pedem DOIS servidores
                                         para a mesma linguagem — hoje lsp/session
                                         e' `linguagem -> um spec` (basedpyright).
                                         Fatia propria: multiplexar didOpen/
                                         didChange/diagnosticos por linguagem e
                                         juntar as code actions; so' depois o ruff
                                         entra como `ruff server`
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
