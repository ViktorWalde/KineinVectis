# 62 — Base pública Kinein Vectis 0.4

> **Classe: PLANO / ESTADO**, conforme indicado em cada seção. Fronteira
> vigente desde 2026-11-02; substitui a exigência de concluir todos os marcos
> 0.4.1–0.4.9 antes da entrega pública. Não altera registros de entregas.

## 1. Fronteira e contabilização

A entrega pública chama-se **Kinein Vectis 0.4**; os metadados SemVer usam
**0.4.0**. Agrega as etapas não publicadas 0.3.6–0.3.9 e o trabalho já
implementado da trilha interna 0.4, incluindo a etapa 0.4.1 ainda incompleta.
Números históricos permanecem em commits, changelog e registros. Renumerar
o produto não conclui nenhuma funcionalidade pendente.

A base prioriza uso confiável em projetos pequenos e médios e no desenvolvimento
da própria IDE: integridade dos documentos, contexto consistente, editor/LSP,
terminal, Git, configurar, compilar, executar, testar e depurar. A consolidação
segue **C e C++ → Rust e integração Rust/C/C++ → Python → especializações**.
C tem corpus e aprofundamento próprios. QML participa da jornada da própria IDE.
O desenho técnico está na
[orquestração e verificação de linguagens](../especificacoes/orquestracao-e-verificacao-de-linguagens.md).
Polimento de UI/HUD/UX e otimização de algoritmos acompanham essas jornadas.
Superfícies funcionais migram dos pop-ups para os docks existentes; menus, tooltips e confirmações
mantêm seu papel. O editor continua a área principal.

A trilha completa de **Sistemas Embarcados Modernos** do [52](52-arquitetura-executavel-da-0.4.md)
continua aberta para contribuições públicas, com marcos 0.4.1–0.4.9 como
referências de escopo, sem exigir releases intermediárias. Funcionalidades
pendentes, cargas extensas de SDK/build e matrizes físicas continuam
compromissos dessa trilha; deixam de bloquear a base pública. As
[quatro tarefas iniciais](../contribuindo/15-tarefas-iniciais-embarcados-0.4.md)
separam implementação, integração, prova em software e prova em hardware.
Nenhuma limitação de emulação demonstra validação física.

O foco vigente é consolidar a orquestração das linguagens já existentes, os
algoritmos complementares e a experiência de uso. Novos SDKs e jornadas de
bare metal/Linux embarcado seguem a base essencial; as regressões dos recursos
especializados já oferecidos continuam obrigatórias. Esta ordem preserva o
roadmap de Embarcados e substitui sua execução integral como prioridade da 0.4.

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
   Consolidar C/C++, depois Rust e suas fronteiras C/C++, depois Python,
   conforme §3.1. O fluxo com contexto externo `/ide` tem aceite próprio (§5).
3. **B3 — Polimento integrado.** Setup primeiro, depois as superfícies com
   maior interrupção do trabalho. Hierarquia, largura, listas, estados,
   atalhos, foco e recuperação; fotos antes/depois e interação na UI nativa.
   Aceite visual permanece separado dos testes de lógica. Instalar Ferramentas
   requer hierarquia cromática e estados mais distinguíveis, conforme sua
   [especificação §0.1](../especificacoes/instalar-ferramentas.md#01-polimento-visual-e-de-interação-da-base-04).
4. **B4 — Primeiro verificador semântico.** Recorte host integrado e provado
   conforme §4 e [40.7 §7.372](40.7-registro-das-entregas.md#7372--verificador-de-correções-lsp-e-guardas-de-contexto-2026-11-02).
   Ampliar pela base das linguagens e pelo núcleo comum da especificação,
   priorizando Rust antes de Python. O assistente completo continua aberto
   no 52 §11.2; o recorte host não encerra esse trabalho.
5. **B5 — Fechamento.** Idiomas e traduções conforme [57 §3](57-mapa-de-versoes-ate-a-1.0.md#3-pedidos-do-autor-e-para-onde-foram),
   metadados 0.4.0, changelog/manual/tutorial coerentes, gate completo estrito,
   pacote local com checksum e validação em ambiente mínimo. Esta fronteira
   não declara a release pronta e não inclui publicação automática.

Quality Center, schema 3, CircuitPython, Banco para embarcados, expansão de
emuladores e as demais jornadas não concluídas do 52 permanecem abertos nos
seus donos. Só bloqueiam esta base quando uma dependência ou regressão
impedir B1–B5. ROS 2, Windows, colaboração remota e o motor semântico profundo
conservam seus planos posteriores. A revisão de acabamento antes associada
à edição especial é incorporada ao polimento desta base, sem exigir outra
versão pública. Os compromissos de idioma não são cancelados pelo recorte.

### 3.1 Matriz de consolidação e prontidão

| Frente, na ordem de aprofundamento | Aceite mínimo a demonstrar |
| --- | --- |
| C | Projeto C puro com biblioteca e executável em várias unidades, contexto/headers corretos, correções verificadas, build/ligação, testes e depuração |
| C++ | Projeto com dependências, configuração efetiva, navegação/rename/correções, build/ligação, testes e depuração |
| Rust | Workspace Cargo com features/target, contexto coerente no rust-analyzer, verificação, build, testes e depuração |
| Rust/C/C++ | Projetos mistos, interfaces e bindings, invalidação nos consumidores, ligação e testes nas fronteiras |
| Python | Ambiente e interpretador corretos, análise/imports, correções, testes e depuração; extensões nativas com prova própria |
| Própria IDE | Jornada Rust/C++/QML e IPC, reconstrução, testes, Git/terminal, recuperação e contexto externo conforme §5 |

Cada frente deve incluir projeto novo e existente, reabertura, troca de workspace,
dependência/ferramenta ausente, falha e cancelamento, sem perda de conteúdo.
Registrar fixtures, versões, passos e resultado observado por capacidade.
Projetos pequenos/médios precisam representar dependências, múltiplas unidades
ou pacotes e configurações reais, não somente exemplos de um arquivo.

Otimização exige comparação reproduzível de latência, consultas, CPU e memória.
Polimento fecha foco, teclado, estados, recuperação e hierarquia visual junto
da jornada. Enquanto esses aceites estiverem abertos, a presença do LSP ou um
build bem-sucedido não constituem prontidão do ecossistema inteiro.

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
