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
protocolo   0.78.0
testes      529 Rust + 23 harnesses QML
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
```

## 4. O que está aberto

```text
27  TimescaleDB e Grafana por HTTP API   o perfil e a introspeccao ja' servem
                                         ao Timescale: ele fala o mesmo
                                         protocolo. Grafana e' AGPL: API,
                                         nunca embutido
28  simulacao: CALCULO sem tela          §5.1 do roadmaps/35 ja' respondida
25  handshake DAP com probe-rs           PARCIAL: precisa de sonda fisica ou
                                         alvo QEMU. O resto do ciclo de
                                         embarcado esta' provado
--  unificar Bibliotecas + Acoes         uma lista so', filtro por LINGUAGEM
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
```

## 6. A lacuna que não é técnica, e continua sendo a mais cara

**O registro de saídas do dogfooding continua VAZIO**
(`docs-privada/diario/19-registro-de-saidas.md`).

E esta sessão deu a prova mais forte que existe de que ele importa: **quatro
defeitos reais** — a busca por arquivo quebrada pelo `fd` 10.4.2, o atalho da
biblioteca que formatava o arquivo, os dezoito alvos de link que não aceitam
link, e o `CMAKE_CXX_STANDARD` escrito onde não faz efeito — **foram achados por
frases do autor usando a IDE**, não pelos dezoito gates.

Uma frase de uso vale mais que uma refatoração da lista.
