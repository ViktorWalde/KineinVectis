# Terminal — ergonomia e ações da 0.3

> **Classe: ALVO EM EXECUÇÃO.** Escrito em 2026-09-22 depois de medir o
> terminal e iniciado em código em 2026-09-23 (`0.130.0`). O código continua
> sendo a fonte do que existe. Esta especificação complementa o roadmap de
> renderização/paridade; não troca o emulador, o PTY ou o renderer.

## 1. Problema de produto

O terminal já é tecnicamente real, mas ainda exige que o usuário conheça
convenções implícitas de terminal Linux. Ações comuns não estão visíveis:

```text
Copiar · Colar · Selecionar tudo · Limpar tela · Limpar histórico
Nova sessão · Fechar sessão
```

Isso é especialmente caro no Remote SSH: se copiar um comando, colar um path ou
limpar a saída for menos natural que abrir outro terminal, a integração remota
perde sua vantagem antes mesmo do fluxo SSH começar.

O objetivo não é esconder que existe um terminal. É preservar semântica de
shell/TUI e, ao mesmo tempo, respeitar o padrão muscular de aplicações de
desktop.

## 2. Baseline medido em 2026-09-22

Na medição anterior à implementação, o terminal já possuía:

- PTY real e emulação `xterm-256color` com `alacritty_terminal`;
- várias sessões e sessões de Run no mesmo host;
- seleção por mouse na grade visível;
- clipboard do sistema;
- `Ctrl+Shift+C` para copiar e `Ctrl+Shift+V` para colar;
- clique do meio para colar;
- bracketed paste quando a aplicação pede;
- `Ctrl+C` traduzido para `0x03` (`SIGINT` no shell);
- scrollback, barra de rolagem, tela alternativa e encaminhamento da roda para
  TUI; não encaminhamento completo de cliques/arrasto.

Naquele baseline ainda não havia:

- menu contextual;
- ações visíveis de Copiar/Colar/Selecionar tudo;
- `Ctrl+V` com semântica de desktop. `Ctrl+C` com semântica de desktop foi
  considerado e **recusado** pelo autor em 2026-09-24: interromper é o gesto
  que não pode competir com copiar (§4.1);
- seleção que represente todo o scrollback;
- ação explícita para limpar tela;
- ação separada para apagar o histórico do emulador;
- busca na saída;
- política de segurança para paste multilinha/com newline final;
- comandos catalogados para as operações locais do terminal.

Ponto particularmente enganoso naquele baseline: `Ctrl+V` caía na regra genérica de teclas
de controle e envia `0x16` (`^V`) ao PTY. Isso é coerente com um terminal
tradicional, mas diverge do gesto de colar esperado pelo autor.

### 2.1 Estado implementado em 2026-09-23

A primeira fatia da v0.3 já entrega, sem declarar a etapa inteira concluída:

- `Ctrl+C` **interrompe sempre** e `Ctrl+Shift+C` copia. Esta lista descrevia,
  até 2026-09-23, um `Ctrl+C` híbrido que copiava e consumia a seleção; o autor
  substituiu essa política em 2026-09-24 e a regra vigente está na §4.1;
- `Ctrl+V`, `Ctrl+Shift+V` e `Shift+Insert` colam; `Ctrl+Alt+V` envia o
  `^V` explícito; AltGr continua protegido;
- `Escape` limpa a seleção. Desde 2026-09-24 isso **não** é pré-requisito para
  interromper: a seleção não muda o significado de `Ctrl+C`;
- clique direito abre menu com Copiar, Colar, Selecionar área visível, Limpar
  tela, Limpar histórico, Novo terminal e Fechar terminal;
- menu também abre por `Shift+F10`/tecla Menu, aceita navegação por teclado,
  usa o espaço da janela (não fica cortado pela altura do terminal), rola se
  a janela não comportar a lista e fecha ao trocar de sessão ou perder foco;
- seleção é invalidada quando o texto selecionado muda, evitando copiar uma
  saída diferente daquela que o usuário marcou;
- colagem arriscada tem preview e confirmação antes do envio (§7);
- `terminal.clearScrollback { id }` (`0.130.0`) apaga o histórico real somente
  da sessão indicada e emite render imediato;
- menu e teclado usam IDs locais estáveis; a entrada dessas ações na paleta
  global fica para a convergência de commands da V3;
- harnesses QML e teste Rust com duas sessões cobrem a política e o isolamento.

Ainda faltam a prova manual em shell/TUI/SSH real, auditoria de acessibilidade,
busca e seleção de todo o scrollback. Portanto “Selecionar tudo” não é exibido:
a UI usa o nome honesto **Selecionar área visível**.

**Esclarecimento vinculante do autor em 2026-09-23:** a ação real
**Selecionar Tudo** é básica e obrigatória para fechar a 0.3.0. O nome honesto
da ação parcial evita uma promessa falsa, mas não satisfaz esse aceite. A
nomenclatura reutilizável das sessões da §5.2 também permanece pendente.

### 2.2 Retomada em 2026-09-24 — protocolo 0.131.0

Selecionar Tudo e nomes reutilizáveis estão implementados no checkout. A
seleção nativa do `alacritty_terminal` 0.26.0 alcança o buffer ativo retido,
inclusive scrollback; `terminal.copySelection` extrai o texto somente no gesto
de copiar. Unicode, wraps e tela alternativa permanecem a cargo do emulador.
Identidade por sessão/gesto impede cópia de seleção obsoleta ou de outra aba.
Saída nova invalida conservadoramente; rolagem conserva a seleção.
O autor revisou a política nesta retomada: `Ctrl+C` sempre envia interrupção
ao terminal, com ou sem seleção; somente `Ctrl+Shift+C`/Copiar copia. A cópia
preserva a seleção. Não há alternância entre copiar e interromper.

Menu/teclado usam `TerminalActionsController`, extraído do painel; nomes
continuam no `RuntimeController`. `terminal`, `terminal1` etc. ocupam a primeira
posição livre, sem reaproveitar IDs nem saída. Títulos específicos permanecem.

O teste com eventos Qt reais encontrou `Shift+F10` interceptado pelo Executar
global. `Keys.onShortcutOverride` agora reserva o gesto com foco no terminal;
o menu consome atalhos enquanto aberto. `Ctrl+A` continua no shell e
`Ctrl+Shift+A` na paleta. Selecionar Tudo pelo teclado usa Menu/`Shift+F10`,
setas e Enter, sem um novo atalho direto. Isso segue a cautela do
[VS Code no Linux](https://code.visualstudio.com/docs/terminal/basics), consultado
em 2026-09-24. O contrato de precedência é o do
[Qt Keys](https://doc.qt.io/qt-6/qml-qtquick-keys.html#shortcutOverride-signal).
Validação e pendências estão no roadmap 40 §7.91.

## 3. Referências verificadas, sem copiar arquitetura

Pesquisa revisada em **2026-09-23**, conforme MODE-B do
[roadmap de adaptação](../roadmaps/adaptacao-de-plugins-abertos.md).
A documentação da JetBrains consultada é da IntelliJ IDEA 2026.2; o código
Code OSS foi lido na revisão `0896e62ebab6f42f24240a9ad72ab3147f7b297e`
(2026-09-23), sob licença MIT. São referências de comportamento: não houve
cópia/transposição de fonte nem adoção de xterm.js ou engine JetBrains.

| Comportamento observado | Decisão na Kinein |
| --- | --- |
| VS Code/Linux usa `Ctrl+Shift+C/V`; Windows tem copiar e consumir seleção por `Ctrl+C` | Decisão de 2026-09-24: `Ctrl+C` sempre interrompe e `Ctrl+Shift+C` copia; `Ctrl+V` cola. |
| JetBrains oferece configuração para `Ctrl+C/V`, copiar ao selecionar e atalhos de terminal | Manter memória muscular pedida; copiar ao selecionar permanece desligado, sem prometer preferência já implementada. |
| VS Code/Linux oferece menu contextual; menu agrupa edição, limpeza e sessão | Reusar `AppMenuPopup`, com ações locais, foco, teclado e limite de altura; não copiar DOM/CSS. |
| VS Code avisa colagem multilinha fora de bracketed paste; oferece colar em uma linha | Preview + Cancelar/Colar em uma linha/Colar. Kinein também confirma newline final e controles; diferenças na §7. |
| VS Code distingue clipboard normal de PRIMARY (`Shift+Insert` no Linux) | Kinein usa o clipboard normal em todos os aliases/clique do meio; PRIMARY ainda não implementado. |
| VS Code tem busca e seleção de todo o buffer; JetBrains mostra sessões em abas e ação de nova sessão | Reusar sessões existentes; busca/buffer completo continuam pendentes. Não chamar seleção visível de “tudo”. |
| JetBrains usa `Alt+F12` para abrir/focar o terminal | Não exibir esse atalho em “Novo terminal”: focar e criar sessão são ações diferentes. |

Fontes oficiais consultadas:

- [VS Code — Terminal Basics](https://code.visualstudio.com/docs/terminal/basics),
  [Appearance](https://code.visualstudio.com/docs/terminal/appearance).
- [Code OSS — contribuição de clipboard](https://github.com/microsoft/vscode/blob/0896e62ebab6f42f24240a9ad72ab3147f7b297e/src/vs/workbench/contrib/terminalContrib/clipboard/browser/terminal.clipboard.contribution.ts),
  [política de colagem](https://github.com/microsoft/vscode/blob/0896e62ebab6f42f24240a9ad72ab3147f7b297e/src/vs/workbench/contrib/terminalContrib/clipboard/browser/terminalClipboard.ts),
  [testes](https://github.com/microsoft/vscode/blob/0896e62ebab6f42f24240a9ad72ab3147f7b297e/src/vs/workbench/contrib/terminalContrib/clipboard/test/browser/terminalClipboard.test.ts),
  [licença MIT](https://github.com/microsoft/vscode/blob/0896e62ebab6f42f24240a9ad72ab3147f7b297e/LICENSE.txt).
- [JetBrains — Terminal settings](https://www.jetbrains.com/help/idea/settings-tools-terminal.html),
  [Terminal tool window](https://www.jetbrains.com/help/idea/terminal-emulator.html).
- [xterm.js — tratamento de clipboard](https://github.com/xtermjs/xterm.js/blob/master/src/browser/Clipboard.ts)
  (consulta ao ramo móvel em 2026-09-23): referência para normalização de Enter
  e neutralização de ESC em bracketed paste, sem código incorporado.
- [Qt — foco de teclado](https://doc.qt.io/qt-6/qtquick-input-focus.html):
  eventos não aceitos propagam; menu/diálogo consomem teclas para não atingir
  o terminal atrás deles.

IntelliJ Community também foi consultado na revisão
`726ef46761f696682ee4a67767bd089b2566c877` (2026-09-23, arquivos sob Apache 2.0):
[TerminalSelectAllAction](https://github.com/JetBrains/intellij-community/blob/726ef46761f696682ee4a67767bd089b2566c877/plugins/terminal/src/org/jetbrains/plugins/terminal/action/TerminalSelectAllAction.kt)
seleciona de zero ao comprimento do documento do editor do terminal reworked;
não só o viewport.
[TerminalTitleUtils](https://github.com/JetBrains/intellij-community/blob/726ef46761f696682ee4a67767bd089b2566c877/plugins/terminal/src/org/jetbrains/plugins/terminal/util/TerminalTitleUtils.kt)
consulta nomes existentes e delega ao
[UniqueNameGenerator](https://github.com/JetBrains/intellij-community/blob/726ef46761f696682ee4a67767bd089b2566c877/platform/util/src/com/intellij/util/text/UniqueNameGenerator.java)
para escolher um nome livre. A forma `terminal`, `terminal1` é decisão da
Kinein, não uma alegação de que essas IDEs usam exatamente esse texto/sufixo.
Não se importam a arquitetura de editor do terminal nem seu gerador de nomes.

## 4. Política de teclado recomendada

### 4.1 `Ctrl+C` e `Ctrl+Shift+C` — decisão de 2026-09-24

```text
Ctrl+C        → enviar 0x03 ao PTY, com ou sem seleção
Ctrl+Shift+C  → copiar a seleção; sem seleção, não fazer nada
```

O autor rejeitou a alternância “primeiro copia, segundo interrompe” por ser
inconveniente e potencialmente problemática. A seleção não muda o significado
de `Ctrl+C`; não é necessário limpá-la antes de interromper. O byte segue ao
PTY e o programa em primeiro plano mantém sua semântica de interrupção.

Copiar pelo menu ou por `Ctrl+Shift+C` preserva a seleção. O menu anuncia
`Ctrl+Shift+C`; Escape continua disponível para desfazer a seleção.

### 4.2 `Ctrl+V`

`Ctrl+V` cola o clipboard no terminal. `Ctrl+Shift+V` e `Shift+Insert`
continuam como aliases. O byte `^V`, quando realmente necessário para quoted
insert do shell, precisa de um gesto documentado/configurável antes de ser
retirado sem substituto. O gesto implementado é **Ctrl+Alt+V**, coberto pelo
harness de teclado e pela verificação de atalhos.

Bracketed paste continua sendo respeitado. A UI não executa nem avalia o
conteúdo; as normalizações de transporte e a única transformação opcional
são explícitas na §7.

### 4.3 Seleção e TUI

- arrastar seleciona localmente o grid renderizado; cliques/arrasto ainda não
  são encaminhados à TUI (lacuna anterior, não paridade completa de mouse);
- a roda respeita o modo de mouse da TUI no core; `Shift` força rolagem local;
- clique direito abre o menu contextual da IDE;
- se uma TUI precisar do botão direito, **Shift+clique direito** precisa ter
  comportamento documentado e testado antes de liberar o evento para ela;
- mudar de sessão limpa a seleção visual da sessão anterior;
- se a saída mudar o texto selecionado, invalidar a seleção, sem copiar novos
  caracteres usando coordenadas antigas. Frames com o mesmo texto a preservam.

## 5. Menu contextual e ações

O clique direito no viewport deve abrir, no mínimo, as ações abaixo. A coluna
de efeito explicita a ação básica que ainda falta no recorte `0.130.0`:

| Ação | Habilitação | Efeito |
| --- | --- | --- |
| Copiar | há seleção | copia exatamente a seleção |
| Colar | clipboard tem texto e sessão está viva | envia paste ao PTY |
| Selecionar Tudo | sessão viva disponível | todo o buffer ativo retido; copiar é um gesto separado |
| Selecionar área visível | sessão tem conteúdo | extra já implementado; não substitui Selecionar Tudo |
| Limpar tela | sessão está viva | equivale ao gesto de terminal `Ctrl+L` |
| Limpar histórico | existe scrollback | apaga o scrollback do emulador |
| Nova sessão | workspace aberto | abre outro shell |
| Fechar sessão | sessão existe | fecha somente a sessão ativa |

Separar **Limpar tela** de **Limpar histórico** é obrigatório:

- limpar tela pede ao programa/shell que redesenhe e pode preservar scrollback;
- limpar histórico altera o buffer mantido pela IDE e precisa de contrato no
  core;
- nenhum dos dois mata o processo.

“Selecionar tudo” significa todo o buffer/scrollback da sessão, não apenas as
linhas visíveis. Enquanto o core não expuser isso corretamente, a UI deve usar
o nome honesto **Selecionar área visível** para a ação parcial. Isso não libera
a 0.3.0 sem a ação completa.

### 5.1 Alcance de Selecionar Tudo — implementado em 0.131.0

- Selecionar todo o texto retido no buffer ativo do terminal selecionado,
  incluindo scrollback, independentemente da posição de rolagem. Copiar é o
  gesto seguinte; selecionar não modifica o clipboard sozinho.
- Não incluir outras abas, sessões encerradas, histórico de comandos do shell
  em disco nem texto já descartado pelo limite de scrollback. Tela alternativa
  não deve trazer conteúdo oculto do buffer normal para uma cópia inesperada.
- Fechar a sessão e criar outra produz um novo buffer: se só existe o prompt,
  só ele é selecionado/copiado. Reutilizar o rótulo da aba não restaura saída.
- Estender o emulador e os controllers atuais: extração sob demanda no dono do
  buffer, sem duplicar todo o histórico em QML. Preservar Unicode/glifos largos
  e distinguir quebra de linha real de wrap visual.
- Associar seleção/resposta ao ID e estado da sessão; saída nova, clear,
  resize e troca de buffer não podem copiar texto diferente sem invalidar ou
  atualizar explicitamente a seleção. Resposta atrasada nunca copia outra aba.
- Menu e teclado passam pela mesma ação. Não capturar `Ctrl+A` do shell
  indiscriminadamente: ele já é usado para início de linha; definir o atalho
  da IDE sem perder essa função, seguindo as referências e o catálogo atual.

### 5.2 Nomes de sessões — implementado em 0.131.0

Primeiro shell: `terminal`; próximos: `terminal1`, `terminal2` etc. Ao criar,
escolher o primeiro nome livre entre as abas existentes. Fechar `terminal`
libera esse nome; fechar `terminal1` libera esse número, sem renomear as outras.
Preservar títulos específicos de Run/serial/container e evitar colisões.

`RuntimeController.qml` escolhe o primeiro nome livre e `tst_multi_terminal.qml`
prova a reutilização, sem outro gerenciador. **IDs técnicos continuam únicos**,
separados dos rótulos; eventos
atrasados de uma sessão fechada não podem atingir a nova de mesmo nome.

O momento de iniciar o primeiro shell ainda precisa de confirmação: ao abrir
o projeto ou ao abrir o painel Terminal. Até essa decisão, preservar o ciclo
sob demanda existente; não iniciar processos extras só para reservar rótulo.

## 6. Ações e commands

As ações locais devem ter IDs estáveis e passar pelo caminho comum da UI:

```text
terminal.copy
terminal.paste
terminal.selectAll
terminal.selectVisible
terminal.clearScreen
terminal.clearScrollback
terminal.new
terminal.close
terminal.find
```

Em `0.131.0`, os oito primeiros IDs usados pelo menu são locais ao painel;
`terminal.find` e o catálogo global ainda não foram implementados.

O clipboard é local à UI. O core recebe as operações sobre o emulador/sessão:
limpar histórico, selecionar o buffer retido e extrair sua seleção sob demanda.

Menu contextual, paleta, toolbar e atalhos devem apontar para a mesma ação. Não
criar lógica diferente em cada superfície.

## 7. Paste seguro — implementado, validação real pendente

- Uma linha sem controles cola imediatamente. Fora de bracketed paste,
  qualquer CR/LF (inclusive newline final/CR isolado) pede confirmação.
- Com bracketed paste ativo, multilinha comum segue sem diálogo; o programa
  receptor continua responsável por interpretá-la. Esse modo não é sandbox.
- Controles C0/DEL, exceto tab/CR/LF, sempre pedem confirmação. O preview é
  texto simples, limitado a 3.000 caracteres e indica truncamento; o envio não
  é truncado. Não há interpretação de markup.
- **Cancelar** é a escolha inicial. **Colar** preserva as linhas;
  **Colar em uma linha** remove CR/LF finais e troca quebras internas por
  espaços, somente por escolha explícita. Não garante que comandos como
  `a; b` sejam inofensivos: o usuário precisa revisar o conteúdo.
- LF/CRLF são normalizados para CR, como Enter de terminal. ESC colado vira
  `␛` (U+241B), com aviso, para não romper o envelope `ESC[200~…ESC[201~`.
  Outros controles só seguem depois de confirmação.
- Trocar/encerrar a sessão ou esconder o terminal cancela o texto pendente;
  enquanto o diálogo está aberto, teclas não são enviadas ao PTY. Outro
  pedido de colagem, inclusive pelo botão do meio, não substitui o pendente
  nem envia conteúdo por trás do diálogo.
- Não registrar clipboard em log/métrica nem manter histórico próprio.

Diferença deliberada do VS Code: sua política automática admite remover
newlines finais de um único comando sem diálogo. Aqui essa transformação é
explícita, preservando o requisito de não modificar comandos silenciosamente.
O estado fica no `TerminalInputController` existente; o diálogo apenas compõe
`KvPanelFrame`/`KvButton`. Shells, Vim/TUI e SSH real ainda precisam de medição
manual antes de declarar a política aprovada para uso diário.

## 8. Outras lacunas ergonômicas a medir

Não viram escopo automaticamente, mas precisam entrar no roteiro de uso:

- busca no scrollback com próximo/anterior;
- `Ctrl+PageUp/PageDown` para trocar sessão;
- renomear uma sessão;
- reabrir shell na pasta do arquivo ou diretório selecionado;
- `Ctrl+clique` em paths/URLs reconhecidos;
- zoom de fonte independente do editor;
- copiar automaticamente ao selecionar, como preferência desligada por padrão;
- confirmação ao fechar uma sessão com processo interativo vivo;
- acessibilidade do grid, menu e seleção;
- indicação visual de aplicação em tela alternativa/captura de mouse.

Para a 0.3, busca, troca de sessão por teclado e confirmação de paste são
alvos. Links clicáveis, drag-and-drop, renomear sessão e zoom entram somente se
as fatias essenciais estiverem fechadas.

## 9. Fatias

### T0 — baseline

**Parcial:** baseline automatizado verde; roteiro real ainda pendente.

- roteiro com shell, processo longo, Vim/TUI e sessão SSH;
- registrar comportamento de `Ctrl+C/V`, seleção, clique do meio e scroll;
- harness atual continua verde;
- verificar colisões no catálogo de atalhos.

### T1 — ações visíveis e memória muscular

**Implementado na superfície local em `0.130.0`:** atalhos, menu e IDs locais.
Catálogo/paleta global permanece na V3 para não criar um segundo dispatcher.

- `Ctrl+C` só interrompe e `Ctrl+Shift+C` copia (revisado em 2026-09-24);
- `Ctrl+V` como paste e aliases preservados;
- menu contextual com Copy/Paste/Nova/Fechar/Limpar tela;
- ações catalogadas e testadas;
- paste simples e bracketed paste sem regressão.

### T2 — buffer completo

**Implementação básica em `0.131.0`:** limpar scrollback, seleção completa e
nomes reutilizáveis; busca é alvo separado. **Prova humana feita: o autor rodou
o roteiro real em 2026-09-24 e deu a fatia por concluída.** O que automação
nunca cobriu aqui — teclas num shell de verdade, TUI, sensação de uso — passou
a ter dono e data.

- contrato mínimo para limpar scrollback;
- seleção de todo o buffer, sem aceitar seleção visível como conclusão;
- nomes de sessão reutilizáveis, sem reutilização de identidade/buffer;
- busca no histórico, se o mesmo contrato permitir sem duplicar o buffer no
  QML;
- testes no core provam que limpar uma sessão não toca as demais.

### T3 — segurança e acabamento

**Parcial:** confirmação de colagem e navegação do menu implementadas e
cobertas automaticamente; validação real e demais itens ainda abertos.

- paste multilinha/newline;
- troca de sessão por teclado;
- foco e menu acessíveis;
- teste com Bash/Zsh, Vim, htop e SSH real;
- manual e tabela de atalhos atualizados.

## 10. Critérios de aceite da 0.3

- o clique direito torna as ações comuns descobríveis;
- `Ctrl+C` interrompe com ou sem seleção, sem alterar o clipboard;
- `Ctrl+V` cola, inclusive em shell remoto;
- `Ctrl+Shift+C/V` continuam funcionando;
- `SIGINT`, `^V` explícito, bracketed paste e AltGr têm testes;
- limpar tela e limpar histórico têm nomes e efeitos distintos;
- “Selecionar Tudo” existe e seleciona o buffer inteiro retido da sessão ativa;
- nova sessão não copia saída de uma sessão encerrada de mesmo nome;
- nomes `terminal`, `terminal1` etc. reutilizam a primeira posição livre;
- paste arriscado não executa várias linhas por acidente sem política medida;
- seleção/cópia funcionam no scrollback e com glifos largos;
- nenhuma ação vaza clipboard para logs;
- o fluxo é exercitado numa sessão SSH porque Remote reutiliza este terminal.

## 11. Decisões congeladas e pendência

Congelado para a primeira fatia da v0.3:

- `Ctrl+C` só interrompe; `Ctrl+Shift+C` copia (decisão de 2026-09-24);
- `Ctrl+Alt+V` como gesto explícito para `^V`;
- copiar ao selecionar desligado — copiar exige ação do usuário;
- aliases `Ctrl+Shift+C/V` e clique do meio preservados;
- Selecionar Tudo completo é básico, não extra nem busca opcional;
- rótulos de sessão usam o primeiro nome livre (§5.2).

Ainda precisa de medição antes de fechar a fatia: a política implementada de
paste multilinha/newline final em shell comum, TUI e SSH, além de
acessibilidade e interação visual. Não confundir testes de lógica com essa
prova de uso real.

## 12. Roteiro de validação visual/manual — ainda não executado

Usar o launcher `scripts/kinein-vectis` depois dos builds debug/release e do
smoke. Conferir no stderr os caminhos efetivamente abertos; não testar um
AppImage antigo supondo que contém a revisão.

1. Abrir um workspace e usar `Alt+F12`. Abrir outro terminal pelo `+`/menu,
   alternar e fechar somente a sessão escolhida. Confirmar que “Novo terminal”
   não anuncia o atalho de apenas abrir/focar o painel.
2. Gerar texto, marcar com mouse e copiar com `Ctrl+Shift+C`: a seleção fica.
   Rodar `sleep 30`, marcar texto e interromper com `Ctrl+C` já na primeira
   tentativa, sem alterar o clipboard. Repetir sem seleção. Conferir também
   `Ctrl+Shift+C/V`, `Ctrl+V`, `Shift+Insert`, botão do meio e `Ctrl+Alt+V`.
3. Abrir o menu por clique direito e `Shift+F10`. Navegar, cancelar e testar
   altura reduzida. Letras não devem chegar ao prompt atrás do menu. Clicar
   no editor precisa fechar o menu sem devolver foco à força ao terminal.
4. Copiar texto inofensivo com várias linhas. Bash/Zsh com bracketed paste
   podem aceitá-lo diretamente como edição; fora desse modo, revisar o
   diálogo, cancelar e testar “Colar em uma linha”. Não usar comandos
   destrutivos para validar a proteção. Trocar/fechar a sessão e esconder o
   painel não podem reaplicar uma confirmação antiga.
5. Produzir histórico, rolar para trás e limpar. O indicador deve voltar ao
   fundo; o conteúdo visível e outra sessão permanecem. “Limpar tela” depende
   da interpretação de `Ctrl+L` pelo programa, não equivale a apagar histórico.
6. Conferir caracteres largos/acentos e seleção durante frames idênticos e
   durante saída que substitui o texto. A primeira deve persistir; a segunda
   deve ser invalidada, não copiar conteúdo novo em coordenadas antigas.
7. Repetir seleção/paste/roda/foco em Vim, htop e SSH real. Registrar shell,
   modo bracketed paste, plataforma e diferenças. Cliques capturados por TUI
   e PRIMARY continuam lacunas explícitas, não resultados aprovados.
8. Verificar leitor de tela, navegação sem mouse e legibilidade em 800×500.
   O harness de geometria ajuda, mas não certifica acessibilidade nem UX.
9. **Bloqueado pela implementação pendente:** gerar saída maior que a tela,
   Selecionar Tudo e copiar; comparar também quando rolado. Fechar a sessão,
   abrir nova com o mesmo nome e confirmar que só seu conteúdo é copiado.
10. **Bloqueado pela implementação pendente:** abrir `terminal`, `terminal1`,
    `terminal2`; fechar a primeira e criar outra, esperando `terminal`. Fechar
    a intermediária e repetir; conferir IDs, output e aba Run independentes.
