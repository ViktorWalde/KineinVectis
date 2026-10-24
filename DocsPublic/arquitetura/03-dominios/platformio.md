# `platformio.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Desenho no
> [52 §14.45](../../roadmaps/52-arquitetura-executavel-da-0.4.md).

## `platformio.*` — o PIO Home que a IDE exibe (0.191.0, 2026-10-23)

Três métodos e dois eventos. O `platformio.compiledb` do mesmo nome é uma
ação de configuração ([`configAction.md`](configAction.md)), não um método
deste domínio.

```text
platformio.homeStart   {}           -> { url, port }
platformio.homeStop    {}           -> { stopped }
platformio.homeRelease { after? }   -> { releasing }      (0.192.0)
```

```text
event.platformio.homeExited   { code, reason }           (reason: 0.192.0)
event.platformio.homeCommand  { method, params }          (0.192.0)
```

**`platformio.homeStart`** sobe o PIO Home do PlatformIO Core detectado, ou
reusa o que já está vivo, e responde quando a página da sessão responde 200.

- **Linha.** `pio home --host 127.0.0.1 --port <porta livre> --no-open
  --session-id <sessão> --shutdown-timeout 300`.
  - A sessão tem 128 bits aleatórios.
  - O timeout desliga o servidor sozinho depois de 5 minutos sem nenhum
    cliente conectado. Desde a 0.192.0, o agente fica conectado enquanto o
    core vive, então esse timeout só vale se o core morrer: o agente vê o
    fim do stdin e sai. Com o core vivo, o servidor sem página se desliga
    por `platformio.homeRelease`.
- **Onde roda.** Na pasta do `pio`, fora de projetos. O PIO Home é a
  interface do próprio PlatformIO, não código do projeto: não exige
  workspace nem confiança.
- **`url`.** `http://127.0.0.1:<porta>/session/<sessão>/`. A raiz do
  servidor responde 403, então só quem tem a URL entra.
- **Prazo.** 120 s, porque a primeira subida instala o `contrib-piohome` pela
  rede.
- **Erros.**
  - Sem `pio`: TOOL_NOT_FOUND, com a sugestão de instalação.
  - Processo que termina antes de ficar pronto, ou prazo esgotado:
    INTERNAL_ERROR com a saída.
- **Concorrência.** Um pedido que chega durante a subida espera a mesma
  subida.

**`platformio.homeStop`** desliga o servidor (POST de shutdown e, depois de
2 s, o fim do processo). Ele responde `stopped: false` quando não havia
servidor. O core também o desliga ao sair.

**`platformio.homeRelease`** (0.192.0) diz que a página saiu da tela.
- O core desliga o servidor `after` segundos depois (inteiro de 0 a 3600;
  sem o campo, 300), pelo relógio monotônico dele, como num
  `platformio.homeStop`.
- Um `platformio.homeStart` antes do prazo cancela o desligamento, e outro
  `platformio.homeRelease` recomeça o prazo: vale o último dos dois. O prazo
  desliga o servidor que estiver rodando quando vence; depois de um
  `platformio.homeStop`, não acha nenhum.
- `releasing: false` quando não há servidor rodando (nem subindo).
- `after` fora do intervalo: INVALID_PARAMS.

O prazo fica no core porque o `Timer` do QML segue o relógio de animação
da janela, não um relógio de parede (52 §14.46).

**`event.platformio.homeExited`** sai quando o servidor deixa de existir sem
um `platformio.homeStop`. Depois dele, o próximo `platformio.homeStart` sobe
outro servidor. `reason` diz por quê (0.192.0):

- `"exited"`: o servidor ou o agente terminou sozinho. Com o agente
  conectado, é uma falha (antes da 0.192.0, também o timeout de
  inatividade). `code` é o código de saída do servidor quando foi ele que
  saiu, senão o do agente, ou `null` quando o processo foi morto por sinal.
- `"idle"`: o prazo de um `platformio.homeRelease` venceu e o core desligou
  o servidor; `code` é `null`.

**O que a página pede à IDE (0.192.0, 52 §14.46).** Junto do servidor, o
core sobe um agente (Python, no interpretador do PlatformIO) que se conecta
ao `/session/<sessão>/wsrpc` e atende os comandos da página. O
`platformio.homeStart` só responde depois de o agente conectar. Servidor e
agente são uma unidade: parar encerra o agente primeiro, e se um sai
sozinho, o core encerra o outro e emite `event.platformio.homeExited` uma
vez.

**`event.platformio.homeCommand`** leva à UI o que ela tem de fazer:

- `open_project`, com `params: { path }`: abrir a pasta como workspace,
  pelo mesmo caminho do "Abrir pasta";
- `open_text_document`, com `params: { path, line, column }`: abrir o
  arquivo na posição.

O core emite o evento e responde à página: `true` para esses dois, a raiz
aberta (quando tem `platformio.ini`) para `get_pio_project_dirs`, e `null`
para o resto. Um `path` que não é absoluto responde `false` e não vira
evento. Cada comando chega à UI uma vez, mesmo que o servidor o entregue de
novo (52 §14.46). `line` e `column` contam de 1, e a coluna que a página não
manda vale 1.
