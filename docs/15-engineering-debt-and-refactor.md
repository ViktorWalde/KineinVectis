# 15 — Dívida Técnica, Modularidade e Refatoração Pós-V1

## Objetivo

Este documento registra riscos de engenharia já visíveis no Kinein Vectis e
define uma obrigação explícita: depois que a V1.0 estiver funcional, o projeto
deve passar por uma fase cirúrgica de polimento, refatoração, organização e
enxugamento.

A prioridade até a V1.0 continua sendo entregar uma IDE usável. Porém, o
projeto não deve aceitar crescimento desorganizado como custo permanente.

## Diagnóstico atual

O projeto tem uma direção arquitetural correta:

- UI Qt/QML separada do core Rust.
- IPC local por JSON-RPC.
- UI proibida de chamar ferramentas externas diretamente.
- Core responsável por orquestrar ferramentas maduras.
- Strict mode documentado.
- Testes Rust cobrindo partes relevantes do core.

Mesmo assim, há sinais claros de dívida técnica estrutural.

## Monólito modular

O core já começa a se comportar como um "monólito modular": existe separação em
arquivos, mas parte importante da coordenação ainda está concentrada em poucos
módulos grandes.

Medição observada em 2026-07-04:

```text
crates/kinein-core/src/lib.rs       2967 linhas
crates/kinein-core/src/lsp.rs       1726 linhas
crates/kinein-protocol/src/lib.rs   1162 linhas
crates/kinein-core/src/fsops.rs      988 linhas
crates/kinein-core/src/workspace.rs  718 linhas
crates/kinein-cli/src/main.rs        597 linhas
```

Isso ainda é aceitável para MVP, mas não é uma forma saudável para o projeto
crescer até Java, Python, Git, debug, IA, embedded, services, refactoring e
quality center.

> **Atualização 2026-07-05 — feito.** Todos esses módulos-monólito foram
> quebrados por responsabilidade, um commit atômico por arquivo, com o gate
> completo (test + clippy estrito + fmt) verde entre cada passo e a superfície
> pública preservada:
>
> - `lib.rs` do core: 2967 → 317 linhas (dispatch central) + `handlers/`.
> - `lsp.rs` (1726) → pasta `lsp/` (`types`, `manager`, `server`, `framing`,
>   `parse`, `edit`, `uri`).
> - `protocol/lib.rs` (1162) → módulos por domínio (`rpc`, `command`, `core`,
>   `tools`, `workspace`, `fs`, `run`, `terminal`, `lsp`, `build`) re-exportados
>   flat.
> - `fsops.rs` (988) → pasta `fsops/` (`error`, `confine`, `ops`, `search`,
>   `find`).
> - `workspace.rs` (718) → pasta `workspace/` (`error`, `detect`, `open`,
>   `create`).
> - `cli/main.rs` (597) → lib target `kinein_cli` (`commands`, `error`) +
>   binário fino.
> - `tests.rs` (955) → pasta `tests/` por domínio.

O risco principal não é "ter arquivos grandes". O risco é os arquivos grandes
virarem pontos onde tudo sabe demais sobre tudo:

- roteamento JSON-RPC;
- estado global da sessão;
- lifecycle de processos;
- adaptação de ferramentas;
- conversão de protocolo;
- lógica de domínio;
- emissão de eventos;
- tratamento de erro;
- testes de várias responsabilidades no mesmo lugar.

## UI concentrada

A UI também apresenta concentração visível em `CoreClient` e em `Main.qml`.
Isso é compreensível no MVP, mas tende a degradar rápido conforme novas abas,
painéis e fluxos forem adicionados.

Riscos:

- `CoreClient` virar um "God object" do lado Qt;
- `Main.qml` acumular estado de editor, problemas, busca, run, terminal,
  workspace, comandos e navegação;
- sinais Qt crescerem sem agrupamento por domínio;
- contrato visual ficar difícil de testar e manter.

Direção desejada:

- clientes/serviços QML por domínio quando fizer sentido;
- componentes QML menores para painéis e fluxos;
- separação explícita entre estado visual e chamadas IPC;
- manter toda regra de negócio no core.

## Gates de qualidade insuficientemente encapsulados

O projeto já tem comandos rigorosos, mas a validação ainda depende de copiar uma
sequência manual de comandos. Isso permitiu um caso concreto: `scripts/verificar-cpp.sh`
falhou em `clang-format`, mas os comandos seguintes continuaram porque não
estavam ligados por `&&` nem rodavam dentro de um script com `set -e`.

Riscos:

- falsa sensação de que "tudo passou";
- release build ser gerado depois de uma falha de lint;
- regressões pequenas entrarem por descuido operacional;
- diferença entre validação local, validação de agente e futura CI.

Direção desejada:

- criar um comando único oficial de verificação completa;
- parar no primeiro erro;
- separar comandos rápidos de comandos completos;
- documentar exatamente o que bloqueia merge/release;
- futuramente espelhar o mesmo fluxo em CI.

## Crescimento físico do projeto

Medição observada em 2026-07-04:

```text
build/   86M
target/ 876M
docs/   448K
crates/ 416K
ui/     2.2M
```

O tamanho total alto vem principalmente de artefatos de build (`target/` e
`build/`), não do código-fonte. Isso é normal em Rust + CMake/Qt, mas precisa
ser tratado explicitamente:

- não confundir tamanho do workspace local com tamanho real do código;
- manter artefatos ignorados pelo Git;
- documentar comandos de limpeza;
- evitar gerar assets pesados sem motivo;
- impedir que documentação ou imagens cresçam sem curadoria.

## Documentação excessiva e dispersa

O projeto tem documentação rica, mas há risco de excesso. Em 2026-07-04, há 39
arquivos `.md` dentro de `docs/`, com cerca de 19.575 linhas somadas.

Isso cria dois problemas:

1. Agentes e humanos podem gastar tempo lendo material histórico que já não é a
   fonte da verdade.
2. Decisões importantes podem ficar duplicadas ou divergentes entre docs
   numerados, planning, quality, subsystems e `ContextoIA.md`.

A ordem de precedência em `docs/README.md` ajuda, mas não resolve tudo.

Direção desejada para pós-V1:

- enxugar docs históricos;
- mover material antigo para arquivo/arquivo histórico quando necessário;
- manter poucos documentos canônicos;
- preferir docs curtos, atualizados e diretamente ligados ao código;
- manter `ContextoIA.md` como estado operacional, não como depósito infinito.

## Critérios para a refatoração pós-V1

Depois que a V1.0 estiver funcional, deve haver uma fase dedicada de polimento
e organização, sem adicionar grandes features novas em paralelo.

Essa fase deve:

- reduzir arquivos centrais grandes;
- separar roteamento, handlers, serviços e adapters;
- separar protocolo por domínio se o arquivo único ficar caro de manter;
- organizar a UI por componentes e domínios;
- transformar a verificação completa em um gate único;
- remover documentação duplicada ou vencida;
- revisar nomes, boundaries e responsabilidades;
- manter comportamento existente coberto por testes antes de mover código;
- evitar refatoração estética sem ganho claro.
- preparar o terreno para modos de compilador e loja de funções pós-V1 sem
  acoplar essa visão ao MVP.

## Possível divisão futura do core

Uma direção plausível para o core, sem compromisso imediato:

```text
crates/kinein-core/src/
├── lib.rs                  # superfície pública mínima
├── app/                    # estado e loop principal da aplicação
├── rpc/                    # parsing, roteamento e responses JSON-RPC
├── workspace/              # abertura, metadata e templates
├── fs/                     # operações confinadas ao workspace
├── build/                  # build e quality runners
├── test_runner/            # execução e parsing de testes
├── run/                    # processos de usuário
├── terminal/               # PTY/shell
├── lsp/                    # manager, framing, requests, parsing
├── tools/                  # detecção e registry de ferramentas
└── events/                 # tipos internos de eventos, se necessário
```

Essa divisão só deve ser feita quando houver testes suficientes e benefício
claro. A meta não é criar pastas por estética, mas reduzir acoplamento real.

> **Estado 2026-07-05.** Parcialmente realizada. `lib.rs` (dispatch mínimo),
> `rpc.rs`, `runtime.rs`, `commands.rs` e `handlers/` já existem; `lsp/`,
> `fsops/` (operações confinadas) e `workspace/` já são pastas. Os runners
> menores (`build.rs`, `test.rs`, `run.rs`, `terminal.rs`, `process.rs`,
> `tools.rs`) continuam como arquivos únicos por ainda não justificarem uma
> pasta — foram deixados assim de propósito, não por esquecimento.

## Regras para evitar dívida técnica antes da V1

Até a V1.0, cada nova feature deve respeitar estas regras:

- não colocar lógica de negócio na UI;
- não chamar ferramenta externa pela UI;
- não aumentar `CoreClient` sem avaliar se o fluxo merece separação;
- não aumentar `kinein-core/src/lib.rs` com lógica que pertence a serviço;
- não duplicar parsing de saída de ferramenta se já houver helper;
- não criar novo documento longo sem atualizar o índice e a precedência;
- não aceitar validação manual ambígua como sinal de qualidade.

## Decisão

Este projeto pode crescer muito, então precisa tratar organização como requisito
de produto, não como luxo.

A V1.0 deve priorizar funcionalidade real. Depois dela, haverá uma etapa
explícita de "polimento de engenharia": refatorar, modularizar, enxugar docs e
fechar gates de qualidade para impedir que a dívida técnica fique absurda.
