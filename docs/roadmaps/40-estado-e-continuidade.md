# 40 — Onde o projeto está, e por onde continuar

> **Classe: ESTADO** (`docs/README.md`). Medido em **2026-09-04**, com o gate
> completo verde. Se divergir do código, o código vence e este documento se
> corrige no mesmo gesto.
>
> **COMECE POR AQUI ao retomar.** Ele substitui o
> [`38`](38-divida-restante-e-continuidade.md) nesse papel; o 38 vira registro
> de como a fila estava quando a dívida foi paga.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada seção carrega o comando.

## 1. O estado, em números

```bash
bash scripts/verificar.sh                 # 18 verificacoes
cat scripts/arquitetura-baseline.txt      # a catraca
cargo test -q --workspace
```

```text
protocolo   0.81.0
testes      563 Rust + 24 harnesses QML
metodos     139 IPC roteados, 35 eventos
catraca     1 arquivo em debito
gate        18 verificacoes
```

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

**A etapa 27 fechou: o Grafana entra pela API** (roadmaps/35 §9.6). Domínio
`grafana.*` com quatro métodos e um evento, cliente HTTP próprio (`ureq` sem
TLS: +5 crates, todas `MIT OR Apache-2.0`, **zero decisão de licença**), e o
cruzamento que justifica o domínio existir:

```text
dev  ->  kinein-dev      banco `kinein` em localhost:5432
```

Exercitado contra um **Grafana 13.0.2 de verdade**, incluindo os quatro modos de
falha (token inválido, sem token, porta errada, `https` recusado). Os dashboards
abrem no navegador — a licença AGPL decide a forma, e a IDE nunca embute.

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
27  bancos relacional/temporal/nao-     QUASE FECHADA. SQLite entrou, o
    relacional, e o Grafana               TimescaleDB aparece por nome e o
                                          Grafana entrou pela HTTP API
                                          (roadmaps/35 §9.6). Falta o MongoDB,
                                          e ele esta' com o AUTOR: sete
                                          perguntas levantadas em 2026-09-04,
                                          porque ele NAO cabe na arvore
                                          esquema->tabela->coluna — ver
                                          roadmaps/35 §9.5.4
--  TLS                                   UMA decisao para `postgres`, `ureq` e
                                          o que vier: duas licencas permissivas
                                          (`subtle` BSD-3-Clause, `webpki-roots`
                                          CDLA-Permissive-2.0) entram na
                                          allowlist, ou o cifrado nao entra.
                                          Politica de licenca e' decisao do
                                          autor, nunca do assistente
28  simulacao: CALCULO sem tela          §5.1 do roadmaps/35 ja' respondida
25  handshake DAP com probe-rs           PARCIAL: precisa de sonda fisica ou
                                         alvo QEMU. O resto do ciclo de
                                         embarcado esta' provado
--  guias de instalacao para arch/suse   a fonte oficial dos tres projetos NAO
                                         cobre essas familias; entrar exige
                                         fonte de comunidade, marcada como tal
```

## 5. As decisões registradas que NÃO se reabrem

```text
IA na IDE                    fora de escopo (2026-07-17)
Python                       adiado
Pylance                      PROIBIDO (licenca)
Docker e banco               NATIVOS, nao plugins
EditorConfig                 auditado com resultado NEGATIVO (2026-07-16)
Grafana embutido             PROIBIDO (AGPL) — integracao por HTTP API
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
```

## 6. A lacuna que não é técnica, e continua sendo a mais cara

**O registro de saídas do dogfooding continua VAZIO**
(`docs-privada/diario/19-registro-de-saidas.md`).

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
