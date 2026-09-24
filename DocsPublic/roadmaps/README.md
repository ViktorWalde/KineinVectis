# roadmaps/ — planos, trilhas e o ESTADO vivo

**Comece pelo [`40-estado-e-continuidade.md`](40-estado-e-continuidade.md)**:
é a fila viva (§4), as decisões que não se reabrem (§5) e o registro de cada
fatia entregue (§7). A Etapa 1 seguiu a ordem do
[`42-trilha-profunda-embarcados.md`](42-trilha-profunda-embarcados.md); a
Etapa 2 (2026-09-18/19, fechada) seguiu o
[`43-etapa2-hud-ui-ux.md`](43-etapa2-hud-ui-ux.md); a Etapa 3 (2026-09-19,
E3-1…E3-6 feitas, E3-7 pendente) segue o
[`44-etapa3-arquitetura-do-frontend.md`](44-etapa3-arquitetura-do-frontend.md);
a **Etapa 4** (o backend de novo: LSP profundo, edição inteligente, a
biblioteca dos compiladores) abre com o
[`45-etapa4-backend-lsp-edicao-compiladores.md`](45-etapa4-backend-lsp-edicao-compiladores.md).
O [`46-frontend-0.3-em-diante.md`](46-frontend-0.3-em-diante.md) é a frente
horizontal de frontend reconciliada em 2026-09-22: orienta a 0.3 e as versões
seguintes sem suspender a Etapa 4 para um rewrite.
O [`47-estrutura-da-v0.3.md`](47-estrutura-da-v0.3.md) cruza as duas frentes em
uma série fechável até a 0.3.5. O
[`48-arquitetura-executavel-da-serie-0.3.md`](48-arquitetura-executavel-da-serie-0.3.md)
aprofunda contratos, estados, migração e prova. Grafana na 0.3.5 e
launcher/interação completa de pastas antes dele estão decididos; bordas mais
naturais entram na 0.3.x. A distribuição exata dos marcos anteriores ainda é
proposta. Selecionar Tudo completo e nomes reutilizáveis são básicos pendentes
do terminal, não extras que possam ser substituídos por seleção visível.

```text
40-estado-e-continuidade.md        ESTADO: números medidos, fila, decisões, entregas
48-arquitetura-executavel-da-serie-0.3.md  contratos, donos, marcos, migração,
                                   rollback e prova até a 0.3.5
47-estrutura-da-v0.3.md            escopo de produto: shell, Remote, terminal,
                                   editor, Grafana e release até a 0.3.5
46-frontend-0.3-em-diante.md       frontend 0.3+: Remote SSH diário, commands,
                                   tool windows, tabs estáveis e área direita
45-etapa4-backend-lsp-edicao-compiladores.md  a Etapa 4 (brief, 2026-09-19): o que o LSP,
                                   o editor e o modelo de compiladores já fazem
                                   (medido no código), o que falta, as fatias L/E/C
                                   e a ordem; a régua é a latência da tecla
44-etapa3-arquitetura-do-frontend.md  a Etapa 3: tool windows à JetBrains (adaptadas),
                                   sete fatias com medida; §8 a etapa seguinte
                                   (compiladores); §9 "posso divulgar?"
43-etapa2-hud-ui-ux.md             a Etapa 2 (HUD/UI/UX): o desenho MEDIDO na IDE
                                   abrindo, a referência JetBrains lida nas fontes,
                                   as fatias F1–F8 com a medida de cada uma
42-trilha-profunda-embarcados.md   a trilha em oito pilares; §8 o "efeito
                                   JetBrains" como critério de pronto; §9 a
                                   trilha Python completa
41-ecossistema-embarcados-e-python.md  o inventário do ecossistema aberto, com
                                   licença lida e o que NÃO entra
35-ambiente-cpp-e-embarcados.md    ambiente C/C++ (catálogo de bibliotecas), a
                                   frente de embarcados e a de banco
39-divida-tecnica-paga.md, 38-...  a dívida paga e a que restou (registro)
34-depois-do-mvp.md, 30-...        o pós-MVP e o caminho até o MVP (fechados)
28, 29                             plataforma/verticais e as verticais de
                                   linguagem (C/C++, Rust, Python)
20, 21, 24, 25, 26                 planos antigos, mantidos como registro
backend-para-ui-ux.md              do backend à UI/UX (2026-07)
motor-semantico-profundo-cpp-rust.md   a especificação do motor semântico (KSWE)
adaptacao-de-plugins-abertos.md    o que adaptar de plugins abertos
```

Regra zero para qualquer item daqui: **medir antes de aceitar como pendente.**
