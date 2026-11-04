# 62 — Base pública Kinein Vectis 0.4

> **Classe: PLANO / ESTADO**, conforme indicado em cada seção. Fronteira
> vigente revista em 2026-11-03: fundamentos obrigatórios na 0.4, com evolução
> na 0.5. Substitui a exigência de concluir indistintamente todos os marcos
> 0.4.1–0.4.9; os mínimos desta base bloqueiam a entrega. Não altera registros.
> Visão de engenharia consolidada em 2026-11-04: **Foundation — Engineering
> Consolidation Release**, conforme §1.1; inventário sistemático em §3.4.

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
Detecção/perfis de toolchain, DAP, QEMU, Windows nativo, SVD, VFS, plotagem,
notebooks e ferramentas de engenharia são fundamentos obrigatórios da 0.4,
com mínimos na [especificação de infraestrutura](../especificacoes/infraestrutura-fundamental-0.4.md).
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
   pacotes Linux/Windows com checksum e validação em ambiente mínimo, depois
   dos aceites B1–B4 e B6–B10. Esta fronteira não declara a release pronta e
   não inclui publicação automática.

Quality Center, schema 3, CircuitPython, Banco para embarcados, expansão de
emuladores além de F3 e as demais jornadas não concluídas do 52 permanecem
abertos nos seus donos. Bloqueiam quando integram os mínimos de §3.2 ou uma
dependência/regressão impede os aceites da base. Windows nativo passa a
bloqueador B7; ROS 2, colaboração remota entre pessoas e o motor semântico
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

| Bloqueador | Entrega exigida | Evidência mínima e dono |
| --- | --- | --- |
| B6 — Ambiente e ciclo embarcado | F1, F2, F3 e F5: perfis sem edição manual de JSON, DAP consolidado, compilar/emular e visualizador SVD | Toolchains reais; jornadas ARM/RISC-V com console e breakpoint; árvore SVD offline e valores no debug; especificação F1–F5 e trilha 52 |
| B7 — Windows | F4: core/UI nativos, jornadas e instalador Windows 11 x86_64 | Matriz nativa, instalação/atualização/desinstalação em ambiente limpo; [60](60-windows-e-edicao-especial.md). WSL2 tem prova separada |
| B8 — Arquivos e contexto | F6: VFS integrada a editor, análise e build em Linux/Windows, SSH e container | Identidade, mapeamento, snapshots, conflitos e reconexão com ferramentas no ambiente correto |
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

A sequência começa por contexto/build/LSP de C de host e verificador no mesmo
projeto, aprofunda C até o aceite C1–C8, segue por C++, Rust e fronteiras C/C++,
depois Python. A jornada C11 isolada não encerra a etapa C. DAP, testes e
QEMU comprovam as etapas de execução/depuração aplicáveis. SVD, notebooks e
plotagem preservam seus bloqueadores próprios, sem impedir o endurecimento
do verificador host. Windows/VFS acompanham seus consumidores desde o desenho.

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

**Registros iniciais — 2026-11-04, classificação de planejamento sobre limites
já documentados; sem nova auditoria funcional:**

- **C-01 — Projeto e ambiente, C1.** Dono e evidência:
  [consolidação C §4.1](../especificacoes/consolidacao-nativa-de-c.md#41-c1--projeto-e-ambiente-recorte-prioritário).
  Severidade alta, por jornada obrigatória parcialmente provada; impacto na
  confiança de criação/abertura e seleção persistida. Causa de eventuais falhas
  ainda não investigada. Testes: projeto novo/existente, escolhas reais,
  persistência e recuperação na UI. Decisão: consolidar e corrigir lacunas
  confirmadas no dono. Estado aberto; próximo recorte, sem aceite completo.
- **C-02 — `typeof(expr)` na gramática C.** Dono e evidência:
  [consolidação C §10.1](../especificacoes/consolidacao-nativa-de-c.md#101-integração-parcial-e-limites-observados--2026-11-04).
  Severidade alta no escopo C23; impacto na estrutura/sintaxe compreendida pelo
  editor. Rejeição observada no Tree-sitter C 0.24.2; causa interna ainda não
  determinada. Testes: corpus C23 válido/inválido e regressões de C11/C17,
  separando prova de gramática, compilador e clangd. Decisão: corrigir ou integrar
  solução no dono da gramática, com limite explícito até a prova. Estado aberto
  em C3, posterior a C1; não equivale a ausência de semântica no clangd.
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
