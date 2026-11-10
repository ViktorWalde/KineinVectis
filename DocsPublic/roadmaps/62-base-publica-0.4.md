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
   conforme §4 e [40.7 §7.372](40.7-registro-das-entregas.md#7372--verificador-de-correções-lsp-e-guardas-de-contexto-2026-11-02).
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
| 1. C | C1–C8 em profundidade, incluindo C23; o contexto incorreto do que já existe é corrigido aqui (C-03–C-05) | **Em andamento.** C1 aberto: C1-01–C1-21, C1-23–C1-25, C1-27 e C-05 corrigidos com provas; abertos C-04 (C1-26, C1-22), C1-07, origem por toolchain file e `CC` do ambiente e consulta de versão bloqueada. C2–C8 abertos (C-02, C-03) |
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
e zsh reais, com fish) é o primeiro item. Estado transversal em 2026-11-09: INF-01
e INF-02 corrigidos no recorte Linux (INF-02 com bash, zsh e fish); INF-03,
INF-04 e INF-05 corrigidos; INF-06 e UI-01 abertos. A dívida
técnica que compromete essas capacidades é paga dentro da 0.4 (§3.4), e as
camadas finais de teste estão em §3.5. As provas mantêm rigidez alta: lógica
e comunicação entre módulos exercitadas até o limite, cenários improváveis,
falhas e casos de borda que façam defeitos aparecer, com mutações das guardas;
quantidade de testes não substitui essa rigidez.

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
  Estado aberto, em sete fatias: C1-18, corrigido em 2026-11-09
  ([40.7 §7.394](40.7-registro-das-entregas.md), protocolo 0.207.0); C1-19–C1-21,
  corrigidos em 2026-11-09 ([40.7 §7.396](40.7-registro-das-entregas.md),
  protocolo 0.209.0); C1-23, corrigido em 2026-11-09
  ([40.7 §7.397](40.7-registro-das-entregas.md), protocolo 0.210.0); C1-24,
  corrigido em 2026-11-09 ([40.7 §7.398](40.7-registro-das-entregas.md),
  protocolo 0.211.0); C1-25, corrigido em 2026-11-10
  ([40.7 §7.399](40.7-registro-das-entregas.md), protocolo 0.212.0); C1-26;
  C1-22. Na
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
  mudar o teste. Estado aberto.
- **UI-01 — Instalar Ferramentas.** Dono e evidência:
  [especificação §0.1](../especificacoes/instalar-ferramentas.md#01-polimento-visual-e-de-interação-da-base-04).
  Severidade média; impacto na hierarquia e distinção dos estados. Causa provável:
  acabamento parcial da apresentação existente, sem concluir falha de protocolo.
  Testes: foco/teclado, carga/vazio/erro/retry, escalas e capturas comparáveis com
  interação nativa. Decisão: corrigir o acabamento nos componentes atuais.
  Estado aberto e obrigatório em B3; não integra a implementação de C1.

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
| Mutação das guardas | Teste verde porque a guarda nunca foi exercida | Mutação de cada guarda crítica com baseline verde, que compile e reprove pela guarda; catálogo versionado em vez de scripts avulsos | Mutações manuais registradas por fatia no 40.7 |
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
mutações e prova nativa estão no [registro](40.7-registro-das-entregas.md#7372--verificador-de-correções-lsp-e-guardas-de-contexto-2026-11-02).

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
