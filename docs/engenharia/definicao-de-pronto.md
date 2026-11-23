# Definição de pronto

Dono: quando uma mudança no core e quando um marco podem ser chamados de
prontos. "Pronto" sem todos os itens não é pronto: é "parcial", com a lista
do que falta.

## Toda mudança

1. Zero aviso no preset `dev` (GCC) e no `lint` (Clang).
2. clang-tidy limpo no `lint`.
3. Testes verdes nos presets `dev`, `asan-ubsan` e `tsan`
   (`cmake --workflow --preset <nome>`, da pasta `core/`).
4. Teste novo para o comportamento novo, e mutante para cada guarda nova.
5. Desempenho medido antes e depois nos quatro tamanhos, quando a mudança toca
   um caminho que depende do tamanho do arquivo ou do projeto.
6. O documento dono do assunto atualizado no mesmo commit, quando a mudança
   muda um fato dele. Thread nova entra na tabela da
   [execução](../arquitetura/execucao-e-concorrencia.md#threads-e-donos).
7. **A lista explícita do que não foi verificado**: ambiente, plataforma ou
   caso que a mudança não exercitou.
8. Commit só quando o Viktor pede.

## Todo marco

- Os critérios de pronto do marco são escritos **antes** do código, com
  número, método e o teste que impõe cada um (A09 das
  [lições](../licoes-da-0.3.md)).
- O marco fecha com a medida de cada critério, ou com NÃO PROVADO explícito.
- Defeito de perda de dados e de segurança achado no caminho fura a fila.

## Critérios por marco

Ainda não definidos. A proposta da ordem dos marcos e dos critérios de cada um
é a Fase D ([decisões](../decisoes-pendentes.md)).
