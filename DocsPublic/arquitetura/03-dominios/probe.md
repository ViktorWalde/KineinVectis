# `probe.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `probe.*` — as sondas de debug conectadas

Domínio novo no protocolo `0.71.0` (etapa 24 do `roadmaps/35`). É o **"plug"**
do plug and play: a IDE detecta em vez de pedir configuração.

```text
probe.list {} -> { probes, toolAvailable, rawOutput, hint? }
```

Executa `<adaptador> list`, onde o adaptador vem do kit (papel `debugAdapter`).
**Exige workspace**, porque sem kit não há qual ferramenta perguntar — e cair no
`PATH` em silêncio esconderia do usuário quem respondeu.

**O `rawOutput` vai SEMPRE, não só no erro.** O formato do `probe-rs list` foi
levantado de fontes da comunidade e **não** verificado contra o binário
instalado (ele não está nesta máquina, medido em 2026-09-03). Um parser rígido
contra formato não verificado quebraria na primeira mudança de espaçamento — e
quebraria dizendo *"nenhuma sonda"*, que é a pior mentira possível aqui.

Por isso: linha que casa vira sonda, linha que não casa é **ignorada**, e a
saída crua volta junto. *"Não entendi o que a ferramenta respondeu, e aqui está
o que ela disse"* é acionável; *"nenhuma sonda"* com uma plugada viola o item 4
do plug and play (`roadmaps/35` §5.1).

**A `hint` é onde o item 4 vira código.** O caso mais importante é o de **udev**:

```text
permissao negada  -> "falta regra de udev; rodar a IDE como root NAO e a solucao"
resposta vazia    -> "sonda desconectada, ou cabo de dados trocado por um de carga"
nao reconhecida   -> "o formato mudou e o parser precisa acompanhar"
```

Sem regra de udev a ferramenta roda, não acha nada, e o usuário conclui que a
placa está com defeito. **Plug and play morre exatamente aí**, e é a lacuna que
`integracoes/36` §5 já tinha nomeado como a mais subestimada.
