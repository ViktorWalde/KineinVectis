# ADR-0007 — ODBC por DSN local e consentimento de sessão

> **Status:** implementado no checkout; provas e aceite no
> [40.7 §7.221](../roadmaps/40.7-registro-das-entregas.md).
> **Data:** 2026-10-06. Protocolo `0.157.0`.

## Contexto e decisão do autor

O [59 §5.7](../roadmaps/59-fechamento-da-0.3.9.md) autoriza **Outro banco
(ODBC)**: listar os DSN do unixODBC, orquestrar console, catálogo padrão e
grade, sem baixar drivers. Carregar driver é gesto explícito com aviso.
ODBC carrega código nativo de terceiro dentro do processo. Essa é uma
exceção registrada à regra geral de integrações sem carregamento de plugins.

## Decisão técnica

Usar `odbc-api = 29.1.1`, fixado exatamente no Cargo, com somente
`odbc_version_3_80`; sem `prompt`, `derive` ou unixODBC vendorizado.
O adaptador Rust chama a API segura da biblioteca. O repositório conserva
`#![forbid(unsafe_code)]`. O gerenciador nativo é o unixODBC do sistema,
ligado dinamicamente; o driver é instalado e configurado pela pessoa.

`SQLDataSources` e `SQLDrivers` enumeram o registro sem conectar. A UI
recebe só DSN, nome do driver e identidade. Não recebe os atributos
arbitrários de conexão. `SQLConnect` recebe DSN, usuário e senha em campos
separados. Não há shell, connection string, instalador nem download de driver.

A primeira operação que abriria conexão retorna `DRIVER_APPROVAL_REQUIRED`
antes do job e da resolução de senha. **Carregar driver** envia autorização
tipada. Cancelar não conecta. O consentimento fica na memória, ligado ao
projeto, perfil completo e identidade atual do driver. Trocar projeto,
remover o perfil ou alterar perfil/driver exige outro gesto. A identidade
considera caminho resolvido, tamanho e modificação da biblioteca e a
precedência `Driver64` no processo de 64 bits. É detecção de mudança,
não verificação criptográfica do conteúdo nem proteção contra alteração
externa entre conferir e conectar.

O catálogo usa `SQLTables`/`SQLColumns`. Buffer comum confere truncamento de
célula, UTF-8, linhas, colunas e memória. A leitura de tabela nasce no core
com o delimitador indicado pelo driver; a UI não acrescenta `LIMIT` de
outro dialeto. SQL desconhecido e escrita usam confirmação genérica pelo
nome da conexão, sem contagem SQL inventada.

## Licenças, custo e fronteira de confiança

| Parte | Licença e distribuição |
| --- | --- |
| `odbc-api` / `odbc-sys` / `atoi` | MIT; fontes e checksums fixados no Cargo.lock |
| Gerenciador `libodbc` do sistema | LGPL-2.1-or-later; ligação dinâmica |
| Driver escolhido pela pessoa | Licença própria; não distribuído nem baixado pela IDE |

A medição no host usou unixODBC 2.3.14. Entraram três crates no lock,
sem runtime de UI, processo auxiliar ou pool de conexão permanente.
Cada operação abre sua conexão. O gerenciador pertence ao processo;
consentimento não cria sandbox para o driver. Ele pode acessar credenciais
e dados. Tracing e logs habilitados na configuração externa do DSN ficam
fora do controle da IDE. Diagnóstico arbitrário do driver não vai à
mensagem do produto; o core retorna texto próprio e SQLSTATE.

`cargo deny` audita as crates Rust. A biblioteca nativa tem uma revisão
separada: os avisos/licença, fontes e substituição exigidos pela LGPL
precisam ser resolvidos no empacotamento, antes do AppImage final (59 §7).
Não se adicionou licença à allowlist do Cargo para esconder essa fronteira.

O mínimo **efetivo** já era Rust 1.88: let chains no código e dependências
MongoDB, ICU e time. A declaração 1.85 foi corrigida para 1.88. Avaliar
1.85.1 confirmou a incompatibilidade; o workspace inteiro, com todos os
alvos e recursos, passou no `cargo +1.88.0 check --locked` em 2026-10-06.
A declaração corrigida também exigiu ajustes mecânicos do Clippy no
código existente, sem supressão. Trocar só o adaptador ODBC não
tornaria o restante do workspace compatível. O pin de desenvolvimento
permanece em `rust-toolchain.toml`.

## Alternativas avaliadas

- **Driver próprio para cada banco:** permanece apropriado para os motores
  nativos; não cobre a extensão genérica solicitada.
- **`isql` e análise de saída de terminal:** dependeria de executável
  adicional e saída textual, sem contrato confiável para NULL e catálogo.
- **ODBC sem aviso:** violaria o gesto explícito para código nativo.
- **Instalar drivers pela IDE:** não autorizado e amplia cadeia de
  suprimento, privilégios e regras de licença por fabricante.

## Limites e verificação

A classificação ODBC aceita uma leitura simples e única; CTE, função,
sequência, SQL não reconhecido e escrita pedem confirmação. Não é parser
universal nem garantia de ausência de efeitos em views/triggers. Leitura
usa transação sem autocommit e rollback; não equivale a `READ ONLY` do
servidor. Driver sem transação ou timeout pode recusar a operação; não há
fallback que remove a proteção. O timeout solicitado não mata código
nativo que o ignora. A grade conserva o primeiro conjunto de resultados.
Qualificação de catálogo/esquema usa ponto: motores com convenção diferente
podem exigir SQL manual. A qualidade do catálogo depende do driver.

O core tem testes de recusa antes do job, autorização obsoleta e entradas
malformadas; a UI, de cancelamento e respostas atrasadas. A prova
`scripts/testar-odbc-real.py --driver /caminho/do/driver.so` usa apenas
biblioteca local, DSN/banco/projeto temporários, HOME real e XDG isolado.
Inclui nome SQL hostil, NULL, teto, célula excessiva e escrita confirmada.
`--ui` permite repetir os gestos na janela e limpa ao fechar. O aceite
integral e a limpeza ficam registrados no 40.7.

## Reversão

Remover o adaptador, seus tipos/roteamento e a dependência exata; perfis
ODBC não devem ser convertidos silenciosamente em motor nativo. A mudança
é aditiva para PostgreSQL, SQLite e MongoDB. Não existe estado de
autorização persistido a migrar ou apagar.

## Fontes primárias

- [Fonte e releases de odbc-api](https://github.com/pacman82/odbc-api/releases).
- [API da versão fixada](https://docs.rs/odbc-api/29.1.1/odbc_api/).
- [unixODBC e licenças das bibliotecas](https://www.unixodbc.org/unixODBC.html).
- Manifestos e arquivos License das crates fixadas; copyright do pacote
  `libodbc2` do sistema usado na prova.
