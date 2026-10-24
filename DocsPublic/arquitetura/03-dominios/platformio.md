# `platformio.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Desenho no
> [52 §14.45](../../roadmaps/52-arquitetura-executavel-da-0.4.md).

## `platformio.*` — o PIO Home que a IDE exibe (0.191.0, 2026-10-23)

Dois métodos e um evento. O `platformio.compiledb` do mesmo nome é uma ação de
configuração ([`configAction.md`](configAction.md)), não um método deste
domínio.

```text
platformio.homeStart {}  -> { url, port }
platformio.homeStop  {}  -> { stopped }
```

```text
event.platformio.homeExited  { code }
```

**`platformio.homeStart`** sobe o PIO Home do PlatformIO Core detectado, ou
reusa o que já está vivo, e responde quando a página da sessão responde 200.

- **Linha.** `pio home --host 127.0.0.1 --port <porta livre> --no-open
  --session-id <sessão> --shutdown-timeout 300`.
  - A sessão tem 128 bits aleatórios.
  - O timeout desliga o servidor sozinho se ninguém o usar por 5 minutos,
    inclusive quando o core morre.
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

**`event.platformio.homeExited`** sai quando o processo termina sem um
`platformio.homeStop`: o timeout de inatividade ou uma falha. `code` é o
código de saída, ou `null` quando o processo foi morto por sinal. Depois dele,
o próximo `platformio.homeStart` sobe outro servidor.

**O que a página pede à IDE** (`open_project`, `open_text_document`,
`get_pio_project_dirs`, pelo agente WebSocket do PIO Home) entra na fatia
seguinte (52 §14.45 item 5). Até lá, valem as alternativas da própria página.
