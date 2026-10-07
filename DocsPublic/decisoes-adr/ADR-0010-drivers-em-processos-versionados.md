# ADR-0010 — Drivers de banco em processos com contrato versionado

> **Status:** arquitetura decidida e revisada; implementação pendente.
> **Data:** 2026-10-07. Base `0214927`, protocolo do produto `0.163.0`.

## Contexto

Após pedir modularidade e atualização de ferramentas pelo usuário, o autor
pediu estudar a orquestração do IntelliJ IDEA Community e aprovou desenhar
e revisar a arquitetura antes de qualquer código de implementação.
O ADR-0009 separou acesso ao banco e LSP, mas deixou drivers compilados no
core e processos de acesso substituíveis como expansão eventual.
Esse limite não atende atualizar a biblioteca do driver sem atualizar a IDE.

A pesquisa distinguiu o conjunto Database Tools oficial, documentado como
driver JDBC em processo JVM, do plugin aberto Database Navigator para a
Community, com interfaces separadas e classloader próprio. Nem o driver
nem a versão do servidor substituem suporte específico de catálogo/dialeto.

## Decisão

Adotar processo adaptador de banco como fronteira alvo para PostgreSQL,
SQLite, MongoDB moderno, InfluxDB 3 e futuros MySQL/MariaDB nativos.
Cada processo reutiliza driver,
biblioteca ou API mantida e expõe uma API pequena, tipada e versionada para
os consumidores reais do serviço Banco. O core permanece dono da seleção,
política, segredo, catálogo público, jobs, leases e autorização de prévia.
A conexão/transação da prévia pertence ao processo até a decisão terminal.

O usuário escolhe instalação/caminho por projeto/conexão e pode manter versões
distintas. Negociação de API antecede segredo e conexão; depois da abertura,
recursos efetivos dependem também do servidor e das permissões. Incompatibilidade
fica no perfil/instância, sem trocar destino ou driver silenciosamente.
Versões de biblioteca/adaptador/servidor não são a versão do contrato da IDE.

LSP mantém registro, contrato e atualização independentes. Reutilizar seu
cliente atual; a UI continua com um CoreClient e um IPC. O serviço Banco
ganha uma fronteira interna de subprocesso, não outro protocolo UI/core.
Extrair uma base compartilhada de propriedade de processo/pipes/encerramento
dos mecanismos existentes, sem copiar um executor nem criar outro LSP manager.
O enquadramento por linhas JSON do adaptador é distinto do framing LSP.

Os contratos externos e de perfil são definidos antes do runtime. O erro
externo segue JSON-RPC numérico com motivo tipado, convertido para o wire UI
existente. Não persistir segredo, não executar programas por descoberta de
manifesto e não abrir ABI Rust dinâmica. A conta real do banco e suas
permissões limitam o executável; separação de processo não é sandbox.

Migração gradual: registro interno sobre os motores atuais; contratos e
preservação dos perfis; processo compartilhado/ponte; extração PostgreSQL com
impacto/prévia; SQLite e MongoDB; InfluxDB nativo. Uma operação usa um backend,
sem executar em dois nem repetir escrita após perda de resposta. Resultado
indeterminado permanece explícito. Troca/encerramento respeita escrita aceita
e a barreira por destino; sucesso exige recursos e processos encerrados.

O backend interno e suas dependências saem conforme cada substituição for
aceita. ODBC permanece no caminho aceito do ADR-0007, com consentimento e
revogação existentes. Não introduzir JDBC/JVM nem antecipar host genérico
de plugins, downloader ou gerenciador de pacotes.

## Consequências e alternativas

Atualizar a biblioteca pode exigir recompilar/instalar o adaptador, sem
recompilar o core quando a API negociada continua compatível. Suporte novo
fora dessa API ainda exige mudança de contrato/IDE; não há promessa de
compatibilidade universal com banco futuro ou legado não testado.

Há custo de manter o contrato e pequenos adaptadores, além de medir pipes,
fila, limites e encerramento. Esse custo substitui o acoplamento das versões
de driver ao core, não a manutenção dos drivers oficiais. Implementar driver
ou parser próprio e importar toda a plataforma IntelliJ foram descartados.

## Revisão e aceite

Donos e limitações conferidos no código atual. Corrigidos no desenho:
store que devolve vazio em schema futuro; executor que fecha stdin; coleta
incompleta de leitores; confusão driver/LSP; erro JSON-RPC; retry de escrita;
prévia atravessando conexões; atualização sem renegociação de capacidades.
Essas correções são plano, não produto consertado nesta entrega.

Exigir duas instalações coexistentes, retorno à anterior, protocolo
incompatível sem conexão, pipes/bytes limitados, resultado indeterminado sem
retry, prévia na mesma conexão, perfil futuro intacto, TLS/política e
desconexão real. Mensagens e orçamentos fecham na fatia de contratos antes
de implementar o runtime. Diagramas, fontes, limites e fila estão no
[39](../arquitetura/39-drivers-externos-e-compatibilidade.md).
