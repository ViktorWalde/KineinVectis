# especificacoes/ — a visão-alvo do produto

**Alvo, não estado.** O que existe hoje se mede no código e está no
[`roadmaps/40`](../roadmaps/40-estado-e-continuidade.md). Entrada:
[`indice-das-especificacoes.md`](indice-das-especificacoes.md).

**Precedência do frontend desde 2026-09-22.**
[`arquitetura-de-frontend-0.3-em-diante.md`](arquitetura-de-frontend-0.3-em-diante.md)
consolida e corrige as specs antigas de UI. Em conflito, ela vence. A Vectis
não terá Assistente/Chat de IA embutido nem telemetria de produto/usuário.
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
arquitetura-de-frontend-0.3-em-diante.md   alvo consolidado do frontend; precedência
remote-ssh-ui-hud.md                       fluxo diário e HUD do Linux remoto
terminal-ergonomia-0.3.md                  ações, atalhos e segurança do terminal
projetos-arquivos-e-integracao-desktop-0.3.md  CLI, árvore, clipboard e drag-and-drop
grafana-ui-ux-0.3.5.md                     conexão e uso diário do Grafana
markdown-preview-0.3.md                     edição, preview e split de Markdown
sistema-de-layout.md                       a tela: regiões, janelas de ferramenta, modos
sistema-de-componentes-de-ui.md            os componentes visuais
sistema-visual-e-icones.md                 identidade visual e a família de ícones
editor-e-inteligencia-de-linguagem.md      editor, LSP, Tree-sitter, diagnósticos
camada-de-editor-tree-sitter.md            a camada estrutural local do editor
fluxos-de-produto-build-run-debug.md       configure → build → run → debug
embarcados-targets-flash-serial-qemu.md    targets, flash, serial, QEMU
arquitetura-interna-core-ipc-jobs.md       core, IPC e jobs (visão-alvo)
onboarding-*.md                            primeira abertura, wizard de projeto/setup,
                                           configurações, camada de setup
fluxo-duplo-e-acoes-de-configuracao.md     ações de configuração com prévia
acoes-de-configuracao-com-escopo-*.md      escopo e links de documentação das ações
recursos-sob-demanda-*.md                  performance e a inteligência sob demanda
plano-de-implementacao-*.md                o plano UI/UX/arquitetura/performance
finalizacao-mvp-e-checklist-de-polimento.md   o checklist de polimento do MVP
```

Os `.svg` ao lado são os diagramas de cada especificação. Specs de features
canceladas não ficam aqui: vão para `DocsPrivate/legado/` e não voltam.
