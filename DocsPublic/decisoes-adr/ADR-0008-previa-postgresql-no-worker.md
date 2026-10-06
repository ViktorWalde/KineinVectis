# ADR-0008 — Prévia PostgreSQL com transação pertencendo ao worker

> **Status:** implementada no checkout; aceite da fatia pendente no
> [40.7 §7.223](../roadmaps/40.7-registro-das-entregas.md).
> **Data:** 2026-10-06. Protocolo `0.159.0`.

## Contexto

O autor pediu transação com prévia no [59 §5.3](../roadmaps/59-fechamento-da-0.3.9.md).
Uma contagem antes da escrita não mostra o resultado real nem conserva uma
transação para COMMIT/ROLLBACK. A interface síncrona `postgres::simple_query`
coleta um Vec; não existe `simple_query_iter` no cliente instalado, apesar
do comentário que o menciona. Manter a conexão no despacho bloquearia IPC.

## Decisão

Um job é dono da conexão, credencial e transação PostgreSQL. Usa a API
de streaming `tokio-postgres::Client::simple_query_raw` pela conexão de
`Transaction::client()`, com um runtime Tokio de uma thread durante o job.
O registro do Core conserva apenas contexto público, reserva e canal de
decisão consumível uma vez. O despacho continua sem runtime obrigatório.

Declarar diretamente `tokio-postgres` 0.7.18, `tokio` 1.53.1 e `futures-util`
0.3.34, já resolvidos no Cargo.lock. A alteração do lock só acrescenta os
três nomes às dependências de kinein-core. Licenças são MIT ou MIT/Apache-2.0;
checksums e escopo constam no registro de componentes abertos. Não há pool
permanente, processo auxiliar, download, shell ou telemetria.

O primeiro recorte aceita uma instrução direta INSERT/UPDATE/DELETE,
validada pelo léxico comum. RETURNING explícito é preservado; na ausência,
compor RETURNING * antes de comentários finais. A amostra não limita a
escrita. Aviso, produção, somente leitura e contexto antecedem senha/job.

Até quatro prévias vivas, uma por projeto/conexão, com decisão em 60 s.
Troca de contexto/cancelamento descarta somente decisão ainda pendente.
COMMIT aceito não é revogável: sucesso exige resposta do servidor; perda
de resposta durante COMMIT informa resultado desconhecido. SQLSTATE de
FATAL não basta para afirmar recusa; erro normal ERROR de COMMIT pode
indicar falha conhecida. TLS obrigatório recusa fallback sem cifra.

## Alternativas e limites

Uma conexão global da sessão exigiria outro ciclo de vida e serializaria
operações do projeto. Reexecutar a escrita depois da amostra seria outra
operação, com risco de resultado diferente. O job com canal mantém o dono
da transação explícito e integra cancelamento já existente.

Não é parser SQL universal nem sandbox. Sequências e efeitos externos de
funções/triggers não são revertidos. Frames do driver são alocados antes
da validação; o teto de retenção não limita absolutamente todos os bytes
na rede. Consulta ordinária síncrona ainda precisa de revisão própria.
Donos, diagrama, orçamentos e provas em [arquitetura/37 §9](../arquitetura/37-banco-de-dados.md).

## Reversão

Retirar os tipos/eventos/controllers e o executor/registro da prévia,
preservando o caminho ordinário e a correção TLS. Nenhuma credencial ou
transação foi persistida. Clientes anteriores mantêm `preview` falso
quando ausente. Reavaliar as três declarações diretas de dependências
apenas se nenhum outro executor passar a usá-las.
