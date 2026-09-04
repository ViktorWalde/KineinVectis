# 38 — A dívida restante, medida, e por onde continuar

> **Classe: ESTADO.** Medido em **2026-09-03**, no fim de uma sessão longa.
> Existe por um motivo específico: **o contexto daquela sessão acabou no meio
> do trabalho**, e este documento é o que impede a próxima de redescobrir tudo.
>
> **Regra zero vale aqui como em tudo:** antes de aceitar qualquer item como
> pendente, MEÇA. Cada linha carrega o comando.

## 0. Este documento foi SUPERADO na parte de dívida (2026-09-04)

A fila descrita na §2 **foi paga**: a catraca saiu de 8 arquivos para 1. O
registro do que aconteceu, com o critério de cada corte, está em
[`39-divida-tecnica-paga.md`](39-divida-tecnica-paga.md) — comece por lá.

Duas coisas deste documento continuam valendo e por isso ele não foi apagado:

- a **§4**, que lista o que está aberto e **não é dívida** (etapas 25 a 28, a UI
  de embarcado, o registro de saídas vazio);
- a **§5**, que responde por escrito ao pedido de "pesquisar as melhores
  arquiteturas".

E uma coisa aqui **estava errada**, o que vale como registro: a §2.3 dizia que
três arquivos não deviam ser cortados porque cortar 20 linhas acima do limite
seria corte por tamanho. A conclusão estava certa, a premissa não — os três
tinham responsabilidade misturada de verdade, e foram cortados por isso. Ver
[`39`](39-divida-tecnica-paga.md) §2.

## 1. O estado, em números

```bash
bash scripts/verificar.sh          # 18 verificacoes (medido em 2026-09-04)
cat scripts/arquitetura-baseline.txt
```

```text
protocolo   0.78.0  (era 0.71.0 quando este documento nasceu)
testes      529 Rust verdes  (eram 467 quando este documento nasceu)
dominios    17 no core        (contagem por pasta; a de 21 misturava modulo
                              de arquivo unico com dominio de pasta)
catraca     1 arquivo em debito   (eram 8 aqui, e 18 no inicio daquela sessao)
gate        18 verificacoes       (eram 14; medido em 2026-09-04)
```

## 2. A dívida restante — 8 arquivos, e cada um tem um diagnóstico DIFERENTE

**Não trate esta lista como uma fila uniforme.** Ela tem três classes, e a
segunda e a terceira não se resolvem cortando.

### 2.1 Tem decisão REGISTRADA de ficar como está (1 arquivo)

```text
ui/qml/editor/EditorController.qml   791/400   +391
```

**NÃO CORTAR sem falar com o autor.** Em 2026-09-03 ele decidiu a saída (a) —
cortar o `ShellWorkspaceHost` primeiro, o que foi feito — e depois, com a
medição na mão, decidiu **continuar congelado e reavaliar mais tarde**. O
registro está em `arquitetura/32` §8.4 e em `roadmaps/34` §3.2.

A medição que sustenta a decisão: as 90 leituras **mudaram de dono**, não
sumiram — total continua 185 em 13 arquivos, mas agora o maior consumidor é o
`ShellEditorHost` (225/400, com folga). Quebrar a fachada hoje empurraria
linhas de volta para um arquivo que tem espaço, sem ninguém entender melhor.

### 2.2 Corte por responsabilidade ainda disponível (4 arquivos)

```text
ui/qml/editor/EditorTextController.qml   574/400  cursor + selecao + indentacao + gestos
ui/qml/editor/EditorDocumentController.qml 548/400 abas + buffers + save + externo
ui/qml/editor/EditorPane.qml             538/300  visual
ui/qml/editor/EditorTextSurface.qml      504/300  visual
```

Os dois controllers têm vocabulário misturado visível no próprio nome do
débito. Os dois visuais são candidatos ao padrão que funcionou em `GitPanel`
(764 → 288): **extrair componente por área visual**, com o posicionamento
ficando no pai.

**O padrão provado nesta sessão, três vezes:** `GitPanel`, `DebugPanel` e
`editor_highlighter.cpp`. Ver `roadmaps/34` §7 etapas 16 e 17.

### 2.3 Perto do limite — cortar aqui seria corte por TAMANHO (3 arquivos)

```text
ui/qml/editor/EditorFindBar.qml            329/300  +29
ui/qml/panels/bottom/SearchPanel.qml       325/300  +25
ui/qml/panels/bottom/TerminalViewport.qml  319/300  +19
```

**Estes três são o caso onde a §4 regra 9 diz para NÃO cortar.** Vinte linhas
acima do limite não é responsabilidade misturada; é um arquivo levemente gordo.
Cortá-los produziria divisão por tamanho, que o contrato proíbe.

O caminho certo: eles encolhem quando uma fatia funcional tocar a área — foi
assim que cinco das dez etapas do roadmap 30 aconteceram. `SearchPanel` já tem
uma fatia esperando (`arquitetura/33` §8a: o `TextArea` do replace).

## 3. O que foi feito nesta sessão, para não ser refeito

```text
roadmap 34  etapas 11, 11.1, 12, 13, 14, 15, 16, 17   TODAS
roadmap 35  etapas 19, 20, 21, 22, 23, 24             feitas; 25 PARCIAL
```

**Cortes que saíram da catraca:** `handlers/lsp.rs` (pasta), `editor_highlighter.cpp`
(5 arquivos), `GitPanel`, `GitController`, `dap/session.rs` (4 arquivos),
`DebugPanel`, `ShellWorkspaceHost`, `AppDomains`, `core_client_requests.cpp`,
`core_client_dispatch.cpp`, `build.rs` (pasta), `lsp/parse.rs`.

**Gates que nasceram, os dois de falha SILENCIOSA medida por mutação:**

```text
verificar-qml-propriedades.sh  binding para propriedade/sinal inexistente.
                               Pegou 3 erros meus na propria sessao.
verificar-qml-duplicacao.sh    catraca de derivacao duplicada. Nasceu de um
                               bug VIVO: severidade desconhecida ficava AZUL
                               no ProblemsPanel e VERMELHA na EditorGutter.
```

## 4. O que está aberto e NÃO é dívida

### 4.1 Etapa 25 — o ciclo de embarcado, parcial

**Provado** (exercitado contra ferramentas reais, `roadmaps/35` §5.6): detecção
de cross-compilador, kit com sysroot/triple/chip, `cmake.configure`,
`build.run` gerando **ELF ARM EABI5**, `probe.list` contra o `probe-rs` 0.32.0.

**Não provado:** o handshake DAP com `probe-rs dap-server` **nunca foi
executado**. Precisa de sonda física ou de alvo QEMU que o probe-rs suporte.

### 4.2 Sem UI (existe no core, ninguém pergunta)

```text
probe.list       deteccao de sonda: responde por IPC, nao tem painel
toolchain kit    sysroot/cross/chip: nao ha tela para escolher
sugerir kit      depende de mapear VID:PID -> chip, e esse mapa NAO existe
```

### 4.3 Frentes registradas e não começadas

```text
26  banco relacional + COFRE DE CREDENCIAL ANTES   §7.3 do roadmaps/35
27  TimescaleDB e Grafana por HTTP API             Grafana e AGPL: API, nunca embutido
28  simulacao: CALCULO sem tela                    §5.1 ja respondida
29  polimento / pente-fino                         porta de entrada em §8
```

**O cofre é pré-requisito, não item paralelo.** Hoje o `.kinein/` guarda
rascunho e toolchain em **texto puro**; senha de banco ali seria regressão de
segurança.

### 4.4 A lacuna que não é técnica

**O registro de saídas do dogfooding está VAZIO**
(`docs-privada/diario/19-registro-de-saidas.md`). Enquanto ele estiver assim, a
ordem da frente C é palpite. O autor instalou o AppImage atual em 2026-09-03 —
a primeira entrada real vale mais que qualquer refatoração desta lista.

## 5. Sobre "pesquisar as melhores arquiteturas"

Pedido do autor em 2026-09-03, e a resposta honesta é que **este projeto não
tem problema de arquitetura desconhecida** — tem 8 arquivos acima de uma linha,
com diagnóstico individual na §2.

A arquitetura já é governada por regras MEDIDAS e registradas: camadas
(`ARCHITECTURE.md` §2), corte por responsabilidade (§4 regra 9), composition
root dividido por área (§4 regra 8), crescimento `função → arquivo → pasta →
crate` (§6). Cada uma nasceu de uma falha concreta, não de leitura.

**A pesquisa que o contrato manda fazer é outra, e é específica:** `integracoes/README.md`
§"Referência profissional é obrigatória" exige estudar **Code OSS, IntelliJ
IDEA Community, Zed, Lapce ou NetBeans** antes de cada feature de IDE, e
**registrar** o que foi aprendido — importando invariante, modo de falha e
estratégia de teste; nunca código.

Buscar "melhores arquiteturas" genéricas produziria conselho que contradiz
decisões já medidas aqui. **O item 2.3 é a prova:** a recomendação genérica
seria "quebre arquivos grandes"; a regra deste projeto diz que cortar 20 linhas
acima do limite é cerimônia, e ela está certa.
