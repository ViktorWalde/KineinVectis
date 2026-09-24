# 46 — Frontend da 0.3 em diante

> **Classe: PLANO.** Extração operacional de `DocsPrivate/arquiKinein/`,
> reconciliada com o código em 2026-09-22.
>
> O alvo completo está em
> [`arquitetura-de-frontend-0.3-em-diante.md`](../especificacoes/arquitetura-de-frontend-0.3-em-diante.md).
> O Remote SSH está detalhado em
> [`remote-ssh-ui-hud.md`](../especificacoes/remote-ssh-ui-hud.md).
> A composição recomendada dessas frentes numa release 0.3 fechável está no
> [`roadmap 47`](47-estrutura-da-v0.3.md); ergonomia do terminal está em
> [`terminal-ergonomia-0.3.md`](../especificacoes/terminal-ergonomia-0.3.md).
> Complementos de 2026-09-23: launcher e interação completa com pastas antes
> da 0.3.5 seguem
> [`projetos-arquivos-e-integracao-desktop-0.3.md`](../especificacoes/projetos-arquivos-e-integracao-desktop-0.3.md);
> bordas/cabeçalho na 0.3.x seguem o
> [layout §6.5](../especificacoes/sistema-de-layout.md#65-bordas-e-cabeçalho-mais-naturais--compromisso-da-03x).
> São extensões dos donos atuais, não autorização para outro shell/explorer.

## 1. Por que este roadmap existe

As cinco partes de `arquiKinein` contêm ideias úteis, mas misturam:

- princípios que já são contrato da Vectis;
- observações corretas sobre acoplamento atual;
- arquitetura-alvo de longo prazo;
- abstrações ainda sem consumidor;
- premissas envelhecidas pela `main` de 2026-09-22;
- uma fronteira de versão baseada em Standard Make que não foi confirmada pela
  fila viva atual.

Este roadmap retém o essencial sem transformar todo o material em fonte da
verdade.

## 2. O que já mudou desde a base analisada

O material privado analisou principalmente a tag `v0.2.0`. Na `main` de
2026-09-22:

- `CommandDispatcher.qml` já existe e recebe Search Everywhere/startup;
- `command.list` e IDs de atalhos já existem;
- `EditorTabsBar` já usa `ListView` horizontal e overflow;
- as abas, porém, ainda são selecionadas/fechadas por índice;
- Símbolos/Structure ainda vive dentro de `EditorPane`;
- `SideRail` ainda conhece domínios nominalmente;
- Projeto/Git já compartilham um slot esquerdo;
- Remote SSH já possui backend amplo, mas sua UI continua uma caixa de setup.

Consequência: não implementar literalmente os PRs sugeridos no documento
privado. Cada fatia começa medindo novamente.

## 3. Linha da 0.3

A 0.3 não fará uma reescrita do frontend. Ela adota fundações onde entregam
valor direto:

```text
F0  reconciliar documentação e baseline
F1  Remote SSH: reorganizar o painel para o fluxo diário
F2  Remote SSH: tool window + HUD de contexto/sync
F3  convergir execução de commands no dispatcher existente
F4  ToolWindowEntry/model mínimo
F5  provar o modelo com Projeto/Git e Remote
F6  tabs com identidade estável
F7  Símbolos/Structure como tool window direita
F8  decidir preview/pin conforme o uso; split vem depois da identidade
```

Essa linha convive com a Etapa 4 (`roadmap 45`). Uma fatia de LSP/editor não
espera F0–F8 inteiras; ela usa o contrato novo apenas quando toca a mesma área.

## 4. Fatias

### F0 — documentação e baseline

- retirar Assistente/IA e Telemetry como features-alvo;
- distinguir observabilidade local/dados do alvo de coleta do produto;
- registrar screenshots e fluxo atual;
- medir catracas e arquivos tocados;
- manter `arquiKinein` como fonte privada, não como contrato.

### F1 — Remote SSH utilizável

- separar Overview, Workspace, Run & Debug e Configurar;
- preservar protocolo e controller;
- mostrar uma ação primária contextual;
- recolher comandos brutos e detalhes raros;
- provar o fluxo em 1024×700 e 1280×800.

### F2 — Remote SSH na HUD

- entrada de tool window contextual;
- status `SSH · alvo · sync` no workspace espelhado;
- job de pull/push/deploy visível;
- distinguir save local de envio remoto;
- declarar honestamente “não verificado”, “última sonda” e limitações atuais.

### F3 — CommandDispatcher como caminho comum

- inventariar quem ainda chama controller diretamente;
- migrar apenas gestos já representados por command ID;
- recusar/registrar ID desconhecido;
- não criar `CommandRegistry` paralelo;
- deixar context keys genéricas para quando houver duplicação medida.

### F4 — Tool Window mínima

- descriptor interno pequeno;
- áreas left/right/bottom;
- disponibilidade, ordem, ativo e componente;
- sem plugin API, detach ou dock graph;
- testes de ordem, ativação, foco e indisponibilidade.

### F5 — migração de prova

- Projeto/Git preservam o slot e a aparência;
- Remote prova uma terceira contribuição;
- shell deixa de ter ramificação nova por domínio;
- caminhos legacy só saem após paridade.

### F6 — identidade estável das abas

- caminho/ID de documento como identidade;
- índice fica restrito à view;
- seleção, close, restore e mudança externa usam identidade;
- harness cobre reordenação/remoção sem agir no documento errado.

### F7 — área direita

- mover a composição de Símbolos/Structure para host de tool window direita;
- reutilizar controllers e protocolo;
- não misturar mudança de LSP;
- manter busca no projeto e outline do arquivo;
- medir foco, largura, colapso e persistência.

### F8 — ergonomia das abas e split

- decidir preview/pin depois da identidade estável;
- primeiro split: dois grupos, sem árvore arbitrária;
- persistência versionada e fallback seguro;
- renderer atual compartilhado, não reescrito.

## 5. Depois da 0.3

Conforme gatilhos reais:

- migrar outras tool windows;
- Settings orientado por schema;
- múltiplas sessões/histórico/pinning;
- Environment/Toolchain com explicação de resolução;
- split adicional;
- modelos Qt incrementais para coleções que comprovadamente precisarem;
- API interna de contribuições e, só depois de estabilizada, avaliar plugins;
- Remote SSH com watcher, conflitos, rename/delete, sistema remoto e execução
  no alvo.

## 6. Fora do plano

- Assistente ou chat de IA embutido;
- telemetria de produto/usuário;
- novo protocolo ou cliente IPC paralelo;
- framework DI;
- Redux/store universal;
- registry para toda classe por simetria;
- API pública de plugins antes das APIs internas estabilizarem;
- reescrita do renderer do editor;
- big-bang do shell;
- agente/servidor proprietário instalado no alvo remoto.

## 7. Critério por fatia

Cada fatia precisa responder:

```text
problema medido
usuário/fluxo afetado
contrato mínimo
estado preservado
risco e rollback
teste/harness
prova visual quando houver UI
impacto de teclado e acessibilidade
medida de desempenho relevante
documento vivo atualizado
```

Se não há problema medido e consumidor real, a abstração não entra.

## 8. Pendências de decisão

As decisões listadas na especificação de frontend §14 e na especificação Remote
§13 não foram resolvidas por inferência. Quando uma fatia depender delas, parar
e perguntar ao autor antes de remover uma opção ou fixar comportamento.
