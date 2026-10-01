# especificacoes/ — a visão-alvo do produto

**Alvo, não estado.** O que existe hoje se mede no código e está no
[`roadmaps/40`](../roadmaps/40-estado-e-continuidade.md). Entrada:
[`indice-das-especificacoes.md`](indice-das-especificacoes.md).

**Precedência do frontend desde 2026-09-22.**
[`arquitetura-de-frontend-0.3-em-diante.md`](arquitetura-de-frontend-0.3-em-diante.md)
consolida e corrige as specs antigas de UI. Em conflito, ela vence. A Vectis
não terá Assistente/Chat de IA embutido nem telemetria de produto/usuário.
Após fechar a série 0.3, a reorganização da casca aprovada pelo autor entra na
0.3.6 segundo o [roadmap 49](../roadmaps/49-frontend-0.3.6-e-sequencia-0.5.md);
a expansão da Biblioteca por capacidades/providers fica na 0.5 segundo o
[roadmap 50](../roadmaps/50-biblioteca-e-providers-0.5.md).
“Assistente de projeto/setup” significa wizard determinístico, não IA; logs,
métricas locais e dados do alvo devem usar nomes do domínio. O desenho diário
de Remote SSH está em [`remote-ssh-ui-hud.md`](remote-ssh-ui-hud.md).
O fluxo prático do Grafana que fecha a 0.3.5 está em
[`grafana-ui-ux-0.3.5.md`](grafana-ui-ux-0.3.5.md).
Launcher e interação completa com arquivos/pastas, antes da 0.3.5, estão em
[`projetos-arquivos-e-integracao-desktop-0.3.md`](projetos-arquivos-e-integracao-desktop-0.3.md).
Bordas/cabeçalho mais naturais na 0.3.x estão no
[`sistema-de-layout.md` §6.5](sistema-de-layout.md#65-bordas-e-cabeçalho-mais-naturais--compromisso-da-03x).

```text
CONSOLIDADO (vence em conflito)
arquitetura-de-frontend-0.3-em-diante.md   alvo consolidado do frontend; precedência

POR VERSÃO (escritos antes do código, a partir de medição)
projetos-arquivos-e-integracao-desktop-0.3.md  CLI, árvore, clipboard e drag-and-drop
terminal-ergonomia-0.3.md                  ações, atalhos e segurança do terminal
markdown-preview-0.3.md                    edição, preview e split de Markdown
simbolos-e-indentacao-0.3.md               símbolos e indentação sem bloquear a tecla (V6)
grafana-ui-ux-0.3.5.md                     conexão e uso diário do Grafana
remote-ssh-ui-hud.md                       fluxo diário e HUD do Linux remoto
modelo-semantico-do-projeto-0.4.md         o modelo semântico do projeto (0.4+); §6 fechada
cache-de-compilacao.md                     cache de compilação orquestrado (PLANO; nada no código)

POR ÁREA (visão-alvo do produto)
sistema-de-layout.md                       a tela: regiões, janelas de ferramenta, modos
sistema-de-componentes-de-ui.md            os componentes visuais e a paleta
sistema-visual-e-icones.md                 identidade visual e a família de ícones
editor-e-inteligencia-de-linguagem.md      editor, LSP, Tree-sitter, diagnósticos
camada-de-editor-tree-sitter.md            a camada estrutural local do editor
fluxos-de-produto-build-run-debug.md       configure → build → run → debug
embarcados-targets-flash-serial-qemu.md    targets, flash, serial, QEMU
arquitetura-interna-core-ipc-jobs.md       core, IPC e jobs (visão-alvo)

AS "PARTES" DA ESPECIFICAÇÃO ORIGINAL (revisadas em 2026-09-22: "assistente" é
wizard determinístico, sem IA nem telemetria; em conflito, o consolidado vence)
onboarding-assistente-de-projeto-e-configuracoes.md   Parte 8: settings, onboarding,
                                           wizard de projeto, setup de CMake/toolchain
onboarding-camada-de-setup-e-otimizacao.md            Parte 8.1: a camada de otimização
fluxo-duplo-e-acoes-de-configuracao.md                Parte 9.1: fluxo duplo, modos de experiência
                                           e ações de configuração com prévia
acoes-de-configuracao-com-escopo-e-links-de-documentacao.md  Parte 9.2: escopo e links
recursos-sob-demanda-estrategia-de-performance.md     Parte 10: performance sob demanda
recursos-sob-demanda-estrategia-de-refatoracao-da-inteligencia.md  Parte 10.1: inspeções
                                           e refatoração profunda
plano-de-implementacao-ui-ux-arquitetura-performance.md  plano UI/UX/arquitetura/performance
finalizacao-mvp-e-checklist-de-polimento.md           "Parte 10" (sic — o mesmo número da de
                                           performance): fechamento do MVP e checklist
indice-das-especificacoes.md               o índice antigo, com o resumo de cada Parte
```

Os `.svg` ao lado são os diagramas de cada especificação. Specs de features
canceladas não ficam aqui: vão para `DocsPrivate/legado/` e não voltam.
