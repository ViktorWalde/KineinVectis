# Arquitetura de Software e Convenções de Crescimento

> **Status:** ativo / contrato de engenharia. **Ler antes de escrever código novo.**
> **Função:** codificar como o código é organizado e como crescer sem precisar de
> outra refatoração massiva. A visão-alvo completa (produto/serviços) está em
> `docs/specs/KINEIN_VECTIS_INTERNAL_ARCHITECTURE_CORE_IPC_JOBS.md`; este documento
> é a ponte entre o que **já existe** e como **evoluir** até lá.

## 1. Por que este documento existe

Em 2026-07 o core, o protocolo e o CLI foram quebrados de arquivos-monólito
(`lib.rs` 2967 linhas, `lsp.rs` 1726, `protocol/lib.rs` 1162, …) para módulos por
responsabilidade. Foi uma refatoração grande que **não deveria ter sido
necessária**. Este documento existe para que o crescimento futuro já nasça
modular. A regra:

> Cada peça de código novo entra na camada certa, no módulo certo, com a
> visibilidade certa, e vira pasta/crate **antes** de virar monólito.

## 2. Arquitetura em camadas (regra inviolável)

```text
┌─────────────────────────────────────────────┐
│ UI Qt/QML  — apresenta, interage, exibe estado │
└───────────────────────┬─────────────────────┘
                        │ JSON-RPC local (stdin/stdout) + eventos
┌───────────────────────▼─────────────────────┐
│ Rust Core  — roteamento, validação, estado    │
└───────────────────────┬─────────────────────┘
                        │ API interna de serviços
┌───────────────────────▼─────────────────────┐
│ Services   — workspace, fs, lsp, build, run… │
└───────────────────────┬─────────────────────┘
                        │ execução como jobs
┌───────────────────────▼─────────────────────┐
│ External Tools — cargo, cmake, clangd, fd…   │
└─────────────────────────────────────────────┘
```

> A UI apresenta, o Core decide, os Services executam, os Jobs registram, os
> Events notificam.

Proibições que sustentam a arquitetura (não negociáveis):

- a UI **nunca** chama ferramenta externa (cargo/cmake/clangd/fd) diretamente;
- a UI **nunca** manipula CMakePresets/Cargo.toml/arquivos sem passar pelo Core;
- a UI **não** contém regra de negócio nem parsing de saída de ferramenta;
- todo dado entre UI e Core passa por **tipos versionados** do `kinein-protocol`
  (nada de JSON solto montado à mão).

## 3. Mapa de crates atual

| Crate | Responsabilidade |
| --- | --- |
| `kinein-protocol` | contratos UI↔Core: requests, responses, eventos, erros. Um módulo por domínio, re-exportado flat (`kinein_protocol::TipoX`). |
| `kinein-core` | dispatch JSON-RPC, estado da sessão e os serviços de domínio. É o cérebro; é o mais testável. |
| `kinein-config` | modelo de configuração strict-by-default. |
| `kinein-cli` | lib (`kinein_cli`) que gera requests JSON-RPC + binário fino. |
| `ui/` (C++/Qt) | frontend Qt/QML; sobe o `kinein-core` como processo filho via `CoreClient`. |

Esta é a base mínima. A visão-alvo (specs) prevê ~18 crates; a Seção 6 descreve
como chegar lá **sem big-bang**.

## 4. Organização interna do `kinein-core` (o que impede o monólito)

```text
kinein-core/src/
├── lib.rs          # SÓ: struct Core, dispatch, RequestOutcome, erro do loop
├── runtime.rs      # loop stdio JSON-RPC
├── rpc.rs          # helpers de response/erro/parse de params
├── commands.rs     # descriptors de command.list
├── handlers/       # roteadores por domínio (impl Core) — finos: parse + delega
├── <dominio>.rs    # lógica de domínio pequena (arquivo único)
├── <dominio>/      # lógica de domínio grande (pasta: mod.rs + submódulos)
└── tests/          # testes de integração, um arquivo por domínio
```

Regras que mantêm isso saudável:

1. **`lib.rs` é fino.** Só o dispatch central e o estado. Lógica de domínio nunca
   entra aqui.
2. **`handlers/<dominio>.rs` é fino.** Roteia `<dominio>.*`, valida params,
   delega para o serviço de domínio, formata a resposta. Sem lógica pesada.
3. **A lógica vive no módulo de domínio.** Começa como arquivo `<dominio>.rs`.
4. **Regra de split (a mais importante).** Quando um arquivo passa a **misturar
   mais de uma responsabilidade** OU cresce além de **~400–500 linhas de código
   fora dos testes**, quebre em pasta `<dominio>/` com `mod.rs` + um submódulo
   por responsabilidade. Precedente já no código:
   - `lsp/` → `types`, `manager`, `server`, `framing`, `parse`, `edit`, `uri`;
   - `fsops/` → `error`, `confine`, `ops`, `search`, `find`;
   - `workspace/` → `error`, `detect`, `open`, `create`.
   O `mod.rs` só declara submódulos, re-exporta a API pública e guarda os
   aliases/constantes compartilhadas.
5. **Visibilidade.** Dentro de uma pasta-módulo: itens internos usados entre
   submódulos irmãos usam `pub(super)` (não `pub(crate)`, que o clippy `nursery`
   rejeita como redundante; não `pub`, que o `unreachable_pub` rejeita). Só a API
   real re-exportada pelo `mod.rs` é `pub`. Em módulo público, `pub(crate)` é ok.
   Um binário que precisa de módulos internos ganha um **lib target** (ver
   `kinein-cli`).
6. **Testes.** Unitários co-localizados (`#[cfg(test)] mod tests` no próprio
   arquivo). Integração em `tests/<dominio>.rs`, com helper compartilhado no
   `tests/mod.rs`.
7. **Lints estritos são inegociáveis** (`unsafe` forbid, warnings/pedantic/nursery
   deny, sem `unwrap/expect/panic` fora de teste). Eles são parte do design.

O `kinein-protocol` segue a mesma ideia: **um módulo por domínio** (`rpc`,
`workspace`, `fs`, `lsp`, `build`, …) re-exportado flat pelo `lib.rs`. Um tipo
novo entra no módulo do seu domínio, não num arquivo gigante.

## 5. Onde colocar código novo (guia de decisão)

Ao adicionar um comando/feature, siga sempre esta ordem:

```text
1. Tipo(s) de contrato → kinein-protocol/src/<dominio>.rs (params + result + evento)
2. Roteamento          → kinein-core/src/handlers/<dominio>.rs (parse + delega)
3. Lógica              → kinein-core/src/<dominio>.rs  (ou .../<dominio>/ se já for grande)
4. Testes              → unit no módulo + integração em tests/<dominio>.rs
5. Se for operação longa → vira JOB (ver Seção 7), não handler síncrono
6. Doc                 → atualizar docs/03-ipc-protocol.md (contrato) e ContextoIA.md (estado)
```

Se o domínio ainda não existe, crie o par `handlers/<dominio>.rs` +
`<dominio>.rs`. Não pendure método novo num domínio que não é o dele.

## 6. Caminho de crescimento até a arquitetura-alvo (sem big-bang)

Os specs preveem serviços dedicados (workspace, project, toolchain, cmake, cargo,
language, build, run, debug, terminal, ai-bridge, target, settings, storage) e,
eventualmente, um crate por serviço (`kinein-cmake`, `kinein-cargo`,
`kinein-language`, …) + `apps/kinein-ui` e `apps/kinein-core-daemon`.

**Não** criar essa estrutura toda agora. A progressão é sempre incremental e
mecânica (baixo risco, como foi o rename):

```text
função  →  arquivo <dominio>.rs  →  pasta <dominio>/  →  crate kinein-<dominio>
```

- Um serviço novo **nasce como pasta-módulo** em `kinein-core/src/` (ex.:
  `src/cmake/`), já dividido por responsabilidade.
- Só vira **crate próprio** (`kinein-cmake`) quando (a) fica grande, (b) é
  independente o suficiente e (c) há ganho real (reuso, tempo de compilação,
  fronteira clara). A extração de crate é uma passada mecânica — nunca um
  big-bang.
- O nome do crate futuro já é conhecido (specs), então **nomeie a pasta-módulo
  igual** desde o início (`src/cmake/`, `src/toolchain/`, `src/language/`), para
  a extração ser trivial.

## 7. Jobs, Events e Risk — construir cedo (ponto crítico anti-refatoração)

Os specs exigem um **Job System**: toda operação longa (configure, build,
index, scan, flash, debug, geração de contexto de IA) é assíncrona, cancelável e
reporta progresso por eventos; nunca bloqueia a UI.

> **Armadilha a evitar:** se continuarmos adicionando handlers **síncronos** para
> operações longas, introduzir jobs depois será exatamente a refatoração massiva
> que este documento existe para prevenir.

Portanto:

- a abstração de **Job/Event** deve ser generalizada **antes** de adicionar os
  próximos serviços longos (cmake configure/build, cargo, debug, targets). O
  `build.run`/`test.run`/`quality.run` (que já fazem streaming via `emit`) são o
  embrião — generalizar em vez de duplicar por comando;
- todo job tem `id`, estado (`queued/running/…/success/failed/cancelled`),
  eventos e logs; job cancelável expõe cancel;
- **eventos não viram pop-up automático** — atualizam status bar, Problems, tool
  window; painel só abre se o usuário pedir;
- todo comando é **classificado por risco** (`low/medium/high/dangerous`);
  `high` exige confirmação, `dangerous` é bloqueado ou exige confirmação muito
  explícita; ação vinda de IA nunca é aplicada automaticamente;
- erros são **estruturados** (código estável + mensagem + detalhes), como já são
  hoje no protocolo.

## 8. Anti-padrões (o que causou a dívida — proibido repetir)

```text
- lib.rs / arquivo de domínio virando monólito multi-responsabilidade;
- God object no lado Qt (CoreClient / Main.qml acumulando tudo);
- UI chamando ferramenta externa ou mexendo em arquivos direto;
- regra de negócio ou parsing de saída de ferramenta na UI;
- operação longa rodando síncrona no handler (deveria ser job);
- JSON montado à mão em vez de tipo do kinein-protocol;
- método pendurado no domínio errado;
- relaxar strict mode sem registrar motivo.
```

## 9. Critérios de aceite (checklist arquitetural)

Uma mudança está arquiteturalmente saudável quando:

```text
[ ] UI não executa ferramenta externa direto; tudo passa por JSON-RPC tipado.
[ ] lib.rs e handlers/ continuam finos; a lógica ficou no serviço de domínio.
[ ] arquivo que cresceu/misturou responsabilidade virou pasta-módulo.
[ ] visibilidade correta (pub(super) interno; pub só na API re-exportada).
[ ] operação longa é job assíncrono, cancelável, com eventos.
[ ] comando classificado por risco; high/dangerous confirmam.
[ ] erro estruturado; nada de unwrap/expect/panic fora de teste.
[ ] testes unit co-localizados + integração por domínio.
[ ] contrato novo documentado em docs/03; estado em ContextoIA.md.
[ ] pasta-módulo de serviço já nomeada como o crate-alvo dos specs.
```

Referência da visão completa: `docs/specs/` (fonte de verdade do produto e da
arquitetura-alvo); estado atual e decisões vigentes: `ContextoIA.md`.
