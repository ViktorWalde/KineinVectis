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
| Arrastar arquivo externo para o editor | intenção de abrir, não importar; resolver explicitamente o caso fora da raiz (§7), sem relaxar confinamento |
| Arrastar para aplicativo externo | oferecer URLs de arquivos locais, sem texto sensível nem exclusão inferida da origem; validar integração nativa |
| Colisões | informar origens/destinos, permitir cancelar, pular ou escolher novo nome; nunca sobrescrever por padrão nem mesclar pastas às cegas |
| Excluir/recuperar | preferir lixeira recuperável quando suportada; exclusão permanente é explícita e confirmada; desfazer nunca promete recuperação inexistente |
| Ações de contexto úteis | copiar path absoluto/relativo, abrir pasta no gerenciador e terminal nessa pasta, usando serviços existentes |
| Mudança interna/externa | árvore, abas, Git e índice refletem o resultado real; alterações externas não substituem buffer sujo sem resolver conflito |
| Erro/cancelamento/lote grande | feedback e progresso observáveis; discriminar o que concluiu/falhou; não congelar UI nem anunciar atomicidade de um lote parcial |

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

- Default de `kinein` sem argumentos e política de nova/reutilizada janela;
  sugestão: CWD no comando curto, sem mudar o launcher de desktop por acidente.
- Arquivos avulsos fora da raiz, múltiplas pastas/multi-root e drop de vários
  projetos: são casos registrados, não descartados; não cabem silenciosamente
  no contrato atual de uma raiz. Pedir decisão antes de implementar restrição.
- Preview/pin de abas e gestos de abertura: o plano anterior os colocava após
  identidade estável. Confirmar se o pedido de interação completa os antecipa;
  separar de docking/split arbitrário, que não é consequência de mover arquivo.
- Lixeira/desfazer: backend de desktop, limites de retenção e fallback honesto.
- Refactoring por rename e mutação remota: dependem de contratos de linguagem
  e sincronização próprios, não de um adaptador de drag-and-drop.

Essas decisões não bloqueiam o registro nem as fatias independentes. Quando
uma delas afetar a próxima implementação, perguntar ao autor antes de cortar
comportamento, enfraquecer proteção ou chamar paridade parcial de completa.
