# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`DocsPublic/README.md`). Se divergir do código, o código
> vence e este documento se corrige no mesmo gesto.
>
> **O estado, em 2026-10-03 (leia isto; o resto do cabeçalho é histórico):**
>
> - **Última versão lançada: 0.3.5**, em 2026-10-01, pré-release "Public Beta"
>   ([`v0.3.5`](https://github.com/ViktorWalde/KineinVectis/releases/tag/v0.3.5),
>   protocolo `0.144.0`; provas no 40.7 §7.147–§7.149).
> - **Série 0.3.6–0.3.9 (a casca) feita no checkout**, protocolo `0.148.0`,
>   não lançada. O autor pensa em lançá-la junto com a 0.4; isso é intenção,
>   não decisão (53 §11). As fatias e as provas estão no 40.7 §7.153–§7.201.
>   A arquitetura da casca para quem chega, com diagramas, está em
>   [`arquitetura/36`](../arquitetura/36-casca-da-ide.md).
>   - **F0–F4 completas.** A **F5** (§7.201) comparou espaço de código (+4 a
>     5 linhas em toda largura), gestos e desempenho, e achou e corrigiu uma
>     regressão de digitação (§7.198). Falta a tarefa real cronometrada pela
>     mão do autor.
>   - **Proposta aberta para o autor decidir:** o primeiro quadro está em
>     400 ms, no limite do orçamento (F0: 372 ms), por acúmulo de fatias.
>     Criar sob demanda o que só existe com projeto aberto o devolve, mas
>     mexe no ciclo de vida de controladores e terminais.
>   - **Dívida anotada:** 20 campos de texto feitos à mão em 18 arquivos (um
>     `KvTextField` é fatia própria, §7.199).
> - **Próximo:** a 0.4 (embarcados, [`52`](52-arquitetura-executavel-da-0.4.md)),
>   depois de o autor revisar a série na tela. Onde cada versão começa no
>   código: [`58`](58-onde-cada-versao-comeca-no-codigo.md).
> - **Gate:** sem vermelho conhecido no Ubuntu 24.04 / Qt 6.4.2 / gdb 15 nem
>   no Ubuntu 26.04 / Qt 6.10 / clang 21 do autor (a exceção do clang-tidy só
>   vale no 18 desde o 40.7 §7.153); o que a máquina não prova sai como NÃO
>   PROVADO ([`contribuindo/04`](../contribuindo/04-os-gates-que-dizem-nao.md)).
> - **Versões até a 1.0** e o que ainda é proposta:
>   [`57`](57-mapa-de-versoes-ate-a-1.0.md). As decisões que não se reabrem:
>   §5 deste documento. O que foi feito, com data e prova:
>   [`40.7`](40.7-registro-das-entregas.md).
> - O foco do produto são **dois contextos**: desenvolvimento de software
>   (Python, C/C++, Rust, banco de dados) e **sistemas embarcados** (MCU bare
>   metal e Linux embarcado); a simulação física/matemática saiu do produto
>   (§5), e emulador como alvo de depuração é outra coisa (0.4).
>
> **Histórico do cabeçalho (registro; não é o estado de hoje).** Os parágrafos
> abaixo foram escritos em datas diferentes, de 2026-09-05 a 2026-09-29, e
> ficam como lição e contexto. Onde contradizerem o bloco acima, vale o bloco
> acima.
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
> **COMECE POR AQUI ao retomar** — e, se for a primeira vez no projeto, antes
> pela [`00-comece-aqui.md`](../00-comece-aqui.md). A ordem das versões está no
> [`57`](57-mapa-de-versoes-ate-a-1.0.md); o registro do que foi feito, no
> [`40.7`](40.7-registro-das-entregas.md).
> O plano escrito **antes** das próximas etapas, com estado parcial de Remote,
> gates, pasta `KV0.3`, commit/push, release e site, está em
> [`51-plano-fechamento-0.3.5.md`](51-plano-fechamento-0.3.5.md).
> Ele substitui o
> [`38`](38-divida-restante-e-continuidade.md) nesse papel; o 38 vira registro
> de como a fila estava quando a dívida foi paga.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada seção carrega o comando.
>
> **Decisão de 2026-09-22 — frontend 0.3+ e Remote SSH.** Os estudos de
> arquitetura do autor foram reconciliados com a `main`, sem promovê-los em
> bloco a fonte da verdade. O alvo consolidado está em
> `especificacoes/arquitetura-de-frontend-0.3-em-diante.md` e a execução em
> `roadmaps/46-frontend-0.3-em-diante.md`. Assistente/Chat de IA e telemetria
> de produto/usuário estão fora. A UI/HUD de Remote SSH é uma frente
> prioritária porque o backend já existe, mas o painel atual mistura setup e
> operação diária; o desenho está em `especificacoes/remote-ssh-ui-hud.md`.
> Isso é frente horizontal e não cancela a Etapa 4 do roadmap 45. Decisões
> ambíguas foram preservadas para pergunta ao autor, não descartadas.
>
> **Estruturação da v0.3 em 2026-09-22:** o cruzamento entre as Etapas 4 e 46,
> Remote SSH, ergonomia do terminal e fechamento de release está proposto no
> `roadmaps/47-estrutura-da-v0.3.md`. Ainda é plano em estruturação: a §13
> preserva as decisões do autor. A lacuna do terminal está detalhada em
> `especificacoes/terminal-ergonomia-0.3.md`. Naquele baseline ainda faltavam
> menu contextual, `Ctrl+V` desktop e limpeza explícita; esse recorte foi
> implementado em 2026-09-23 (§7.87–§7.89). Selecionar todo o scrollback e nomes
> reutilizáveis foram implementados na retomada de 2026-09-24 (§7.91) e o gate
> integrado fechou verde no mesmo dia (§7.92); a fila da série está na §4.2.7.
>
> **Decisão de 2026-09-24:** o autor pausou o avanço da série e substituiu a
> política híbrida. `Ctrl+C` somente interrompe, mesmo com seleção;
> `Ctrl+Shift+C` copia. Código, menu e testes já refletem essa decisão (§7.91).
> O autor autorizou continuar; a validação integrada foi concluída com essa
> política vigente (§7.92).
>
> **Decisão de 2026-09-24 — o que fica para a 0.4:** os atalhos do trilho da
> esquerda pedem análise de uso e de implementação, e o **ecossistema de
> embarcados ganha uma versão inteira só para ele** (layout, atalhos, fluxo).
> Nenhuma das duas bloqueia a 0.3.5; ver roadmap 47 §10.1.
>
> **Decisão posterior, 2026-09-29 — após fechar a série 0.3:** o autor colocou
> a reorganização e otimização da casca do frontend na **0.3.6**, antes da 0.4,
> e aprovou a direção visual/estrutural de dois estudos do autor (o
> conteúdo aproveitável está nos roadmaps 49 e 50). A auditoria do trilho migra da 0.4 para a
> 0.3.6; a versão dedicada aos embarcados permanece 0.4. A Library por
> capacidades e providers segue na 0.5. Escopo e provas: roadmaps
> [49](49-frontend-0.3.6-e-sequencia-0.5.md) e
> [50](50-biblioteca-e-providers-0.5.md). São planos, não entregas já medidas.
>
> **Decisão de 2026-09-22 — série 0.3 até 0.3.5 e Grafana:** Grafana entra no
> fechamento **0.3.5** e precisa ser prático no uso diário, não apenas receber
> o acabamento visual da E3-7. O produto e a arquitetura estão nos roadmaps
> `47`/`48`; o fluxo em `especificacoes/grafana-ui-ux-0.3.5.md`. A mesma régua
> vale para Remote: a IDE deve descobrir/reutilizar um SSH existente e guiar a
> configuração do zero, sem guardar senha; detalhes em
> `especificacoes/remote-ssh-ui-hud.md`. A distribuição das outras fatias entre
> 0.3.0–0.3.4 continua proposta, não decisão congelada.
>
> **Complemento de ergonomia da série 0.3:** arquivos Markdown ganham modos
> Editar, Preview e Lado a lado. A versão mínima Qt 6.4 já possui renderer
> Markdown, portanto o alvo não adiciona WebEngine. HTML executável e imagens
> remotas ficam bloqueados por padrão; o plano está em
> `especificacoes/markdown-preview-0.3.md` e foi encaixado nos roadmaps 47/48.
>
> **Decisão de 2026-09-22 — token Grafana na sessão:** fechar a tool window
> mantém o token digitado somente em memória para evitar novo login; trocar ou
> fechar workspace, confirmar outra URL, esquecer credencial/instância,
> receber rejeição de autenticação ou encerrar a aplicação o invalida. Ele é
> vinculado a workspace + URL, não inicia requisição sozinho e continua fora de
> perfil/log/view. Proteções e limite de apagamento da string QML estão em
> `especificacoes/grafana-ui-ux-0.3.5.md` §7.
>
> **Complemento de 2026-09-23:** terminal básico significa Selecionar Tudo
> da sessão atual e rótulos `terminal`, `terminal1` com primeira posição
> livre. CLI/abertura e interação completa com arquivos/pastas entram antes de
> Grafana 0.3.5. Bordas/cabeçalho mais naturais entram na 0.3.x. O planejamento
> e as dúvidas preservadas estão na §7.90 e nos roadmaps 47/48; não são entregas.
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
bash scripts/verificar.sh                 # 24 etapas (medidas em 2026-09-24)
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
protocolo   0.148.0 (2026-10-03; o historico das versoes e' o arquitetura/03)
metodos     174 IPC roteados, 57 eventos (scripts/verificar-fiacao-ipc.sh,
            2026-10-01); 98 harnesses QML em scripts/qml-harness (2026-10-02)
testes      874 Rust aprovados; 1 C++; 67 harnesses QML (medicao de 2026-09-24, §7.99)
historico   168 IPC roteados, 56 eventos em 2026-09-24 (remote.parseCommand em 0.134.0;
            remote.command kind copyId em 0.133.0;
            remote.discover/resolve em 0.132.0;
            terminal.selectAll/copySelection em 0.131.0;
            terminal.clearScrollback em 0.130.0;
            datasource.destroy e event.datasource.destroyed em 0.129.0;
            run.stdin e event.run.* sairam em 0.125.0;
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
gate        24 etapas (contagem de 2026-09-24; a mais nova sao os testes C++
            da UI, §7.97; a fiacao IPC de ponta a ponta entrou em 2026-09-18)
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
protocolo    0.129.0 · 162 metodos IPC · 56 eventos · 36 dominios (todos no arquitetura/03)
testes       843 Rust · 55 harnesses QML · 24 verificacoes no gate, todas verdes
binario      linux-clang-debug-strict abre em ~720-840 ms offscreen (debug);
             release-hardened 386-479 ms em 2026-09-19 (§7.79; 318 ms na §7.54)
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

Fechamento da etapa                                       FEITO em 2026-09-19 (§7.74-7.79)
    Ln:Col na status bar (§7.74); rename/codeActions fora do laco e
    container.status adiado (§7.75); Grafana/Setup/Biblioteca na moldura comum
    (§7.76); o trilho compacto/expandido (§7.77); a foto a 1024 px e a faixa de
    abas que nao bate no x (§7.78); as tres telas fotografadas de novo e o
    release-hardened remedido (§7.79). Ficou dito: o foco da StartScreen contra
    o TerminalPanel (caso raro), a mensagem do job que cede antes da barra, a
    primeira linha (KvPanelHeader) nos paineis de baixo e nos tres de ambiente
    que so' ganharam a moldura.

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

**(c) Só o autor, com a placa na mão**: o passo do
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

**Etapa 3 decidida em 2026-09-19 (a partir do teste do autor): a
arquitetura do frontend** — tool windows à JetBrains adaptadas (Git em pé
à esquerda, Símbolos à direita, o trilho sem repetidos, Embarcados/Banco/
Containers/Grafana polidos). O desenho e as sete fatias estão no
[`44`](44-etapa3-arquitetura-do-frontend.md); a etapa seguinte
(integração profunda com os compiladores) está anotada no `44` §8, e a
resposta a "posso divulgar?" no `44` §9.

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
`build.run`) abre um painel logo depois do workspace, como o gate faz;
`index.symbols=parse_` abre a aba Símbolos já buscando (E3-2).

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

#### 4.2.7 Fila da série 0.3 — retomada autorizada em 2026-09-24

A §7.89 registra a revisão do primeiro recorte, não o fechamento da versão.
**Selecionar Tudo real** e nomes `terminal`, `terminal1` etc. estão no checkout,
sem reutilizar identidade/conteúdo, e o gate integrado fechou verde em
2026-09-24 (§7.92). O que ainda bloqueia a 0.3.0 **não é automação**: falta o
dogfooding do autor em shell/TUI, um SSH real e a auditoria de acessibilidade.
Gate verde e harness verde não substituem esse roteiro.
Política vigente: `Ctrl+C` só interrompe; `Ctrl+Shift+C` copia.

**V1 fechada em 2026-09-24:** o autor rodou o roteiro real de shell/TUI e deu a
fatia do terminal por validada. Era o único obrigatório da §4 do roadmap 47 cujo
bloqueio não era código. Continua pendente a auditoria de acessibilidade.

O [roadmap 48 §3](48-arquitetura-executavel-da-serie-0.3.md#3-trem-de-versões-proposto)
organiza as demais frentes. Launcher e interação completa de arquivos/pastas
ficam em 0.3.0–0.3.4; Grafana permanece na 0.3.5. Refinamento de bordas/header
é compromisso da 0.3.x, proposto na 0.3.4. A matriz de projeto cobre teclado,
seleção, menus, clipboard e arrasto, além de proteção de dados — não só drop.

Launcher/pastas/bordas ainda são planejamento (§7.90), não comportamento entregue.
Reaproveitar os donos medidos na especificação antes de acrescentar
código; perguntar ao autor quando defaults ou limites de contrato estiverem
em aberto. Detalhes e fontes na §7.90.

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
                             simulacao que estavam aqui sairam do
                             repositorio. O foco
                             sao DOIS contextos: software (Python, C/C++, Rust,
                             banco) e sistemas embarcados. "Futuramente vejo
                             algo sobre simulacao" — reabrir e' do autor
emulador de alvo: NA 0.4     DECISAO DO AUTOR em 2026-10-01, e NAO reabre a
                             linha acima: QEMU, Renode e QEMU da Espressif
                             entram como ALVO onde o firmware roda (o
                             debugServer do kit, como o QEMU ja' entra no
                             gate). A IDE nao modela nada; sobe o emulador
                             que o usuario escolheu e o depura (52 §10.1)
PlatformIO: TIER 1 NA 0.4    DECISAO DO AUTOR em 2026-10-01: as cinco
                             jornadas da 0.4, nao so' o compiledb (52 §5.10;
                             o que falta por arquivo: 58 §4.4)
versoes 0.6-1.0: PLANEJADAS  DECISAO DO AUTOR em 2026-10-01: o agrupamento
                             proposto foi aceito (57 §2); o criterio da 1.0
                             continua PROPOSTA (57 §4)
inicio da 0.3.6              DECISAO DO AUTOR em 2026-10-01: F0 -> layout
                             -> V-1; wayland-egl investigado na 0.3.6;
                             varredura grande entre fatias (53 §13.0)
gdb >= 16; qmllint >= 6.5    DECISAO DO AUTOR em 2026-10-01: abaixo do gdb 16
                             a falta de globais no DAP e' degradacao
                             EXPLICADA (protocolo 0.145.0); abaixo do qmllint
                             6.5 o lint e' NAO PROVADO (contribuindo/04)
documentacao: DUAS ARVORES   DECISAO DO AUTOR em 2026-09-12: `DocsPublic/` (toda a
                             documentacao do projeto, versionada; pastas com
                             nome explicito; documento = numero + nome
                             explicito) e a arvore interna do autor (no .gitignore: log,
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

**O registro de saídas do dogfooding continua VAZIO**.

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

## 7. O registro das entregas, por fatia — mudou de arquivo (2026-10-01)

O registro datado (§7.1 em diante) mora em
[`40.7-registro-das-entregas.md`](40.7-registro-das-entregas.md), com a
**mesma numeração**: "40 §7.148" é a entrada `### 7.148` de lá. Ele saiu
daqui porque é **LOG** (nunca se reescreve) e este arquivo é **ESTADO** (tem
de ser verdade hoje) — duas classes que o `DocsPublic/README.md` separa, e que
juntas faziam deste o maior documento do projeto (7.602 linhas em 2026-10-01).

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
