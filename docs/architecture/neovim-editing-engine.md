# Motor de edição: o Neovim

Dono: a divisão de responsabilidades entre o Neovim, o core e a UI, as
condições da troca de editor e os requisitos herdados da prova. A avaliação
completa continua no 65; aqui fica só o que o core em C++ precisa cumprir.

## Quem é dono do quê

Do 65 §5 (65:185-200).

| Responsabilidade | Dono | Os outros |
| --- | --- | --- |
| Texto e versão do documento | Neovim (`changedtick`) | O core e a UI recebem eventos versionados e nunca editam por conta própria |
| Mudança de texto pela Kinein | Função Lua que confere o `changedtick` esperado e recusa o velho | — |
| Gravação do arquivo | Core, pelo `BufWriteCmd`, com a gravação guardada | Neovim avisa |
| Recuperação depois de queda | Core, pelo rascunho do espelho, sem swap | — |
| Processo do servidor LSP | Core, pelo lançador | Neovim conecta por socket e não cria processo |
| Protocolo LSP do documento | Neovim | O core não sincroniza o mesmo documento; o relé não manda `didOpen` |
| Configuração do servidor | Core (kit, preset, confiança) | Neovim recebe pronta |
| Diagnóstico do servidor | O servidor publica; o Neovim mostra | O core lê pelo relé para o painel de problemas |

## Decisões do Viktor de 2026-11-15 (65:890-903)

1. Sem modo por padrão; o modo do Vim é opção da pessoa.
2. O core grava o arquivo, pelo `BufWriteCmd`.
3. A recuperação é pelo rascunho do espelho, sem swap.
4. Sem Landlock, o projeto não confiável abre sem o editor novo.
5. O `nvim` vai fixado dentro do AppImage.
6. e 7. Multi-cursor e o verificador de correções seguem abertos.

## Condições para tirar o editor atual (65:870-883)

O editor atual fica ao lado do novo até o produto integrado passar nas quatro:
R1 e R12 pelo método da prova, R10 com o documento inteiro, IME real com fcitx5
ou ibus, e os testes do Windows com o Job Object. Se uma reprovar, a decisão
volta ao Viktor com a medida.

## O que já existe

O ramo `origin/prova/eng-60` (`8e35e471`) tem a prova dos 15 itens (65 §20).
O lado da UI já é C++: msgpack, cliente RPC e o `NvimView` (65:709). O
supervisor, que faz o papel do core, é Rust (Landlock, relé, espelho, cerca de
1.700 linhas). O que falta é o supervisor dentro do core em C++, no produto
integrado.

**A ordem.** O `nvim` nasce pelo lançador com Landlock, e por isso a
integração vem depois do lançador e dos prazos (65:885-888). A prova isolada
não depende da linguagem do backend; a integração depende do lançador.

## Requisitos da integração (65:777-805)

Oito defeitos que a prova achou e corrigiu, cada um com teste obrigatório na integração:

1. Depois de cada ação composta, a camada sem modo volta à inserção.
2. Digitar ou colar sobre a seleção não muda a área de transferência.
3. Com snippet ativo, o `Tab` salta de campo.
4. O contorno do `selection=exclusive` com o `vim.snippet`.
5. O lançador espera a sessão LSP anterior terminar antes de aceitar a nova.
6. O socket fica num caminho curto, e caminho acima de 108 bytes falha com mensagem.
7. A entrada trata o texto por ponto de código, não por unidade UTF-16.
8. A fábrica de acessibilidade é instalada antes do motor QML, e o R10 é lido pela AT-SPI.

E o limite do Landlock (65:814-821): ele barra o `execve`, não a carga de
biblioteca. O teste hostil cobre os vetores passivos (`runtimepath`,
`package.path`, `package.cpath`, modeline e `exrc`).
