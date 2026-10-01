# Projetos, arquivos e integração com o desktop — série 0.3

> **Classe: ALVO, ainda não implementado como fluxo completo.** Decisões do
> autor em 2026-09-23: abrir a Kinein pelo terminal e trabalhar com pastas deve
> entrar na 0.3.0 ou em 0.3.1–0.3.4, sem esperar o Grafana da 0.3.5. A interação
> com o projeto precisa cobrir o ciclo diário completo, não só aceitar um drop.
> A distribuição exata segue como proposta no
> [roadmap 48](../roadmaps/48-arquitetura-executavel-da-serie-0.3.md).

## 1. Referências e critério de paridade

Pesquisa oficial em 2026-09-23, sem incorporar código de terceiros:

- [VS Code — CLI](https://code.visualstudio.com/docs/configure/command-line):
  `code .` abre a pasta corrente; paths relativos usam o diretório de invocação.
  Abrir janela nova e reutilizar janela são opções distintas.
- [VS Code — Explorer](https://code.visualstudio.com/docs/editing/getting-started/userinterface#_explorer-view):
  mover dentro da árvore, copiar arquivos externos para ela, seleção múltipla
  e menus contextuais são partes do mesmo fluxo. O destino do arrasto importa.
- [IntelliJ — Project tool window](https://www.jetbrains.com/help/idea/project-tool-window.html):
  criar, recortar/copiar/colar, copiar caminhos, renomear e excluir ficam
  acessíveis pelo contexto do item; não apenas por uma toolbar global.
- [IntelliJ — abertura pela CLI](https://www.jetbrains.com/help/idea/opening-files-from-command-line.html):
  o launcher recebe o caminho do arquivo ou projeto e precisa estar acessível
  ao shell. Não é uma segunda instância da arquitetura de workspace.

Esses produtos não têm todos os defaults iguais. Cada gesto precisa de regra
explícita na Kinein, teste e justificativa quando divergir. “Paridade completa”
nesta frente significa cumprir a matriz abaixo; não significa declarar todos
os recursos de duas IDEs inteiras implementados. Dúvidas de escopo ficam
registradas na §7 e são levadas ao autor, não descartadas silenciosamente.

## 2. Base existente, medida no checkout em 2026-09-23

| Dono existente | O que reaproveitar | Lacuna deste fluxo |
| --- | --- | --- |
| `scripts/kinein-vectis` | seleciona UI/core e encaminha argumentos, preservando o diretório de invocação | comando curto instalado e contrato CLI documentado/testado |
| `ui/src/core_client_process.cpp` | reconhece argumento de pasta existente e chama `openWorkspace` quando o core inicia | parsing explícito, diagnóstico de erro e política de janela |
| `ProjectExplorer.qml` / `ProjectTreeController.qml` | listagem, expansão, seleção simples, abertura, menus e pedidos de criação/rename/delete | seleção múltipla, teclado completo, clipboard de arquivos e drag-and-drop |
| `ProjectTreeRequestRouter.qml` / `CoreClient` | transporte único de intenções para `fs.*` | estender contratos somente para capacidades ausentes |
| `crates/kinein-core/src/fsops/` | confinamento, criação, escrita, rename/move sem sobrescrita e delete | cópia/importação, lote, recuperação e progresso quando necessários |
| controllers de editor/workspace | documentos, abas e atualização após rename/delete | provar buffers sujos e respostas atrasadas em todos os novos caminhos |

O `fs.delete` atual é exclusão permanente, inclusive recursiva para pastas.
Não há autorização para apresentá-lo como lixeira/desfazer. O `fs.rename`
existente já move dentro do workspace; não criar um segundo motor de rename.

## 3. P0 — abrir pelo terminal e pelo desktop

> **Concluído em 2026-09-26** — o comando curto `kinein` passou a ser instalado
> pelos dois caminhos: `scripts/instalar-atalho.sh` (checkout) e o instalador
> que viaja com o AppImage. Ele acrescenta **uma** coisa ao binário: sem
> argumento nenhum, abre a pasta corrente. Todo o resto passa intacto, porque o
> dono do contrato de argumentos é `ui/src/cli_args`, com teste C++. O modelo
> tem um dono só (`scripts/kinein.in`) e o empacotador o embute no instalador
> entregue, para que um download de dois arquivos baste; o gate de distribuição
> cobra essa etapa.
>
> **Decidido pelo autor em 2026-09-26:** *mesma pasta foca a janela aberta;
> outra pasta abre janela nova.* Implementado no mesmo dia. Cada janela que tem
> um workspace aberto escuta num socket de domínio Unix dentro do
> `XDG_RUNTIME_DIR`, nomeado por um digest do caminho canônico; quem chega
> depois pergunta, e **o caminho inteiro viaja na mensagem e é conferido** —
> colisão de nome não faz uma pasta se passar por outra.
>
> O que isso protege não é conforto: **não há lock de workspace**, e duas
> janelas na mesma pasta são duas donas do `.kinein/`, escrevendo o mesmo
> `session.json` e o mesmo índice. A última a fechar apaga o que a outra
> gravou, sem erro e sem aviso.
>
> **Como era em 2026-09-24.** O contrato de argumentos passou a
> ter dono (`ui/src/cli_args`), teste (o primeiro teste C++ do projeto) e
> diagnóstico. `--help` e `--version` respondem sem subir UI, core ou rede;
> caminho inválido é recusado com o motivo e sem criar nada. Faltava o comando
> curto instalado, que é o que a nota acima fecha. Registro em
> [`roadmap 40`](../roadmaps/40-estado-e-continuidade.md) §7.97.
>
> Decisão respeitada: o binário sem argumento **não** trata o CWD como projeto —
> é o que esta seção manda, porque o atalho do desktop roda sem argumento de um
> diretório qualquer. O default de CWD é do comando curto, e a §7 o registra como
> sugestão a confirmar.

Contrato de produto a implementar:

| Entrada | Resultado esperado |
| --- | --- |
| `kinein .` | abrir a pasta corrente como workspace |
| `kinein caminho` | resolver path relativo ao terminal de origem; aceitar espaços/Unicode |
| `kinein` sem argumentos | proposta de adaptação ao pedido: abrir a pasta corrente; não atribuir esse default ao VS Code |
| pasta vazia | abrir normalmente, mostrar estado vazio e ações de criar; não exigir manifesto |
| pasta com conteúdo | mostrar a árvore, detectar o projeto e disponibilizar conteúdo sob demanda |
| path inválido/inacessível | erro útil, sem criar pasta nem substituir o projeto atual silenciosamente |
| `--help` / `--version` | responder sem iniciar UI/core nem fazer rede |

“Ler tudo” significa disponibilizar o projeto e seus arquivos, não carregar
recursivamente todos os conteúdos em memória nem executar scripts encontrados.
Preservar as regras existentes de detecção/indexação e explicar filtros da
árvore; arquivos binários/grandes não devem bloquear a abertura.

A instalação do comando deve ser explícita, reversível e funcionar no artefato
distribuído, não apenas no checkout. Não editar automaticamente `.bashrc`,
`.zshrc` ou sobrescrever executável homônimo. O launcher existente permanece o
ponto de partida; esta especificação não instala nada no computador do autor.

Janela já aberta, abertura pelo menu do desktop e execução pelo shell precisam
de políticas separadas. Abertura do desktop sem path não deve tratar um CWD
arbitrário como projeto. Antes de reutilizar janela, resolver documentos não
salvos pelo fluxo comum de Salvar/Descartar/Cancelar; cancelamento preserva o
workspace. Não anunciar `--reuse-window` sem transporte entre instâncias e
testes. A decisão final de defaults está na §7.

## 4. P1–P3 — matriz obrigatória de interação com o projeto

Todas as linhas são alvo; a existência de uma operação básica na §2 não prova
o fluxo completo. Não fechar esta frente enquanto houver linha sem teste ou
decisão explícita do autor que a reprograme.

| Contexto/gesto | Contrato de interação e aceite |
| --- | --- |
| Navegar na árvore | setas, expandir/recolher, Home/End, foco visível, busca pelo nome digitado; mouse e teclado alcançam os mesmos itens |
| Selecionar | seleção simples, Ctrl para itens independentes, Shift para intervalo; Ctrl+A seleciona itens da árvore focada, nunca texto do terminal/editor |
| Abrir arquivo | gesto consistente de clique/Enter/duplo clique; abrir item já aberto o ativa; nunca duplicar nem descartar buffer sujo |
| Menu contextual | ação no item/seleção sob contexto, inclusive tecla Menu/Shift+F10; habilitação correta, motivo de indisponibilidade e retorno de foco |
| Criar/renomear | destino explícito, validação de nome/colisão/permissão; cancelar não escreve; preservar identidade e conteúdo das abas afetadas |
| Copiar/recortar/colar | clipboard de arquivos distinto de copiar texto/path; recorte só move no paste bem-sucedido; falha não perde origem |
| Arrastar dentro da árvore | mover por padrão; Ctrl explicita cópia no Linux; realçar destino e operação, autoscroll e expansão por hover; Escape cancela |
| Soltar em arquivo ou área vazia da árvore | indicar antes do drop a pasta pai ou raiz que receberá a operação; nunca usar destino implícito diferente do realce |
| Arrastar arquivos/pastas de fora para árvore com workspace | importar/copiar no destino escolhido, com colisões tratadas; não mover a origem externa nem trocar o projeto |
| Arrastar pasta para acolhimento/área de abertura de projeto | abrir como workspace pelo mesmo fluxo de proteção; não confundir com importar para a árvore |
| Arrastar arquivo da árvore para o editor | abrir/ativar documento; não mover o arquivo no disco |
| Arrastar arquivo externo para o editor | uma URL local única abre aba somente leitura, sem importar nem escrever na origem; ver decisão da §7 |
| Arrastar para aplicativo externo | oferecer URLs de arquivos locais, sem texto sensível nem exclusão inferida da origem; validar integração nativa |
| Colisões | informar origens/destinos, permitir cancelar, pular ou escolher novo nome; nunca sobrescrever por padrão nem mesclar pastas às cegas |
| Excluir/recuperar | preferir lixeira recuperável quando suportada; exclusão permanente é explícita e confirmada; desfazer nunca promete recuperação inexistente |
| Ações de contexto úteis | copiar path absoluto/relativo, abrir pasta no gerenciador e terminal nessa pasta, usando serviços existentes |
| Mudança interna/externa | árvore, abas, Git e índice refletem o resultado real; alterações externas não substituem buffer sujo sem resolver conflito |
| Erro/cancelamento/lote grande | feedback e progresso observáveis; discriminar o que concluiu/falhou; não congelar UI nem anunciar atomicidade de um lote parcial |

**Estado medido em 2026-09-29:** a árvore tem seleção por path com Ctrl,
Shift, Ctrl+Shift e Ctrl+A quando está focada. O cursor pode andar com
Ctrl+seta sem alterar a seleção; refresh preserva paths existentes e remove
os que saíram da árvore visível. `Menu` e `Shift+F10` abrem o mesmo menu do
clique direito; setas, Enter e Escape funcionam nele, e Escape devolve o foco
à árvore. Em seleção múltipla, renomear/excluir e os botões globais de criar
ficam indisponíveis com motivo visível: essas operações ainda não têm contrato
de lote. Criar pelo menu usa o item sob contexto como destino. Clipboard de
arquivos, drag-and-drop, lixeira e colisões de lote seguem como alvo, sem
serem inferidos dessa fatia.

**Primeira fatia de P2 em 2026-09-29:** o explorador usa o `Clipboard` já
exposto ao QML para colocar uma URL local no clipboard do sistema, com marca
de recorte quando couber. Menu e Ctrl+C/Ctrl+X/Ctrl+V chegam ao mesmo
`ProjectFileClipboard`; a colagem abre o diálogo de nome já usado para
renomear, mostra origem e destino e permite escolher outro nome antes de
escrever. Enquanto o pedido está pendente, o diálogo indica a operação.
Copiar chama `fs.copy`; recortar usa o `fs.rename` existente e só limpa a
marca depois do sucesso. Colisão volta ao diálogo sem apagar a origem. A
primeira fatia aceita **um item do workspace corrente**; a indisponibilidade
de seleção múltipla ou fonte externa aparece no menu. A cópia usa o JobManager
existente para progresso por bytes e cancelamento entre blocos; ao cancelar,
remove o staging e preserva a origem. Durante a cópia, **Ver em Jobs** fecha o
diálogo e abre o painel onde o job pode ser cancelado; o resultado ainda volta
à mesma intenção de colagem. Drag-and-drop, importação externa,
lote, decisões de colisão em lote e prova nativa do clipboard seguem
pendentes. P2 e P3 não estão fechadas.

**Lote de clipboard em 2026-09-30:** `fs.transferBatch` aceita pares explícitos
de cópia ou movimento dentro do workspace. Valida o conjunto antes da primeira
mutação, usa os motores `fs.copy`/`fs.rename`, publica progresso pelo JobManager
e devolve resultado por item quando uma falha de execução torna o lote parcial.
Seleção múltipla em Ctrl+C/Ctrl+X e no menu chega ao diálogo de revisão: cada
item pode ter nome editado ou ser pulado, sucessos não são reenviados após
falha parcial e o recorte preserva no clipboard apenas as fontes ainda não
movidas. O diálogo abre Jobs para acompanhar ou cancelar. Colisão detectada
no preflight volta como erro do lote; uma colisão tardia aparece no item
afetado. Arrastar e soltar, importação externa e prova nativa continuam
pendentes.

**Encaminhamento do arrasto interno em 2026-09-30:** a árvore agora anuncia
URLs locais e paths internos; linha e espaço vazio compartilham o mesmo
destino de drop. O destino aparece antes da soltura, com expansão da pasta por
hover e rolagem nas bordas. O drop usa a intenção de cópia/movimento e o
diálogo de transferência acima, sem alterar o clipboard. O parser de MIME
foi exercitado em QML; os demais destinos e a fonte externa ainda precisam
de prova nativa.

Em X11, o arrasto interno real foi provado em 2026-09-30: mover A→B e
Ctrl+arrastar B→A abriram os diálogos corretos e produziram os arquivos
esperados. O Qt entrega o MIME do gesto nativo em `formats`, não em `keys`; o
parser agora valida o formato, o Item de origem e seus paths. Wayland e
AppImage ainda precisam de prova.

**Importação para a árvore em 2026-09-30:** o mesmo destino de drop recebe
URLs `file:` locais de fora do aplicativo e força cópia, inclusive quando a
fonte propõe mover. A revisão por item do lote é usada também para um único
arquivo externo; renomear, pular, colisão, progresso e cancelamento seguem o
mesmo fluxo. `fs.transferBatch(operation: "import")` mantém a origem externa,
conserva o destino dentro do workspace e abre a árvore de origem por
descritores com `O_NOFOLLOW`; symlinks e entradas especiais são recusados no
preflight. URLs remotas ou com host são rejeitadas pela ponte Qt. Testes Rust,
C++ Qt e harness QML passaram. O gesto real X11 com Nautilus foi provado em
2026-09-30: a origem permaneceu e o destino recebeu uma cópia. A ponte Qt
retorna `QStringList`; o drop o converte a `Array` antes de chamar a validação
compartilhada em QML. Falta provar o gesto em Wayland e no AppImage; a
abertura de projeto e o drop no editor são fluxos distintos, provados em X11
logo depois.
O MIME interno só autoriza movimento quando o evento traz o Item de origem
da própria árvore e os paths coincidem com os anunciados por ele; uma aplicação
externa que forje a chave interna entra no fluxo de importação por URL local,
que sempre copia.

**Ações de caminho em 2026-09-30:** o menu de contexto agora copia paths
absolutos ou relativos à raiz para o clipboard de texto, inclusive seleção
múltipla em linhas separadas. A mesma lista de ações atende clique e teclado;
o menu ajusta sua posição à altura real para manter os novos itens na tela.
Abrir a pasta no gerenciador e abrir terminal nela usam o item sob contexto:
arquivo aponta para o pai, pasta aponta para si. O primeiro chama o serviço
de URL local do Qt; o segundo estende `terminal.open` com `cwd` opcional,
confinado pelo core à raiz. Caminho inválido não inicia um PTY. A integração
no checkout X11 foi exercitada com um arquivo de `/tmp`: a chamada D-Bus
`org.freedesktop.Application.Open` ao Nautilus recebeu a URL da pasta pai,
e o terminal abriu com o mesmo diretório no prompt. Falta a prova no AppImage.

**Destinos adicionais em 2026-09-30:** uma URL local única solta na tela
inicial só é aceita após a ponte Qt verificar que aponta para uma pasta; então
segue para `workspace.open`, cuja validação no core permanece. Soltar um arquivo
da árvore sobre o editor força ação de cópia
do drag e usa `EditorController.openDiagnostic` para abrir ou ativar a aba,
sem alterar o disco. O payload interno tem parser único para árvore/editor.
**Decisão do autor em 2026-09-30:** soltar uma URL local única fora da raiz no
editor abre uma aba somente leitura, sem importar. O fluxo usa
`fs.readExternal` (`0.142.0`) e o mesmo modelo de documentos, mas o marcador
de somente leitura bloqueia edição, salvamento, formatação, rascunhos, LSP e
persistência na sessão. Repetir o drop relê o disco na mesma aba. O core só
lê arquivo regular UTF-8 até 1 MiB e recusa symlink nos componentes; URL
remota e seleção múltipla não entram no pedido. `fs.read` e `fs.write` seguem
confinados à raiz. Os destinos internos e a tela inicial foram provados por
gesto real em X11 em 2026-09-30. Em 2026-10-01, a rota externa no editor foi
provada com o mouse real em X11 e em Wayland nativo, a partir do Nautilus da
sessão (roadmap 40 §7.144). O AppImage ainda exige prova.

**Saída para outro aplicativo em X11:** um receptor Qt separado recebeu a URL
local em `text/uri-list`, interpretou-a como `QUrl` e aceitou cópia. O Nautilus
isolado aceitou a ação, mas não criou a cópia. Em 2026-10-01, em Wayland
nativo, um receptor GTK4 recebeu `GdkFileList` com a URL correta, inclusive
lido 500 ms depois do drop. Com o mouse sintético, o Nautilus 50.2.2 realçava
o destino e não copiava. Com o gesto feito pelo autor, a cópia funcionou em
Wayland (roadmap 40 §7.147), e a falha era do método de teste. A origem
permaneceu intacta.

**Proteção de buffer em exclusão, 2026-09-30:** o roteador de remoção
consulta os documentos abertos antes de enviar o pedido e mantém o diálogo
com motivo quando houver conteúdo diferente do salvo ou salvamento pendente
sob o caminho. Se uma edição acontecer enquanto a resposta do core está em
trânsito, o editor conserva a aba alterada em vez de fechá-la e limpar seu
rascunho. Esta proteção não substitui a decisão de lixeira/recuperação nem a
prova de conflito externo do P3.

**Lixeira recuperável em 2026-09-30:** a ação principal do diálogo envia
`fs.trash` ao core, que aplica o mesmo confinamento de `fs.delete` e usa a
lixeira FreeDesktop do sistema. A falha não apaga o item e volta ao diálogo;
"Excluir permanentemente" permanece uma escolha separada, explicitamente
rotulada. O mesmo bloqueio de documentos sujos protege as duas ações. Um teste
moveu arquivo Unicode para uma lixeira isolada em `/tmp` e conferiu conteúdo e
metadados de recuperação. A IDE ainda não oferece um comando de desfazer;
recuperação deve ser feita pelo gerenciador de arquivos do desktop. Integração
nativa do envio pelo diálogo passou em X11 com arquivo sob `/tmp`; a recuperação
visual no gerenciador segue pendente, pois o `gio` desta sessão não listou a
lixeira específica desse volume. Outro teste isolado tornou a lixeira
indisponível e verificou que a origem permanece intacta; o harness QML cobriu
o erro que volta ao diálogo. O caminho de remoção compartilhado canoniza só a
pasta pai: apagar ou mover um link simbólico remove o link selecionado, sem
seguir o alvo, inclusive quando ele fica fora do workspace.
O diálogo mantém a remoção pendente até resposta: cliques repetidos e
cancelamento durante o pedido não disparam outra mutação; falha reabre a escolha.
Respostas tardias de outro caminho ou após trocar de workspace não fecham abas.

Antes da implementação de abertura por clique, confrontar o uso atual com o
modo preview de abas das referências. A decisão de preview/pin não pode ser
substituída por “duplo clique faz o mesmo” sem discussão (§7).

## 5. Fronteiras e proteções

O caminho continua: gesto → controller existente → router/CoreClient → core
→ resultado/evento → atualização de árvore/documentos. Não fazer IO de disco
em QML, executar comandos montados com paths, criar outra árvore de workspace
ou duplicar o buffer do editor. Menu, teclado e drop usam a mesma intenção.

No core, validar o conjunto antes de agir e novamente ao efetivar quando o
estado puder ter mudado. Recusar mover raiz, mover pasta para si/descendente,
fontes duplicadas/ancestrais conflitantes, fuga por symlink e destinos fora do
escopo. `fs.rename` existente não implica atomicidade entre filesystems.
Importação externa exige autorização delimitada aos paths selecionados;
não desabilitar a proteção de `fs.*` para permitir drag-and-drop.

Lotes/copias demoradas reutilizam Jobs. Cada resposta carrega contexto
suficiente para não atualizar outro workspace depois de uma troca. Depois de
uma mutação, atualizar paths de documentos descendentes e estado de árvore;
notificar LSP/índice/Git pelas rotas suportadas. Rename de arquivo não promete
refactoring semântico/imports corrigidos automaticamente.

Não tratar URL remota como path local nem baixar ao soltar. Em espelho SSH,
move/delete locais não passam a ser sincronização remota bidirecional: manter
visível a limitação do Remote e pedir decisão antes de ampliar esse contrato.

Desfazer de arquivo não é apenas desfazer texto do editor. Definir retenção,
precondições e recuperação antes de expor a ação; mudanças externas impedem
reversão destrutiva automática. Não fabricar sucesso de rollback.

## 6. Ordem de execução e prova

- **P0 / proposta 0.3.1:** launcher, pasta existente/vazia, argumentos e
  abertura protegida. Pode antecipar para 0.3.0 se não atrasar seus bloqueios.
- **P1 / proposta 0.3.2:** seleção/foco/teclado e menu convergentes; arquitetura
  de mutações, colisão e recuperação sobre os donos existentes.
- **P2 / proposta 0.3.3:** clipboard de arquivos e drag-and-drop interno/externo;
  identidade estável das abas é pré-requisito dos fluxos que a necessitem.
- **P3 / proposta 0.3.4:** concluir toda a matriz, integração desktop, falhas,
  escala e prova real. Não anunciar “interação completa” no fim de P0/P1.

Testar em diretórios temporários: vazio, preenchido, Unicode/espaços, homônimos,
symlinks, arquivo aberto/sujo, seleção de pai+filho, permissão negada, disco
indisponível e troca de workspace durante operação. Provar que Cancelar e
operações recusadas preservam arquivos e buffers; falha parcial identifica
precisamente os resultados. Testar múltiplos filesystems quando houver move
entre eles, sem depender só de mocks.

Harnesses QML cobrem intenção/destino/foco; testes Rust cobrem IO, confinamento
e falhas; o artefato distribuído prova CLI, MIME/clipboard e arrasto real no
desktop. X11/Wayland e escalas de tela suportadas precisam de registro; não
confundir teste de lógica com integração nativa certificada.

## 7. Decisões a confirmar antes de ampliar contratos

- ~~Default de `kinein` sem argumentos e política de nova/reutilizada janela.~~
  **Decidido em 2026-09-26:** sem argumentos abre o CWD (e o launcher de desktop
  continua sem esse default); mesma pasta foca a janela aberta, outra pasta abre
  nova. Implementado e medido no mesmo dia; ver §3 e o roadmap 40 §7.116.
- ~~Arquivo avulso fora da raiz solto no editor.~~ **Decidido em 2026-09-30:**
  aba somente leitura, sem importação; ver §4 e IPC `0.142.0`.
- Múltiplas pastas/multi-root e drop de vários projetos: são casos registrados,
  não descartados; não cabem silenciosamente no contrato atual de uma raiz.
- Preview/pin de abas e gestos de abertura: o plano anterior os colocava após
  identidade estável. Confirmar se o pedido de interação completa os antecipa;
  separar de docking/split arbitrário, que não é consequência de mover arquivo.
- Lixeira/desfazer: backend de desktop, limites de retenção e fallback honesto.
- Refactoring por rename e mutação remota: dependem de contratos de linguagem
  e sincronização próprios, não de um adaptador de drag-and-drop.

Essas decisões não bloqueiam o registro nem as fatias independentes. Quando
uma delas afetar a próxima implementação, perguntar ao autor antes de cortar
comportamento, enfraquecer proteção ou chamar paridade parcial de completa.
