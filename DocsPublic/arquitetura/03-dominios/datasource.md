# `datasource.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

## `datasource.*` — os perfis de banco, e a senha que não mora em disco

Domínio da etapa 26/27 (`../roadmaps/35` §9), ampliado no passo 7 do 59.
A arquitetura e os fluxos estão no [37](../37-banco-de-dados.md).

```text
datasource.discover   {}                      -> { candidates: [DataSourceCandidate], containerEngine?, containerHint?, suggestedPorts?: { postgres?, mongo? }, hint? }  (0.124.0; 0.169.0)
datasource.create     { kind: sqliteFile, name, path? } -> { profile }                                       (0.124.0)
                      { kind: containerServer, engine, name, port } -> { jobId, command }  + event.datasource.created
datasource.destroy    { name, data?, password?, clientContext?, expectedContext?, confirmation? } -> { profiles, note?, clientContext? } | { jobId, command, clientContext? } + event.datasource.destroyed (0.158.0)
datasource.odbc.sources {}                     -> { sources: [{ dsn, driver, identity }] } (0.157.0; sem carregar driver)
datasource.odbc.authorize { name, identity, workspace } -> { name, identity, workspace } (0.157.0; consentimento em memória)
datasource.console    { name }                -> { path, created }   (0.149.0; arquivo em .kinein/consoles/)
datasource.list       {}                      -> { profiles: [DataSourceProfile], workspace, consoleBindings, providers: [DataSourceProviderDescriptor], unavailable: [DataSourceUnavailableProfile] } (0.165.0)
datasource.save       { profile }             -> DataSourceWriteResult
datasource.remove     { name }                -> DataSourceWriteResult
datasource.test       { name, password?, clientContext?, expectedContext? }     -> DataSourceTestAccepted   (job)
datasource.impact    { name, password?, sql, clientContext?, expectedContext? } -> aceite + job + event.datasource.impact  (0.150.0; so' leituras)
datasource.introspect { name, password?, clientContext?, expectedContext? }     -> aceite + job
datasource.query      { name, password?, sql, maxRows?, confirmWrite?, clientContext?, expectedContext?, confirmation?, preview? } -> { jobId, name, clientContext?, preview } + job (0.159.0)
datasource.preview.decide { previewId, decision, name, clientContext, expectedContext } -> { jobId } + queried com desfecho (0.159.0)
datasource.disconnect  { name, clientContext, expectedContext } -> { jobId, name, clientContext } + job (0.163.0)
datasource.history    { name }                -> { name, entries: [DataSourceHistoryEntry] }  (0.166.0)
datasource.history.clear { name }             -> { name }                                     (0.166.0)
```

**Histórico de consultas (`0.166.0`, passo 13b).** O core anota cada
`datasource.query` que rodou (o job terminou, com sucesso ou falha do motor);
recusa antes do job não entra. `DataSourceHistoryEntry { sql, at, outcome,
rows? }`: `at` em segundos desde a época (UTC), `outcome` é `ok | failed`,
`rows` são as linhas lidas ou afetadas quando o motor as diz. A mais recente
vem primeiro; a repetição imediata do mesmo texto atualiza a do topo; no
máximo 100 por conexão, e texto acima de 64 KiB não entra. Exigem workspace
aberto; nome sem perfil salvo responde `INVALID_PARAMS`. O arquivo fica em
`$XDG_STATE_HOME/kinein-vectis/datasource-history/` (sem a variável,
`~/.local/state`), um por projeto, nunca no projeto; um arquivo ilegível fica
intacto e o histórico daquele projeto aparece vazio.

Teste, impacto, catálogo, consulta, decisão de prévia e desconexão exigem workspace
aberto; o perfil mora no projeto.

**O motor do servidor em container é o que responde (`0.168.0`, 16b.1,
`roadmaps/59` §2.5.2).** `datasource.create { kind: containerServer }` e
`datasource.destroy` escolhem o motor como o domínio `container.*` (o
primeiro, na ordem `docker`, `podman`, cujo `ps` responde), fora do laço.
Motor que existe e não responde: `create` responde `TOOL_NOT_FOUND` com a
dica e `details: { engine, reachable: false, hint, rawOutput }`, antes do
`run`. `destroy` só pergunta ao motor com `data: true` num PostgreSQL ou
MongoDB; motor que não responde, ou `ps` que falha, é o mesmo erro, e nada
é apagado, nem o perfil — não dá para saber se o servidor é um container
da IDE. Sem motor no `PATH`, não há container da IDE e o plano segue sem
ele. O container do servidor sai com `rm -f -v` (os volumes anônimos).

**O "Criar banco" diz o motor e a porta (`0.169.0`, 16b.2, `roadmaps/59`
§2.5.2).** `datasource.discover` escolhe o motor como o `create` (o que
responde ao `ps`): `containerEngine` só vem quando ele responde e lista; sem
ele vem `containerHint`, o motivo ("o Podman não respondeu: <dica>", ou sem
Podman/Docker no `PATH`). `suggestedPorts` traz, por motor, a primeira porta
de 20 a partir da padrão (5432, 27017) em que um `bind` em `127.0.0.1`
funciona; sem nenhuma livre, o motor fica de fora. `datasource.create {
kind: containerServer }` recusa a porta em que o `bind` falha com "em uso"
duas vezes, com 50 ms entre elas (um `fork` de outra thread herda o socket
de teste até o `exec`, e a primeira resposta pode ser falsa; 40.7 §7.286),
antes de escolher o motor: `INVALID_PARAMS` com a porta e a primeira livre
depois dela.

**Descritores de provedores (`0.164.0`, D1, contrato anterior ao código).**
`datasource.list/save/remove` acrescentam `providers`, em ordem de apresentação.
O formulário e o menu de conexão consomem esse campo pelo CoreClient/roteador existentes; nenhum
segundo método/cliente é criado. Cada `DataSourceProviderDescriptor` tem `id`
(identidade da implementação), `engine` (valor atual do perfil),
`connectionKind: server | file | dsn` e `profileFeatures`, lista tipada com
`credentials | verifiedTls | sampling`. Esses são campos que a implementação
atual aceita; não são permissões efetivas de uma conexão, nem capacidades LSP.
Os IDs transitórios são `builtin.postgres`, `builtin.sqlite`, `builtin.mongo`
e `system.odbc`. Nenhum executável é selecionado, carregado ou sondado nessa
leitura; versão de servidor não é inferida. A lista contém somente os quatro
motores implementados. LSPs pesquisados e InfluxDB ainda não entram nela.

`DataSourceProfile`, seus valores de `engine` e o arquivo schema 1 permanecem
iguais. Não há ID de ferramenta persistido nesta fatia. O formulário deriva
seleção e campos dos descritores; seus rótulos/padrões de preenchimento
continuam com dono único na UI. Motor desconhecido não adota os padrões
PostgreSQL. Sem descritor recebido, edição/salvamento no formulário aguardam
a lista, sem inventar um provedor. Migração de perfis, processo/erros/limites,
instâncias e LSP seguem no [39](../39-drivers-externos-e-compatibilidade.md).

**Catálogo protegido (D1a.1, sem mudança do wire).** Arquivo ausente permite
criar perfis; erro de leitura, JSON inválido, schema/opções não reconhecidos,
entrada não regular ou arquivo acima de 1 MiB impedem sobrescrita.
`datasource.list` responde `INTERNAL_ERROR` nesse caso, em vez de sucesso com
lista vazia. Salvar/remover e criar/destruir recusam antes de seus efeitos;
usam os códigos existentes e mensagem pública sem conteúdo bruto do arquivo.
Schema 1 reconhecido mantém o formato e usa publicação atômica compartilhada.
O projeto continua abrindo; perfil extensível/migração permanecem no 39.

**Perfis indisponíveis e formato 2 (`0.165.0`, D1a.4).** `DataSourceWriteResult`
e o resultado de `datasource.list` ganham `unavailable`, lista de
`DataSourceUnavailableProfile { name, engine, adapter, reason }`, ordenada por
nome; `reason` é `unknownProvider | unsupportedInstallation |
unsupportedOptions | invalidOptions`. Nenhuma opção, valor ou texto do arquivo
atravessa o fio: o perfil fica preservado no disco, sem conectar nem editar.
`datasource.save` com o nome de um indisponível responde `INVALID_PARAMS`;
`datasource.remove` o remove. Ler nunca grava; a primeira gravação sobre um
arquivo schema 1 migra para o formato 2 com cópia
`.kinein/datasources.schema1.json` e releitura verificada (39 §5.1). Os
métodos, os códigos de erro e `DataSourceProfile` não mudam.

```text
event.datasource.tested        { jobId, name, ok, message, clientContext?, ... }
event.datasource.disconnected  { jobId, name, clientContext, success, message }
event.datasource.impact        { jobId, name, sql, severity, clientContext?, confirmationTarget?, requiresConnection, previewEligible,
                                 statements: [{ text, kind, targets, column?, filter?, severity, rows?, totalRows?, note? }] }  (0.158.0)
event.datasource.introspected  { jobId, name, schemas | collections, clientContext?, ... }
event.datasource.previewed     { jobId, name, clientContext, previewId, expiresInSeconds, sql, executedSql,
                                 columns: [string], rows: [[string | null]], affected, truncated, elapsedMs } (0.159.0)
event.datasource.queried       { jobId, name, success, columns: [string], rows: [[string | null]],
                                 rowCount, affected?, truncated, elapsedMs, message?, secretRequired, confirmationSql?, clientContext?, access,
                                 previewOutcome?: committed | rolledBack | expired | cancelled | failed | unknown,
                                 catalogUpdate: none | reload } (catalogUpdate desde 0.162.0; ausente = none)
event.datasource.created       { jobId, success, profile?, message }                          (0.124.0)
event.datasource.destroyed     { jobId, success, message, profiles?, clientContext? }                          (0.129.0)
```

**Descobrir e criar (`0.124.0`).** `DataSourceCandidate { kind: localServer
| container | file, label, detail, running, profile }` — cada achado traz o
perfil que o alcança, pronto para o formulário (adotar não salva; salvar é
do autor). O `create` em container usa autenticação `trust` **só no
loopback** (`-p 127.0.0.1:…`): a IDE não guarda senha, e a porta não sai da
máquina; imagens pinadas (`postgres:16`, `mongo:7`). O MongoDB cria banco na
primeira escrita — não há `create` para ele além do servidor.

**Contexto e política (`0.158.0`).** `expectedContext` é a cópia pública
`{ workspace, profile }`; o core compara antes da senha/job. `clientContext`
aceita até 128 bytes ASCII alfanuméricos, `:._-`; é ecoado nos eventos e nas
recusas de política. Campos são opcionais para clientes anteriores.
A UI atual sempre os envia. Produção pede aviso em qualquer escrita;
remoção/alteração global/impacto desconhecido exigem `confirmation` com nomes
exatos. `readOnly` recusa escrita mesmo confirmada. O léxico comum confere o
lote inteiro e CTE, além da proteção nativa de leitura. Detalhes no 37 §8.

**Desconectar (`0.163.0`).** Contexto/token são obrigatórios; campos extras,
inclusive password, são recusados. O aceite ainda não significa encerrado.
O job aguarda todas as operações reservadas para workspace/nome, depois
libera a barreira e emite `disconnected`. Enquanto espera, novos trabalhos
desse destino e nova autorização ODBC são recusados. Revoga consentimento
ODBC e prévia pendente; COMMIT já aceito é aguardado, não revogado. Perfil,
arquivos e dados permanecem. Operações comuns já aceitas podem concluir;
não se promete rollback nem interrupção imediata do driver. O pool MongoDB
é encerrado explicitamente antes de liberar sua operação. Outro destino e
o despacho continuam livres. Próximo pedido explícito após o evento usa
o fluxo normal de conexão. Donos e provas no 37, desenho no 59 §5.1.5.

**Executar o que o autor escreveu (`0.121.0`, `../roadmaps/35` §7.4).** A
classificação do lote inteiro pelo léxico comum decide o caminho —
`SELECT`, `WITH`, `VALUES`, `TABLE`, `SHOW`, `EXPLAIN` são candidatos a leitura;
CTE mutante, controle de transação e instrução desconhecida são escrita.
No PostgreSQL a leitura roda em `BEGIN READ ONLY`,
no SQLite o arquivo abre com `SQLITE_OPEN_READ_ONLY`. O teto (`maxRows`,
padrão 500, máximo 10.000) vem de fora do texto: `SELECT * FROM (<sql>) AS
kinein_q LIMIT n+1` quando é uma instrução só de `SELECT`/`WITH`/`VALUES`/
`TABLE`, o `step` até n+1 no SQLite; `truncated` diz quando cortou. A política de confirmação desde `0.156.0` está no [37 §3](../37-banco-de-dados.md).
Remoção, substituição destrutiva, alteração sem filtro e operação desconhecida
são recusadas antes do job com `WRITE_CONFIRMATION_REQUIRED`. Em
Desenvolvimento, inserção e criação rodam direto; alteração filtrada tem
medição silenciosa num job. Produção exige aviso para toda escrita.
Se atinge tudo ou não pode ser medida, o evento traz `confirmationSql`, sem
executar a escrita. A UI abre o mesmo diálogo e reenvia o texto com
`confirmWrite: true` depois do gesto explícito.

**Prévia (`0.159.0`).** É execução real dentro de transação PostgreSQL, para
uma instrução direta INSERT/UPDATE/DELETE elegível; não é a contagem do aviso.
Contexto/token e confirmação são obrigatórios antes da escrita.
`preview.decide` rejeita campos desconhecidos, inclusive senha e autorização;
COMMIT confere o perfil salvo novamente e a decisão só é aceita uma vez.
SQLSTATE em FATAL não basta para afirmar recusa: perda de resposta de COMMIT
é `unknown`. Sequências e efeitos externos não são recuperados por rollback.
O ciclo, streaming e orçamentos estão no [37 §9](../37-banco-de-dados.md).

**Releitura após execução (`0.162.0`).** `catalogUpdate: reload` indica que
o catálogo pode estar desatualizado: alteração estrutural/instrução relacional
desconhecida, escrita válida Mongo ou operação ODBC que não é leitura.
O campo ausente equivale a `none`, para eventos de versões anteriores.
O sinal pode acompanhar erro de lote parcialmente aplicado; não afirma
commit. Recusa de política/preflight, senha necessária e prévia não o geram.
A UI confere contexto, consome o resultado terminal uma vez e agrupa
invalidações enquanto a introspecção está ocupada. Mudanças externas e
efeitos indiretos ainda usam F5. Donos e limites no [37](../37-banco-de-dados.md).

A execução mantém o texto do autor (`simple_query`/`execute_batch`), e
`affected` é o que o motor contou (SQLite: a última instrução). Células são
texto; `NULL` é `null`; `BLOB` SQLite vira `<N bytes>`; num lote, a grade
mostra o último conjunto de resultados. O MongoDB lê e escreve desde
`0.155.0`, pela gramática JSON do [manual](../../manual.md); a forma anterior
`coleção {filtro}` continua lendo. O impacto inclui `mongoInsert`,
`mongoUpdate`, `mongoDelete`, `dropCollection` e `replace` (SQLite).
Vazio é `INVALID_PARAMS`; a senha segue a política do perfil
(`SECRET_REQUIRED` síncrono, `secretRequired` no evento).

**TLS no PostgreSQL (`0.121.0`).** `DataSourceProfile.tls: disable |
require` (ausente = `disable`, o de sempre) e `caFile` (PEM em que confiar,
para o servidor autoassinado; ausente = raízes públicas do `webpki-roots`).
`require` é o `verify-full` do libpq — cadeia **e** nome do host conferidos
—; não existe "cifra sem conferir" (o `sslmode=require` do libpq). Só vale
para o motor `postgres`: noutro motor o `save` descarta. Desde a correção
`0.159.0`, conexão comum e prévia definem `SslMode::Require` explicitamente:
servidor sem TLS é recusado, sem fallback sem cifra. Implementação:
`tokio-postgres-rustls` 0.14 (MIT) sobre o `rustls` que o `mongodb` já
trazia — +11 crates medidos em 2026-09-18, `cargo deny` verde
(`../integracoes/37`).

**A senha nunca entra no perfil.** O `DataSourceProfile` guarda motor, host,
porta, banco, usuário e um `SecretSource` — *de onde* o segredo vem —, e o
`password` viaja só no parâmetro do método que precisa dele, por chamada. É a
decisão de `../seguranca/40`, e a UI abre o diálogo de senha por
`secretRequired` no erro, **nunca casando texto de mensagem**.

**Duas formas de resultado, porque há dois tipos de banco.** O relacional
devolve `schemas → tables → columns` lido do `information_schema`; o MongoDB
devolve `collections → fields` com profundidade, tipo **plural** e presença em
%, e a origem do esquema declarada:

```text
DECLARADO   veio do validador `$jsonSchema` da colecao
INFERIDO    veio de `$sample` sobre a colecao
```

**A tela nunca deixa os dois parecidos**, e o custo da leitura vai junto:
quantos documentos foram lidos, e se a amostragem obrigou o servidor a varrer a
coleção inteira (`$sample` varre tudo quando N não é menor que 5% dela). Tetos
de RAM — 2.000 campos, 8 níveis, 10 elementos de array — aparecem como aviso na
coleção em vez de a deixarem com cara de completa.

## Console e árvore (0.160.0, desenho e aceite em 2026-10-06)

Aceito no 40.7 §7.225. datasource.list/save/remove devolvem também workspace e
consoleBindings: lista de {name, paths}, calculada pelo core. paths contém o
console novo e, quando sem colisão, o console legado. Não há leitura de dados
ou senha nessa derivação. datasource.console aceita clientContext e
expectedContext opcionais para clientes antigos; a UI os envia e a resposta
traz name, path, created, clientContext e expectedContext.

datasource.console.statement exige {name, path, text, cursor, selectionStart,
selectionEnd, clientContext, expectedContext}; offsets em unidades UTF-16.
Resposta: {name, path, statement, clientContext, expectedContext}. Recusa antes
de qualquer conexão/job; detalhes de falha contêm só nome/token públicos.
A UI conserva preview no pedido local, valida token/contexto da resposta e
passa statement para datasource.query existente. Não se adiciona autorização
de escrita nem se persiste o texto nesta chamada. Limite: 1 MiB de texto.
Desenho no roadmap 59 §5.12; provas e limites no 40.7 §7.225.
