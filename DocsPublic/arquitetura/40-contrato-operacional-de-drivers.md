# 40 — Contrato operacional de drivers externos

<!-- caminhos-conferidos -->

> **Classe: CONTRATO.** Desenho D1a.3 em 2026-10-07, anterior ao código,
> sobre D1a.1 em `1a9b10f` e o handshake D1a.2 em preparação; integrado
> após D1a.2 em `b29c549`. Tipos e guardião puro implementados.
> Complementa o [39](39-drivers-externos-e-compatibilidade.md) §4.4 e o
> [37](37-banco-de-dados.md). É contrato tipado e validação pura; não
> representa processo, conexão externa ou runtime implementados.

## 1. Fronteira e donos

API externa 1.0, JSON por linha em stdio privado. O envelope é o pedido
JSON-RPC existente e `driver::Reply<T>` da D1a.2. IPC UI/core permanece
`0.164.0`: os nomes `driver.*` não são métodos da IDE. Não há callback que
peça ao core executar SQL, instalação de programa ou segundo sistema de jobs.
O job atual do core coordena uma operação; a operação externa recebe somente
um identificador de correlação, não cria um job concorrente independente.

Tipos operacionais: `kinein-protocol/src/driver/operation.rs`. Guardião puro:
`datasource/driver_stream.rs`, com validação de conteúdo em dono separado
quando necessária. Testes e fixtures JSON v1 próprios medem ausência inicial
dos tipos, formato estrito de pedido, credencial redigida, contexto e ordem
incorretos, terminal repetido, limites cumulativos, NULL e catálogo Mongo.
O runtime D1b terá de limitar bytes antes da leitura/parse e aplicar política,
tempos, fila e coleta real; estas funções não provam esses efeitos.

## 2. Contexto, abertura e pedidos

Todo pedido/chunk/terminal de sucesso carrega `context`:
`{ instanceId, sessionId, generation, operationId }`. `instanceId` é opaco,
gerado pelo core para workspace + perfil + instalação resolvida; motor ou
nome do perfil isolados não são identidade suficiente. `generation` é a
geração integral de configuração/credencial, conferida por igualdade.
`sessionId` é nulo somente em `open` e `shutdown`. Os demais métodos recebem
a sessão opaca devolvida por `open`. `operationId` identifica o pedido atual;
decisão/cancelamento referenciam uma operação alvo explicitamente.

| Método | Parâmetros além de `context` | Resultado terminal de sucesso |
| --- | --- | --- |
| `driver.open` | `engine`, `options`, `restrictions`, `credential` opcional | sessão, versão pública do servidor opcional, operações observadas |
| `driver.test` | nenhum | versão pública do servidor opcional |
| `driver.introspect` | nenhum | `truncated` |
| `driver.query` | `text`, `maxRows` | afetados opcionais, corte, tempo e atualização de catálogo |
| `driver.impact` | `text` | severidade e `SqlStatementImpact` existentes |
| `driver.preview` | `text`, `maxRows` | id da prévia, desfecho existente e atualização de catálogo |
| `driver.decide` | `previewOperationId`, `previewId`, `decision` existente | ids alvo e decisão aceita |
| `driver.cancel` | `targetOperationId` | id alvo e cancelamento aceito |
| `driver.close` | nenhum | confirmação de fechamento |
| `driver.shutdown` | nenhum | confirmação de encerramento |

Pedidos são fechados: desconhecidos e campos de autoridade adicionais são
recusados. `options` separa `schemaVersion` e mapa de valores públicos
tipados (texto, booleano, inteiro), sem shell/código nem senha. O schema do
adaptador validará nomes/valores na fatia de perfis D1a.4; ser desserializável
não autoriza abrir banco. `restrictions` carrega `readOnly` e os limites
negociados. `credential` é texto transitório enviado somente após política
e negociação, com `Debug` redigido. Nunca pertence ao perfil persistido.

O core já possui texto autorizado, política e confirmação. Impacto não
concede autorização. Templates/instruções permanecem nos donos atuais do
core; campos de instrução que os tipos antigos admitem não são aceitos em
catálogo externo nesta primeira API. O adaptador executa operações dentro
das restrições recebidas; ele não pede ao core outra execução por callback.

## 3. Chunks, prévia e conclusão

Notificação `driver.chunk` recebe `{ context, sequence, payload }`.
`sequence` começa em zero e cresce de um em um. O terminal de sucesso contém
`{ context, sequence, result }`, com a próxima sequência. `result` distingue
a operação (`kind`); não aceita uma resposta de query para close, por exemplo.
Só existe um terminal. Erro numérico termina o mesmo pedido sem payload de
sucesso; o runtime correlaciona o id pendente e chama a finalização de falha
com o contexto armazenado. Nunca repassa texto livre de erro ao guardião/UI.

| `payload.kind` | Operação | Dados e regra |
| --- | --- | --- |
| `catalogue` | introspect | `schemas` relacionais **ou** `collections` Mongo existentes; não converte campos/documentos em colunas SQL |
| `rows` | query, preview | `columns` e `rows: Vec<Vec<Option<String>>>`; NULL permanece `null`, vazio permanece texto vazio |
| `previewReady` | preview | `previewId`, `executedSql`, `expiresInSeconds`, afetados e corte; amostra já veio nos chunks de rows |

Colunas são idênticas em todos os chunks de uma operação; cada linha tem
exatamente essa largura. Um stream de catálogo conserva sua forma relacional
ou Mongo. Catálogo externo não carrega instruções `statements`/`readSql`.
Mongo conserva declarado/inferido e valida presença finita no intervalo
[0,1]. Objetos, colunas/campos, nomes e tipos entram nos respectivos budgets.
Tipos de objeto aceitos: tabela `table`/`view`, coleção
`collection`/`view`/`timeseries`; outro valor recusa o chunk.
Diagnósticos livres `MongoCollection.truncated`/`SqlStatementImpact.note`
ficam vazios/ausentes nesta API. Corte é o booleano terminal; contagem ausente
continua impacto desconhecido, com explicação pública gerada no core.

`previewReady` ocorre uma vez, após a amostra. Recusar rows ou segundo ready
após ele. O prazo anunciado deve ser de 1 a 60 segundos; `executedSql` é
dado limitado para comparação/apresentação, não comando adicional a executar.
A autorização e o aceite da decisão pertencem ao consumidor futuro do core;
o guardião confere o stream e não é outro autorizador. Esses efeitos não
foram provados nesta fatia. No runtime futuro, a conexão/transação permanece
no adaptador até o terminal original da prévia. Aceitar `decide` não afirma
commit; somente o desfecho terminal da
prévia o faz. `cancel` também não substitui o terminal da operação alvo.
Uma amostra não pode ter mais linhas que o total afetado; sem corte,
esses valores devem ser iguais. `executedSql` vazio não anuncia prévia pronta.
Prévia que não ficou pronta não pode anunciar committed/rolledBack/expired;
failed/cancelled/unknown continuam possíveis e não autorizam repetição.
`close`/`shutdown` são confirmações de sucesso do adaptador; D1b deve ainda
provar liberação/coleta, inclusive quando o transporte morre sem resposta.

## 4. Guardião e limites

`StreamGuard::new(context, operation, limits)` rejeita contexto inválido e
limites acima dos tetos locais. Decisão e cancelamento exigem `for_request`,
que conserva os alvos exatos do pedido; query/prévia também aplicam seu
`maxRows` quando esse construtor é usado. `accept_chunk`/`accept_terminal` conferem
contexto exato, sequência, forma e budgets antes de atualizar estado. As
versões `*_bytes` recebem o payload `Chunk`/`Terminal`, não o envelope:
D1b limita o frame externo inteiro antes de extrair `params`/`result`.
Elas conferem o tamanho do payload antes de desserializar;
retornam o objeto validado ao futuro consumidor. O guardião conserva somente
estatísticas, colunas e identidade da prévia, sem reter linhas/catálogo.
Erro não avança sequência, budget ou estado; o runtime encerrará a operação
incompatível em vez de continuar seu stream. `finish_failure` aplica contexto
exato e terminal único à falha do pedido correlacionado. O erro bruto é
descartado e seu texto não entra no budget retido. Identidades únicas e
merge do snapshot de catálogo pertencem ao futuro consumidor core; o
guardião puro não prova essas responsabilidades.

Tetos negociados do [39](39-drivers-externos-e-compatibilidade.md): mensagem
1 MiB, 10.000 linhas, 128 colunas, célula 16 KiB, retenção acumulada 8 MiB,
5.000 itens de catálogo. Retenção soma o tamanho JSON de todas as mensagens
aceitas, inclusive metadados e terminal; é conservadora e inclui repetições
de cabeçalho. O caminho bytes usa bytes reais, inclusive campos adicionais;
o caminho tipado mede serialização sem construir outro buffer integral.
Contagens e bytes são cumulativos, não limites renovados por chunk.

Uma célula acima do teto recusa a mensagem inteira. Não cortar texto e
publicá-lo como valor íntegro; esta versão só permite corte do conjunto de
linhas/samples, sinalizado por `truncated`. Prévia e consultas não usam o
catálogo como amostra ilimitada. Nomes/textos/tipos também recebem limites,
e statements de impacto contam no budget de itens. Não há execução de SQL,
parse de dialeto, IPC novo, migração ou persistência nesta fatia.

## 5. Aceite da integração — 2026-10-07

40.7 §7.237: um agente adicional autorizado desenvolveu esta fatia em
worktree separado; o principal revisou e integrou após `b29c549`. Sete
testes operacionais do protocolo e 15 do core passaram na árvore principal.
Regressão real de Context como array posicional falhou antes da correção;
desserialização comum exige objeto para os tipos operacionais novos.

Gate completo/estrito em continuação verde: 1056 testes Rust, Clippy com
todos os targets/features, cargo-deny, arquitetura, documentos, ferramentas
reais, sete CTest e 132 harnesses por Qt 6.10.2/6.4.2. Fontes/configuração
C++ idênticos à prova D1 permitiram reutilizar sua análise estática; demais
etapas executadas. Debug/release em 351/352 ms, 33 superfícies sem avisos
cada; terminal 32 ms. Não houve alteração nas fontes UI/C++/QML.

Estas provas cobrem tipos e validação pura. Runtime, autorização da decisão,
coleta de processos e execução/rollback externos ainda exigem integração
e provas próprias. Perfil extensível/migração segue em D1a.4, antes de D1b.
