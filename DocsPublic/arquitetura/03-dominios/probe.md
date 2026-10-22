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

**Correção de consulta (0.4.1h4b2b4a, 2026-10-22):** o mesmo método/shape da
0.187.0 passa a aguardar a ferramenta fora do laço do core, com prazo de 5 s e
retenção de saída de até 64 KiB. Falha de saída, prazo ou excesso não produz
sondas reconhecidas a partir de resposta parcial; hint informa a causa e a
saída retida continua disponível. toolAvailable indica somente se o processo
pôde iniciar. Ao finalizar, raiz/adaptador fixado diferentes recusam a resposta
antiga. Esta consulta não valida acesso, exclusividade nem destino de gravação.

**Ponte Qt (h4b2b4a2):** somente o id da última consulta pode produzir sinais
de sucesso/falha; respostas anteriores são descartadas antes dos efeitos.
Fechamento aceito, abertura aceita com outra raiz e desconexão invalidam o
pedido. Abertura recusada e reabertura da mesma raiz conservam a consulta
atual. API pública, sinais e formato IPC permanecem os mesmos.

**Formato 0.32.0 e campos novos (0.188.0, 0.4.1h4b2b4b1):** `probes[]` ganha
`interface?: number` e `inaccessible?: true`. O parser segue a linha medida no
código e no binário oficiais da 0.32.0 (2026-10-22):
`[N]: <nome> -- vvvv:pppp[-IF]:<serial> (<fábrica>)[ (inaccessible)]`. O
marcador `(inaccessible)` não vira mais tipo nem parte do serial, e a interface
FTDI sai do pid. Linhas desconhecidas continuam ignoradas aqui, com a saída crua
junto. Só a validação nativa de gravação exige o formato inteiro e recusa o
resto. A lista continua descoberta: não prova identidade, acesso ou exclusividade.

