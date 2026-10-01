# 09 — Glossário de identificadores (português → inglês)

> Criado em 2026-10-01 para a **varredura de idioma** (roadmap
> [53 §G0](../roadmaps/53-arquitetura-executavel-da-0.3.6.md)). Identificador é
> sempre em inglês ([08](08-convencoes-codigo-testes-commits.md)). Este glossário
> existe para que a mesma palavra vire o **mesmo** nome em todo o repositório:
> sem ele, `pasta` vira `folder` num arquivo e `dir` noutro, e a busca por
> conceito deixa de funcionar.
>
> Comentários e textos de tela continuam em português; isto vale só para nomes.

## Como usar

- Antes de traduzir um nome, procure a palavra aqui. Se ela não está, decida a
  tradução, **acrescente a linha** neste arquivo no mesmo commit, e use-a.
- Traduza o **conceito**, não letra por letra, e na ordem do inglês:
  `caminhoAbsoluto` → `absolutePath`, `linhaAtual` → `currentLine`,
  `pedidoPendente` → `pendingRequest`.
- Siga a convenção da linguagem: `camelCase` em QML/JS/C++ (membros),
  `snake_case` em Rust e Python, `PascalCase` em tipos.
- Nome que está no **protocolo** (`kinein-protocol`, campo serializado, método
  IPC) não se renomeia na varredura: é mudança de contrato, com versão e
  catraca de fiação, numa fatia própria.
- A ferramenta `scripts/rename_identifiers.py` troca só em código (nunca em
  comentário ou string); o compilador, o `qmllint` e os testes conferem o resto.

## As palavras mais frequentes (medidas em 2026-10-01)

| Português | Inglês | Nota |
| --- | --- | --- |
| raiz | root | `raiz` do workspace → `root` |
| linha / linhas | line / lines | |
| caminho | path | |
| texto | text | |
| erro | error | |
| indice | index | |
| saida | output | saída de processo; `exit` só para código de saída |
| pedido / pedidos | request / requests | |
| alvo / alvos | target / targets | |
| resultado | result | |
| arquivo / arquivos | file / files | |
| entrada / entradas | entry / entries; input | `entry` em lista/árvore, `input` em processo |
| achado / achados | finding / findings | |
| evento / eventos | event / events | |
| lista | list | |
| porta | port | |
| campo | field | |
| conferir | check / verify | `verify` quando confronta um valor esperado |
| ambiente | environment | |
| corpo | body | |
| falha / falhas | failure / failures | |
| falhou | failed | |
| mensagem | message | |
| aberto | open / opened | |
| resposta | response | |
| coluna | column | |
| atual | current | |
| cenario | scenario | em teste: `fixture` quando é o ambiente montado |
| projeto | project | |
| chave | key | |
| perfil | profile | |
| fonte | source | `font` só para tipografia |
| inicio | start | |
| resto | rest / remainder | |
| padrao | default; pattern | `default` para valor de fábrica, `pattern` para regex/glob |
| modelo | model | |
| tipo | kind / type | `kind` quando `type` colide com palavra reservada |
| escrever | write | |
| controlador | controller | |
| comando | command | |
| limpo | clean | |
| vistos | seen | |
| baixo | bottom; low | `bottom` para posição |
| titulo | title | |
| regras | rules | |
| binario | binary | |
| executavel / executaveis | executable / executables | |
| falso | fake (teste); false | `fake` para dublê de teste |
| outro | other | |
| destino | destination; target | |
| argumentos | arguments | |
| bloco | block | |
| conteudo | content | |
| detalhe | detail | |
| prazo | deadline / timeout | |
| novo | new | |
| busca | search | |
| tamanho | size | |
| passo | step | |
| rotulo | label | |
| colecao | collection | |
| itens | items | |
| abrir | open | |
| programa | program | |
| vazio | empty | |
| metodo | method | |
| objeto | object | |
| prefixo | prefix | |
| serializar | serialize | |
| pacote | package | |
| imagem | image | |
| receita | recipe | |
| versao | version | |
| banco | database | |
| guia | tab / guide | `tab` para aba de interface |
| nada | nothing / none | |
| ultimo | last | |
| lidos | read (particípio) | `readCount`, `readItems` |
| catalogo | catalog | |
| intencao | intent | |
| limite | limit | |
| tecla / teclas | key / keys | |
| codigo | code | |
| ferramenta / ferramentas | tool / tools | |
| registro | record; log | `record` para dado, `log` para registro de eventos |
| acao / acoes | action / actions | |
| plano | plan | |
| estado | state | |
| motivo | reason | |
| simbolo / simbolos | symbol / symbols | |
| cabecalho | header | |
| contexto | context | |
| unidade | unit | |
| familia | family | |
| espelho | mirror | |
| maquina | machine | |
| teclado | keyboard | |
| depois | after | |
| filho | child | |
| parado | stopped | |
| ativo | active | |
| relativo | relative | |
| aceito | accepted | |
| usuario | user | |
| sucesso | success | |
| espera | wait | |
| selecao | selection | |
| dica | hint | |
| esperado | expected | |
| pasta | folder; dir | `folder` na UI e no domínio; `dir` em API de sistema de arquivos |
| secao | section | |
| aba / abas | tab / tabs | |
| painel | panel | |
| janela | window | |
| botao | button | |
| trilho | rail | |
| espaco | space / spacing | |
| largura / altura | width / height | |
