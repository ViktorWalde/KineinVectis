# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`DocsPublic/README.md`). Tem de ser verdade hoje; se
> divergir do código, o código vence e este arquivo se corrige no mesmo gesto.
> Até 2026-10-15 acumulava cabeçalhos de cada retomada; o texto daquela data está em
> [`registro/40-o-estado-ate-2026-10-15.md`](registro/40-o-estado-ate-2026-10-15.md),
> e uma citação "40 §1–§4, §6 ou §8" anterior a ela se lê lá
> ([61](61-organizacao-da-documentacao.md) §3.3). Este cabeçalho é só o
> handoff: cabe em 60 linhas (gate), e o que é decisão ou plano mora no dono.

> **HANDOFF (2026-10-22). Leia isto primeiro.**
>
> **Checkout:** `/home/hugh/KineinVectis`, branch `main`, sem push.
> **0.4.1h4b2b4b2 concluída:** alvo e sonda do probe-rs nativo na aba Gravar,
> validados na IDE real do autor (52 §14.35; 40.7 §7.344); core em §7.343.
> `AppMenuBar.qml`, `.directory`, `eim_config.toml` e `log/` locais preservados.
> Protocolo **0.189.0**, binário **0.3.9**; probe-rs 0.32.0 instalado no usuário (§7.343).
>
> **Objetivo autorizado:** concluir a **0.4.9**, executar a **edição especial** em seguida;
> só após validá-la gerar/testar o **AppImage local** (40.7 §7.335), sem publicação.
> Decisão de 2026-10-20 substitui o fechamento anterior na 0.4.5. A entrega
> 0.4 inclui o trabalho não publicado 0.3.6–0.3.9; a 0.5 herda essa base e
> anuncia apenas seu próprio delta ([57 §1.1](57-mapa-de-versoes-ate-a-1.0.md#11-fronteira-da-entrega-049--decisão-de-2026-10-20)).
>
> **Fechado:** 0.3.9 no checkout, sem lançamento (59); fundação 0.4.0;
> cross 0.4.1a, inspector/deriva 0.4.1b/c, dock 0.4.1d, contextos 0.4.1e e acesso 0.4.1f.
> **Último gate completo estrito, 0.4.1h4b2b4b2:** 1297 Rust, análise C++ limpa,
> 8 CTest, qmllint estrito e **168 harnesses QML**, sem NÃO PROVADO; target isolado (§7.344).
> UI/core debug/release, 33 superfícies por preset, primeiro frame 327/335 ms (§7.344).
> Acesso físico continua dependente de renovar a sessão (40.7 §7.326).
>
> **b4b1/b4b2:** core probe-rs (stdio + probe-rs real) e UI provados; validação na tela do autor (§5).
> **Próxima h4b2b4b3:** depuração probe-rs com o seletor escolhido e reserva da mesma sonda (52 §14.35).
> Depois: demais motores com prova própria e matriz integrada; h4b2b e a 0.4.1 seguem abertas.
> Depois: reformular Instalar Ferramentas junto do Setup no dock; schema 3;
> construir/C + Quality Center inicial; demais escopos. Ordem no **52 §11.1**.
> Quality Center: Q1 na 0.4.2; gate local/Q2–Q3 concluídos até a 0.4.6.
> Navegação compacta aguarda feedback (52 §14.16); serial completa/flash físico pendentes.
>
> **Escopo:** bare metal e Linux embarcado; PlatformIO em destaque dentro
> de Embarcados; C puro com assistente/padrão C e jornada visual completa;
> CPython, MicroPython e CircuitPython distintos; integrações nativas
> ativáveis/desativáveis. Terminal como escolha; UI moderna, discreta e contextual.
> Cada escopo fecha com UI/UX prática (52 §11, §14.14–14.17).
> Banco **0.4.5**, consolidação **0.4.6**, qualidade **0.4.7–0.4.8**;
> estabilização funcional **0.4.9**; edição especial imediatamente depois,
> base polida da 0.5 em diante; AppImage somente após essa edição.
>
> **Provas:** build ESP-IDF via core e ciclo CPython/debugpy passaram;
> C17 passou build/run/CTest/clang-format/clangd numa fixture (52 §14.14).
> Última medida: CP2102 com sessão sem acesso; flash físico não provado. CircuitPython,
> preferências de integração e assistente C ainda não implementados.
> **Operação:** validação visual na IDE real do autor (§5, 2026-10-22); QA com target próprio, sem compartilhar com o principal; Xvfb só em provas automáticas
> nem fechar a janela original. Sem sudo/instalação silenciosa ou gravação
> na placa sem pedido específico. Não compilar UI durante `verificar-cpp.sh`.
> Objetivo contínuo autorizado (autor, 2026-10-21); cota/retomada dependem da plataforma.
> MCP/ROS 2/colaboração ficam após 0.4.9; inglês acompanha o fechamento (57).
> Inglês no produto/divulgação: 0.5 (57 §3 item 11); Windows pausado na W6b4 (60).
> EditorController congelado (§2). Pesquisa de Embarcados recebida como rascunho (52 §11.1).

## 1. O estado, em números

```text
protocolo   0.189.0 (crates/kinein-protocol, medido em 2026-10-22; o
            historico das versoes e' o arquitetura/03)
metodos     187 roteados e 61 eventos (os titulos do arquitetura/03, que o
            verificar-fiacao-ipc.sh confere contra o core)
harnesses   168 tst_*.qml em scripts/qml-harness (medido em 2026-10-22)
testes      1284 Rust no gate completo estrito de 2026-10-22 (40.7 §7.342);
            920 no Windows com 73 adiados e 1006 no Fedora do WSL (2026-10-14,
            §7.277); 1191 no gate rápido estrito local de 2026-10-20 (§7.321).
            1194 Rust no gate rápido estrito integral da 0.4.1e em
            2026-10-20 (§7.325), sem NÃO PROVADO.
            1200 Rust na 0.4.1f em 2026-10-20 (§7.326), gate estrito integral verde.
            1200 Rust na 0.4.1g em 2026-10-21 (§7.327), gate completo estrito verde.
            1227 Rust na 0.4.1h3 em 2026-10-21 (§7.330), gate rápido estrito verde.
            1241 Rust na 0.4.1h4a em 2026-10-21 (§7.332), gate rápido estrito verde.
            1251 Rust na 0.4.1h4b1 em 2026-10-21 (§7.333), gate completo estrito verde.
            1256 Rust na 0.4.1h4b2a em 2026-10-21 (§7.337), gate completo estrito verde em cópia isolada.
            1259 Rust na 0.4.1h4b2b1 em 2026-10-21 (§7.338), gate rápido estrito verde em cópia isolada.
            1278 Rust na 0.4.1h4b2b2 em 2026-10-22 (§7.339), gate rápido estrito verde em cópia isolada.
            1278 Rust na 0.4.1h4b2b3 em 2026-10-22 (§7.340), gate completo estrito verde com target independente.
            1297 Rust na 0.4.1h4b2b4b1 em 2026-10-22 (§7.343), gate rápido estrito verde em cópia isolada.
            1297 Rust na 0.4.1h4b2b4b2 em 2026-10-22 (§7.344), gate completo estrito verde em cópia isolada.
```

Os comandos que medem estão no
[`build/comandos-de-build-e-verificacao.md`](../build/comandos-de-build-e-verificacao.md);
número sem data aqui é afirmação sobre hoje e o `verificar-docs.sh` o cobra.

## 2. A dívida da catraca: 1 arquivo, e ele tem decisão do autor

```text
ui/qml/editor/EditorController.qml   791/400
```

**Não corte sem falar com o autor.** A decisão de mantê-lo congelado está em
[`../arquitetura/32`](../arquitetura/32-editor-por-responsabilidade.md) §8.4, e
as duas saídas estão medidas em [`39`](39-divida-tecnica-paga.md) §6: dissolver
a fachada (~190 pontos de chamada em 15 arquivos) ou corrigir a categoria.
Mexer em limite é decisão explícita dele, nunca do assistente.

## 3. O que este arquivo já guardou

As seções §3 (a sessão de 2026-09-03/04), §4.1 e §4.2 (as ordens e panoramas
de setembro), §6 (a lacuna do dogfooding) e §8 (a varredura de 2026-09-10),
e os cabeçalhos de cada retomada, estão no
[registro](registro/40-o-estado-ate-2026-10-15.md). São registro do dia:
antes de aceitar um item de lá como pendente, meça.

## 4. O que está aberto

Este arquivo diz qual é a próxima fatia; a ordem mora no dono.

**Embarcados na série 0.4:** a 0.4.1 é a fundação; 0.4.2 e 0.4.3 completam
construção/qualidade e gravação/depuração; até a 0.4.6 consolidam-se as jornadas
bare metal e Linux embarcado. 0.4.7–0.4.9 auditam e estabilizam. Provas de software,
emulação e hardware são estados distintos; colaboradores ampliam a matriz
física. Ordem/aceite no 52 §11.1 e limites no §10.1 (esclarecido em 2026-10-22).


| O quê | Dono da ordem e do detalhe |
| --- | --- |
| Embarcados: acesso, ciclo de vida, reservas e armazenamento fechados nas fatias anteriores; intenção/UI esptool h4b2b3, consulta de sondas A1/A2 e probe-rs nativo b4b1 (core) e b4b2 (UI, validada na tela do autor) fechados. Próxima h4b2b4b3: depuração probe-rs com o seletor escolhido e reserva da mesma sonda; depois demais motores e recuperação. A medida física ainda requer renovar a sessão (2026-10-22) | [52](52-arquitetura-executavel-da-0.4.md) §14.19–14.35 e [40.7](40.7-registro-das-entregas.md) §7.326–§7.344 |
| C puro: pendências básicas incluídas pelo autor no fechamento da 0.4, com jornada visual de criar/configurar/compilar/executar/testar/depurar; terminal como escolha (2026-10-20) | [52 §14.14](52-arquitetura-executavel-da-0.4.md#1414-auditoria-de-c-puro-antes-do-fechamento--2026-10-20) |
| Instalar Ferramentas: reformular junto da migração Setup ao dock na 0.4.1; aplicar referências ao catálogo existente. Quality Center inicial: Q1 na 0.4.2, gate local e consolidação Q2/Q3 até a 0.4.6. Ambos planejados, ainda sem implementação (autor, 2026-10-21) | [52 §11.1](52-arquitetura-executavel-da-0.4.md#111-fluxo-de-execução-reorganizado--2026-10-21); [Instalar Ferramentas](../especificacoes/instalar-ferramentas.md) e [Quality Center](../especificacoes/quality-center-e-gates.md) |
| Edição especial associada à 0.5: otimização/acabamento depois da 0.4.9; inglês como idioma plenamente suportado e divulgação internacional na 0.5. ROS 2 continua em frente posterior, sem versão fixada | [52 §14.17.1](52-arquitetura-executavel-da-0.4.md#14171-direção-da-edição-especial-associada-à-05--2026-10-21), [57](57-mapa-de-versoes-ate-a-1.0.md) §3 item 11 e [integracoes/41](../integracoes/41-ros2-hud-ux-analise-projeto.md) |
| HUD por contexto 0.4.1e fechada: PlatformIO/Linux e aplicação CMake com kit Linux, gate integral estrito verde. Schema 3 dos kits segue após os bloqueadores de orquestração e Setup. CPython revalidado, MicroPython corrigido, CircuitPython ainda por separar/implementar (2026-10-20) | [52 §14.13](52-arquitetura-executavel-da-0.4.md#1413-hud-por-contexto-e-destaque-platformio--desenho-em-2026-10-20) e [41 §3.4.1](41-ecossistema-embarcados-e-python.md#341-ampliação-confirmada-e-revisão-medida-em-2026-10-20) |
| Prévia de navegação compacta continua sem escolha visual aprovada. Pesquisa de Embarcados recebida em 2026-10-21: cinco imagens conceituais, com a 5 mais próxima da UI atual segundo o autor, ainda sujeita a refinamento/aprovação. Estudos de frontend são rascunhos; evoluir a UI real, sem copiar mockups. Trabalho funcional independente pode seguir | [52](52-arquitetura-executavel-da-0.4.md) §11.1/§14.16–14.17; [rascunho de Embarcados](../especificacoes/embarcados-hud-ux-0.4.md) e [regra de aprovação visual](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md#11-aprovação-visual-e-referências--decisão-de-2026-10-21) |
| CI pública: camada rápida preparada e validada localmente; execução no GitHub pelo autor, camadas UI/pesada e matriz Arch pendentes; porta para colaboradores em preparação, com ideia de cards no site para outra sessão | [contribuindo/07 §2.1](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md#21-a-camada-rápida-da-ci-2026-10-19) e [contribuindo/11](../contribuindo/11-ci-publica-e-tarefas-para-colaboradores.md); decisão no §5 |
| ROS 2: frente e ensaio por jornadas depois da 0.4.9 (limite atualizado em 2026-10-20); escolher versão exata a partir da pesquisa medida (autor, 2026-10-19) | [integracoes/40 §1.2](../integracoes/40-ros2-ecossistema-e-integracao.md#12-decisão-recuperada-escopos-por-ferramentas-e-melhorias-próprias-2026-10-19) |
| Algoritmos complementares de análise e refatoração: estudo de longo prazo autorizado, sem substituir motores ou reservar versão (autor, 2026-10-19) | [56 §1.1](56-motor-semantico-profundo-cpp-rust.md#11-direção-de-longo-prazo-recuperada-em-2026-10-19) e [pesquisa/02](../pesquisa/02-analise-e-refatoracao-complementares.md) |
| Pendências do handoff confirmadas dentro da 0.4.x em 2026-10-21 (alocação no 52 §11). Banco na 0.4.5, consolidação nativa na 0.4.6, auditoria/polimento geral na 0.4.7–0.4.8 e estabilização funcional na 0.4.9. Edição especial imediatamente em seguida; AppImage só depois dela | [52](52-arquitetura-executavel-da-0.4.md) §11/§11.1; fronteira no [57](57-mapa-de-versoes-ate-a-1.0.md) §1.1 |
| Entrega acumulada: trabalho não publicado 0.3.6–0.3.9 pertence à série 0.4; edição especial sobre a 0.4.9 estabelece a base polida da 0.5 em diante. Alinhar identificação/versões/notas e gerar/testar o AppImage somente após essa edição; publicação com o autor (atualizado em 2026-10-21) | o autor; [57](57-mapa-de-versoes-ate-a-1.0.md) e [53](53-arquitetura-executavel-da-0.3.6.md) §11; os requisitos do pacote no [59](59-fechamento-da-0.3.9.md) §8 (registro) |
| Os adiados do Banco que seguem sem versão: MySQL/MariaDB nativo, a edição de célula, os drivers em processo externo (9b/9c); o LSP por conexão (10) e o InfluxDB 3 (11) voltaram na 0.4.5 | [59](59-fechamento-da-0.3.9.md) §2.3 (registro); a arquitetura no [arquitetura/38](../arquitetura/38-provedores-de-banco-e-linguagem.md) e [39](../arquitetura/39-drivers-externos-e-compatibilidade.md) |
| O porte para o Windows (pausado na W6b4): W6c, depois W7–W12 | [60](60-windows-e-edicao-especial.md) §3.3 e §3.4; os testes adiados na catraca `scripts/verificar-adiados-windows.sh` |
| A edição especial Windows + WSL2, depois da 0.4.0 | [60](60-windows-e-edicao-especial.md) §4 |

As dívidas antigas, de antes da 0.3.9 (o TLS do `postgres`, o DAP com
probe-rs, as fatias de embarcados da trilha), estão na lista datada do §4 do
[registro](registro/40-o-estado-ate-2026-10-15.md) e nos donos delas
([35](35-ambiente-cpp-e-embarcados.md), [42](42-trilha-profunda-embarcados.md),
[52](52-arquitetura-executavel-da-0.4.md)). Quem abrir a 0.4 confere ali o
que ainda vale, medindo antes.

## 5. As decisões registradas que NÃO se reabrem

```text
rascunhos e aprovacao visual DECISAO DO AUTOR em 2026-10-21: estudos/imagens
                             de frontend sao rascunhos e nao retratam a UI
                             real nem aprovam desenho. Melhorar/incrementar/
                             polir sobre a IDE atual; a direcao visual vem
                             da aprovacao dele. Imagem 5 de Embarcados e' a
                             aproximacao preferida, ainda por refinar/aprovar.
                             Dono: frontend §1.1; pesquisa: spec
                             embarcados-hud-ux-0.4; ordem: 52 §11.1.
AppImage apos especial       DECISAO DO AUTOR em 2026-10-21: fechar a 0.4.9,
                             executar a edicao especial imediatamente depois
                             e so' entao gerar/testar o AppImage. A edicao
                             especial estabelece a base polida da 0.5 em
                             diante, para oferecer aos usuarios de teste.
                             Substitui o pacote logo apos a 0.4.9; versao/
                             identificacao do artefato ainda nao fixadas.
                             Dono: 57 §1.1; execucao: 52 §11/§11.1.
Quality Center e instalador  INCLUIDOS na 0.4.x pelo autor em 2026-10-21:
                             reformular Setup junto da migracao ao dock e
                             expor verificacoes existentes com gate local
                             simples. Ordem 52 §11.1; detalhes nas specs
                             instalar-ferramentas e quality-center-e-gates.
                             A edicao especial associada a 0.5 melhora/
                             otimiza/pole detalhes da base entregue.
ingles e divulgacao 0.5      DECISAO DO AUTOR em 2026-10-21: ingles plenamente
                             suportado no produto, mantendo portugues;
                             marco internacional na 0.5. Preparacao dos
                             componentes/documentacao antes, sem alterar
                             permissao de publicar/enviar mensagens.
                             Dono: 57 §1.1 e §3 item 11.
analise futura de CLI        após 0.4.9 e após revisão especial HUD/UX/UI;
                             medir utilidade nos fluxos embarcados/ROS 2,
                             sem novos comandos aprovados. Autor 2026-10-21;
                             dono: especificacao de projetos/desktop §3.1,
                             índice 57 §3 item 9.
comando curto de terminal    `kine` passa a ser a entrada principal; `kinein`
                             mantém compatibilidade. Decisão 2026-10-21;
                             contrato/estado em especificacoes/projetos-arquivos-
                             e-integracao-desktop-0.3.md §3; entrega 0.4.6,
                             verificação no AppImage apos a edicao especial (52 §11).
IA na IDE                    fora de escopo (2026-07-17): sem painel de IA,
                             o agente roda no terminal. So' o servidor MCP
                             para agentes externos entra, depois da 0.4.9
                             (limite atualizado em 2026-10-20; 57 §1.1)
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
modo autonomo (2026-10-15)   o autor PAUSA o desenvolvimento da IDE (projetos de
                             portfolio; CLion e Rider no dia a dia) e AUTORIZA o
                             agente a fechar a 0.3.9 e a 0.4 sozinho, com mouse e
                             teclado no Xvfb (o aceite fica dito como do agente);
                             SDKs fora do apt no usuario, sem sudo (ESP-IDF,
                             Zephyr, pico-sdk, QEMU Espressif, Renode,
                             PlatformIO); SEM placa (o que nao tem emulador fica
                             NAO PROVADO); AppImage autorizado so' apos a
                             edicao especial pos-0.4.9 (2026-10-21); sem push/tag/release;
                             organizar a documentacao junto, sem pular
0.4: C1 ao salvar            LIGADO por padrao em C/C++ com CDB; Rust so' com
                             `cargo check` quando o rust-analyzer nao esta'
                             ativo (52 §13.1, 2026-10-15)
0.4: perifericos (SVD)       aba da area Embarcados (52 §13.2, 2026-10-15)
0.4: delta de tamanho        so' do ultimo build (52 §13.3, 2026-10-15)
toolchain automatica         NOME NEUTRO, e o automatico LE o que o projeto
                             aberto de fato usa (o cache do CMake), sem
                             suposicao (autor, 2026-10-16; 59 §7.1.7, F7). O
                             "primeiro candidato detectado" deixa de ser o
                             que a tela diz que vai ser usado
Banco para embarcados        DE VOLTA, dentro da 0.4, na 0.4.5, antes do
(2026-10-16)                 fechamento (52 §11, 0.4.5): o LSP por conexao
                             (passo 10 do 59) e o InfluxDB 3 nativo (passo
                             11). MySQL/MariaDB, edicao de celula e drivers
                             em processo externo seguem adiados (59 §2.3)
versao e AppImage            o binario diz 0.3.9 desde 2026-10-16 ("pode
(2026-10-16)                 gerar e atualizar"); AppImage SO' com a 0.4
                             finalizada. Tag e release continuam com o autor
documentacao para agentes    APROVADOS (2026-10-16): um arquivo curto de
(2026-10-16)                 entrada na raiz e o 03 dividido por dominio (61
                             §4, O6 e O7). A entrada e' o `CLAUDE.md`: o nome
                             que a proposta usava e' o de uma nota interna do
                             autor pela regra de 2026-10-01 (gate de links e
                             copia limpa)
Rust e Qt                    MANTIDOS (autor, 2026-10-08, 40.7 §7.241;
                             reconfirmado em 2026-10-16, §7.311): a pilha do
                             projeto nao muda
fechamento da 0.4            (autor, 2026-10-17; limite atualizado para
(2026-10-20/21)              0.4.9 em 2026-10-20) A 0.4.9 fecha funcionalmente;
                             AppImage apos a edicao especial; marco de
                             divulgacao na 0.5 e TRANSICAO DA DOCUMENTACAO PARA O
                             INGLES (prioridade; o alemao talvez na 0.5, sem
                             decisao). Ate' la a documentacao segue em
                             portugues. DEPOIS da 0.4.9: a Kinein como
                             servidor MCP para agentes externos (o Claude Code
                             por `claude mcp add`), o que revisa, so' nisto, o
                             "IA na IDE fora de escopo" de 2026-07-17 — o
                             agente continua no terminal, sem painel de IA; o
                             desenho, com o primeiro corte so' de leitura, vem
                             antes do codigo. O site (outra sessao) fica em
                             ingles, com aba de colaboradores, dizendo so' de
                             passagem que o projeto vem do Brasil. O `/ide` da
                             Kinein (protocolo interno do Claude Code, sem
                             especificacao publica) NAO foi decidido
0.9 antecipada em parte      (autor, 2026-10-17) A CI publica ENTRA AGORA,
(2026-10-17)                 numa fatia curta antes da 0.4.1b, em camadas (a
                             rapida em todo push; a UI; a pesada agendada); o
                             gate estrito local continua a regra de cada
                             commit. A porta para novos colaboradores (modelos
                             de issue e PR, labels, issues de entrada em
                             INGLES): ver "porta de colaboracao" abaixo,
                             com a atualizacao do autor de 2026-10-19.
                             Push, workflow e issue no GitHub sao do autor
ROS 2                        ENTRA na IDE (autor, 2026-10-17): o ambiente ROS 2
(2026-10-17)                 tao bom quanto o do VS Code ou melhor, orquestrando
                             as ferramentas abertas que ja existem. Versao a
                             decidir; antes, a pesquisa medida (o que o VS
                             Code faz, o que a Kinein ja tem, o que falta)
                             Pedido posterior recuperado em 2026-10-19:
                             escopos por ferramentas, jornadas e melhorias;
                             dono: integracoes/40 §1.2. Versao nao escolhida
                             Esclarecido pelo autor em 2026-10-19: a frente
                             ROS 2 e seu ensaio ficam DEPOIS da 0.4.9
                             (limite atualizado em 2026-10-20, 57 §1.1)
profundidade semantica       DIRECAO de longo prazo do autor (pedido apos o
(recuperada 2026-10-19)       commit de 2026-10-17): refatoracao e compreensao
                             de compilador com a regua JetBrains, ao longo
                             dos anos. Esclarecido em 2026-10-19: AUTORIZADO
                             o estudo de algoritmos proprios COMPLEMENTARES
                             de analise/refatoracao, sem substituir motores;
                             refina a restricao absoluta anterior no ponto
                             da pesquisa. Donos: 56 §1.1 e pesquisa/02
porta de colaboracao         IDEIA documentada a pedido do autor (2026-10-19):
(2026-10-19)                 tarefas no GitHub, cards informativos no site
                             com botao para a issue e "Tasks realizadas".
                             Recomendacao no contribuindo/11; nenhuma issue,
                             configuracao GitHub ou mudanca no site nesta
                             fatia. Atualizacao do autor em 2026-10-19:
                             trabalha SOZINHO ate o fechamento, agora 0.4.9
                             (2026-10-20); porta e tarefas
                             para colaboradores DEPOIS dela. O prompt do
                             site foi entregue na conversa, sem ampliar
                             esta documentacao com implementacao do site
continuidade ate 0.4.9       AUTORIZADA pelo autor em 2026-10-20, por fatias,
                             com documentacao atualizada a cada etapa; o
                             limite anterior 0.4.5 foi ampliado. Primeiro
                             a bancada ESP32 e Embarcados integrado a HUD;
                             botao por arquivo continua exclusivo de scripts.
                             Reuso de componentes e animacoes existentes.
                             Retomada solicitada a cada cinco horas: objetivo
                             continuo ATIVO em 2026-10-21, sem cronograma
                             nem gatilho confirmado para renovacao da cota.
                             Continuar assim que a plataforma permitir;
                             registrar progresso e limites sem prometer
                             retomada exata ou contornar limite de uso.
modo automatico renovado     DECISAO DO AUTOR em 2026-10-20/21: desenvolver ate
                             a 0.4.9, executar a edicao especial em seguida
                             e so' depois gerar/testar o AppImage (57 §1.1).
                             Em cada 0.4.x: procurar falhas de logica e bugs
                             visuais, otimizar o frontend e melhorar os paineis
                             para uma UX moderna, clara e intuitiva. Embarcados
                             tem prioridade; Python inclui CPython,
                             MicroPython e CircuitPython. Sem push/publicacao;
                             firmware fisico ainda exige pedido especifico.
HUD por contexto (2026-10-20) Embarcados tambem cobre Linux embarcado; a UX
                             deve distinguir as jornadas. PlatformIO fica
                             em DESTAQUE DENTRO DE EMBARCADOS, sem novo icone
                             separado. Animacoes acompanham os fluxos com
                             os componentes e tokens existentes. Atualizar
                             a documentacao ao final de CADA tarefa.
C e integracoes nativas      DECISAO DO AUTOR em 2026-10-20: as pendencias
                             basicas de C entram na 0.4; embarcados integra
                             as ferramentas existentes, com jornadas visuais
                             praticas para bare metal e Linux, mantendo o
                             terminal como escolha. Integracoes ativaveis/
                             desativaveis pelo usuario, profundamente ligadas
                             aos fluxos da IDE; refinamento dessa gestao e
                             ampliacao do escopo na 0.5 (52 §14.14–14.15).
evolucao e projetos legados  REFORCO DO AUTOR em 2026-10-21: modularidade e
                             compatibilidade de projetos antigos/novos fazem
                             parte dos aceites da 0.4.x. Matriz medida e pontos
                             de extensao no 52 §11; abrir nao migra o projeto
                             nem comprova suporte a toda versao de ferramenta.
fronteira da entrega 0.4.9   DECISAO DO AUTOR em 2026-10-20: fechar a serie
                             na 0.4.9 antes do AppImage. O trabalho nao
                             publicado 0.3.6–0.3.9 entra na entrega 0.4;
                             preservar commits/registros. A 0.5 herda a base,
                             mas anuncia so' o delta proprio. Dono: 57 §1.1.
                             Banco continua marco funcional 0.4.5. Ordem
                             operacional 0.4.6–0.4.9 no 52 §11.
auditoria geral de frontend  PEDIDO DO AUTOR em 2026-10-20: componentes,
                             comportamento, HUD/UI/UX, espaco dos paineis,
                             animacao/interacao, resposta, informacoes,
                             cores semanticas legiveis e confortaveis,
                             atalhos/nomes/localizacao/acesso lateral.
                             Entra antes do fechamento funcional 0.4.9;
                             AppImage apos a edicao especial (52 §14.17).
                             Previa de navegacao compacta para feedback
                             ainda pendente (52 §14.16); nao tratar como
                             layout aprovado nem como arquivo ja' entregue.
eficiencia e escopo fechado  ESCLARECIMENTO DO AUTOR apos o handoff, em
(2026-10-20)                 2026-10-20: concluir cada escopo sem acumular
                             pendencias. A 0.4.9 deve entregar tudo redondo,
                             incluindo UI/UX necessaria ao uso. As revisoes
                             finais conferem a integracao e as regressoes;
                             nao recebem lacunas deixadas pelos marcos.
                             Confirmacao posterior: "edicao especial"
                             revisa HUD/UX/UI APOS concluir a 0.4.9, buscando
                             componentes/estilos/estados esquecidos ou fora
                             do padrao moderno. Atualizacao 2026-10-21:
                             associada a 0.5; patch/artefato e cronograma
                             ainda nao fixados; sem transferir pendencias.
                             Criterios no 52 §11 e §14.17; fronteira no 57 §1.1.
validacao na IDE real        DECISAO DO AUTOR em 2026-10-22: validacao visual e
(2026-10-22)                 de fluxo usa a IDE real na tela do notebook dele,
                             com mouse/teclado reais e ele acompanhando ("eu quero
                             que use a IDE real no meu notebook para que possa
                             validar de fato e eu visualizar tambem"). Xvfb fica
                             para provas automatizadas. Instancia separada, sem
                             fechar a janela dele; permissao "Remote Control" do
                             Plasma e' dele. Tambem liberou instalar o necessario
                             no usuario; sudo vira comando para ele rodar.
                             Gravar placa continua exigindo pedido especifico.
janelas de trabalho em dock  PEDIDO DO AUTOR em 2026-10-20: Instalar ferramentas
                             e demais janelas funcionais viram paineis;
                             menus, dicas e confirmacoes seguem compactos
                             (clarificacao explicita). Reusar componentes e
                             estender os donos existentes por open/closed.
                             Erros com mensagem clara, destaque e animacao
                             leve. Primeiro Setup e acesso ESP32 (52 §14.18).
```

## 7. O registro das entregas

O registro datado (§7.5 em diante) é o
[`40.7-registro-das-entregas.md`](40.7-registro-das-entregas.md): o índice das
partes fechadas em [`registro/`](registro/README.md) e a fase aberta. A
numeração é a mesma desde que saiu daqui em 2026-10-01: "40 §7.148" e "40.7
§7.148" são a entrada `### 7.148`. Ele é LOG (nunca se reescreve) e este
arquivo é ESTADO.
