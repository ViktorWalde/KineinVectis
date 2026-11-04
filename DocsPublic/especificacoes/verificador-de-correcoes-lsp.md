# Verificador complementar de correções LSP

**Escopo implementado e evolução:** este documento descreve a primeira fatia
host C/C++. O alvo comum, o aprofundamento próprio de C e a ordem C/C++ → Rust
e integração Rust/C/C++ → Python estão na
[orquestração e verificação das linguagens](orquestracao-e-verificacao-de-linguagens.md).
Suporte LSP existente de uma linguagem não implica provedor de verificação
implementado; a ampliação especializada de SDKs conserva provas próprias.

## Contrato S0/S1

O consumidor é a prévia transacional de `lsp.applyCodeAction`.
A confiança pode ser concedida nessa própria prévia pela ação existente
`workspace.trust`, identificada por `reasonCode: "untrustedWorkspace"`; exige
nova consulta e não aplica a sugestão. O LSP continua
responsável pelas sugestões e pelos ranges UTF-16; `lsp::transaction` prepara
os conteúdos e confirma a escrita somente após gesto explícito. O verificador
reutiliza `index::context`, `build::syntax_check`, o parser de diagnósticos e o
encerramento de processos existentes. Diagnósticos experimentais não substituem
os problemas do documento nem alteram configurações do projeto.

Entrada: workspace confiável, transação, caminho canônico, conteúdos antes/depois,
edits originais, versão LSP e unidade exata da CDB atual. A identidade inclui
conteúdos, geração da consulta e SHA-256 da CDB, argv e diretório efetivos.
Consultas sem mudança de texto preservam a versão LSP. Conteúdo, contexto ou
consulta substituídos invalidam ações antigas, inclusive respostas adiadas.

A prévia inclui `verification`:

| `status` | Texto apresentado | Significado |
| --- | --- | --- |
| `newError` | Erro novo observado | Há diagnóstico adicional no conteúdo editado. |
| `noNewError` | Sem erro novo observado | A comparação não observou erro adicional. |
| `notVerified` | Não verificado | `reason` explica a falta de evidência utilizável. |

A evidência contém compilador, argumentos, diretório, identidade do contexto,
versão, duração, contagens e até 32 erros adicionais. `obsolete: true` desativa
a confirmação. Ausência de erro novo não prova equivalência, link ou testes.
Nenhuma correção é aplicada automaticamente; erro novo exige confirmação
explicitamente identificada na prévia.

## Comparação e identidade

A comparação usa um multiconjunto de arquivo, mensagem e posição. Os offsets
são mapeados pelos edits UTF-16 validados pelo próprio dono LSP; um erro em texto
inserido não pode consumir um erro antigo apenas por repetir sua mensagem.
Inserir linhas ou corrigir outro trecho preserva a identidade de diagnósticos
em texto inalterado. GCC usa colunas em bytes e localização na chamada de macro.
Ranges fora do documento, sobrepostos ou entre unidades de um mesmo caractere
não recebem verificação positiva nem transação aplicável.

Os limites padrão de erros dos compiladores são removidos para esta análise.
Saída excedida ou análise interrompida não produzem `noNewError`. A aplicação
de muitos edits e a localização dos diagnósticos usam índices, evitando varrer
o documento inteiro para cada posição.

Headers são capturados por conteúdo, usando o rastreamento de includes e a
lista de dependências produzidos pelo próprio compilador. As passagens são
repetidas até que ambas usem o mesmo conjunto de cópias imutáveis. SHA-256,
metadados e namespaces de busca são conferidos novamente antes de aplicar:
preservar tamanho/data não oculta mudança de bytes; criar um header opcional
antes ausente também invalida o contexto. Datas dos arquivos-sombra são
preservadas. Passagens que atravessam uma mudança de relógio são repetidas,
sem substituir o ambiente de tempo efetivo do compilador.

## Execução e limites

A fatia aceita uma unidade `.c/.cc/.cpp/.cxx` com GCC/G++ ou Clang/Clang++ e
opções interpretadas da CDB, incluindo as flags de endurecimento do build host.
Nunca troca o compilador selecionado por outro para obter resultado positivo.
Comando ambíguo, wrapper não interpretável, resposta indireta, módulo/PCH ou
ferramenta incompatível resultam em `notVerified`.

Clang usa uma sobreposição VFS com `use-external-names: false`; GCC usa
bubblewrap com filesystem original somente para leitura, cópias montadas nos
nomes originais e área privada gravável para dependências. Includes relativos,
symlinks, nomes de arquivo e diretório da CDB são preservados. Uma sonda com
`#error` comprova a leitura do documento isolado antes da comparação. Ausência
ou indisponibilidade do isolamento GCC fica explícita; não há instalação
implícita. O formato VFS está no [LLVM](https://llvm.org/docs/doxygen/classllvm_1_1vfs_1_1RedirectingFileSystem.html),
e a opção no [Clang](https://clang.llvm.org/docs/ClangCommandLineReference.html).

A área temporária fica no cache em disco e é removida por RAII. O compilador
roda sem shell e com locale de diagnósticos fixado. Limites iniciais:

- 1 MiB por conteúdo, CDB até 16 MiB e headers capturados até 64 MiB;
- 4.096 arquivos observados e 200 mil entradas por namespace de includes;
- saída e lista de dependências até 1 MiB, com limite nos leitores dos pipes;
- orçamento total de 5 s, incluindo captura e comparação; o dono do processo
  mantém seu prazo adicional de drenagem no encerramento.

Exceder um limite, perder contexto, não encontrar ferramenta ou receber uma
edição obsoleta resulta em `notVerified`. Nova verificação, edição, mudança de
workspace e cancelamento encerram o trabalho anterior. Aplicação durante a
verificação é recusada. Invalidação de code actions preserva prévias de rename
não relacionadas. Outros buffers modificados devem ser salvos; a UI invalida
a evidência imediatamente e confere novamente o texto vivo antes de enviar
`apply`, inclusive antes do debounce de `didChange`.

## Corpus e reprodução

O corpus host está em `crates/kinein-core/src/build/edit_verification/tests/`;
o despacho e as transações em `crates/kinein-core/src/tests/edit_verification.rs`.
O harness `scripts/qml-harness/tst_edit_verification.qml` exercita a apresentação
e a invalidação do consumidor. Executar com Rust fixado, target de QA separado
e temporários em disco:

```bash
cargo test --locked -p kinein-core edit_verification -- --nocapture
```

Os casos cobrem correção válida C/C++, erro inserido, include necessário
removido, includes relativos/forçados, macros/flags, Unicode, erro antigo
deslocado, erro novo com mensagem idêntica, erro além do limite padrão,
metadados restaurados, header opcional criado, datas, contextos obsoletos,
respostas adiadas, cancelamento, timeout, ferramenta ausente e saída excessiva.
Compiladores reais são obrigatórios para o aceite host; ferramentas falsas
produzem apenas sugestões ou falhas controladas. O corpus mede duração e falso
`newError`; `scripts/check_edit_verification_mutations.py` testa a remoção da
comparação e das guardas numa cópia descartável, com baseline previamente verde.

SDKs requerem corpus e builds próprios com versões identificadas. A fatia host
não conclui S2–S4 nem o assistente completo do
[52 §11.2](../roadmaps/52-arquitetura-executavel-da-0.4.md#112-escopo-dedicado--assistente-semântico-decisão-do-projeto-2026-10-23-posição-de-2026-10-24).

**Integração C de host (2026-11-03):**
`crates/kinein-core/src/tests/c_host.rs` usa a
[fixture C11](../../scripts/fixtures/c-host/README.md), CMake/Clang/CTest reais
e sugestões LSP controladas. A CDB é produzida pelo build, com biblioteca,
executável, várias unidades e headers públicos/gerados. O corpus confere
correção válida, erro novo, invalidação após alterar configuração/header,
aplicação explícita, regeneração, falha de ligação e recuperação. Esse aceite
exercita o verificador existente sem ampliar seu escopo para edições de headers
ou múltiplos arquivos. A prova separada com clangd e GDB reais consta no
[registro §7.376](../roadmaps/40.7-registro-das-entregas.md#7376--jornada-c-de-host-e-sincronização-do-contexto-após-build-2026-11-03).
