# Markdown — edição e preview legível na série 0.3

> **Classe: ALVO / PLANO.** Adicionado em 2026-09-22 à frente de ergonomia da
> série 0.3. A IDE hoje abre `.md` como texto; não há preview renderizado.

## 1. Resultado de produto

Um arquivo Markdown continua sendo texto editável, mas pode ser visto como
documento formatado:

```text
Editar       fonte Markdown
Preview      documento renderizado
Lado a lado  fonte + documento
```

Não é uma imagem estática. O preview deve ser selecionável, pesquisável,
rolável, acompanhar o conteúdo ainda não salvo e permitir abrir links com
política explícita.

## 2. Base técnica medida

O frontend declara Qt 6.4 como versão mínima, liga `Core`, `Gui`, `Qml` e
`Quick` e não usa WebEngine. A API Qt disponível nessa base expõe:

- `Text.MarkdownText` / `TextEdit.MarkdownText`;
- `baseUrl` para resolver referências relativas;
- `QTextDocument::setMarkdown` com dialeto GitHub;
- `QTextDocument::MarkdownNoHTML`;
- `QTextDocument::setResourceProvider` para controlar recursos.

Portanto o primeiro corte não precisa de Chromium, servidor local, processo
Node ou parser novo. Uma pequena ponte C++ pode configurar o documento e a
política de recursos sem levar estado de edição para fora dos controllers
atuais.

## 3. Experiência

### 3.1 Descoberta

Ao abrir `.md`/`.markdown`, a barra do editor oferece três modos compactos:

```text
[ Editar ] [ Preview ] [ Lado a lado ]
```

Também existem commands para paleta/atalho:

```text
markdown.preview.toggle
markdown.preview.side
```

`Ctrl+Shift+V` é candidato para alternar o preview, depois do gate de colisão.
O terminal mantém seu paste porque atalhos são resolvidos pelo contexto/foco.
O modo pode ser lembrado por arquivo durante a sessão; persistência entre
sessões só entra junto do layout versionado.

### 3.2 Preview

- largura de leitura confortável, centralizada, sem esticar parágrafos por toda
  a janela;
- títulos, parágrafos, listas, citações, tabelas, tarefas, links, imagens e
  blocos de código distinguíveis pelo tema;
- texto e código selecionáveis, com Copy e menu contextual;
- zoom de leitura pode reutilizar a política futura do editor, não nasce como
  escala arbitrária isolada;
- empty/error states dizem por que algo não foi renderizado;
- ao alternar modos, foco e posição não saltam sem motivo.

### 3.3 Lado a lado

- editor à esquerda e preview à direita por padrão;
- splitter com mínimo utilizável e retorno simples a Editar;
- atualização com debounce curto sobre o buffer atual, sem exigir save;
- scroll sincronizado por bloco é alvo posterior se não houver mapeamento
  confiável. A primeira versão preserva posição de cada lado em vez de fingir
  sincronização por porcentagem que diverge com imagens/blocos.

## 4. Links e imagens

### 4.1 Links

```text
#âncora                 navega dentro do preview
arquivo.md / relativo  abre no editor, resolvido a partir do arquivo atual
http(s)                 abre no navegador externo, com indicação ↗
file fora do workspace  bloqueia ou pede confirmação conforme Workspace Trust
outros esquemas         recusados por padrão
```

Links não executam comandos. O path é normalizado e validado antes de abrir;
`../` não pode escapar silenciosamente do workspace.

### 4.2 Imagens

- caminho relativo é resolvido a partir da pasta do `.md`;
- somente arquivo local permitido pelo workspace é carregado no primeiro corte;
- `http(s)` remoto fica bloqueado por padrão para evitar tracking e acesso à
  rede ao apenas abrir documentação;
- ausência/erro mostra alt text e caminho, sem quebrar o documento;
- tamanho respeita a largura do preview;
- recurso carregado passa por provider controlado, não pelo acesso irrestrito
  do documento.

Uma preferência futura pode permitir imagem remota em workspace confiável, com
origem visível. Não é default da 0.3.

## 5. HTML e segurança

Markdown embutindo HTML usa `MarkdownNoHTML` inicialmente. Isso evita script,
iframe, pixel remoto e diferenças de CSS imprevisíveis. HTML aparece ignorado
ou como conteúdo seguro segundo o renderer; nunca ganha execução.

O preview não é navegador:

- sem JavaScript;
- sem cookies/storage;
- sem WebEngine;
- sem navegação embutida para sites;
- sem execução de Mermaid, PlantUML ou blocos de código;
- sem leitura fora do workspace por URL construída no documento.

Workspace Trust continua sendo uma camada adicional, não justificativa para
liberar tudo em projeto confiável.

## 6. Arquitetura

```text
EditorPane
 ├─ EditorTextSurface
 └─ MarkdownPreviewPane
      ├─ toolbar de modo
      ├─ Flickable + TextEdit read-only
      └─ MarkdownPreviewDocument (ponte pequena C++)
             ├─ setMarkdown(buffer, MarkdownNoHTML)
             ├─ base URL do arquivo atual
             ├─ stylesheet do tema
             └─ resource provider restrito
```

Dados de entrada:

```text
documentId · path · bufferVersion · content · workspaceRoot · trusted
```

- conteúdo vem do mesmo model do editor, inclusive alterações não salvas;
- `documentId` evita que debounce de uma aba atualize outra;
- `bufferVersion` descarta atualização obsoleta;
- modo, scroll e splitter são estado de apresentação;
- nenhum método IPC é necessário para renderizar texto;
- se imagens precisarem de leitura mediada pelo core, entra método de leitura
  binária restrito e reutilizável, nunca `file://` irrestrito no QML;
- links internos usam o caminho existente de abrir arquivo;
- links externos passam por uma única política de URL do shell.

`MarkdownPreviewPane` não entra dentro de `EditorController.qml`. O host/editor
compõe a superfície e o controller de documentos fornece identidade/conteúdo.

## 7. Renderização e desempenho

- debounce inicial proposto: 100–200 ms, medido com arquivos pequenos e grandes;
- não renderizar preview oculto;
- mudança de tema recompõe estilo sem reler disco;
- imagens têm limite de dimensões/memória e carregamento cancelável ao trocar
  de documento;
- arquivo muito grande pode pausar atualização automática e oferecer
  **Atualizar preview**, com motivo explícito;
- latência da tecla continua sendo a régua: renderização não roda no caminho
  síncrono do `TextEdit` de código.

O primeiro corte pode usar o renderer Qt no thread da UI somente se a medição
mostrar orçamento seguro. Se documento grande quebrar a régua, parse/render é
agendado ou limitado; não se declara “leve” sem benchmark.

## 8. Fidelidade declarada

O renderer Qt cobre Markdown comum/GitHub, mas o preview da 0.3 não promete
paridade total com GitHub, VS Code ou JetBrains. O painel informa de forma
discreta recursos não suportados quando encontrados, sem modificar o arquivo.

Fora do primeiro corte:

- Mermaid/diagramas executáveis;
- matemática LaTeX;
- plugins de Markdown;
- HTML arbitrário;
- exportar PDF/HTML;
- preview colaborativo;
- edição WYSIWYG.

## 9. Fatias

### M0 — baseline e corpus

- corpus com headings, listas, tabela, tarefa, quote, links, imagem local,
  código, Unicode e documento grande;
- medir abertura, digitação e troca de aba;
- registrar quais extensões o Qt 6.4 realmente apresenta.

### M1 — Preview

- detecção `.md`/`.markdown`;
- modo Preview, buffer não salvo, tema e seleção/cópia;
- links internos/externos com política;
- HTML desabilitado;
- imagens inicialmente omitidas se o provider seguro ainda não estiver pronto.

### M2 — Lado a lado e recursos locais

- splitter e preservação de posição/foco;
- provider para imagem local dentro do workspace;
- debounce, cancelamento por documento/versão e limite de tamanho;
- commands e atalhos após gate de colisão.

### M3 — polimento

- toolbar/context menu acessíveis;
- teclado, tab order, zoom/pesquisa se confirmados pelo teste;
- 1024×700, tema claro/escuro e AppImage;
- documentação de fidelidade e segurança.

## 10. Critérios de aceite

- `.md` pode alternar Editar/Preview/Lado a lado sem abrir aplicativo externo;
- preview reflete buffer não salvo sem degradar a digitação;
- conteúdo continua selecionável e copiável;
- link relativo abre o arquivo correto; URL web abre externamente;
- HTML/script não executa;
- abrir documento não busca imagem remota silenciosamente;
- imagem local fora do escopo permitido não é lida;
- resposta atrasada nunca aparece na aba errada;
- documento grande tem comportamento limitado e explicado, não congelamento;
- modo e ação são encontráveis por mouse, teclado e paleta;
- o AppImage inclui exatamente a mesma capacidade, sem dependência WebEngine.

## 11. Decisões a confirmar

1. Preview ou Editar é o modo inicial de um `.md` nunca aberto? A recomendação
   é **Editar**, preservando que a IDE é editor; a escolha fica visível.
2. `Ctrl+Shift+V` pode ser o atalho contextual de preview Markdown?
3. Imagens remotas permanecem sempre bloqueadas ou podem ser liberadas por
   documento/workspace confiável?
4. M1–M3 fecham juntos na 0.3.3, como propõe o roadmap 48, ou o preview simples
   entra antes e o lado a lado depois?
