# 61 — A organização da documentação por contexto

> **Classe: PLANO** (`DocsPublic/README.md`). Dono do que o
> [60](60-windows-e-edicao-especial.md) §6 deixou sem desenho. Escrito em
> 2026-10-15, no Ubuntu 26.04 LTS, antes de mover qualquer arquivo; cada
> fatia abaixo diz o que prova e entra no registro (40.7) ao terminar.

> **Situação em 2026-10-15:** O1–O5 feitas (40.7 §7.278–§7.281). O que
> fica de fora está no §5, sem decisão.

## 1. O pedido

Do autor, em 2026-10-09 (60 §1): "separar a documentação por contexto,
deixar tudo explícito e bem separado", para que uma pessoa ou um agente de
IA "não fique lendo partes desnecessárias onde não vai mexer". Em
2026-10-14 o autor voltou ao Linux e pôs o fechamento da 0.3.9 na frente,
com a organização planejada sem bloqueá-la (40.7 §7.277). Em 2026-10-15
pediu a organização agora, "para otimizar o desenvolvimento das versões
futuras e a finalização da 0.3.9".

## 2. O ponto de partida, medido em 2026-10-15 no `6daacd4`

```text
Markdown versionado          97747 linhas em 153 arquivos
  40.7, o registro           12952 linhas, 760 KB, 273 entradas (§7.5–§7.277)
  03, o protocolo IPC         4914 linhas, 276 KB; 949 delas antes do
                              "## Objetivo", só notas por versão
  59, a 0.3.9                 1812 linhas
  40, o estado                1639 linhas; 472 antes do "## 1."
```

O que isso custa, visto nas sessões de 2026-10-09 a 2026-10-15:

- **Saber onde o projeto está** pede o cabeçalho do 40. Ele tem oito
  "retomadas" empilhadas (2026-09-15 a 2026-10-14), cada uma com um pedaço
  de estado que outra mais nova já desmentiu. A seção §1 ("o estado, em
  números") é de 2026-10-03.
- **O 40.7** não cabe numa leitura de agente; só se lê por trechos. Quem quer
  as últimas entregas lê as 12 mil linhas para achar o fim.
- **A ordem das versões** aparece, com diferenças, no cabeçalho do 40, no 57,
  no 59, no 47 e no 60.
- **Os seis defeitos dos containers** (o próximo passo da 0.3.9, 16b) moram
  no cabeçalho do 40, não no dono da 0.3.9 (o 59 diz "(40, topo)").
- **Quem mexe num domínio do protocolo** (o Banco, por exemplo) lê 949
  linhas de notas por versão antes de chegar à seção do domínio.

## 3. O desenho

### 3.1 Os contextos e o que cada um lê

Os contextos são os do código, já medidos e com gate: as 13 entradas de
`CONTEXTS` em `scripts/module_map.py`, que viram as seções "Contextos" do
[01](../arquitetura/01-mapa-de-modulos.md). A documentação não inventa outra
divisão. Somam-se quatro contextos transversais, que não são domínio do
core: **plataforma** (Linux/Windows), **gates e build**, **a documentação**
e **a fila** (versões e o que vem a seguir).

O mapa "contexto → leia → dispensa" mora no
[`00-comece-aqui.md`](../00-comece-aqui.md) §5, que já é "por onde começar,
conforme o que você veio fazer". Ele não contém fato; aponta para o dono. A
leitura comum a todos encolhe para: 00 → o cabeçalho do 40 (o handoff) → a
linha do contexto. O registro (40.7) e as partes fechadas dele saem da
leitura obrigatória: só se leem para responder "por que isto é assim?".

### 3.2 O registro dividido por fase, sem reescrever o LOG

O 40.7 continua sendo o ponto de citação ("40.7 §7.259" segue valendo), mas
passa a ser **índice + fase aberta**:

- o topo do 40.7 tem a tabela "faixa de entradas → arquivo";
- as fases fechadas vão, **palavra por palavra**, para
  `roadmaps/registro/`, um arquivo por fase. Só os links relativos mudam,
  porque a pasta é outra; o texto não;
- as entradas novas continuam no 40.7, a partir da §7.278.

As fases, pelas datas e versões das entradas:

| Parte | Entradas | Período | O que cobre |
| --- | --- | --- | --- |
| 40.7.1 | §7.5–§7.79 | 2026-09-10 a 09-19 | Etapas 1 (backend e toolchains) e 2 (HUD) |
| 40.7.2 | §7.80–§7.152 | 2026-09-19 a 10-01 | Etapas 3 e 4 e a série 0.3 até o lançamento da 0.3.5 |
| 40.7.3 | §7.153–§7.219 | 2026-10-01 a 10-04 | a casca da 0.3.6–0.3.9, Remoto e Grafana acoplados |
| 40.7.4 | §7.220–§7.260 | 2026-10-05 a 10-09 | o Banco da 0.3.9 até o 9a |
| 40.7.5 | §7.261–§7.277 | 2026-10-09 a 10-14 | o porte para o Windows (W1–W6b4) |

**Regra de fechamento:** quando a fase aberta passa de 1500 linhas ou a
versão em curso fecha, as entradas dela viram a parte seguinte, no mesmo
gesto que atualiza a tabela. Nenhuma parte fechada recebe entrada nova; uma
correção é entrada nova na fase aberta, que cita a antiga.

### 3.3 O 40 só com o estado de hoje

O 40 volta a ter o tamanho de um estado: o handoff curto, o estado em
números com data, a dívida da catraca, a fila aberta (apontando para o
dono de cada item) e as decisões que não se reabrem (§5, intacta). O texto
que sai (os cabeçalhos empilhados e as §1, §3, §4, §6 e §8 antigas) vai,
palavra por palavra, para `roadmaps/registro/40-o-estado-ate-2026-10-15.md`,
classe LOG. As citações antigas ("40 §4.1", "40 §8") são registro do dia em
que foram escritas: a nota no topo do 40 diz onde lê-las. Nos documentos de
classe ESTADO e PLANO que ainda as usam como fonte viva, a citação passa a
apontar o arquivo do registro.

O que estava no cabeçalho e é decisão ou plano vai para o dono antes de sair:
os seis defeitos dos containers e a decisão do "Remover" viram o 59 §2.5
(o passo 16b).

### 3.4 Um dono para cada ordem

- **A ordem das versões:** o [57](57-mapa-de-versoes-ate-a-1.0.md).
- **A ordem dos passos da 0.3.9:** o [59](59-fechamento-da-0.3.9.md) §2.
- **A ordem do porte:** o 60 §3.3.

O 40 diz qual é a próxima fatia e aponta o dono; não repete a ordem. O 47 e
o 60 §1 ficam como histórico das decisões que registraram.

### 3.5 O protocolo sem o histórico no caminho

As 949 linhas de notas por versão do topo do 03 vão, palavra por palavra,
para `arquitetura/03.1-notas-por-versao-do-protocolo.md`. O 03 fica com o
formato, as seções por domínio e a lista canônica (que o
`verificar_fiacao_ipc.py` lê e continua lendo do 03). A nota de uma versão
nova entra no topo do 03.1; o contrato do domínio, na seção do domínio no 03,
como já é a regra. O 03.1 é do mesmo contrato, não é LOG: algumas notas
trazem detalhe que a seção do domínio ainda não tem, e quem mexe num domínio
procura o domínio nas duas.

### 3.6 O gate

Estende o `scripts/verificar-links-docs.sh`, sem gate novo:

- **o registro inteiro:** cada `### 7.N` existe uma vez só, somando o 40.7 e
  as partes; a tabela do 40.7 diz exatamente a primeira e a última entrada
  de cada parte;
- **citação resolvida:** toda citação "40.7 §7.N" num `.md` aponta uma
  entrada que existe;
- **sem depósito:** o cabeçalho do 40 (antes do "## 1.") cabe em 60 linhas e
  a fase aberta do 40.7 em 1500. Passou, fecha a parte ou enxuga o handoff.

O `verificar-docs.sh` passa a tratar `roadmaps/registro/` como registro,
como já trata as ADRs: número lá dentro é do dia em que foi escrito.

Cada verificação nova é provada por mutação antes do commit.

## 4. As fatias

| # | Fatia | Prova |
| --- | --- | --- |
| O1 | Este desenho | os dois gates de documentação verdes |
| O2 | O 40.7 em partes (§3.2) e o gate do registro (§3.6) | as 273 entradas idênticas antes e depois (comparação byte a byte, só links relativos reescritos); mutações: entrada duplicada, faixa errada na tabela, citação a entrada inexistente, fase aberta acima do teto |
| O3 | O 40 só com o estado (§3.3), o 59 §2.5 e o dono de cada ordem (§3.4) | o texto que saiu do 40 está inteiro no registro; mutação do teto do cabeçalho |
| O4 | O 03 sem o histórico no caminho (§3.5) | as 949 linhas idênticas no 03.1; o `verificar-fiacao-ipc.sh` lê a lista canônica (onde o Rust estiver instalado) |
| O5 | O mapa de contextos no 00 §5 e a ordem de leitura do `contribuindo/05` (§3.1) | os links do mapa existem (gate) |

## 5. O que fica de fora, por enquanto

- **Dividir o 03 por domínio** (um arquivo por `dominio.*`). Pouparia mais
  leitura, mas o `verificar_fiacao_ipc.py` e o `module_map.py` leem o 03;
  é fatia própria, com o gate mudado antes. Proposta, sem decisão.
- **Enxugar o 59.** As seções dos passos feitos (§3–§6, §5.8–§5.15) são o
  desenho do que já está no código. Ao fechar a 0.3.9, o 59 inteiro vira
  registro, como o 51 virou depois da 0.3.5; não vale reescrevê-lo antes.
- **A arquitetura do código.** O 60 §6 dizia "se a medição pedir". O mapa de
  módulos já separa os contextos e tem gate; nada medido aqui pede mudança.
