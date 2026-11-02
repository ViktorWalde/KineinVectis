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

A base prioriza desenvolver a própria IDE: integridade dos documentos,
contexto consistente, editor/LSP, terminal, Git, configurar, compilar,
executar, testar e depurar C/C++, Rust e o frontend QML. Polimento de
UI/HUD/UX e otimização acompanham essas jornadas. Superfícies funcionais
migram dos pop-ups para os docks existentes; menus, tooltips e confirmações
mantêm seu papel. O editor continua a área principal.

A trilha completa de **Sistemas Embarcados Modernos** do [52](52-arquitetura-executavel-da-0.4.md)
continua aberta para contribuições públicas, com marcos 0.4.1–0.4.9 como
referências de escopo, sem exigir releases intermediárias. Funcionalidades
pendentes, cargas extensas de SDK/build e matrizes físicas continuam
compromissos dessa trilha; deixam de bloquear a base pública. As
[quatro tarefas iniciais](../contribuindo/15-tarefas-iniciais-embarcados-0.4.md)
separam implementação, integração, prova em software e prova em hardware.
Nenhuma limitação de emulação demonstra validação física.

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
3. **B3 — Polimento integrado.** Setup primeiro, depois as superfícies com
   maior interrupção do trabalho. Hierarquia, largura, listas, estados,
   atalhos, foco e recuperação; fotos antes/depois e interação na UI nativa.
   Aceite visual permanece separado dos testes de lógica.
4. **B4 — Primeiro verificador semântico.** Implementar um recorte com
   consumidor real e critérios abaixo; o assistente completo continua
   aberto no 52 §11.2.
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

## 4. Primeiro algoritmo: fronteira anterior ao código

**Finalidade:** avaliar uma correção textual sugerida pelo LSP usando o
compilador e o contexto já disponíveis, sem substituir parser, LSP ou build.
Primeiro consumidor: a prévia/aplicação de correções C/C++ no dono atual de
ações de linguagem. O algoritmo completo continua o [52 §11.2](52-arquitetura-executavel-da-0.4.md#112-escopo-dedicado--assistente-semântico-decisão-do-projeto-2026-10-23-posição-de-2026-10-24).

**Entradas:** conteúdo e versão do documento, edição LSP, CDB e compilador
efetivos, diretório de trabalho e diagnósticos anteriores. Congelar a
identidade desses dados; resposta obsoleta não libera aplicação.
**Saídas:** erro novo observado, sem erro novo observado ou não verificado,
com origem e motivo. Ausência de erro de sintaxe não prova equivalência
semântica nem sucesso de link/testes; não usar o rótulo genérico “segura”.

**Recorte determinístico inicial:** uma unidade C/C++, edição em memória,
checagem antes/depois com flags da CDB e arquivo-sombra que preserve a
resolução de includes. Sem escrita no documento original. Limites iniciais
propostos: uma unidade de até 1 MiB, 5 s por verificação, cancelamento por
troca de versão/contexto. Comando não interpretável, compilador ausente,
timeout, mudança em múltiplos arquivos ou contexto incompleto resultam em
“não verificado”. Definir e revisar o contrato IPC antes da implementação.

**Corpus mínimo anterior ao aceite:** correção válida de host C e C++, erro
de sintaxe inserido, include removido necessário, ordem de includes FreeRTOS,
macro de SDK ausente, flags/CDB divergentes, Unicode, edição obsoleta,
timeout e ferramenta ausente. Casos de SDK usam versões identificadas e
build real; fixtures sintéticas não os substituem. Medir falso bloqueio e
tempo; mutações que removem a comparação ou aceitam contexto obsoleto devem
reprovar. Ampliar depois para os SDKs e consumidores de S2–S4.

## 5. Integração opcional de contexto com Codex

Objetivo de fim de ciclo: investigar e provar uma ponte externa para o
contexto de editor consumido por `/ide`. Segundo a
[documentação oficial](https://learn.chatgpt.com/docs/developer-commands?surface=cli),
o comando inclui arquivos abertos e seleção no prompt. Essa descrição não
estabelece um protocolo público de adaptador para qualquer IDE.

A integração exige investigação do mecanismo disponível, versão suportada
e teste real de descoberta/contexto. Ainda não implementada nem comprovada.
Delimitar transporte e acesso antes de código; não criar um endpoint que se
anuncie compatível sem prova. Preservar execução externa e uso opcional,
sem painel de IA ou dependência de provedor no funcionamento da IDE.
