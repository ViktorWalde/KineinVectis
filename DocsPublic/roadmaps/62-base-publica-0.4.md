# 62 — Base pública Kinein Vectis 0.4

> **Classe: PLANO / ESTADO**, conforme indicado em cada seção. Fronteira
> vigente revista em 2026-11-03: fundamentos obrigatórios na 0.4, com evolução
> na 0.5. Substitui a exigência de concluir indistintamente todos os marcos
> 0.4.1–0.4.9; os mínimos desta base bloqueiam a entrega. Não altera registros.
> Visão de engenharia consolidada em 2026-11-04: **Foundation — Engineering
> Consolidation Release**, conforme §1.1; inventário sistemático em §3.4.
> Revisto em 2026-11-08: ordem completa da 0.4 com estado por etapa (§3),
> camadas finais de teste (§3.5), Windows básico validado em VM no fim e porte
> completo por colaboração (§3.2), e conferência do planejamento (§2.1).

## 1. Fronteira e contabilização

A entrega pública chama-se **Kinein Vectis 0.4**; os metadados SemVer usam
**0.4.0**. Agrega as etapas não publicadas 0.3.6–0.3.9 e o trabalho já
implementado da trilha interna 0.4, incluindo a etapa 0.4.1 ainda incompleta.
Números históricos permanecem em commits, changelog e registros. Renumerar
o produto não conclui nenhuma funcionalidade pendente.

**Fechamento por vertical comprovada:** a 0.4 só se conclui com as ferramentas
orquestradas, os consumidores integrados e as jornadas robustas demonstradas,
incluindo a etapa final do algoritmo efetivo no escopo declarado. Depois da
0.4.0, versões públicas **0.4.1 em diante até a 0.5** aprofundam as etapas,
integrações complementares, polimento e estabilização, preservando essa base.
Essas versões públicas são distintas dos identificadores internos históricos
0.4.1–0.4.9 usados para organizar trabalhos ainda não lançados.

A base prioriza uso confiável em projetos pequenos e médios e no desenvolvimento
da própria IDE: integridade dos documentos, contexto consistente, editor/LSP,
terminal, Git, configurar, compilar, executar, testar e depurar. A consolidação
segue **C → C++ → Rust e integração Rust/C/C++ → Python → especializações**.
C tem uma [etapa profunda dedicada](../especificacoes/consolidacao-nativa-de-c.md),
com C23, integração nativa e matriz de aceites C1–C8 antes da consolidação C++.
A primeira jornada C11 é uma prova parcial dessa etapa, que permanece aberta.
QML participa da jornada da própria IDE.
O desenho técnico está na
[orquestração e verificação de linguagens](../especificacoes/orquestracao-e-verificacao-de-linguagens.md).
Polimento de UI/HUD/UX e otimização de algoritmos acompanham essas jornadas.
Detecção/perfis de toolchain, DAP, QEMU, SVD, VFS, plotagem, notebooks e
ferramentas de engenharia são fundamentos obrigatórios da 0.4, com mínimos na
[especificação de infraestrutura](../especificacoes/infraestrutura-fundamental-0.4.md).
Desde 2026-11-08, Windows nativo exige na 0.4 só a validação funcional básica
numa VM, penúltima etapa da ordem (§3); a matriz completa e o instalador seguem
como frente de colaboração (§3.2, B7), e a 0.4 não declara mais do que provar.
A 0.5 aprofunda, melhora e amplia essa base; não recebe esses mínimos por
adiamento implícito. A preparação para colaboradores acompanha o fechamento.
Superfícies funcionais migram dos pop-ups para os docks existentes; menus, tooltips e confirmações
mantêm seu papel. O editor continua a área principal.

A trilha completa de **Sistemas Embarcados Modernos** do [52](52-arquitetura-executavel-da-0.4.md)
continua aberta para contribuições públicas, com marcos 0.4.1–0.4.9 como
referências de escopo, sem exigir releases intermediárias. Funcionalidades
pendentes, cargas extensas de SDK/build e matrizes físicas continuam
compromissos dessa trilha. O que excede os mínimos F1–F9 não bloqueia esta
base por pertencer ao roadmap inteiro; os mínimos continuam exigidos. As
[quatro tarefas iniciais](../contribuindo/15-tarefas-iniciais-embarcados-0.4.md)
separam implementação, integração, prova em software e prova em hardware.
Nenhuma limitação de emulação demonstra validação física.

O foco vigente é consolidar a orquestração das linguagens já existentes, os
algoritmos complementares e a experiência de uso. Novos SDKs e jornadas de
bare metal/Linux embarcado seguem a base essencial; as regressões dos recursos
especializados já oferecidos continuam obrigatórias. Esta ordem preserva o
roadmap de Embarcados, com ciclo mínimo integrado obrigatório, e substitui sua
execução integral como pré-condição indistinta da 0.4.

### 1.1 Visão da 0.4 — Foundation

**Kinein Vectis 0.4 — Foundation**

**Engineering Consolidation Release**

Uma versão destinada a auditar, corrigir, aprofundar e estabilizar a
infraestrutura existente, transformando o conjunto de funcionalidades
implementadas em uma base coerente, verificável e extensível.

A 0.4 trata uma deficiência acumulada no processo de engenharia desde as
primeiras versões: funcionalidades implementadas precisam de integração,
profundidade de suporte delimitada e evidência de comportamento para formar
uma base confiável. Auditar e consolidar por capacidade, corrigindo causas nos
donos existentes e integrando ferramentas maduras onde necessário.

**Critério central de conclusão:** a versão somente será publicada quando os
recursos que declara suportar estiverem integrados, documentados e validados,
com limitações conhecidas explicitamente registradas. Uma limitação precisa
delimitar o suporte e a reação da IDE; sua documentação não aprova uma jornada
obrigatória que continua incorreta ou não provada.

**A 0.4 deve eliminar a dívida técnica que compromete a correção, a
confiabilidade, a manutenção e a evolução das capacidades declaradas. Não
precisa eliminar toda possibilidade de melhoria futura.** A conclusão exige
os aceites desta base e o tratamento da dívida que os compromete; melhorias
posteriores conservam escopo, justificativa e dependências identificados.

Declarar análise semântica de C, por exemplo, exige registrar quais construções
e operações são compreendidas, quais casos não têm suporte e como a IDE reage
a uma situação desconhecida. Distinguir parser/estrutura, clangd, compilador
e verificador: êxito de um produtor não comprova a cobertura dos demais.
Identificar versão, dialeto, alvo e contexto, com casos aceitos, recusados e
indisponíveis. Uma situação desconhecida deve preservar conteúdo, explicar o
limite e manter operações independentes; não pode produzir aparência de
análise válida nem reutilizar evidência obsoleta como atual.

## 2. Auditoria inicial do checkout — 2026-11-02

Esta tabela registra inspeção de código e as provas anteriores identificadas.
Não equivale a uma nova aprovação das jornadas nem ao gate completo desta base.

| Capacidade | Implementado / integrado | Lacuna e prova exigida | Prioridade |
| --- | --- | --- | --- |
| Abrir, editar, salvar e recuperar | Editor, drafts, identidade documental, conflito externo e persistência têm donos e harnesses | Repetir edição/salvamento/reabertura e troca de projeto; arquivo externo alterado e documento sujo sem perda | Bloqueador B1 |
| Próprio projeto: QML/C++/Rust e LSP | Workspace, CDB, clangd, rust-analyzer e contexto de compilação integrados | Configurar o checkout, navegar/editar os três contextos e recuperar ferramenta ausente; não confundir sintaxe QML com LSP funcional | Bloqueador B2 |
| Terminal e Git | PTY, múltiplas sessões, correlação de comandos, status/diff/commit integrados | Entrada/saída, redimensionamento, cancelamento, diff atualizado e foco com o editor aberto | Bloqueador B2 |
| Configurar/compilar/executar/testar/depurar | Donos CMake, Cargo, RunConfig, Jobs, Test e DAP existentes | Exercitar jornadas reais de C/C++ e Rust; falha/cancelamento e ferramenta incompatível visíveis; medir depuração separadamente | Bloqueador B2 |
| Toolchains/kits | Detecção, seleção, origem efetiva, importação/exportação e deriva implementadas | Revalidar kit efetivo no próprio projeto; schema 3 continua aberto na trilha 52 | B2 para contexto atual; evolução para schema 3 |
| Setup/Instalar Ferramentas | `setup.list`, catálogo por distro e guias oficiais integrados; host flutuante na auditoria inicial | Dock, busca/filtro, carga, erro/recuperação, detalhes, teclado e envio explícito ao terminal | Polimento B3; primeira fatia |
| Embarcados | esptool, idf.py e PlatformIO nativos; probe-rs, DFU e picotool com produtores; provas históricas discriminadas no 40.7 | Regressão de software dos recursos oferecidos; novas placas/sondas não são prova herdada. `west` tem fluxo legado, mas produtor nativo ausente em `flash/native.rs` | Regressões bloqueiam; expansão nas tarefas públicas |
| Algoritmo semântico complementar | LSP/compilador e aplicação transacional existentes; assistente do 52 §11.2 ainda planejado | Definir corpus e integrar verificador inicial conservador; nunca rotular sugestão não verificada como segura | Bloqueador B4 |

O gate completo estrito de 2026-10-27 e a validação Linux de 2026-11-01
estão registrados no [40](40-estado-e-continuidade.md). Não validam mudanças
posteriores. O produto ainda se identifica como 0.3.9 nesta auditoria;
protocolo IPC 0.197.0 tem ciclo independente da versão pública.

### 2.1 Conferência do planejamento de fundamentos — 2026-11-08

Três documentos de planejamento do autor (recuperação do escopo da 0.4,
catálogo de fundamentos maduros e orquestração em Rust, motor semântico
intermediário por linguagem) foram conferidos com o checkout por leitura de
código, sem executar jornadas. Não são donos: as decisões vigentes estão em §3
e os problemas confirmados em §3.4.

| Afirmação do planejamento | Estado no checkout |
| --- | --- |
| Motores maduros já orquestrados: Tree-sitter, clangd, rust-analyzer, basedpyright/Ruff, qmlls, CMake File API, Cargo, Git, `fd`, PTY com `alacritty_terminal`, SQLite, `notify`, formatadores, cargo test/CTest/pytest | Código identificado; o aceite por jornada continua aberto. Consolidar, não reimplementar |
| A CDB guarda uma unidade por arquivo | Confirmado: C-03 |
| Diretório de build único e só a primeira configuração do CMake | Confirmado: C-04 |
| CodeAction sem `range` da seleção, `only` e `resolve` | Confirmado: CPP-01 |
| Uma thread por job, busca literal própria e fuzzy ausente | Confirmado no código; mudar só com medição que demonstre a falta, sem registro de defeito |
| Cobertura C/C++ | Recusa rotulada em `coverage/`; aceite pertence à etapa C |
| Detectado, selecionado e efetivo | Compilador C provado (C1-11–C1-14); origem por toolchain file e `CC` do ambiente aberta |
| Notebook, API de plotagem e VFS unificada | Ausentes, como na [infraestrutura §2](../especificacoes/infraestrutura-fundamental-0.4.md#2-ponto-de-partida--inspeção-de-2026-11-03) |
| Achado desta conferência | CDB fora de `.kinein/build`, raiz, `build`, `builddir` e `out/build` não é localizada: C-05 |

## 3. Ordem e critérios de fechamento

1. **B1 — Integridade e estabilidade.** Corrigir perdas de dados, crashes,
   contexto cruzado e operações bloqueadas encontrados na auditoria.
2. **B2 — Uso da IDE no próprio desenvolvimento.** Jornada reproduzível
   abrir → editar → salvar → configurar → compilar → executar/testar →
   depurar, com LSP, Git e terminal. Declarar explicitamente qualquer etapa
   NÃO PROVADA e o motivo; compilação sozinha não aprova esta jornada.
   Consolidar a etapa C, depois C++, Rust e suas fronteiras C/C++, depois Python,
   conforme §3.1. O fluxo com contexto externo `/ide` tem aceite próprio (§5).
3. **B3 — Polimento integrado.** Setup primeiro, depois as superfícies com
   maior interrupção do trabalho. Hierarquia, largura, listas, estados,
   atalhos, foco e recuperação; fotos antes/depois e interação na UI nativa.
   Aceite visual permanece separado dos testes de lógica. Instalar Ferramentas
   requer hierarquia cromática e estados mais distinguíveis, conforme sua
   [especificação §0.1](../especificacoes/instalar-ferramentas.md#01-polimento-visual-e-de-interação-da-base-04).
4. **B4 — Verificador consolidado.** Recorte host inicial integrado e provado
   conforme §4 e [40.7 §7.372](registro/40.7.9-a-base-publica-e-o-c-da-0.4.md#7372--verificador-de-correções-lsp-e-guardas-de-contexto-2026-11-02).
   Ampliar pela base das linguagens e pelo núcleo comum da especificação,
   priorizando Rust antes de Python. O assistente completo continua aberto
   no 52 §11.2; o recorte host não encerra esse trabalho.
   A fase dedicada de consolidação do algoritmo ocorre após a orquestração
   das ferramentas, com consumidores, corpus e aceites das linguagens da base.
   A versão não fecha somente pela existência do primeiro recorte host.
5. **B5 — Fechamento.** Idiomas e traduções conforme [57 §3](57-mapa-de-versoes-ate-a-1.0.md#3-pedidos-do-autor-e-para-onde-foram),
   metadados 0.4.0, changelog/manual/tutorial coerentes, gate completo estrito,
   pacote Linux com checksum e validação em ambiente mínimo, depois dos
   aceites B1–B4 e B6–B10 (B7 no recorte básico de §3.2). Esta fronteira não declara a release pronta e
   não inclui publicação automática.

**Ordem do autor (2026-11-08).** A 0.4 consolida primeiro o essencial da IDE,
sem atalhos, orquestrando os motores maduros já integrados (Tree-sitter,
clangd, rust-analyzer, basedpyright/Ruff, CMake File API, CDB, Cargo, Git,
PTY/VT, SQLite, `notify`, `fd`, DAP) em vez de criar parser, compilador,
terminal, índice ou depurador próprios. Cada etapa só avança com a anterior
confiável no recorte declarado. A coluna de estado é a visão concreta da 0.4;
cada atualização datada a substitui, e o detalhe fica nos donos:

| Etapa | Conteúdo | Estado em 2026-11-09 |
| --- | --- | --- |
| 1. C | C1–C8 em profundidade, incluindo C23; o contexto incorreto do que já existe é corrigido aqui (C-03–C-05) | **Em andamento.** C1 aberto: C1-01–C1-06, C1-08–C1-45, C-04 e C-05 corrigidos com provas; abertos C1-07, C1-46–C1-48, origem por toolchain file e `CC` do ambiente e consulta de versão bloqueada. C2–C8 abertos (C-02, C-03) |
| 2. C++ | Matriz de §3.1 e contrato de CodeAction (CPP-01) | Não iniciada como etapa; recursos existentes mantidos |
| 3. Rust e Rust/C/C++ | Workspaces, features/target, fronteiras e a própria IDE | Não iniciada como etapa; recursos existentes mantidos |
| 4. Python | Interpretador/venv, análise, testes e depuração | Não iniciada como etapa; recursos existentes mantidos |
| 5. DAP e QEMU | F2/F3 sólidos sobre as jornadas anteriores; as partes de depuração (C5) e QEMU (C6) de cada linguagem fecham aqui (confirmado pelo autor em 2026-11-08) | Base existente (`dap/`, GDB, LLDB, probe-rs, debugpy; QEMU `lm3s6965evb` no gate embarcado); aceites F2/F3 abertos |
| 6. F5–F9 | SVD, VFS, plotagem, notebooks e engenharia (B6, B8, B9) | SVD, VFS e engenharia parciais; plotagem e notebooks ausentes ([infraestrutura §2](../especificacoes/infraestrutura-fundamental-0.4.md#2-ponto-de-partida--inspeção-de-2026-11-03)) |
| 7. Algoritmo semântico de ambiente | Camada preparatória: leitura básica/intermediária da configuração essencial de cada ecossistema (presets e toolchain file, Cargo e `rust-toolchain`, `pyproject` e venv), com contratos para vários alvos, dependências, evidências e configurações simultâneas ([orquestração §13.1](../especificacoes/orquestracao-e-verificacao-de-linguagens.md#131-etapa-7--leitura-e-normalização-de-ambientes)) | Não iniciada; base em `toolchain/`, `project/` e `index/context/` |
| 8. Algoritmo especializado | Núcleo profundo delimitado e obrigatório: correlação de evidências, propagação de invalidação, análise de impacto contextual e verificação entre configurações ([orquestração §13.2](../especificacoes/orquestracao-e-verificacao-de-linguagens.md#132-etapa-8--núcleo-profundo-obrigatório-na-04)); fase dedicada do verificador/assistente (B4, §3.3) e última etapa de implementação | Recorte host S0/S1 integrado (40.7 §7.372); fase dedicada não iniciada |
| 9. Camadas finais de teste | Varredura rigorosa de §3.5 sobre o produto inteiro | Gates atuais em uso por fatia; camadas novas de §3.5 a desenhar |
| 10. Windows básico em VM | Compatibilidade ao menos funcional, conforme B7 em §3.2 | Porte pausado na W6b4 ([60](60-windows-e-edicao-especial.md) §3.3); VM a preparar |
| 11. Fechamento | Polimento restante e B5 | Não iniciada |

A fundação geral (integridade do editor, terminal, Git, busca e Jobs) é
transversal: cada jornada corrige o que exercitar, e INF-02 (terminal com bash
e zsh reais, com fish) é o primeiro item. Estado transversal em 2026-11-13: INF-01
e INF-02 corrigidos no recorte Linux (INF-02 com bash, zsh e fish); INF-03,
INF-04, INF-05, INF-07, INF-08, INF-09, INF-10, INF-11, INF-12, INF-13,
INF-14, INF-15, INF-17, INF-20, INF-21 e INF-18 corrigidos; INF-06, INF-16,
INF-19, UI-01 e UI-02 abertos. A dívida
técnica que compromete essas capacidades é paga dentro da 0.4 (§3.4), e as
camadas finais de teste estão em §3.5. As provas mantêm rigidez alta: lógica
e comunicação entre módulos exercitadas até o limite, cenários improváveis,
falhas e casos de borda que façam defeitos aparecer, com mutações das guardas;
quantidade de testes não substitui essa rigidez.

**Ordem do autor (2026-11-12).** A 0.4 paga toda a dívida técnica e de
arquitetura inventariada: nenhuma passa para a versão seguinte. A engenharia
do [63](63-regras-e-plano-de-engenharia-da-0.4.md) vem inteira antes de
retomar a etapa 1 (C), na ordem do §E.1 de lá: INF-15 primeiro, depois o gate,
o core, a ponte com a QML, e os testes com as listas congeladas. O C1 retoma
em seguida, do C1-47; os testes adiados do Windows são pagos na etapa 10. As
regras de engenharia passam a morar na Parte A do 63, e nenhuma etapa desta
tabela começa sem o plano dela escrito (R-PROC-01).

Quality Center, schema 3, CircuitPython, Banco para embarcados, expansão de
emuladores além de F3 e as demais jornadas não concluídas do 52 permanecem
abertos nos seus donos. Bloqueiam quando integram os mínimos de §3.2 ou uma
dependência/regressão impede os aceites da base. Windows nativo, bloqueador
B7 desde 2026-11-03, passou em 2026-11-08 a exigir só a validação básica em
VM (etapa 10), com o porte completo por colaboração; ROS 2,
colaboração remota entre pessoas e o motor semântico
profundo conservam seus planos posteriores. A revisão de acabamento antes associada
à edição especial é incorporada ao polimento desta base, sem exigir outra
versão pública. Os compromissos de idioma não são cancelados pelo recorte.

### 3.1 Matriz de consolidação e prontidão

| Frente, na ordem de aprofundamento | Aceite mínimo a demonstrar |
| --- | --- |
| C | Matriz C1–C8 da [consolidação nativa de C](../especificacoes/consolidacao-nativa-de-c.md): reconhecimento de dialetos sem lista fechada, C23 como referência e regressões antigas, criação/configuração, GCC/Clang, CMake/Make, editor/headers, análise/memória, build/ligação/testes/DAP, embarcados e plataformas |
| C++ | Projeto com dependências, configuração efetiva, navegação/rename/correções, build/ligação, testes e depuração |
| Rust | Workspace Cargo com features/target, contexto coerente no rust-analyzer, verificação, build, testes e depuração |
| Rust/C/C++ | Projetos mistos, interfaces e bindings, invalidação nos consumidores, ligação e testes nas fronteiras |
| Python | Ambiente e interpretador corretos, análise/imports, correções, testes e depuração; extensões nativas com prova própria |
| Própria IDE | Jornada Rust/C++/QML e IPC, reconstrução, testes, Git/terminal, recuperação e contexto externo conforme §5 |

Na etapa C, o recorte prioritário é **C1 — Projeto e ambiente**, detalhado na
[consolidação C §4.1](../especificacoes/consolidacao-nativa-de-c.md#41-c1--projeto-e-ambiente-recorte-prioritário).
A criação com escolha de instalação/padrão é uma base parcial; completar e
provar a jornada antes de ampliar os demais aceites.

Cada frente deve incluir projeto novo e existente, reabertura, troca de workspace,
dependência/ferramenta ausente, falha e cancelamento, sem perda de conteúdo.
Registrar fixtures, versões, passos e resultado observado por capacidade.
Projetos pequenos/médios precisam representar dependências, múltiplas unidades
ou pacotes e configurações reais, não somente exemplos de um arquivo.

Otimização exige comparação reproduzível de latência, consultas, CPU e memória.
Polimento fecha foco, teclado, estados, recuperação e hierarquia visual junto
da jornada. Enquanto esses aceites estiverem abertos, a presença do LSP ou um
build bem-sucedido não constituem prontidão do ecossistema inteiro.

Após a aceitação de C1–C8, C continua em maturação e polimento por uso real,
conforme a [consolidação C §5](../especificacoes/consolidacao-nativa-de-c.md#5-corpus-e-registro-de-prontidão).
Isso não encerra as outras linguagens nem os fundamentos obrigatórios da 0.4.

### 3.2 Fundamentos obrigatórios e porta de contribuição

**Escopo de 2026-11-03.** Os requisitos F1–F9 e sua sequência pertencem à
[infraestrutura fundamental](../especificacoes/infraestrutura-fundamental-0.4.md).
Esta tabela vincula os aceites à release; nenhum destes bloqueadores é opcional.
B7 bloqueia desde 2026-11-08 só no recorte básico em VM; o restante do porte é
frente de colaboração.

| Bloqueador | Entrega exigida | Evidência mínima e dono |
| --- | --- | --- |
| B6 — Ambiente e ciclo embarcado | F1, F2, F3 e F5: perfis sem edição manual de JSON, DAP consolidado, compilar/emular e visualizador SVD | Toolchains reais; jornadas ARM/RISC-V com console e breakpoint; árvore SVD offline e valores no debug; especificação F1–F5 e trilha 52 |
| B7 — Windows básico (recorte de 2026-11-08) | Etapa 10: Windows 11 x86_64 numa VM, com core e UI nativos compilados, a IDE abrindo, abrir/editar/salvar/reabrir projeto, terminal, Git e configurar/compilar/executar um projeto C e um Rust de host, sem processos órfãos. Mínimo confirmado pelo autor em 2026-11-08. Matriz completa e instalador seguem por colaboração | Matriz e instalador no [60](60-windows-e-edicao-especial.md) §1.1; código Windows e catraca de adiados preservados; o que não for exercitado fica NÃO PROVADO. WSL2 não substitui a VM nativa |
| B8 — Arquivos e contexto | F6: VFS integrada a editor, análise e build em Linux, SSH e container | Identidade, mapeamento, snapshots, conflitos e reconexão com ferramentas no ambiente correto |
| B9 — Ciência e engenharia | F7–F9: plotagem reutilizável, notebook Python/Jupyter, bibliotecas e utilitários, relatório Flash/RAM | Fluxo numérico até C/C++, buffers e desempenho medidos, kernel recuperável, resultados de engenharia conferidos e artefatos identificados |
| B10 — Contribuição e demonstração | README coerente, guias, CI, templates e tarefas revisadas com labels; demonstração curta do ciclo integrado | [contribuindo/11](../contribuindo/11-ci-publica-e-tarefas-para-colaboradores.md); distinguir artefatos locais de configuração/publicação efetiva no GitHub |

O estado inicial da infraestrutura está na especificação §2. Ele não declara
novas provas funcionais. Ferramenta, plataforma ou hardware indisponível fica
NÃO PROVADO com seu alcance; ausência de evidência não encerra um bloqueador.
A demonstração de lançamento usa somente capacidades aceitas, com versão e
ambiente identificados. A 0.5 evolui esses contratos e consumidores em uso.

### 3.3 Sequência de consolidação das ferramentas e do algoritmo

O foco inicial é consolidar a orquestração/integração das ferramentas e o
contexto de cada jornada. O verificador existente acompanha as fatias para
exercitar as entradas e preservar suas garantias; a fase dedicada de evolução
e fechamento do algoritmo vem depois dessa consolidação, ainda na 0.4.
As pré-condições, a primeira fixture C e os aceites finais estão na
[orquestração §12.1](../especificacoes/orquestracao-e-verificacao-de-linguagens.md#121-dependências-entre-ferramentas-e-verificador).

A sequência segue a ordem do autor em §3: C de host até o aceite C1–C8, C++,
Rust e fronteiras C/C++, Python, DAP e QEMU, F5–F9, o algoritmo semântico de
ambiente e, por último, a fase dedicada do algoritmo especializado. A jornada
C11 isolada não encerra a etapa C. Testes comprovam a execução em cada
linguagem; SVD, notebooks e plotagem preservam seus bloqueadores próprios,
sem impedir o endurecimento do verificador host. VFS acompanha seus
consumidores desde o desenho. Depois do algoritmo especializado vêm as camadas
finais de teste (§3.5) e a validação básica do Windows em VM.

O fechamento B4 exige evidências nos contextos suportados e consolidados,
com regressões, mutações das guardas, medições e prévia/aplicação na UI.
O recorte S0/S1 existente conserva seu alcance até cada ampliação ser provada;
o assistente S0–S4 permanece com seus aceites no 52 §11.2. Um resultado positivo
não compensa ferramenta, configuração ou dependência divergente. B5 continua
dependendo de todos os bloqueadores, inclusive os que não alimentam o verificador.

### 3.4 Inventário sistemático de dívida técnica

Antes de refatorar uma capacidade, registrar sua dívida e as lacunas de prova,
conferindo o código e os resultados atuais. O inventário cresce por fatia na
ordem de §3.1; não exige uma reauditoria indistinta do histórico nem autoriza
refatorações gerais. Quantidade de linhas alteradas e redução de tamanho não
demonstram correção de comportamento. Reusar inventários, donos e fixtures
existentes e vincular cada intervenção a um problema reproduzível ou a uma
limitação de engenharia demonstrada.

| Área | O que investigar |
| --- | --- |
| Linguagem C | Tratamento próprio de C e fronteira com C++, parsing, semântica, diagnósticos, toolchains e matriz C1–C8 |
| Linguagem C++ | Análise semântica, padrões, indexação e integração com compiladores |
| Outras linguagens existentes | Profundidade real do suporte, funcionalidades incompletas e fronteiras entre linguagens |
| Algoritmos | Correção, complexidade, cobertura, casos extremos e limites de recursos |
| DAP | Protocolo, estados, falhas, comunicação e consistência da depuração |
| Sistemas embarcados | Toolchains, QEMU, SDKs, configuração e integração de projetos |
| Infraestrutura da IDE | Concorrência, gerenciamento de recursos, persistência e tratamento de erros |
| Interface | Responsividade, consistência, carregamento, foco, recuperação e diagnósticos |

Cada problema tem **ID, área/capacidade e dono; evidência/contexto de reprodução;
severidade e impacto; causa provável; testes necessários; decisão e justificativa;
estado e evidência de fechamento**. Causa não investigada fica assim marcada,
separada de hipótese e causa confirmada. Falta de prova não é, por si só,
evidência de defeito no código. O detalhe pertence ao documento da capacidade;
este inventário conserva sua referência e classificação, sem duplicar o relato.

Severidade: **crítica** para perda/corrupção de conteúdo ou resultado incorreto
tratado como seguro; **alta** para capacidade declarada inviável, inconsistente
ou sem evidência obrigatória; **média** para degradação com recuperação viável;
**baixa** para melhoria localizada sem comprometimento desses aceites. Registrar
o impacto concreto e a prioridade no contexto; não classificar só pelo tamanho
do arquivo ou pela dificuldade da mudança.

As decisões possíveis são **corrigir, substituir, limitar o suporte ou adiar
com justificativa**. Corrigir/substituir exige evidência antes/depois, regressões
e prova real do consumidor; mutações exercitam guardas relevantes, e mudanças
de desempenho precisam de medições. Limitar exige fronteira pública explícita
e comportamento seguro fora dela. Adiar exige motivo, alcance, dependências e
critério de retomada. Limitação ou adiamento não dispensam C1–C8, F1–F9 nem os
demais aceites obrigatórios: um bloqueador permanece aberto até ser resolvido.
Melhoria futura sem impacto nesses compromissos não impede o lançamento.

**Registros — classificação inicial em 2026-11-04 sobre limites documentados;
auditorias e provas posteriores datadas nos donos:**

- **C-01 — Projeto e ambiente, C1.** Dono e evidência:
  [consolidação C §4.1](../especificacoes/consolidacao-nativa-de-c.md#41-c1--projeto-e-ambiente-recorte-prioritário).
  Severidade alta, por jornada obrigatória parcialmente provada; impacto na
  confiança de criação/abertura e seleção persistida. A auditoria localizada
  de criação e seus defeitos, causas e testes estão na
  [consolidação C §12](../especificacoes/consolidacao-nativa-de-c.md#12-c1-dialeto-na-criação-e-validação-do-driver--desenho-de-2026-11-04):
  token de dialeto, modo C do driver, validação fora do laço e sincronização da
  regressão de contexto. Decisão: corrigir nesses donos e conservar abertura,
  divergências e recuperação ainda não comprovadas na matriz de §4.1.
  A auditoria de projetos existentes está na
  [consolidação C §13](../especificacoes/consolidacao-nativa-de-c.md#13-c1-validade-do-ambiente-de-projetos-existentes--desenho-de-2026-11-05):
  configure incompleto, comando histórico/efetivo, instalação removida e
  recarga tardia de outro workspace. C1-05/C1-06/C1-08 têm correção localizada;
  C1-07 conserva o cancelamento da sonda de criação aberto. Estado C1 aberto;
  provas parciais e limites no mesmo documento.
  C1-09/C1-10 e suas provas estão na
  [consolidação C §14](../especificacoes/consolidacao-nativa-de-c.md#14-c1-seleção-visual-de-opções-conhecidas--desenho-de-2026-11-05):
  seleção visual com escape personalizado e preservação do buffer ao retornar
  pela árvore. Não fecham configuração livre de instalações/presets manuais,
  divergências, cancelamento da sonda submetida ou matriz Windows.
  C1-11–C1-14 estão na
  [consolidação C §15](../especificacoes/consolidacao-nativa-de-c.md#15-c1-instalação-do-compilador-em-projetos-existentes--desenho-de-2026-11-08):
  instalação por caminho no kit de projetos existentes, com prévia; escolha
  indisponível recusada; kit do último configure conservado; e CDB do
  compilador anterior, deixada pelo CMake ao recriar o cache, sem passar por
  atual. Configurações manuais/transitivas amplas, C1-07 e Windows seguem abertos.
- **C-02 — `typeof(expr)` na gramática C.** Dono e evidência:
  [consolidação C §10.1](../especificacoes/consolidacao-nativa-de-c.md#101-integração-parcial-e-limites-observados--2026-11-04).
  Severidade alta no escopo C23; impacto na estrutura/sintaxe compreendida pelo
  editor. Rejeição observada no Tree-sitter C 0.24.2; causa interna ainda não
  determinada. Testes: corpus C23 válido/inválido e regressões de C11/C17,
  separando prova de gramática, compilador e clangd. Decisão: corrigir ou integrar
  solução no dono da gramática, com limite explícito até a prova. Estado aberto
  em C3, posterior a C1; não equivale a ausência de semântica no clangd.
- **C-03 — CDB com uma unidade por arquivo, C2.** Dono e evidência:
  `crates/kinein-core/src/index/context/cdb.rs` (`parse_cdb` devolve um mapa
  caminho → unidade). Conferido em 2026-11-08 por leitura: uma segunda entrada
  do mesmo arquivo substitui a primeira e só marca `ambiguous`; headers e
  verificador recusam a unidade ambígua, sem tratá-la como segura. Severidade
  alta: o contexto de um fonte compilado em dois alvos ou configurações fica
  inviável e as alternativas se perdem. Causa confirmada no modelo de dados.
  Testes: CDB real com o mesmo `.c` em dois alvos com `-D`/`-I` distintos, GCC
  e Clang, seleção explícita, invalidação ao reconfigurar, consumidores de
  header e verificador; mutação que volte a descartar unidades. Decisão:
  corrigir na etapa C, conservando todas as unidades com identidade de
  alvo/configuração. Estado aberto.
- **C-04 — Diretório de build único e primeira configuração, C1/C2.** Dono e
  evidência: `cmake::build_dir` (`crates/kinein-core/src/cmake/mod.rs`) passa
  `-B .kinein/build` também com `--preset`, e `CmakeModel` lê só
  `/configurations/0/targets` (`cmake/model.rs`). Conferido em 2026-11-08 por
  leitura. Severidade alta: o `binaryDir` dos presets do projeto é ignorado no
  configure da IDE, trocar de preset reconfigura o mesmo diretório e um gerador
  multi-config expõe só a primeira configuração. Causa confirmada no desenho do
  diretório canônico. Testes: dois presets com `binaryDir` distintos, troca e
  retorno sem reconfigurar o outro, gerador multi-config, build dir de preset
  configurado fora da IDE, contexto e Project Health coerentes. Decisão:
  corrigir na etapa C, com contrato antes de mudar o IPC. Medido em 2026-11-09
  com CMake 4.4.5: o `-B` da IDE vence o `binaryDir`, e com Ninja
  Multi-Config a CDB traz uma entrada por configuração, das quais o clangd usa
  a primeira. Inventário C1-18–C1-23 e decisões do autor de 2026-11-09:
  configure no `binaryDir`; configuração ativa ao começar o configure; com
  multi-config, a que o clangd vê; cache comum com as definições da IDE à
  vista; preset sem `binaryDir` em `.kinein/presets/<preset>`; reparar o build
  dir remove a pasta de um preset só se o cache dela for deste projeto e ela
  não for a raiz nem a contiver. Desenho na
  [consolidação C §17](../especificacoes/consolidacao-nativa-de-c.md#17-c1-configure-no-binarydir-configuração-ativa-e-include-de-presets--desenho-de-2026-11-09).
  Estado corrigido no C1, em sete fatias: C1-18, corrigido em 2026-11-09
  ([40.7 §7.394](40.7-registro-das-entregas.md), protocolo 0.207.0); C1-19–C1-21,
  corrigidos em 2026-11-09 ([40.7 §7.396](40.7-registro-das-entregas.md),
  protocolo 0.209.0); C1-23, corrigido em 2026-11-09
  ([40.7 §7.397](40.7-registro-das-entregas.md), protocolo 0.210.0); C1-24,
  corrigido em 2026-11-09 ([40.7 §7.398](40.7-registro-das-entregas.md),
  protocolo 0.211.0); C1-25, corrigido em 2026-11-10
  ([40.7 §7.399](40.7-registro-das-entregas.md), protocolo 0.212.0); C1-26,
  corrigido em 2026-11-10 ([40.7 §7.400](40.7-registro-das-entregas.md),
  protocolo 0.213.0); C1-22, corrigido em 2026-11-10
  ([40.7 §7.401](40.7-registro-das-entregas.md), protocolo 0.214.0). Trocar
  de configuração numa pasta multi-config segue no C-03, no C2. Na
  segunda fatia, cada configuração
  configura na pasta dela: o `binaryDir`, ou `.kinein/presets/<preset>` sem ele.
  A configuração fica ativa ao começar o configure, e build, run, debug,
  ctest, flash, `qmlls`, o contexto e o Project Health usam a pasta dela
  ([§17.2](../especificacoes/consolidacao-nativa-de-c.md#172-c1-19c1-21-integração-e-provas--2026-11-09)).
  Decisão do autor de 2026-11-09: o C1-23 vem antes do C1-22. Na terceira
  fatia, um chip do cabeçalho mostra a configuração ativa e abre o menu: as
  configurações com a pasta, a ativa marcada e "Automático". A faixa e o menu
  Ambiente abrem o mesmo menu, e o registro órfão sem presets tem a volta ao
  automático
  ([§17.3](../especificacoes/consolidacao-nativa-de-c.md#173-c1-23-integração-e-provas--2026-11-09)).
  A prova achou o C1-24: o CMake recusa o arquivo de presets inteiro por uma
  macro inválida em qualquer preset, e a IDE só expandia o `binaryDir` do
  preset usado. Decisões do autor de 2026-11-09: corrigir antes do C1-22, em
  todos os tipos de preset. Na quarta fatia, a IDE expande cada preset de cada
  tipo como o CMake, avalia a `condition` herdada e tira da lista o preset com
  `$vendor{}` ou condição falsa; com `${fileDir}` na versão 12, configura na
  mesma pasta que o `cmake --preset`
  ([§17.4](../especificacoes/consolidacao-nativa-de-c.md#174-c1-24-integração-e-provas--2026-11-09)).
  A medida do C1-24 achou o C1-25, aberto: o CMake recusa o arquivo por um campo
  fora do esquema ou de versão maior. Decisão do autor: o C1-25 vem depois do
  C1-24 e antes do C1-22. Achou também o C1-26: `matches` e `notMatches` usam a
  expressão regular do CMake. Decisões do autor de 2026-11-10: a IDE pergunta
  ao `cmake --list-presets` do projeto quando a `condition` usa essas formas,
  e o C1-26 vem depois do C1-25 e antes do C1-22. Decisões do autor de
  2026-11-10 para o C1-25: perguntar ao CMake do projeto, o que o kit padrão
  fixa, senão o do `PATH`. Na quinta fatia, a lista, o menu e o configure
  seguem o `cmake --list-presets` desse CMake, na ordem dele; a recusa dele
  recusa os presets com o motivo, e sem CMake vale a leitura nativa. Os
  mutantes acharam o C1-27, corrigido na mesma fatia por decisão do autor: o
  preset oculto pedido pelo nome é recusado antes do job
  ([§18.1](../especificacoes/consolidacao-nativa-de-c.md#181-c1-25-integração-e-provas--2026-11-10)).
  Na sexta fatia, sem resposta do CMake, o preset de `condition` com `matches`
  ou `notMatches` fica no menu, e a dica do item diz que só o CMake a decide;
  depois de uma forma indecisa, uma macro inválida ou um `$vendor{}` não
  recusam nem ignoram mais o preset, como no CMake, que não chega neles
  ([§19.1](../especificacoes/consolidacao-nativa-de-c.md#191-c1-26-integração-e-provas--2026-11-10)).
  Na sétima fatia, com gerador multi-config, o build, o `ctest`, o run e o
  debug usam a configuração que o clangd lê, e o contexto mostra a entrada
  dela na CDB; a faixa informa quando o `cmake --build` do terminal compila
  outra. Corrigidos na mesma fatia, por decisão do autor, os achados da
  regressão: o Catch2 precisa se identificar (C1-28), o caso gtest conta uma
  vez, com o nome da lista (C1-29), o run e o debug não contam mais as
  bibliotecas compartilhadas (C1-30), o caso Catch2 roda só ele (C1-31), e a
  faixa cortada mostra a mensagem inteira ao pairar (C1-33).
  O C1-32, a árvore que repete os casos de um teste que já é um caso, ficou
  para o commit seguinte
  ([§20.1](../especificacoes/consolidacao-nativa-de-c.md#201-c1-22-integração-e-provas--2026-11-10)),
  que o corrigiu com o C1-35, o caso gtest desativado que dizia que passou, e
  o C1-36, os testes do 1 ao 9 que sumiam da árvore numa suíte de dez ou
  mais, e o C1-37, o desfecho do caso de dentro que pintava outra linha
  ([§21.1](../especificacoes/consolidacao-nativa-de-c.md#211-c1-32-integração-e-provas--2026-11-10),
  protocolo 0.215.0). O C1-34, o caso rodado sem o ambiente e os argumentos
  do teste do ctest, ficou para o commit seguinte, que o corrigiu com o C1-38,
  o comando mostrado do caso, o C1-40, o teste cujo desfecho o CTest decide de
  outro jeito, o C1-41, o comando mostrado do ctest, e o C1-42, a linha que
  cai ou passa do prazo
  ([§22.1](../especificacoes/consolidacao-nativa-de-c.md#221-c1-34-integração-e-provas--2026-11-10),
  protocolo 0.216.0). O C1-39, a árvore que não distinguia o caso de dentro
  nem o ignorado, veio no seguinte, com o C1-44, a árvore que voltava ao topo
  ao rodar: a linha mostra o id inteiro, o ignorado é âmbar e a árvore fica
  do tamanho que tinha
  ([§23.1](../especificacoes/consolidacao-nativa-de-c.md#231-c1-39-e-c1-44-integração-e-provas--2026-11-10),
  protocolo 0.217.0). O C1-43, a listagem de um binário que não termina, veio
  depois do INF-09: cada chamada da listagem tem 5 s e 64 MiB e para com o
  cancel do job, e o cancel dá "listagem cancelada"
  ([§24.1](../especificacoes/consolidacao-nativa-de-c.md#241-c1-43-integração-e-provas--2026-11-11),
  protocolo 0.218.0). A prova na UI achou o C1-45, o motivo da listagem que só
  o log do job mostra: as linhas da listagem vão à aba Testes pelo
  `event.test.output`, e listar apaga a saída anterior
  ([§25.1](../especificacoes/consolidacao-nativa-de-c.md#251-c1-45-integração-e-provas--2026-11-12),
  protocolo 0.219.0). Ficam abertos o C1-46 (run e listagem ao mesmo tempo),
  o C1-47 (a saída da aba que para um lote antes do fim) e o C1-48 (a
  contagem dos casos depois do cancel).
- **C-05 — CDB fora dos diretórios conhecidos, C1.** Dono e evidência:
  `crates/kinein-core/src/cdb.rs` procura só em `.kinein/build`, na raiz, em
  `build`, `builddir` e `out/build`. Observado em 2026-11-08: as CDBs deste
  repositório ficam em `build/<preset>/`, fora da lista; por leitura, o contexto
  da IDE neste checkout, sem configure pela própria IDE, não encontra CDB.
  Reprodução na jornada pendente. Severidade alta: um projeto existente
  configurado por presets ou por outra ferramenta perde o contexto C/C++ e é
  orientado a configurar de novo. Causa confirmada por leitura: lista fixa, sem
  o `binaryDir` dos presets. Testes: preset com `binaryDir` próprio já
  configurado, várias CDBs candidatas, escolha visível com origem, CDB obsoleta
  e retorno ao diretório da IDE. Decisão: corrigir junto de C-04. Reproduzido
  em 2026-11-09 com `cmake --preset` real, e com dois achados vizinhos: CDB em
  `builddir/` sem `--compile-commands-dir` para o clangd, e codemodel de outro
  build dir. Inventário C1-15–C1-18, decisões do autor (várias candidatas:
  perguntar; `.clangd` que fixa a base vence; configure no `binaryDir` no C-04)
  e desenho na
  [consolidação C §16](../especificacoes/consolidacao-nativa-de-c.md#16-c1-cdb-de-configurações-existentes-e-coerência-com-o-clangd--desenho-de-2026-11-09).
  Estado corrigido em 2026-11-09 ([40.7 §7.393](40.7-registro-das-entregas.md)),
  com protocolo 0.206.0. Um dono decide a CDB em uso, e o clangd recebe essa
  pasta e reinicia quando ela muda. Com várias candidatas, a faixa pergunta, e
  escolher não reconfigura. O codemodel vem da mesma pasta. Quatro regressões
  com CMake, GCC, Clang e clangd reais cobrem o argv e diagnósticos que
  dependem da CDB. Dezesseis mutantes do core e seis do QML reprovam, e a UI
  nativa isolada mostra a escolha
  ([§16.1](../especificacoes/consolidacao-nativa-de-c.md#161-integração-e-alcance-das-provas--2026-11-09)).
  C1-18 (`include`) segue com C-04, que também traz a troca da configuração
  já escolhida pela UI.
- **CPP-01 — Contrato de CodeAction incompleto.** Dono e evidência:
  `crates/kinein-core/src/lsp/manager.rs` (pedido no ponto do cursor) e
  `lsp/parse.rs` (aceita só ação literal com `edit` inline). Conferido em
  2026-11-08: sem `range` da seleção, sem filtro `only` e sem
  `codeAction/resolve` anunciado; ações só de comando e as que dependem de
  resolve são descartadas. Severidade média: o comportamento é seguro, mas
  correções e refatorações oferecidas pelos servidores não chegam à UI. Causa
  confirmada no handshake e no parser. Testes: clangd e rust-analyzer reais com
  seleção, kinds, resolve e comando; ação obsoleta após edição; range UTF-16
  inválido recusado antes da escrita; transação existente. Decisão: corrigir na
  etapa C++; a jornada C usa o mesmo cliente quando exigir. Estado aberto.
- **INF-01 — Testes do terminal dependentes do shell do usuário.** Dono e
  evidência: `crates/kinein-core/src/tests/terminal.rs` e
  `terminal/session.rs`. Observado em 2026-11-08, também no commit anterior a
  C1-11: com `$SHELL` zsh, `terminal_open_accepts_only_workspace_directory_as_cwd`
  não via a saída de `pwd`. Causa confirmada em PTY: a inicialização interativa
  do usuário (assistente do Powerlevel10k sem configuração; `zsh-newuser-install`
  sem `.zshrc`) consome a entrada digitada logo após abrir. O terminal da IDE se
  comporta como qualquer emulador; o defeito era de prova. Três testes que
  digitavam `echo` também passavam pelo eco da entrada, sem o comando executar.
  Severidade média: prova falsa e gate dependente do ambiente. Correção: em
  Unix, os testes de despacho injetam `/bin/sh` no gerenciador; um teste próprio
  confere que, sem injeção, `terminal.open` sobe e informa o `$SHELL` do usuário;
  as agulhas só aparecem se o comando executar. Prova: 50 testes do terminal
  verdes com zsh real; trocar o shell por `cat` reprova os quatro testes de
  execução (com as agulhas antigas, só um), e ignorar a injeção reprova quatro
  no zsh. Windows mantém o shell padrão nesses testes e não foi exercitado.
  Estado corrigido no recorte Linux.
- **INF-02 — Terminal com bash, zsh e fish reais.** Dono e evidência:
  `crates/kinein-core/src/terminal/`, `tests/terminal_shells.rs` e
  `ui/qml/runtime/TerminalPendingCommands.qml`. Pedido do autor em 2026-11-08:
  qualquer configuração de zsh deve funcionar na IDE tanto quanto a de bash,
  não só a de uma máquina; fish entra na matriz por pedido do autor.
  Severidade alta: jornada B2 sem evidência. Causa confirmada por reprodução
  em 2026-11-08, com HOME, ZDOTDIR e XDG isolados: o comando pedido ao abrir
  um terminal era digitado assim que `terminal.opened` chegava
  (`handleTerminalOpened`), antes de a inicialização do shell terminar. Sem
  arquivos de inicialização, o `zsh-newuser-install` consumiu o primeiro
  caractere e o zsh executou o comando mutilado (`cho …`, comando não
  encontrado); com o tema powerlevel10k instalado e sem configuração, o
  assistente consumiu o comando como respostas. bash sem rc e com
  `PROMPT_COMMAND`, zsh com rc lento, p10k configurado com prompt instantâneo,
  oh-my-zsh com plugins e fish com e sem `config.fish` executaram. Decisão do
  autor em 2026-11-08: o core guarda o comando até o primeiro prompt, marcado
  por OSC 133;A pela integração de shell (bash por `--init-file`, zsh por
  ZDOTDIR próprio que devolve o do usuário, fish por `--init-command`); a
  integração preserva o `zsh-newuser-install` de quem não tem arquivos de zsh;
  shell sem integração conhecida recebe o comando na abertura, limite
  documentado; sem marcador, o terminal mostra o comando aguardando com
  "Enviar agora" e "Descartar". Testes: matriz com o shell real de cada perfil
  e agulhas de execução (bash sem rc, `PROMPT_COMMAND` e tema que reescreve
  `PS1`; zsh sem arquivos, rc lento, p10k sem configuração e com prompt
  instantâneo, oh-my-zsh, ZDOTDIR do usuário e `.zshenv` que o define; fish
  sem e com `config.fish`; rc de bash que troca de shell), assistentes
  respondidos pelo teste, comando sem marcador enviado e descartado a pedido,
  sessão que sai com comando pendente, marcador partido entre leituras, e
  mutações das guardas. Estado corrigido no recorte Linux em 2026-11-08
  ([40.7 §7.390](40.7-registro-das-entregas.md)): matriz verde com bash 5.3,
  zsh 5.9 e fish 4.9; na UI real sob Xvfb, com zsh sem arquivos, a faixa
  aparece sobre o menu do `zsh-newuser-install`, o comando executa depois do
  `q` e "Descartar" deixa o prompt sem ele; 12 mutações do core e 13 da UI
  reprovadas, e uma sobrevivente com motivo (o `$?` preservado pelo marcador do bash só importa
  na forma em texto do `PROMPT_COMMAND`, usada antes do bash 5.1). Sem prova
  local: bash anterior a 5.1, fish anterior a 4 (sem marcador nativo, cobre-o a
  função injetada) e Windows, cujo shell não tem integração e recebe o comando
  na abertura.
- **INF-03 — SQLite embutido com leitura de memória não inicializada.** Dono e
  evidência: `Cargo.toml` (`rusqlite` 0.32 com `bundled`, que traz o SQLite
  3.46 do `libsqlite3-sys` 0.30.1), `db/mod.rs` e `datasource/sqlite.rs`. O
  `cargo-deny` do gate rápido de 2026-11-09 acusou RUSTSEC-2026-0459
  (CVE-2026-119375), publicado depois do gate verde anterior: antes do SQLite
  3.54.0, abrir um banco com um `-wal` irmão de page size diferente pode
  devolver memória antiga do processo em consultas comuns. A IDE abre os dois
  casos: `.kinein/kinein.db`, em WAL, dentro do projeto, que pode vir de um
  repositório não confiável, e o arquivo SQLite que a pessoa escolhe como
  fonte de dados. Severidade alta: gate bloqueado e exposição local ao abrir
  arquivos de terceiros. Causa confirmada pelo advisory; a correção só existe
  no `libsqlite3-sys` 0.39 (SQLite 3.54), com `rusqlite` 0.41. Decisão do autor
  em 2026-11-09: atualizar a dependência num commit próprio, sem ignorar o
  advisory no `deny.toml`. Testes: os existentes de `db` e `datasource`, mais um par banco
  e `-wal` de page size diferente pelos dois donos e a versão mínima do SQLite
  ligado. Estado corrigido em 2026-11-09
  ([40.7 §7.392](40.7-registro-das-entregas.md)): `rusqlite` 0.41.1 liga o
  SQLite 3.54.1, que zera o resto da página lida do WAL. O `foldhash`, Zlib,
  entra por exceção própria no `deny.toml`, decisão do autor. Com
  `MALLOC_PERTURB_`, o par forjado lê zeros na fonte de dados, e com o 3.46.0
  os mesmos testes reprovam. Limite: o par forjado esconde o rascunho do
  projeto nas duas versões, porque o `SQLite` não o detecta e a IDE não valida
  páginas por conta própria.
- **INF-04 — Fim da sessão de terminal antes da saída final.** Dono e
  evidência: `crates/kinein-core/src/terminal/session/threads.rs`. Observado
  em 2026-11-09: `finished::a_finished_session_stays_readable_but_takes_no_input`
  falhou uma vez sob carga (um `cargo check` de editor em paralelo), sem repetir
  em 40 rodadas sozinho e 15 com a suíte do terminal; já tinha falhado uma vez
  na sessão do INF-02. Causa confirmada: a thread que espera o processo zera
  `running` e emite `event.terminal.closed` assim que ele sai, sem esperar a
  leitora drenar o PTY, embora a leitora prometa o render do estado final antes
  do `closed`. Uma sonda que segura o lock do estado, o que para a leitora, e
  libera o processo por um FIFO recebeu `closed` antes do render com a última
  linha. No teste, a rolagem acontece com o histórico ainda crescendo.
  Severidade média: o fim da sessão é anunciado com saída por entrar, e quem a
  lê no `closed` perde o final. Decisão do autor em 2026-11-09: corrigir agora,
  em commit próprio. Correção: a leitora sinaliza o EOF, e a espera do processo
  aguarda esse sinal antes de `running` e `closed`, com teto para o filho em
  segundo plano que mantém o PTY aberto. Testes: a ordem com a leitora parada,
  o teto com um filho que segura o PTY, a matriz `terminal_shells` e mutações
  da espera e do teto. Estado corrigido em 2026-11-09
  ([40.7 §7.391](40.7-registro-das-entregas.md)): a leitora sinaliza o EOF
  depois do render final, e o `closed` espera esse sinal, com teto de 2 s. Com
  a leitora parada pelo lock do estado, não há `closed` até ela terminar, e o
  render da última linha vem antes dele; com um filho que segura o PTY, a
  sessão termina no teto. Três mutações reprovam. A carga de CPU sozinha não
  reproduziu a falha original, nem antes nem depois.
- **INF-05 — Linha da tela do terminal juntada por heurística nos testes.**
  Dono e evidência: `render_lines` em `crates/kinein-core/src/tests/mod.rs`.
  Observado em 2026-11-09, no `cargo test` sem o `TMPDIR` do gate:
  `serial::ownership_run::port_and_script_names_remain_literal_in_both_micropython_run_methods`
  reprova sempre, com o caminho `…/file it's safe.py` partido em `…/file` e
  `it's safe.py`; com o `TMPDIR` do gate, passa. Causa confirmada: o render não
  diz se a linha continua na seguinte, e o ajudante só junta uma linha cheia
  que não termina em espaço. Aqui o espaço cai exatamente na coluna 80 (79
  caracteres antes dele, com `/tmp` e PID de 7 dígitos). O `run.script` passa o
  argv sem shell; o defeito é da prova. Severidade média: o veredito depende do
  comprimento do caminho temporário e do PID, e o gate não o vê. Decisão do
  autor em 2026-11-09: cada linha do `event.terminal.render` passa a dizer se
  continua na seguinte, pela marca `WRAPLINE` que o `alacritty_terminal` já põe
  na última célula da linha quebrada, com contrato e versão nova do protocolo;
  o ajudante junta só por essa marca. Commit próprio, depois do C1-18. Testes:
  quebra exatamente num espaço, linha cheia sem quebra, quebra dupla e
  redimensionamento, com mutações. Estado corrigido em 2026-11-09
  ([40.7 §7.395](40.7-registro-das-entregas.md)), protocolo 0.208.0: o render
  emite `wrapped`, e o ajudante junta só pela marca, devolvendo o espaço final
  que o render descarta. Com o ajudante anterior, o teste reprova sem o
  `TMPDIR` do gate e passa com ele; agora passa nos dois. Oito mutações
  reprovam.
- **INF-06 — Prazo de relógio nos testes da verificação de edição.** Dono e
  evidência: `crates/kinein-core/src/build/edit_verification/tests/mod.rs`, que
  dá 5 s a cada verificação. Observado em 2026-11-09, no gate rápido do
  C1-19–C1-21: cinco testes reprovaram com `duration_ms` 5000 e "Contexto
  alterado ou prazo excedido.". Foram
  `edit_verification_real_host_corpus` e os casos de driver GCC/G++, erro de
  macro, header restaurado e includes relativos e forçados. Na mesma árvore,
  passaram em outras três rodadas do gate e, sozinhos, em duas seguidas.
  Causa provável: cada verificação roda o compilador real ao menos três vezes
  (sonda, antes e depois) dentro do prazo. O gate roda a suíte com paralelismo
  pleno, junto de regressões com CMake e compiladores reais. O produto responde
  certo ("prazo excedido"), mas o teste espera o veredito. Severidade média: o
  gate reprova de forma intermitente, sem defeito no produto. Testes: a
  reprodução sob carga controlada e uma prova que não dependa do relógio da
  máquina. Decisão do autor de 2026-11-09: fica aberto por enquanto, sem
  mudar o teste. Decisão do autor de 2026-11-12: pagar na 0.4, com o prazo
  como parâmetro do teste: o teste que não é sobre o prazo recebe um prazo
  folgado, e o que é sobre o prazo usa um compilador falso que nunca termina
  ([63](63-regras-e-plano-de-engenharia-da-0.4.md) §E.5). Estado aberto.
- **INF-07 — `settings.json` de esquema desconhecido sobrescrito.** Dono e
  evidência: `crates/kinein-core/src/settings.rs`. `read_values` devolve o
  vazio para o arquivo sem `schemaVersion` ou com outro, e `set_workspace` e
  `set_global` juntam o que chega a esse vazio e gravam por cima. Observado
  em 2026-11-10, no preparo da prova na UI do C1-34: o
  `.kinein/settings.json` escrito à mão com `{"autoSave": false}`, sem o
  `schemaVersion`, foi ignorado, e o primeiro salvamento do layout o
  substituiu, sem o `autoSave`. Pela mesma leitura, uma versão da IDE com o
  esquema 1 apagaria o arquivo de uma versão com o esquema 2 no mesmo
  projeto. Severidade média: perde, sem aviso, preferências que a IDE não
  soube ler. Causa confirmada por leitura: a escrita não distingue o arquivo
  ausente do que ela não entende. Testes: o arquivo sem `schemaVersion`, o de
  esquema maior e o JSON inválido, cada um mantido no disco depois de um
  `settings.set`, com mutações. Decisão do autor de 2026-11-10: a IDE não
  grava por cima do arquivo que não entende e diz por quê, num commit
  próprio, fora do C. Medido em 2026-11-12, com o core 0.219.0, `HOME` e
  `XDG_CONFIG_HOME` isolados: o arquivo sem `schemaVersion`, o de
  `schemaVersion` 2, o JSON cortado, o que não é UTF-8 e o de esquema 1 com
  `"autoSave": "nao"` são lidos como vazios, o `autoSave: false` deles não
  vale, e o primeiro `settings.set` os troca por `{"schemaVersion": 1,
  "explorerWidth": 300}`. Decisões do autor de 2026-11-12: o
  `settings.get` e o `settings.set` dizem o motivo por escopo, num campo novo
  do `SettingsResult` (protocolo 0.220.0); o `settings.set` daquele escopo é
  recusado com o mesmo motivo; a faixa de saúde do projeto mostra o problema
  do `.kinein/settings.json`, e o diálogo de Configurações o de cada escopo;
  INF-11 e INF-12 entram no mesmo commit. Desenho: a leitura segue a do
  `runconfig.rs`, que já distingue o arquivo ausente do que não entende
  (o que não é arquivo regular, o que passa de 1 MiB, o que não é UTF-8 nem
  JSON, o esquema que não conhece e o valor de tipo errado). Correção de
  2026-11-12, protocolo 0.220.0 (40.7 §7.410): a leitura e a escrita saíram
  para `settings/file.rs`; o `settings.get` leva `globalProblem` e
  `workspaceProblem`, com o caminho e o motivo; o `settings.set` daquele
  escopo é recusado com `INVALID_REQUEST`, e o arquivo fica como estava; a
  faixa mostra o do projeto, depois dos avisos de build, com o botão
  **[Configurações]**, e o diálogo, o de cada escopo e a gravação recusada.
  Provas: os cinco casos medidos e outros quatro motivos da leitura (o
  `schemaVersion` em texto, a lista, o arquivo vazio e o de mais de 1 MiB),
  mais a pasta no lugar do arquivo, nos dois escopos, cada um idêntico no
  disco depois do `settings.set`; o teste do IPC com o arquivo sem
  `schemaVersion`; o harness `tst_settings_problems`; 27 mutantes que
  compilam, todos reprovados; na UI nativa, o projeto sem `schemaVersion` (o
  caso de 2026-11-10) e o global de esquema 2, recusados sem mudar um byte,
  e o arquivo corrigido por fora usado na gravação seguinte. Estado
  corrigido.
- **INF-11 — Chave desconhecida de um `settings.json` de esquema 1 apagada.**
  Dono e evidência: `crates/kinein-core/src/settings.rs`. O `SettingsFile`
  lê os campos conhecidos e ignora os outros, como o comentário do
  `SettingsValues` promete para a compatibilidade com um core mais novo, mas
  a escrita só grava os conhecidos. Medido em 2026-11-12, no preparo do
  INF-07: `{"schemaVersion": 1, "autoSave": false, "chaveFutura": 7}` vira,
  depois de um `settings.set`, um arquivo sem a `chaveFutura`. Severidade
  média: um core mais velho apaga, sem aviso, o que um mais novo gravou no
  mesmo esquema. Causa confirmada pela medida: a escrita parte dos valores
  conhecidos, e não do objeto lido. Testes: a chave desconhecida e o valor
  dela ficam no disco depois de um `settings.set`, nos dois escopos, com
  mutações. Decisão do autor de 2026-11-12: corrigir junto do INF-07; a
  escrita guarda as chaves que não conhece. Correção de 2026-11-12 (40.7
  §7.410): a escrita parte do objeto lido e só troca as chaves que conhece;
  a conhecida que vinha `null` fica até ser gravada, sem sair em dobro.
  Provas: a chave desconhecida, com objeto e número, nos dois escopos, e a
  UI nativa com a `chaveFutura` mantida. Estado corrigido.
- **INF-12 — `settings.json` do projeto gravado fora dele por symlink.**
  Dono e evidência: `crates/kinein-core/src/settings.rs`, que lê com
  `read_to_string` e grava com `fs::write`, os dois seguindo o symlink.
  Medido em 2026-11-12, no preparo do INF-07, com o core 0.219.0 e pastas
  isoladas: com `.kinein/settings.json` apontando para um arquivo que não
  existe fora do projeto, um `settings.set` do escopo do projeto o cria; com
  o link para um arquivo de texto existente, o substitui pelo JSON das
  configurações. O salvamento do layout é automático e não espera a
  confiança no projeto: abrir um repositório clonado com esse link basta.
  Severidade alta: a IDE cria ou sobrescreve, sem aviso, um arquivo da
  pessoa fora do projeto. Causa confirmada pela medida: o escopo do projeto
  não confere o tipo do caminho, como o `runconfig.rs` confere. Testes: o
  link pendente, o link para um arquivo existente e para uma pasta, cada
  um sem nada gravado fora do projeto, e o global por symlink lido e gravado
  no alvo, com mutações. Decisão do autor de 2026-11-12: corrigir junto do
  INF-07; no escopo do projeto, o symlink e o que não é arquivo regular são
  recusados, com o motivo na faixa; o global, que é da pessoa e pode vir de
  dotfiles, segue o link e recusa o alvo que não é arquivo regular.
  Correção de 2026-11-12 (40.7 §7.410): no projeto, a leitura usa o
  `symlink_metadata` no `.kinein` e no arquivo, e a escrita é atômica e não
  segue um link que apareça depois da leitura; o global segue o link na
  leitura e na escrita. Provas: o `.kinein` ligado a uma pasta de fora com um
  `settings.json` e ao que não existe, o arquivo ligado ao que não existe e a
  uma pasta, sem nada criado nem mudado fora; o global por uma pasta ligada,
  gravado no alvo com o link de pé, e o ligado a uma pasta, recusado. Os
  links são de pasta ou para o que não existe, e o Windows os faz com
  junção, sem privilégio: os testes rodam nos dois sistemas. Na UI nativa, o
  `.kinein/settings.json` ligado a um arquivo de texto de fora foi recusado,
  e o arquivo ficou com o mesmo sha256. Estado corrigido.
- **INF-13 — Outros arquivos do `.kinein` gravados por symlink.** Dono e
  evidência: além do `settings.rs` e do `runconfig.rs`, 26 arquivos do core
  citam o `.kinein` e gravam arquivos (entre eles `cmake_configuration.rs`,
  `workspace/session.rs`, `kits/store.rs`, `datasource/store.rs` e
  `project/artifacts.rs`), numa busca grosseira de 2026-11-12, no preparo do
  INF-07: nem todos gravam no `.kinein`. O `runconfig.rs` confere o tipo do
  caminho; os outros não foram lidos nem medidos. Medido em 2026-11-12, com o
  core 0.220.0 e pastas isoladas, repetindo as requisições que a UI nativa
  manda ao abrir um projeto (sem a confiança): com o `.kinein` ligado a uma
  pasta de fora, a abertura cria nela o `workspace.json` e o `kinein.db` dos
  rascunhos; com cada arquivo do `.kinein` ligado a um caminho de fora que
  não existe, a abertura cria os mesmos dois lá; com o `workspace.json`
  ligado a um arquivo de texto existente, a abertura o sobrescreve com o JSON
  do workspace (sha256 diferente). O `kinein.db` ligado a um arquivo
  existente não o muda: o `DraftStore` apaga o link e cria o banco no lugar.
  O `runconfig.rs` e o `datasource/store.rs` recusam o link; o `settings.rs`
  recusa desde o INF-12. Por leitura, gravam com `fs::write` ou
  `create_dir_all`, seguindo o link, sem medir ainda pela ação de cada um:
  `workspace/open.rs` (`persist`), `workspace/session.rs`,
  `workspace/project_options.rs` (`python.json`), `kits/store.rs`
  (`toolchain.json`), `remote/store.rs`, `remote/mirror.rs`,
  `grafana/store.rs`, `db/mod.rs` (`Connection::open`), e o
  `cmake_configuration.rs`, que grava atômico, mas cria o `.kinein` pelo
  link. Severidade alta: abrir um repositório clonado basta para criar
  arquivos fora do projeto ou sobrescrever um arquivo da pessoa, e o
  `kinein.db` guarda o texto dos buffers não salvos, que iria para onde o
  link aponta. Causa confirmada pela medida: cada dono monta o caminho e
  grava por conta própria, e só três conferem o tipo. Testes: o `.kinein`
  ligado a uma pasta de fora, o arquivo ligado ao que não existe e a um
  arquivo existente, em cada dono, sem nada criado nem mudado fora, com
  mutações. Decisões do autor de 2026-11-12: a varredura vem num commit
  próprio, logo depois do INF-07, antes do C1-47, junto do INF-14; a regra é
  a do INF-12, num guarda só por onde passa todo arquivo que o core grava no
  `.kinein` (recusa o `.kinein` e o arquivo que são link ou não regulares,
  grava atômico, nada fora do projeto); com o link, o projeto abre, nada se
  grava por ele, e a faixa diz o caminho e o motivo, num campo novo do
  `workspace.open` (protocolo 0.221.0). Corrigido em 2026-11-12, com o
  protocolo 0.221.0 e o contrato no 03 workspace: o guarda é o
  `kinein_dir.rs`, um módulo folha como o `fsops` (os donos dependem dele, e
  ele só do `fsops`; desenhado primeiro dentro do `workspace`, ele fechava um
  ciclo novo de 20 módulos no mapa de módulos, porque o `kits`, o `cmake` e
  o `cmake_configuration` passavam a depender do `workspace`, que depende
  deles), por onde passam o `workspace.json`, a sessão, o
  `kinein.db`, a toolchain, os alvos remotos, o Grafana, a configuração
  ativa, o `python.json`, o LCOV, os marcadores e a trava da pasta de build
  do configure e o `settings.json` do INF-12. Ele confere cada componente do
  caminho: o ausente e o do tipo certo passam; o link, a pasta do meio que
  não é pasta e o último que não é arquivo regular voltam com o caminho a
  partir do projeto e o motivo, sem nada criado; a escrita que passa é
  atômica. Os arquivos que o `SQLite` cria ao lado do banco ele mesmo abre
  sem seguir o link e sem criar o alvo (medido com o `-wal`, o `-shm` e o
  `-journal`). O `workspace.open` abre o projeto assim mesmo, não grava o
  `workspace.json` e leva no `stateProblem` o motivo do `.kinein`, ou o da
  primeira entrada logo dentro dele que é link; a faixa de saúde o mostra sem
  botão, antes do aviso do `settings.json`, e o zera ao trocar de projeto. O
  `runconfig.rs` e o `datasource/store.rs` seguem com a conferência deles; o
  `remote/mirror.rs` grava no espelho do cache, fora do projeto. Provas: o
  guarda em cada componente, na escrita e no resumo; cada dono chamado
  direto, e o `workspace.open` e o `workspace.saveSession` pelo IPC, com a
  pasta de fora vazia no fim; a pasta de build e a trava do configure
  ligadas; o banco e cada arquivo do `SQLite` ligados ao que não existe; o
  harness `tst_workspace_state_problem`, novo, com o router e o controller
  reais; 29 mutantes que compilam, todos reprovados. Na UI nativa isolada, o
  projeto com o `.kinein` ligado a uma pasta de fora com um arquivo de texto
  da pessoa abriu com a faixa; a sessão e o rascunho do buffer editado foram
  recusados com o motivo, e a pasta de fora ficou com a mesma listagem e o
  mesmo sha256; trocado o link por uma pasta, a faixa sumiu e o estado foi
  gravado no projeto. Estado corrigido.
- **INF-14 — `settings.json` lido pela UI na partida sem as regras do core.**
  Dono e evidência: `ui/src/startup_web.cpp`, `readOption`, chamado pelo
  `main.cpp` antes do `QGuiApplication` para decidir o contexto OpenGL
  compartilhado. Lido em 2026-11-12, no INF-07, sem medir: ele abre o
  `.kinein/settings.json` do projeto da linha de comando e o global com
  `QFile`, que segue o link simbólico, sem conferir se é arquivo regular,
  sem limite no `readAll` e sem o `schemaVersion`; e deixa o
  `grafanaWebView` e o `platformioWebView` do projeto vencerem o global,
  quando o core só usa o global (`settings.rs`, `resolve`). Medido em
  2026-11-12, com a UI 0.3.9 e pastas isoladas, abrindo pela linha de
  comando um projeto com um FIFO no lugar do `.kinein/settings.json`: sem
  primeiro quadro em 20 s, com o processo parado no `open` do FIFO
  (`wait_for_partner`); com o arquivo regular, o primeiro quadro sai em
  949 ms. Severidade alta: um repositório clonado trava a IDE aberta por
  ele; o resto custa só o contexto compartilhado (~13 ms e ~9 MiB, 59
  §7.1.6). Causa: a leitura é uma cópia à parte da do core, anterior ao
  INF-07. Testes: o FIFO, o link e o arquivo sem `schemaVersion` no
  `tst_startup_web`, com a partida medida na UI. Decisões do autor de
  2026-11-12: entra no commit do INF-13, na mesma varredura do `.kinein`; na
  partida vale só o global, como no core, e o arquivo do projeto deixa de
  ser lido ali. Corrigido em 2026-11-12: o `wantsWebView` recebe só o
  arquivo global, e o `main.cpp` não monta mais o caminho do projeto; a
  leitura segue o link, recusa sem abrir o que não é arquivo regular, lê no
  máximo 1 MiB e um byte, e só aceita o objeto JSON com `schemaVersion` 1.
  Ficam duas diferenças do core, que custam só o contexto a mais: o valor de
  tipo errado em outra chave não é conferido, e o `schemaVersion` 1.0 passa,
  porque o Qt o guarda como o inteiro 1. Provas: o `tst_startup_web` com o
  FIFO (com prazo, e o outro lado aberto se a leitura prender), a pasta no
  lugar do arquivo, a pasta global ligada, o limite de 1 MiB nos dois lados
  do byte, e o sem `schemaVersion`, o de versão 2, o de versão em texto, a
  lista, o JSON quebrado, o que não é UTF-8 e a opção em texto; seis
  mutantes que compilam, todos reprovados. Na UI, a mesma medida com o FIFO
  dá o primeiro quadro em 697 ms. Estado corrigido.
- **INF-15 — Arquivo do `.kinein` lido sem conferir o tipo: o FIFO prende o
  core.** Dono e evidência: as leituras do `.kinein` no core, cada dono por
  conta própria, com `fs::read_to_string` ou parecido, sem conferir o tipo
  antes do `open`. Achado em 2026-11-12, na medida do INF-14, e medido com o
  core 0.220.0 e pastas isoladas, mandando o `workspace.open`, o
  `settings.get` e outras leituras de um projeto CMake: com um FIFO no lugar
  do `workspace.json`, do `session.json`, do `toolchain.json`, do
  `configuration.json` ou do `python.json`, nem o `workspace.open` responde
  em 8 s, e o processo fica parado no `open` do FIFO (`wait_for_partner`); no
  controle, sem FIFO, e com ele no lugar do `settings.json`, do
  `remotes.json`, do `grafana.json`, do `runconfigs.json` ou do
  `datasources.json`, o `workspace.open` e o `settings.get` respondem.
  Severidade alta: o core atende os pedidos em ordem, então um repositório
  clonado com esse FIFO trava a IDE inteira ao abrir o projeto. Causa
  provável: a leitura não passa por um guarda, como a escrita passa desde o
  INF-13; a confirmar lendo cada dono, e o link na leitura também. Testes: o
  FIFO, o link e a pasta no lugar de cada arquivo lido, sem prender nem
  seguir, com mutações; no Windows o caminho não tem FIFO, e vale a pasta.
  Decisões do autor de 2026-11-12: é o primeiro item da engenharia da 0.4;
  a leitura passa pelo `kinein_dir`, que abre sem bloquear e sem seguir o
  link e confere o tipo no descritor
  ([63](63-regras-e-plano-de-engenharia-da-0.4.md) §E.2). Medido de novo em
  2026-11-12, no 61934697 (protocolo 0.221.0), com os quatro casos no lugar
  de cada arquivo: o FIFO prende o core inteiro no `session.json`, no
  `toolchain.json`, no `configuration.json`, no `python.json` e nas três
  marcas do configure em `.kinein/build/` (`.kinein-preset`, `.kinein-kit` e
  `.kinein-configure-pending`), já no `workspace.open`, parado no `open` do
  FIFO; no `remotes.json`, no `grafana.json` e no `remote-contacts.json`, no
  pedido do dono (`remote.list`, `grafana.get`). O `workspace.json`, que o
  core só grava, deixou de prender com o INF-13. A leitura segue o link: o
  `session.json` e o `python.json` ligados a arquivos de fora foram usados.
  Decisões do autor de 2026-11-12, na abertura da execução: a leitura
  guardada vale para os dez, para o `remote-contacts.json`, para o
  `datasources.schema1.json` e para as três marcas quando a pasta de build
  está dentro do `.kinein`; as marcas numa pasta de build fora dele e o
  `coverage.lcov` são o INF-18 e o INF-19. O `stateProblem` passa a nomear
  também a primeira entrada logo dentro do `.kinein` que não é arquivo
  regular nem pasta (o FIFO), sem campo novo (protocolo 0.222.0); a pasta no
  lugar do arquivo e o arquivo acima do limite aparecem no erro do método que
  lê, e os donos que toleram o arquivo ilegível (a sessão, a toolchain, os
  remotos, os contatos e o Grafana, como ausentes; o `python.json`, como
  nenhum interpretador escolhido) seguem assim até o catálogo do ENG-02.
  Corrigido em 2026-11-13: o `kinein_dir::read` abre a raiz canônica do
  projeto e cada componente com `openat`, sem seguir link e sem bloquear,
  confere no descritor que o do meio é pasta e o último é arquivo regular, e
  lê no máximo 1 MiB e um byte; o motivo tem o caminho a partir do projeto,
  como na escrita. As marcas do configure acham o projeto pelo `.kinein`
  mais de fora do caminho, porque um preset pode se chamar `.kinein`.
  Provas: o guarda em cada componente, o limite nos dois lados do byte e o
  FIFO no lugar do arquivo e de uma pasta do meio, sem escritor do outro
  lado; a reprodução deste registro pelo IPC, com o arquivo válido, o FIFO,
  o link para o mesmo arquivo fora, a pasta e o arquivo acima do limite no
  lugar de cada um dos quinze, conferindo a faixa, que o conteúdo só é usado
  no controle, o motivo nos donos que erram e a pasta de fora intacta; a
  prévia da receita da toolchain no dono. Os pedidos rodam com prazo, e o
  teste abre o outro lado do FIFO se a leitura prender. 24 mutantes que
  compilam, todos reprovados: dez do guarda e catorze dos donos, um por
  leitura que volta a ler direto. Medido de novo com o core corrigido, os 44
  casos e o FIFO no lugar de cada marca respondem todos, sem processo parado
  no `open`, e o `session.json` e o `python.json` ligados não são usados
  ([40.7](40.7-registro-das-entregas.md) §7.413). O pronto do 63 (nenhuma
  leitura do `.kinein` fora do `kinein_dir`) não se cumpriu para o
  `.kinein-rigor` e o `remote-mirror.json`, achados depois: INF-20. Estado
  corrigido.
- **INF-16 — Mapa de módulos conta o `tests.rs` de um módulo como código.**
  Dono e evidência: `scripts/module_map.py`, que exclui só os caminhos com uma
  pasta `tests` (linha 267). Achado em 2026-11-12, no INF-13: o mapa deu ao
  `kinein_dir` uma dependência do `platform` que só o `kinein_dir/tests.rs`
  tem (o `canonicalize` da pasta temporária). Todo `<módulo>/tests.rs` e
  `<módulo>/<arquivo>/tests.rs` entra assim nas dependências medidas, e um
  ciclo só de teste pode aparecer como de produção. Severidade baixa: o mapa
  e o `--check` exageram arestas, sem esconder nenhuma. Causa confirmada por
  leitura. Testes: um módulo com dependência só no `tests.rs`, fora do mapa,
  e o mapa regenerado sem diferença no resto. Ampliado em 2026-11-12, na
  revisão de arquitetura: o extrator erra nos dois sentidos. Medido nessa
  data, ele dá 184 arestas e 2 ciclos; só o código de produção, com os grupos
  `use crate::{a, b}` lidos, tem 196 arestas e 0 ciclos. Os dois ciclos
  conhecidos só existem em comentário, 10 arestas vêm de comentário ou de
  teste, e 22 arestas reais em import agrupado não são vistas: um ciclo
  fechado por elas passaria. Severidade média. Decisão do autor de
  2026-11-12: corrigir o extrator e apagar a lista de ciclos conhecidos
  ([63](63-regras-e-plano-de-engenharia-da-0.4.md) §E.2). Recontado em
  2026-11-13, com o critério do 63 §E.2, nas árvores do 61934697 e do
  3952b8ad: na da revisão, as falsas eram 10, não 11 (4 de comentário ou
  literal, 4 de `tests.rs` e 2 de teste inline), o que fecha
  184 − 10 + 22 = 196. No 3952b8ad, o extrator dá 200 arestas e os mesmos 2
  ciclos, e o código de produção tem 213 arestas e 0 ciclos, com 9 falsas e
  as 22 perdidas. As 17 novas são da Fase 0: 4 até o cfae82b9, para o
  `kinein_dir` e dele para o `platform`, e 13 do INF-18, 11 delas para o
  `fsops`.
  O exemplo do achado, `kinein_dir → platform`, virou aresta real no INF-15;
  as de `tests.rs` que sobram são `datasource → stderr_tail`,
  `platformio → platform` e `platformio → tools`. Decisão do autor de
  2026-11-13: o aceite passa a ser o da árvore do 3952b8ad (63 §0, D-18).
  Estado aberto.
- **INF-17 — Catálogo do Banco e contatos remotos gravados fora do projeto
  pelo link.** Dono e evidência: `datasource/store.rs` (`save`, `write`,
  `keep_legacy_copy`) e `remote/contacts.rs` (`store`). Achado em
  2026-11-12, na abertura do INF-15. Medido nessa data, com o core do
  61934697 e pastas isoladas: num projeto com o `.kinein` ligado a uma pasta
  de fora vazia, o `datasource.save` responde sem erro e grava o
  `datasources.json` nela. A conferência própria do dono só olha o último
  componente com `symlink_metadata`, que segue o `.kinein` ligado, e a
  escrita é a `fsops::atomic_write` do caminho montado à mão. Lido no
  código, sem medir: o `remote-contacts.json` é gravado com `fs::write`, que
  segue o link do próprio arquivo e o do `.kinein`. O INF-13 deixou o
  `datasource/store.rs` com a conferência dele, por engano, e não viu o
  `remote/contacts.rs`. Severidade alta, a do INF-13: abrir um repositório
  clonado e salvar uma conexão basta para criar ou sobrescrever um arquivo
  fora do projeto. Causa confirmada: cada dono grava por conta própria (a
  causa do ENG-02). Testes: o `.kinein` e cada arquivo ligados, pelo IPC e no
  dono, com a pasta de fora vazia no fim, com mutações. Decisão do autor de
  2026-11-12: a regra do INF-13 (`kinein_dir::write`), num commit próprio
  logo depois do INF-15, antes do ENG-01
  ([63](63-regras-e-plano-de-engenharia-da-0.4.md) §E.1). Desde o INF-15, a
  leitura guardada antes de gravar já recusava o `.kinein` e o catálogo
  ligados; o `remote-contacts.json`, que vale como vazio quando recusado,
  seguia gravado pelo link. Corrigido em 2026-11-13: o catálogo, a cópia e os
  contatos gravam pelo `kinein_dir::write`, todo o catálogo por um ajudante
  só; a conferência própria e a criação de pasta do dono saíram, e o erro da
  cópia recusada passa a trazer o motivo do guarda. Provas: pelo IPC, o
  `datasource.save` com o `.kinein`, o catálogo e a cópia ligados, e a sonda
  remota com o `ssh` falso e o `remote-contacts.json` ligado, com a pasta de
  fora vazia no fim; no dono, o ajudante com o `.kinein` e cada arquivo
  ligados, a restauração da migração com o `.kinein` trocado por um link no
  meio, e o contato com o `.kinein` e o arquivo ligados. Seis mutantes que
  compilam: quatro reprovados (o ajudante, a restauração, o motivo da cópia
  e os contatos gravando direto) e dois equivalentes, a gravação principal e
  a da cópia gravando direto, porque a leitura guardada do mesmo caminho vem
  antes e só uma corrida os separa; a conferência de vocabulário do ENG-02
  pega esse desvio ([40.7](40.7-registro-das-entregas.md) §7.414). Estado
  corrigido.
- **INF-18 — Arquivos do projeto e das pastas de build lidos sem conferir o
  tipo: o FIFO prende o core.** Dono e evidência: as leituras que não são de
  arquivo próprio do `.kinein`. Achado em 2026-11-12, na abertura do
  INF-15, e medido nessa data com o core do 61934697 e pastas isoladas: um
  FIFO no lugar do `CMakeLists.txt` da raiz do projeto, ou do
  `CMakeCache.txt` da `.kinein/build/` (o que o CMake grava, lido pelo
  `ProjectCache::read` do `toolchain/project.rs`), prende o `workspace.open`,
  parado no `open` do FIFO; o `CMakePresets.json`, o `CMakeUserPresets.json`
  e o `build/CMakeCache.txt` de uma pasta que nenhum preset usa não prendem.
  As marcas do configure (`.kinein-configure-pending`, `.kinein-preset`,
  `.kinein-kit`, no `cmake_configuration.rs`) passam pelo guarda do INF-15
  na pasta de build do `.kinein`; numa pasta de build de preset fora dele,
  seguem com `fs::read`, inferido do mesmo código, sem medir. Severidade
  alta, a do INF-15: o repositório clonado traz o arquivo. Causa provável:
  o guarda conhece só os arquivos próprios do `.kinein`, e os leitores dos
  arquivos do projeto e das ferramentas abrem com `fs::read`; o leitor do
  `CMakeLists.txt` na abertura está por confirmar. Testes: o FIFO no lugar
  de cada arquivo que a abertura e o status do CMake leem, sem prender. A
  regra R-DADOS-02 do [63](63-regras-e-plano-de-engenharia-da-0.4.md) cobre
  só o `.kinein`. Decisões do autor de 2026-11-13: o `CMakeLists.txt` e os
  outros arquivos do sistema de build são a verdade do projeto, e o
  `.kinein` só complementa, sem atrapalhar nem tomar o lugar deles; a regra
  nova R-DADOS-04 lê o arquivo do projeto e o da ferramenta como o sistema
  de build os lê, seguindo o link e sem limite próprio, e só evita que a
  leitura prenda e diz o motivo; o item entra logo depois do INF-17, antes
  do ENG-01 (63 §0, D-14, e §E.2). Na abertura, em 2026-11-13, medido com o
  core do cfae82b9, pastas isoladas e um projeto configurado de verdade, cada
  arquivo trocado por um FIFO, uma pasta, um link para fora e um arquivo
  acima de 1 MiB, com dezoito pedidos de leitura um por vez: o FIFO no lugar
  do `CMakeLists.txt`, do `platformio.ini`, do `.clangd`, do
  `CMakeCache.txt` da pasta da configuração ativa e das marcas do preset, do
  kit e da tentativa nela prende o `workspace.open`; no do `Cargo.toml`,
  prende o `configAction.list` e o `python.status`; no do `pyproject.toml`,
  o `python.status`; no do `CMakePresets.json`, o `configAction.list`; no de
  um include dos presets ou do `CMakeUserPresets.json`, prende o
  `cmake --list-presets` que a abertura roda, e o core espera por ele sem
  prazo; no do `CMakeCache.txt`, o configure também prende no CMake. O leitor do `CMakeLists.txt` na abertura
  é o detector de frameworks (`project/detect.rs`), confirmado pelo
  backtrace. O link e o arquivo grande eram lidos como o sistema de build os
  lê. A lista dos leitores de produção achou os mesmos `fs::read` no
  `.clangd`, na CDB, na resposta do file-api, nos artefatos e no ELF, no
  linker script, na lista de dependências da verificação, no índice, na
  busca, na substituição, no git, nos rascunhos, no flash e no banco
  `SQLite` do projeto. Decisões do autor de 2026-11-13, na abertura: o item
  cobre todo leitor de produção de arquivo do projeto ou de ferramenta, num
  commit; o motivo vai num campo novo, o `projectProblem`; o detector, a CDB
  e o file-api perdem o teto próprio, e o trabalho da IDE (o índice, a busca,
  a substituição e a verificação) mantém o dele (63 §0, D-16, e §A.9).
  Corrigido em 2026-11-13 (protocolo `0.223.0`): a leitura comum
  `fsops::open_project_file` abre sem bloquear, seguindo o link, confere no
  descritor que é arquivo regular e lê sem limite próprio; todo leitor da
  lista passa por ela. Os presets com um arquivo recusado são recusados com
  o motivo sem rodar o `cmake --list-presets`; o `begin_configure` recusa o
  `CMakeCache.txt` que não é arquivo regular antes do CMake. O
  `projectProblem` do `workspace.open`, do `workspace.createProject` e do
  `cmake.status` diz o primeiro arquivo recusado entre os marcadores de
  build, os presets com os includes, o `.clangd` e, na pasta da
  configuração ativa, a CDB, o cache e as marcas fora do `.kinein`, e a
  faixa de saúde o mostra. O sinal novo da ponte fica redeclarado nos três
  harnesses que provam a faixa e conta na ENG-23, que passa a 37 sinais
  (decisão do autor de 2026-11-13, no fechamento). Medido de novo com o core corrigido, nenhum FIFO
  prende um pedido, o `projectProblem` nomeia cada FIFO e cada pasta que a
  abertura lê, e o `build.run` com o FIFO no cache termina recusado com o
  motivo, sem CMake filho. Provas pelo IPC, com dezesseis donos em cada
  forma, e no dono de cada leitor; cinquenta e três mutantes, quarenta e
  sete reprovados e seis equivalentes, os leitores que só leem o que acabaram
  de conferir como arquivo regular (40.7 §7.417). Estado corrigido.
- **INF-19 — LCOV da cobertura lido fora do `kinein_dir`.** Dono e
  evidência: `coverage/mod.rs` (`run`), que lê o `.kinein/coverage.lcov` com
  `fs::read_to_string`, sem limite, depois que a ferramenta o grava. Achado em
  2026-11-12, na abertura do INF-15, lido no código. O caminho passa pelo
  guarda do INF-13 antes da ferramenta, que recusa o link e o FIFO, e o
  arquivo é apagado antes de ela gravar; a leitura de volta não passa pelo
  `kinein_dir` (R-DADOS-01), e o limite de 1 MiB dos outros arquivos não
  serve: o LCOV de um projeto grande passa dele sem defeito. Severidade
  baixa: o arquivo é da ferramenta que a IDE acabou de rodar. Causa: o
  `kinein_dir` não tem limite por arquivo. Testes: o LCOV acima de 1 MiB
  lido inteiro; o link e o FIFO no lugar dele entre a ferramenta e a
  leitura. Decisão do autor de 2026-11-13: o limite de cada arquivo é
  declarado no catálogo do ENG-02, e o LCOV entra nele com o limite dele,
  lido de volta pelo guarda (63 §0, D-14, e §E.2). Estado aberto.
- **INF-20 — Marcas do configure, remoções e marcador do espelho fora do
  guarda do `.kinein`.** Dono e evidência: `cmake/rigor.rs` (`applied_rigor`,
  que o `needs_configure` do `build.run` chama, e `record_rigor`: o
  `.kinein-rigor` com `fs::read_to_string` e `fs::write`), `cmake/mod.rs`
  (`record_preset`, que grava e apaga o `.kinein-preset`, e
  `write_file_api_query`, que cria a query do file-api), `cmake/attempt.rs`
  (as marcas da tentativa e do kit gravadas e apagadas com `fs`, depois da
  conferência), `cmake_configuration.rs` (`select_configuration` sem preset
  apaga o `.kinein/configuration.json` com `fs::remove_file`),
  `configaction/builddir.rs` (o reparo apaga a pasta de build da IDE com
  `fs::remove_dir_all`, depois de uma conferência só do caminho) e
  `remote/mirror.rs` (`read_marker` e `write_marker`: o
  `.kinein/remote-mirror.json` com `fs`). Achado em 2026-11-13, na abertura
  do INF-18, listando as leituras e as escritas de produção do core; as
  remoções, a query e as marcas da tentativa e do kit, na abertura deste
  item, listando as escritas e as remoções. Medido nessa data, com o core do
  deb3ed5b e pastas isoladas: com o cache do configure presente, um FIFO no
  lugar do `.kinein-rigor` prende o `build.run` no `open`, sem fim; com a
  marca do rigor ligada para fora, o configure que deu certo grava `strict`
  no alvo, e com a do preset ligada e um preset escolhido, grava `dev`; com
  `.kinein/presets` ligado a uma pasta de fora, o `cmake.repairBuildDir`
  responde sem erro e apaga lá fora a pasta do preset inteira, com o arquivo
  da pessoa dentro, e o `build.run` cria nela a query do file-api antes de
  recusar; com o `.kinein` ligado, o `cmake.selectConfiguration` sem preset
  apaga o `configuration.json` de fora. Com o `.kinein` ou o `.kinein/build`
  ligados, o reparo já recusava, pela leitura guardada do INF-15; um FIFO no
  lugar do `kinein.db` ou de um arquivo do `SQLite` ao lado dele não prende a
  abertura (o `kinein_dir::file` recusa). O marcador do espelho só é lido e
  gravado no espelho que a IDE cria no cache dela, e o `.kinein` não
  sincroniza com o alvo: só um arquivo posto ali à mão o alcança; lido no
  código. Severidade alta, a do INF-13 e do INF-15, e maior no reparo, que
  apaga fora do projeto: o repositório clonado traz o `.kinein`; baixa para o
  marcador do espelho. Causa: os donos recebem a pasta de build ou a raiz do
  espelho e montam o caminho à mão, e o guarda não tinha remoção; escaparam
  às listas do INF-13 e do INF-15, feitas por medida e por leitura. Testes:
  o FIFO e o link no lugar de cada marca no build, com prazo; o reparo, o
  build e a volta à escolha automática pelo link, com a pasta de fora
  intacta; o marcador do espelho ligado e como FIFO; com mutações. Decisões
  do autor de 2026-11-13: entra logo, num commit próprio, antes do INF-18;
  na abertura, o item cobre também as remoções, a query e as marcas da
  tentativa e do kit ([63](63-regras-e-plano-de-engenharia-da-0.4.md) §0,
  D-15, e §E.2). Corrigido em 2026-11-13: o guarda ganha o
  `kinein_dir::remove`, que confere o caminho como a escrita; as marcas e a
  query passam por dois ajudantes do `cmake_configuration.rs`, pelo guarda
  dentro do `.kinein` e como antes numa pasta do projeto fora dele; o
  `begin_configure` confere também a marca do preset e a do rigor; o
  registro da configuração e a pasta do reparo são apagados pelo guarda, e o
  reparo confere o caminho na prévia e ao aplicar; o marcador do espelho
  passa pelo guarda. Medido de novo com o core corrigido: o build com o FIFO
  termina recusado com o motivo, sem thread parada, e cada caso de antes
  recusa com o motivo, sem gravar nem apagar fora. A varredura das escritas e
  remoções de produção não achou outra que saia do projeto: o
  `runconfigs.json`, o export do toolchain e os consoles do Banco conferem o
  caminho por conta própria, e o literal que sobra fora do guarda é a
  migração do ENG-02. Provas: pelo IPC, o `build.run` com o FIFO e o link no
  lugar da marca do rigor e da do preset, recusado antes do CMake e sem
  prender; o reparo, o build e a volta à escolha automática com
  `.kinein/presets` ou o `.kinein` ligados, com a pasta de fora intacta; no
  dono, o `kinein_dir::remove` com o `.kinein`, uma pasta no meio e o
  arquivo ligados, a entrada do tipo errado e o FIFO; as marcas da tentativa
  e do kit ligadas depois do `begin_configure`; as marcas do rigor e do
  preset e a query com `.kinein/presets` ligado; o marcador do espelho
  ligado, como FIFO e com o `.kinein` ligado. Dezoito mutantes que compilam:
  dezesseis reprovados e dois equivalentes, a marca da tentativa gravada e a
  pasta do reparo apagada direto. Eles vêm logo depois da conferência do
  guarda na mesma função, que recusa antes de gravar e faz a prévia recusar
  como a aplicação, e só uma corrida os separa; a forma deles, o dono que
  monta o caminho à mão, é um mutante que o ENG-02 tem de reprovar
  ([40.7](40.7-registro-das-entregas.md) §7.415). Estado corrigido.
- **INF-21 — Verificador de idioma lê a string crua de bytes do Rust e a
  string crua do C++ como código.** Dono e evidência: `strip_c_like` em
  `scripts/check_identifier_language.py` (linha 137), que só reconhece a
  string crua num `r` que não vem depois de letra: o `br"…"` e o `br#"…"#`
  do Rust e o `R"(…)"` do C++ passam pelo ramo das aspas comuns. Achado em
  2026-11-13, no gate rápido do INF-18: o teste novo do `cmake/model.rs`, com
  `br#"{"toolchains": [{"language": "C", …}]}"#`, reprovou com
  `toolchainslanguage` e `ccompilerpath`, palavras feitas do conteúdo da
  string. Medido nessa data, chamando a função: `br#"{"language": "C"}"#`
  sobra como `br#languageC#`; depois de `br#"a"b"#` ou de `R"(a"b)"`, o resto
  da linha some, e um `resposta_nova` nela passa; o `r#"…"#` sai inteiro. A
  árvore do cfae82b9 não tem `br"` em Rust; o C++ tem cinco `R"(…)"`, em
  `ui/src/editor_highlighter_rules.cpp` e `ui/src/markdown_policy.cpp`, e
  neste a aspa de dentro esconde o fim da linha. Severidade baixa: o erro
  fica na linha da string, e nas que existem quase nada vem depois dela; o
  falso negativo deixa passar identificador em português, e o falso positivo
  reprova código certo. Causa confirmada por leitura e pela chamada. Testes:
  o `br"…"`, o `br#"…"#` e o `R"delim(…)delim"` com aspas dentro, sem
  palavra da string e com o identificador depois dela visto, no
  `scripts/test_identifier_language.py`. Decisão do autor de 2026-11-13: entra
  logo, num commit próprio, antes de fechar o INF-18
  ([63](63-regras-e-plano-de-engenharia-da-0.4.md) §0, D-17, e §E.2).
  Corrigido em 2026-11-13: a string crua do Rust (`r`, `br`, com ou sem
  `#`) e a do C++ (`R"fim(…)fim"`, com os prefixos `u8`, `u`, `U` e `L`) são
  puladas inteiras, quando não vêm coladas num nome; a contagem do legado
  fica igual (14070). Provas: cada forma com aspas dentro, sem palavra da
  string e com o identificador depois dela visto; o nome raw do Rust e o
  nome terminado em `r` ou `R` colado numa string seguem como código; o gate
  inteiro sem reprovar o texto da string e reprovando o `RESPOSTA_NOVA`
  depois dela. Oito mutantes, numa cópia, todos reprovados
  ([40.7](40.7-registro-das-entregas.md) §7.416). Estado corrigido.
- **INF-08 — Linha do terminal juntada por heurística no ajudante Python do
  gate.** Dono e evidência: `_texto_do_render` em
  `scripts/verificar_python_debug.py`, que o `verificar_micropython_porta.py`
  também usa. Observado em 2026-11-10, no gate rápido do C1-39: o
  `verificar-python-debug.sh` reprovou dois casos com `mpremote recebeu:
  mpremote-falso connect …/port-one`, sem o `run main.py`. Sozinho, passou
  três vezes. Causa confirmada: o ajudante só junta uma linha cheia que não
  termina em espaço, a heurística que o INF-05 tirou do ajudante Rust, e
  com o `TMPDIR` do gate e um PID de 5 dígitos o espaço cai exatamente na
  coluna 80. Reproduzido sem o gate e sem carga, com um `TMPDIR` de 102
  caracteres, que põe o espaço na coluna 160: as mesmas duas falhas. O
  produto está certo: o render dá o `wrapped` desde 0.208.0. Severidade
  média: o veredito depende do PID e do caminho temporário, e o gate reprova
  sem defeito no produto. Testes: a mesma reprodução, que passa a passar, e
  o ajudante conferido com a quebra num espaço, sem quebra, com quebra dupla
  e com a linha cheia que o programa terminou, com mutações. Decisão: a do
  autor no INF-05, de 2026-11-09, aplicada ao ajudante que ficou de fora: ele
  junta só pela marca `wrapped`, com o espaço final que o render descarta.
  Commit próprio. Estado corrigido em 2026-11-10
  ([40.7 §7.404](40.7-registro-das-entregas.md)): o ajudante, agora
  `_render_text`, junta só pela marca, e um teste dele roda no
  `verificar-python-debug.sh` antes das provas. Com o ajudante anterior, a
  reprodução com o `TMPDIR` de 102 caracteres reprova nos dois casos; agora
  passa. Quatro mutações reprovam. O gate verde que veio depois teve um PID de
  6 dígitos, que não põe o espaço na coluna: a prova do caso é a reprodução.
- **INF-09 — Saída dos jobs cortada no primeiro byte que não é UTF-8.** Dono
  e evidência: `forward_output` em `crates/kinein-core/src/process.rs`, que o
  build, o configure do CMake, os testes e os outros jobs usam. Sem o limite
  de bytes, a linha que não é UTF-8 encerra a leitura daquele pipe, sem aviso;
  antes do 0743e1d1, o `lines().map_while(Result::ok)` fazia o mesmo.
  Observado em 2026-11-10, no desenho do C1-43, e medido no dono: depois de
  uma linha com o byte 0xFF, as linhas seguintes somem, e o programa que
  continua escrevendo morre por SIGPIPE. Com as ferramentas reais, o GCC 16
  copia crua no diagnóstico a linha de um fonte em Latin-1. Num projeto CMake
  com Ninja e 42 objetos, um `-Wunused-variable` nessa linha para o
  `cmake --build` lido pelo `process` no 1/42, com `SIGPIPE` e "build tool
  execution failed", sem gerar a biblioteca. O mesmo build no terminal vai
  até o fim. O CTest 4.4.5 passa crua a saída do teste no `-V`. O Clang
  escapa os bytes (`<E7><E3>`). Severidade alta: no perfil Equilibrado, no
  Livre ou com o `COMPILE_WARNING_AS_ERROR` desligado pelo projeto, um
  projeto válido, com fonte em Latin-1, não compila na IDE quando um aviso
  cai numa linha acentuada, e o erro parece do projeto. No Estrito, o padrão,
  o aviso já é erro, e se perde o resto da saída, com os erros dos outros
  arquivos. Pela mesma leitura, o teste que imprime esses bytes derruba o
  `ctest`. Causa confirmada: a decodificação
  estrita sem limite termina o leitor, e o fim dele fecha o pipe; com o
  limite, o byte inválido já é uma falha declarada, que a verificação de
  edição usa. Testes: no dono, a linha inválida no meio, no fim e com muita
  saída depois, nos dois pipes, e o build real com o fonte Latin-1 pelo core,
  com mutações. Decisão do autor de 2026-11-10: o leitor sem limite troca o
  byte inválido por U+FFFD e segue lendo, como o `from_utf8_lossy` que a
  listagem dos casos já usa; com o limite, o byte inválido segue sendo a falha
  declarada. Commit próprio, antes do C1-43. Estado corrigido em 2026-11-10
  ([40.7 §7.406](40.7-registro-das-entregas.md)): os dois leitores sem limite
  decodificam a linha com U+FFFD e seguem lendo, e o contrato está no
  [03 job](../arquitetura/03-dominios/job.md), sem versão nova do protocolo.
  A regressão `latin1_source`, com o CMake 4.4.5, o Ninja e o GCC 16 reais,
  compila o fonte em Latin-1 e recebe a saída inteira do teste que imprime o
  byte; com o código de antes, ela reprova. Seis mutações reprovam. Limite: o
  texto noutra codificação mostra U+FFFD no lugar dos bytes, sem adivinhar a
  codificação.
- **INF-10 — `rustix` com advisory de comportamento indefinido.** Dono e
  evidência: `Cargo.toml` (`rustix = "1.1"`, que o `Cargo.lock` fixa em
  1.1.4), usado pelo `kinein-core`, pelo `alacritty_terminal`, pelo `polling`
  e pelo `rustix-openpty`. O `cargo-deny` do gate rápido de 2026-11-11, no
  C1-43, acusou RUSTSEC-2026-0486, publicado depois do gate verde anterior,
  sem mudança local nas dependências: as funções de atributo estendido de
  `rustix::fs` (`getxattr`, `listxattr` e as variantes `l` e `f`) marcam como
  iniciados bytes que não foram escritos quando o buffer tem tamanho zero.
  Por leitura, o código do core não chama essas funções: usa `mkfifoat`,
  `flock` e o resto de `fs` e `process`. Severidade média: o gate rápido e o
  completo ficam bloqueados, e a exposição, pelo que o core chama, é nula.
  Causa confirmada pelo advisory: corrigido no `rustix` 1.1.6, compatível com
  o `"1.1"` declarado; o 1.1.6 não estava no cache local do Cargo. Testes: os
  existentes de terminal, console e plataforma, e o `cargo-deny` verde.
  Decisão do autor de 2026-11-12: atualizar o `rustix` para o 1.1.6 num
  commit próprio, antes do C1-43, sem exceção no `deny.toml`. Estado
  corrigido em 2026-11-12 ([40.7 §7.407](40.7-registro-das-entregas.md)): o
  `Cargo.lock` muda só no `rustix`, e o `cargo-deny` passa. O `windows-sys`
  dos crates do Windows fica como estava, e o build no Windows segue para a
  VM da etapa 10.
- **UI-01 — Instalar Ferramentas.** Dono e evidência:
  [especificação §0.1](../especificacoes/instalar-ferramentas.md#01-polimento-visual-e-de-interação-da-base-04).
  Severidade média; impacto na hierarquia e distinção dos estados. Causa provável:
  acabamento parcial da apresentação existente, sem concluir falha de protocolo.
  Testes: foco/teclado, carga/vazio/erro/retry, escalas e capturas comparáveis com
  interação nativa. Decisão: corrigir o acabamento nos componentes atuais.
  Estado aberto e obrigatório em B3; não integra a implementação de C1.
- **UI-02 — A faixa de saúde e o despacho do `CoreClient` crescem um ramo
  por motivo.** Dono e evidência: `ui/qml/workspace/ProjectHealthController.qml`
  (`update()`) e `ui/src/core_client_dispatch.cpp`. Medido em 2026-11-12, no
  INF-13: o `update()` é uma cadeia de prioridade com 23 `apply()`, em 376 das
  400 linhas permitidas a um controller; dos últimos 20 commits que tocaram o
  arquivo, 14 acrescentaram ramos, e o INF-07 e o INF-13 acrescentaram um
  cada. O `core_client_dispatch.cpp` tem 486 das 500 linhas e 56 ramos por
  nome de método; o `handleWorkspaceOpened` cresce a cada campo novo do
  `workspace.open`. Severidade média: cada motivo novo edita a mesma função,
  e a ordem entre eles só existe na leitura da cadeia; os dois arquivos estão
  perto do limite da catraca, e o próximo item que passar teria de dividi-los
  às pressas. Causa: falta de um ponto de extensão (princípio aberto/fechado):
  o motivo e o método não são unidades próprias. Testes: os harnesses da
  faixa e do router, e os testes do despacho, iguais antes e depois, mais um
  motivo e um método novos sem editar a cadeia. Decisões do autor de
  2026-11-12: item próprio, logo depois do INF-13; o desenho vai para a
  revisão de arquitetura com o autor, antes de seguir a etapa C, e a
  execução segue o desenho. Desenho decidido em 2026-11-12: cada motivo é
  uma unidade numa lista ordenada, e o despacho passa à tabela do objeto de
  domínio da ponte ([63](63-regras-e-plano-de-engenharia-da-0.4.md) §C.4 e
  §E.4). Estado aberto.

**Registros da revisão de engenharia de 2026-11-12.** Medidos nessa data, no
commit 6baddbdf. O dono de cada um é o
[63](63-regras-e-plano-de-engenharia-da-0.4.md): a evidência medida está no
§B.1 de lá, e os testes, os mutantes e o critério de pronto, na Parte E.
Decisão do autor de 2026-11-12, para todos: corrigir na 0.4, na ordem do 63
§E.1; o ENG-16 é a única troca consciente. Estado de todos: aberto.

- **ENG-01 — Mutantes avulsos, sobre o checkout vivo.** Severidade média.
  Causa: a mutação nunca virou ferramenta do repositório, com catálogo.
- **ENG-02 — Cada dono monta o caminho do `.kinein`.** Severidade alta: é a
  causa comum do INF-12, do INF-13 e do INF-15. Causa: falta de um catálogo
  dos arquivos próprios, com leitura e escrita por um ponto só.
- **ENG-03 — Checagem barata no fim do gate, e folha que grava no checkout.**
  Severidade média. Causa: a ordem dos passos cresceu por acréscimo.
- **ENG-04 — Três checagens do gate com custo quadrático.** Severidade baixa.
  Causa: um regex por nome sobre o texto inteiro.
- **ENG-05 — clang-tidy sem cache no gate rápido.** Severidade baixa.
- **ENG-06 — Gate de docs que lê arquivo fora do git.** Severidade média: o
  veredito local pode diferir do da CI.
- **ENG-07 — A CI nunca executou.** Severidade alta: o perfil da CI nunca foi
  provado no ambiente dela. Causa confirmada: o job não inicia, por bloqueio
  da conta no GitHub, fora do repositório; e o ambiente da CI não se reproduz
  localmente.
- **ENG-10 — Roteamento do core por cadeia.** Severidade alta: o `lib.rs`
  está no limite, e o próximo método não cabe. Causa: falta de um ponto de
  extensão para método.
- **ENG-11 — Reações a evento e à troca de workspace numa função central.**
  Severidade média. Causa: falta de um ponto de extensão para reação.
- **ENG-12 — Infraestrutura dentro de domínio, e borda que conhece domínio.**
  Severidade média. Causa: as camadas do core nunca foram declaradas.
- **ENG-13 — `workspace` com duas responsabilidades.** Severidade média.
- **ENG-14 — `impl Core` espalhado.** Severidade alta: todo handler alcança
  todo campo. Causa: o corte dos handlers foi físico, e não lógico
  ([35](../arquitetura/35-crescer-sem-god-object.md) §2).
- **ENG-15 — Estado de domínio sem dono no `Core`.** Severidade alta.
- **ENG-16 — Tipo do fio como tipo interno.** Decisão do autor de
  2026-11-12: troca consciente, com gatilho escrito no 63 §B.3. Não é
  corrigido na 0.4.
- **ENG-20 — `CoreClient` como classe única.** Severidade alta: todo método
  e todo sinal editam o mesmo cabeçalho. Causa: a fachada única da
  ARCHITECTURE §3.1, regra 5, revista nessa data.
- **ENG-21 — Raízes de composição da UI no limite.** Severidade média.
  Causa: o módulo por domínio do [27](../arquitetura/27-modulos-por-dominio.md)
  §4 segue aberto.
- **ENG-22 — QML de área que fala com a ponte.** Severidade média. Causa: a
  regra da ARCHITECTURE §5 não tinha gate.
- **ENG-23 — Cliente falso repetido nos harnesses.** Severidade média.
  Desde 2026-11-13, 37 sinais: o INF-18 acrescentou o `workspaceProjectProblem`,
  redeclarado nos três harnesses que provam a faixa (decisão do autor;
  [63](63-regras-e-plano-de-engenharia-da-0.4.md) §0, D-16).
- **ENG-24 — Fachada do `EditorController.qml`.** Severidade média. Decisão
  do autor de 2026-11-12: dissolver a fachada; a decisão anterior, de mantê-la
  congelada ([32](../arquitetura/32-editor-por-responsabilidade.md) §8.4),
  deixa de valer.
- **ENG-25 — Limite de linhas usado como alvo.** Severidade média. Causa: os
  arquivos no limite são os que todo commit edita, por falta dos pontos de
  extensão acima.
- **ENG-30 — Espera fixa em teste.** Severidade média.
- **ENG-31 — Ajudante de teste duplicado.** Severidade baixa.
- **ENG-32 — A UI repete a regra do `settings.json` na partida.** Severidade
  média. Decisão do autor de 2026-11-12: uma regra só, no core; a partida
  pergunta a ele.
- **ENG-33, ENG-34 e ENG-35 — As três linhas de base da QML** (funções sem
  consumidor, derivações repetidas e literais fora do `Theme`). Severidade
  baixa.
- **ENG-36 — Supressões e listas de aceitos sem motivo na linha.** Severidade
  baixa.
- **ENG-37 — Identificadores em português.** Severidade média.
- **ENG-38 — Checklist de riscos sem a verificação apontada.** Severidade
  média. Causa não investigada: a auditoria é o próprio item.
- **ENG-39 — Testes adiados no Windows.** Severidade média; pago na etapa 10.

As demais áreas da tabela ainda exigem inventário por capacidade; esta lista
inicial não certifica ausência de dívida nelas. Atualizar classificação,
causa e estado conforme a evidência, encerrando problemas pelo comportamento
comprovado e pela manutenção/evolução viável, não pela conclusão de um rewrite.

### 3.5 Camadas finais de teste

Pedido do autor de 2026-11-08: depois do algoritmo especializado, uma varredura
rigorosa do produto inteiro evita regressões, falhas de lógica, código que
funciona mas está mal escrito e causa atraso ou bug desnecessário, e falhas por
coisas básicas não feitas. A rigidez já vale em cada fatia (§3); esta etapa a
repete sobre o produto consolidado e acrescenta o que o gate por fatia não
cobre. Os gates existentes estão no
[catálogo](../contribuindo/04-os-gates-que-dizem-nao.md) e o fluxo no
[07](../contribuindo/07-fluxo-e-responsabilidades-dos-gates.md); uma camada
nova entra lá quando for implementada.

| Camada | Defeito que deve revelar | Técnica e evidência | Ponto de partida |
| --- | --- | --- | --- |
| Regressão de jornadas | Fluxo que funcionava e quebrou | Jornadas de C, C++, Rust, Python, DAP/QEMU e F5–F9 com fixtures reais, positivas, negativas e de recuperação, no gate completo estrito | Testes de despacho, harnesses QML, `verificar-exercitacao.sh` e `verificar-embarcado.sh` |
| Mutação das guardas | Teste verde porque a guarda nunca foi exercida | Mutação de cada guarda crítica com baseline verde, que compile e reprove pela guarda; catálogo versionado em vez de scripts avulsos | Mutações manuais registradas por fatia no 40.7; a ferramenta e o catálogo são o ENG-01 do [63](63-regras-e-plano-de-engenharia-da-0.4.md) |
| Propriedades e fronteiras | Caso não imaginado em parsers, IPC, offsets Unicode, caminhos e CDB | Geração de entradas e fuzzing nas fronteiras, com regressão para cada falha encontrada | Casos fixos atuais; ferramenta nova só com registro de componente |
| Ordem temporal e concorrência | Resposta antiga publicada, troca de workspace, cancelamento, processo morto | Sequências forçadas sem `sleep` como sincronização; repetição em lote para revelar intermitência; nenhum processo órfão ao final | Guardas de workspace e testes de cancelamento atuais |
| Desempenho e atraso | Espera fixa, polling, trabalho repetido, custo quadrático, bloqueio do laço do core ou da UI | Baseline e p50/p95 de abrir, digitar, IPC, indexar, configurar e compilar; piora acima do limite medido reprova | Marcador de primeiro frame (G0.5) e medições por fatia |
| Qualidade do código | Lógica duplicada, erro engolido, `unwrap` fora de teste, estado com dois donos, responsabilidade inchada | Revisão dirigida por domínio com o [checklist de riscos ocultos](../arquitetura/16-checklist-de-riscos-ocultos.md), Clippy pedante e catracas de arquitetura, fiação e duplicação | Gates atuais de Clippy, arquitetura, fiação IPC e QML |
| Básico feito | Erro sem mensagem, operação sem cancelamento ou prazo, recurso sem limpeza, dado sem persistência, botão sem efeito | Matriz por superfície: vazio, carregando, erro, nova tentativa, cancelamento, teclado e reabertura, na UI nativa isolada | `verificar-qml-alcance.sh`, `verificar-fiacao-ipc.sh` e provas nativas por fatia |
| Pacote e ambiente limpo | Funciona só na máquina de desenvolvimento | Gate completo estrito em checkout limpo e AppImage em ambiente mínimo | `verificar-appimage.sh` e `testar-appimage` |

Uma camada só conta quando seu defeito-alvo foi reproduzido e reprovado por
ela, por mutação ou falha injetada. NÃO PROVADO por ambiente ausente fica
explícito e mantém a etapa aberta. Contagem de testes não mede esta etapa.

## 4. Primeiro algoritmo: fronteira anterior ao código

**Finalidade:** avaliar uma correção textual sugerida pelo LSP usando o
compilador e o contexto já disponíveis, sem substituir parser, LSP ou build.
Primeiro consumidor: a prévia/aplicação de correções C/C++ no dono atual de
ações de linguagem. O algoritmo completo continua o [52 §11.2](52-arquitetura-executavel-da-0.4.md#112-escopo-dedicado--assistente-semântico-decisão-do-projeto-2026-10-23-posição-de-2026-10-24).

**Estado em 2026-11-02:** S0 e primeira fatia S1 host implementadas na
[especificação própria](../especificacoes/verificador-de-correcoes-lsp.md).
Clang/Clang++ e GCC/G++ reais comparam diagnósticos antes/depois, com headers
imutáveis, ranges UTF-16 estritos e guardas de conteúdo/contexto. A prévia
apresenta evidência, confiança do workspace e confirmação explícita. Corpus,
mutações e prova nativa estão no [registro](registro/40.7.9-a-base-publica-e-o-c-da-0.4.md#7372--verificador-de-correções-lsp-e-guardas-de-contexto-2026-11-02).

**Entradas:** conteúdo e versão do documento, edição LSP, CDB e compilador
efetivos, diretório de trabalho e diagnósticos anteriores. Congelar a
identidade desses dados; resposta obsoleta não libera aplicação.
**Saídas:** erro novo observado, sem erro novo observado ou não verificado,
com origem e motivo. Ausência de erro de sintaxe não prova equivalência
semântica nem sucesso de link/testes; não usar o rótulo genérico “segura”.

**Recorte determinístico inicial:** uma unidade C/C++, edição em memória,
checagem antes/depois com flags da CDB e arquivo-sombra que preserve a
resolução de includes. Sem escrita no documento original. Limites implementados:
uma unidade de até 1 MiB, 5 s por verificação, cancelamento por
troca de versão/contexto. Comando não interpretável, compilador ausente,
timeout, mudança em múltiplos arquivos ou contexto incompleto resultam em
“não verificado”. O contrato IPC 0.198.0 precede o produtor e está nos
documentos de protocolo e LSP.

**Corpus host executável:** correção válida C/C++, erro inserido, include
necessário removido, macros e flags/CDB divergentes, Unicode, edição/contexto
obsoletos, cancelamento, timeout e ferramenta ausente. Também cobre mensagens
idênticas em posições novas, erros além do limite padrão do compilador e
mudanças de headers com metadados restaurados. Duração e falso `newError`
são medidos; seis mutações da comparação e das guardas reprovam.

**Expansão do corpus:** ordem de includes FreeRTOS e macros de SDK exigem
dependências e versões identificadas, com build real; fixtures sintéticas não
os substituem. SDKs e consumidores de S2–S4 conservam seus aceites próprios.
Antes dessa expansão especializada, aprofundar C/C++ de host, introduzir Rust,
provar Rust/C/C++ e então Python, pela especificação comum. A infraestrutura
compartilhada deve nascer de consumidores reais; LSP, compiladores e donos de
build existentes continuam responsáveis por suas verificações.

## 5. Integração opcional de contexto com Codex

Objetivo da jornada de autodesenvolvimento da 0.4: investigar e provar uma ponte
externa para o contexto de editor consumido por `/ide`. Segundo a
[documentação oficial](https://learn.chatgpt.com/docs/developer-commands),
o comando inclui arquivos abertos e seleção no prompt. Essa descrição não
estabelece um protocolo público de adaptador para qualquer IDE.

A integração exige investigação do mecanismo disponível, versão suportada
e teste real de descoberta/contexto. Ainda não implementada nem comprovada.
Delimitar transporte e acesso antes de código; não criar um endpoint que se
anuncie compatível sem prova. Preservar execução externa e uso opcional,
sem painel de IA ou dependência de provedor no funcionamento da IDE.

O aceite do fluxo com `/ide` exige identificar o mecanismo e a versão realmente
suportados, conectar a instância correta e observar no cliente externo o arquivo
ativo, arquivos abertos e seleção atuais. Testar edição ainda não salva conforme
as capacidades do mecanismo, seleção vazia, Unicode, troca de arquivo/workspace,
desconexão e reconexão, impedindo contexto antigo ou cruzado. Declarar os dados
que o mecanismo não transporta. Paths/ranges e identidade do contexto precisam
ser coerentes com os documentos vivos.

Uso do CLI no terminal ou leitura explícita de arquivos não aceita essa integração.
Ela continua não implementada e não comprovada; sua existência não é deduzida da
documentação de uso do comando. A prontidão da jornada com contexto externo só
pode ser declarada após a prova, preservando a operação independente da IDE.
