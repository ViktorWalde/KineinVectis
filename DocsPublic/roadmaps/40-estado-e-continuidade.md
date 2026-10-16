# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`DocsPublic/README.md`). Tem de ser verdade hoje; se
> divergir do código, o código vence e este arquivo se corrige no mesmo gesto.
> Até 2026-10-15 ele acumulava cabeçalhos de cada retomada; o texto inteiro
> daquela data está em
> [`registro/40-o-estado-ate-2026-10-15.md`](registro/40-o-estado-ate-2026-10-15.md),
> e uma citação "40 §1–§4, §6 ou §8" anterior a ela se lê lá
> ([61](61-organizacao-da-documentacao.md) §3.3). Este cabeçalho é só o
> handoff: cabe em 60 linhas (gate), e o que é decisão ou plano mora no dono.

> **HANDOFF (2026-10-16). Leia isto primeiro.**
>
> **Máquina:** Ubuntu 26.04 LTS, checkout `/home/hugh/KineinVectis`, branch
> `main`, que contém o `porte-windows` (fast-forward local em 2026-10-15,
> sem push). Medido em 2026-10-15: Qt 6.10.2, clang 21.1.8, GCC 15.2, CMake
> 4.2.3, Python 3.14.4, Rust 1.96.1 (rustup), cargo-deny 0.20.2, Podman
> 5.7.0; o Docker 29.9.1 nega o socket (usuário fora do grupo). O
> `CMakeUserPresets.json` local foi gerado como o `instalar-ambiente.sh` faz;
> o gate rápido passa TUDO VERDE, sem NÃO PROVADO, desde 2026-10-16 (o
> autor instalou a lista do apt e fixou o `rustup default 1.96.1`: QEMU ARM,
> `debugpy`, clangd cross, `fd` e monitor serial provados, 40.7 §7.302). A UI
> compila e abre: os aceites da 16.1 a dirigiram no Xvfb (2026-10-16).
>
> **A 0.3.9 está encerrada no checkout (2026-10-16, 40.7 §7.304), sem
> lançamento** (o pacote único da 0.3.6–0.3.9 é do autor; 57). O
> [59](59-fechamento-da-0.3.9.md) virou registro; o que ficou aberto está no
> §4. **Próximo passo:** a 0.4.0, a fundação
> ([52](52-arquitetura-executavel-da-0.4.md) §11 e §14): a 0.4.0a, o contexto
> efetivo com a origem, está feita (40.7 §7.307); seguem o preset no
> configure automático, o diagnóstico do compilador ao salvar, L6/L7 e o
> `compiledb` do PlatformIO, cada desenho no 52 §14 antes do código. **Confiança no projeto (16.4b):** cada projeto aberto
> antes dela pede um clique em "Confiar neste projeto" (a faixa de saúde).
> **Decisões de 2026-10-16** (§5): o Banco para embarcados volta na 0.4.5,
> o binário diz 0.3.9, o AppImage só com a 0.4 finalizada, o 03 por domínio
> e a entrada `CLAUDE.md` (61, O6 e O7, feitas), e a pesquisa de C#/.NET 10.
>
> **Modo autônomo (decisão do autor, 2026-10-15, §5):** o agente fecha a
> 0.3.9 e a 0.4 sozinho, fatia por fatia, sem push; SDKs no usuário, sem
> placa, sem AppImage. A organização da documentação (61 §4, O1–O5) está
> feita; ela continua junto de cada fatia. **Pausado:** o porte para o
> Windows, na W6b4, a retomar numa VM (60 §3.3).
>
> **Aceites com mouse e teclado** de 7, 7b, 14a, 14b, 13a, 13b e 9a.3:
> feitos pelo agente na 16.1 (`scripts/xvfb_ide.py`, 40.7 §7.292), com três
> defeitos corrigidos; o autor não os repetiu.

## 1. O estado, em números

```text
protocolo   0.173.0 (crates/kinein-protocol, medido em 2026-10-16; o
            historico das versoes e' o arquitetura/03)
metodos     187 roteados e 60 eventos (os titulos do arquitetura/03, que o
            verificar-fiacao-ipc.sh confere contra o core)
harnesses   148 tst_*.qml em scripts/qml-harness (medido em 2026-10-16)
testes      1120 Rust no Linux (gate completo de 2026-10-09, 40.7 §7.259);
            920 no Windows com 73 adiados e 1006 no Fedora do WSL (2026-10-14,
            §7.277); 1158 no Ubuntu 26.04 (gate rápido de 2026-10-16,
            §7.302).
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

| O quê | Dono da ordem e do detalhe |
| --- | --- |
| A 0.4 (embarcados), em curso desde 2026-10-16: 0.4.0 a 0.4.4, e a 0.4.5 com o Banco para embarcados e o fechamento | [52](52-arquitetura-executavel-da-0.4.md) §11; as versões seguintes no [57](57-mapa-de-versoes-ate-a-1.0.md) |
| O lançamento da 0.3.6–0.3.9 (tag e release; o binário já diz `0.3.9`) e o AppImage, só com a 0.4 finalizada (autor, 2026-10-16) | o autor; [57](57-mapa-de-versoes-ate-a-1.0.md) e [53](53-arquitetura-executavel-da-0.3.6.md) §11; os requisitos do pacote no [59](59-fechamento-da-0.3.9.md) §8 (registro) |
| Os adiados do Banco que seguem sem versão: MySQL/MariaDB nativo, a edição de célula, os drivers em processo externo (9b/9c); o LSP por conexão (10) e o InfluxDB 3 (11) voltaram na 0.4.5 | [59](59-fechamento-da-0.3.9.md) §2.3 (registro); a arquitetura no [arquitetura/38](../arquitetura/38-provedores-de-banco-e-linguagem.md) e [39](../arquitetura/39-drivers-externos-e-compatibilidade.md) |
| A migração possível para C#/.NET 10: pesquisa, sem decisão | [pesquisa/02](../pesquisa/02-migracao-para-csharp-dotnet.md) |
| O porte para o Windows (pausado na W6b4): W6c, depois W7–W12 | [60](60-windows-e-edicao-especial.md) §3.3 e §3.4; os testes adiados na catraca `scripts/verificar-adiados-windows.sh` |
| A edição especial Windows + WSL2, depois da 0.4.0 | [60](60-windows-e-edicao-especial.md) §4 |
| O AppImage novo: o autor considera depois da 0.4.0, sem decisão | [59](59-fechamento-da-0.3.9.md) §8 (registro; os requisitos guardados) |

As dívidas antigas, de antes da 0.3.9 (o TLS do `postgres`, o DAP com
probe-rs, as fatias de embarcados da trilha), estão na lista datada do §4 do
[registro](registro/40-o-estado-ate-2026-10-15.md) e nos donos delas
([35](35-ambiente-cpp-e-embarcados.md), [42](42-trilha-profunda-embarcados.md),
[52](52-arquitetura-executavel-da-0.4.md)). Quem abrir a 0.4 confere ali o
que ainda vale, medindo antes.

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
modo autonomo (2026-10-15)   o autor PAUSA o desenvolvimento da IDE (projetos de
                             portfolio; CLion e Rider no dia a dia) e AUTORIZA o
                             agente a fechar a 0.3.9 e a 0.4 sozinho, com mouse e
                             teclado no Xvfb (o aceite fica dito como do agente);
                             SDKs fora do apt no usuario, sem sudo (ESP-IDF,
                             Zephyr, pico-sdk, QEMU Espressif, Renode,
                             PlatformIO); SEM placa (o que nao tem emulador fica
                             NAO PROVADO); SEM AppImage; sem push, tag, release;
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
Banco para embarcados        DE VOLTA, dentro da 0.4, no fim, antes do
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
C#/.NET 10                   ANOTADA como pesquisa, sem decisao (2026-10-16):
                             a migracao possivel de Rust + Qt, em
                             pesquisa/02; a 0.4 segue em Rust e Qt
```

## 7. O registro das entregas

O registro datado (§7.5 em diante) é o
[`40.7-registro-das-entregas.md`](40.7-registro-das-entregas.md): o índice das
partes fechadas em [`registro/`](registro/README.md) e a fase aberta. A
numeração é a mesma desde que saiu daqui em 2026-10-01: "40 §7.148" e "40.7
§7.148" são a entrada `### 7.148`. Ele é LOG (nunca se reescreve) e este
arquivo é ESTADO.
