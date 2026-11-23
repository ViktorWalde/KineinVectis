# Regras de C++23

Dono: os métodos de C++ do core e o mecanismo que impõe cada um. A defesa
contra cada defeito clássico da linguagem está em
[erros-conhecidos-de-cpp-e-defesas.md](erros-conhecidos-de-cpp-e-defesas.md).
As flags estão em [perfis-de-compilacao-e-hardening.md](perfis-de-compilacao-e-hardening.md).

"Revisão" na última coluna quer dizer que nenhuma ferramenta impõe a regra:
quem revisa confere. Toda regra marcada assim é candidata a mecanismo.

## Linguagem

| Regra | Por quê | Quem impõe |
| --- | --- | --- |
| C++23 sem extensões (`CMAKE_CXX_EXTENSIONS OFF`) | Código portável entre GCC, Clang e MSVC, que o clangd entende | `build.contract` reprova `-std=gnu++` |
| C++26 só por ADR, um recurso por vez, quando GCC, clangd e MSVC o suportarem sem a etiqueta de experimental, protegido por macro de teste de recurso (`__cpp_*`) | O C++26 não foi publicado (a votação DIS de 2026-10-23 reprovou); o GCC 16 marca o suporte como experimental; o Clang oficial 23 não tem reflexão (P2996) | Revisão; a sondagem `cxx26-probe` só detecta incompatibilidade |
| Identificadores em inglês; comentários e documentação em português | ENG-37 da 0.3: 14 mil ocorrências de identificador em português viraram dívida | Revisão |

## Recursos e posse

| Regra | Por quê | Quem impõe |
| --- | --- | --- |
| RAII para todo recurso; nada de `new` e `delete` soltos | Vazamento e liberação dupla | `cppcoreguidelines-owning-memory`, `cppcoreguidelines-no-malloc`; ASan e LSan |
| `QObject` vive na árvore de pais do Qt; todo o resto em RAII; nunca os dois no mesmo objeto | Dois donos para o mesmo objeto é liberação dupla | Revisão; só existe em `core/qt` |
| `std::string_view` e `std::span` só como parâmetro, nunca guardados em membro nem devolvidos de função que recebeu temporário | Referência pendurada | `bugprone-dangling-handle`, `-Wdangling-reference` (GCC), `-Wdangling` (Clang); ASan `stack-use-after-return` |
| Regra do zero; se uma operação especial é declarada, as cinco são | Cópia rasa e liberação dupla | `cppcoreguidelines-special-member-functions` |

## Erros

| Regra | Por quê | Quem impõe |
| --- | --- | --- |
| `std::expected` para erro esperado (arquivo ausente, ferramenta que falhou, prazo vencido) | O erro faz parte do tipo e não é esquecido | Resultado descartado reprova: `bugprone-unused-return-value` com `std::expected` (`core/.clang-tidy`) |
| Exceção só para o que não é esperado, e nunca atravessa thread nem o laço de eventos do Qt | Exceção que escapa de thread chama `std::terminate`; pelo laço do Qt, o comportamento não é definido | `bugprone-exception-escape` em função `noexcept`; corpo de thread e slot do adaptador são `noexcept` |
| Cada causa de falha tem nome próprio (`DeadlineExceeded`, `RequestCancelled`, `WorkspaceUntrusted`...) | L19: o prazo vencido saía como "obsoleto" | Teste de cada caminho de falha |

## Concorrência e execução

As invariantes E1 a E10 estão em
[execucao-e-concorrencia.md](../arquitetura/execucao-e-concorrencia.md). Em
termos de código:

| Regra | Quem impõe |
| --- | --- |
| `std::jthread` com `std::stop_token` para trabalho cancelável | Revisão |
| Um dono por dado; threads trocam mensagens | TSan; revisão |
| Variável global não constante é proibida | `cppcoreguidelines-avoid-non-const-global-variables`; `-Wglobal-constructors` e `-Wexit-time-destructors` em `core/lib` (Clang) |
| Função da biblioteca C que não é segura em thread é proibida | `concurrency-mt-unsafe` |
| O laço nunca bloqueia; toda chamada externa tem prazo e cancelamento | Guarda de E/S no laço (a projetar) |

## Fronteiras

| Regra | Quem impõe |
| --- | --- |
| `core/lib` sem Qt, testável sem laço de eventos | O preset `dev` não procura o Qt |
| O moc nunca vê `core/lib` | AUTOMOC só nos alvos de `core/qt` |
| Processo só pelo lançador | clang-tidy (`CustomFunctions`), ativo e medido ([processos](../arquitetura/processos-e-confianca.md)) |
| Dependência entre módulos só para camada de baixo | `kinein_add_module`, proposto ([módulos](../arquitetura/modulos-e-camadas.md)) |
| Nada executa antes da confiança no workspace | Teste do projeto hostil (RNF-SEG-01) |
| Dependência externa nova só com aprovação do Viktor, com licença conferida | Revisão |

## Supressões

- `NOLINT` só na forma `NOLINT(<checagem>): <motivo>`, na linha, nunca em bloco.
- Aviso do compilador não se desliga por arquivo; se uma flag não serve, ela
  sai do perfil, com o motivo no `core/cmake/`.
- O `-Werror` dos presets de desenvolvimento não se desliga para passar um build.
