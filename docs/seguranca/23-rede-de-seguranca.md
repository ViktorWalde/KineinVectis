# 23 — Rede de segurança contra perda de dado (SQLite + escrita atômica)

> **Status vivo desta fatia.** Registra EXPLICITAMENTE os problemas
> encontrados/a encontrar, qual está EM ANDAMENTO, e o plano. Atualizar a
> cada passo. Design canônico resumido aqui (a fatia é grande demais para
> caber só no docs-privada/diario/18; docs-privada/diario/18 aponta para cá).

## Por quê

O maior bloqueador do "usar sem se preocupar" (ver ContextoIA e docs-privada/diario/18) é
o medo de **perda de dado**. Antes do dogfooding, construir a rede de
segurança. Decisão do usuário (2026-07-11): persistência local com
**SQLite** ("escala exponencialmente, o SQLite segura bem").

## Problemas (registro explícito)

| # | Problema | Onde | Status |
|---|---|---|---|
| **P1** | `fs.write` NÃO é atômico: `fs::write` trunca o arquivo e só então escreve — crash/kill no meio **zera o arquivo em disco**. É o vetor do bug histórico "arquivo zerado". | `crates/kinein-core/src/fsops/ops.rs:166` | **✅ FEITO** |
| **P2** | Buffer não salvo vive só na RAM da UI. A M4.3-A cobre crash do CORE (a UI segura as abas), mas se a **UI** cair (ou power loss), as edições não salvas somem. | UI-only buffer | **✅ FEITO** |
| **P3** | (histórico) 3 bugs de perda de dado da refatoração abandonada de lifecycle — JÁ RESOLVIDOS em 2026-07-11. Esta fatia é a rede contra a **classe** deles. | ContextoIA "ponte" | ✅ RESOLVIDO |
| **P4** | `session.json`/`settings.json` reescrevem o blob JSON inteiro a cada save (não atômico via mesmo mecanismo, não escala). | `workspace/*.rs`, `settings.rs` | 🟡 RADAR (migrar pro mesmo SQLite depois) |

## Decisões de design (dois pilares)

### Pilar 1 — Escrita atômica em disco (mata P1) — FEITO

- `write_file` grava num **temp no MESMO diretório**, faz `fsync`, e
  `rename` por cima do alvo. `rename` no mesmo filesystem é atômico: um
  crash no meio deixa o arquivo ORIGINAL intacto (nunca truncado). Sem
  dependência externa (temp manual: `.<nome>.kinein-tmp-<pid>-<seq>`).
- Vale para todo save (`fs.write`); `create_file` continua como está
  (criar do zero não tem conteúdo a perder).

### Pilar 2 — Autosave de rascunhos em SQLite (mata P2) — EM ANDAMENTO

- **Crate:** `rusqlite` com feature `bundled` (SQLite vendorizado, sem dep
  de sistema, reprodutível). É o padrão maduro do ecossistema Rust. O FFI
  C fica na DEPENDÊNCIA — `kinein-core` segue `#![forbid(unsafe_code)]`.
- **Onde:** módulo `crates/kinein-core/src/db/` (conexão + migrações via
  `PRAGMA user_version` + `DraftStore`). DB por-workspace em
  `.kinein/kinein.db` (já no `.gitignore`). `PRAGMA journal_mode=WAL` +
  `synchronous=NORMAL` (durável contra crash de app; escrita atômica por
  transação). DB corrompido → recria (padrão inválido→default do storage);
  nunca derruba o core.
- **Schema v1:** `drafts(path TEXT PRIMARY KEY, content TEXT NOT NULL,
  saved_at INTEGER NOT NULL)`. Só rascunhos de buffers SUJOS (não salvos).
- **Contrato (protocolo 0.40.0):**
  - `draft.save { path, content }` → `{ savedAt }` — autosave de um buffer
    sujo. `path` confinado à raiz (como `fs.read`); chave = path absoluto.
  - `draft.clear { path }` → `{ ok }` — remove o rascunho (save/close
    limpo). `fs.write` **também** limpa o rascunho do path salvo (core-side).
  - Recuperação: embutida na resposta de `workspace.open` como
    `drafts: [{ path, content, savedAt }]`, filtrando os que DIFEREM do
    disco (rascunho == disco → obsoleto, é apagado). A UI, após o session
    restore, sobrepõe o conteúdo do rascunho na aba (marca modified) e
    mostra aviso discreto "N arquivo(s) recuperado(s)".
- **Cadência da UI:** debounce ~1.5s após edição (novo `autosaveDebounce`
  no `EditorController`, espelhando o `sessionSaveDebounce`) →
  `draft.save`. Ao salvar (`fs.write` ok) o core limpa; ao fechar aba, a
  UI manda `draft.clear`. Rascunho, portanto, **só sobrevive a CRASH**
  (saída não-graciosa), que é exatamente a rede de segurança.
- **Semântica:** um crash da UI/power loss → no próximo `workspace.open`
  os buffers sujos voltam como abas modificadas. Save/close limpo não
  deixa rascunho.
- **Troca de workspace (corrigido em 2026-08-29).** A store é POR-WORKSPACE,
  então trocar o workspace ativo tem de trocar a store junto. Só o
  `workspace.open` fazia isso: `workspace.createProject` deixava o projeto novo
  gravando autosave no banco do projeto ANTERIOR (ou sem autosave nenhum, se
  não houvesse anterior), e `workspace.close` mantinha a store aberta. Hoje há
  um dono único da transição (`activate_workspace`/`deactivate_workspace` em
  `handlers/workspace.rs`), travado por `scripts/verificar-transicao-workspace.sh`.
- **Arquivo apagado não volta (corrigido em 2026-08-29).** Um rascunho só nasce
  contra arquivo existente (`fsops::confine_file`); se o arquivo sumiu do disco
  desde o autosave, o usuário o apagou ou renomeou. `recover_drafts` comparava
  conteúdo com o disco e, como ler arquivo ausente devolve `None` (que "difere"
  do rascunho), oferecia de volta o buffer de um arquivo deliberadamente
  apagado. Agora esses rascunhos são descartados.
- **O rascunho acompanha o `fs.rename` (2026-08-29).** A chave da store é o
  caminho absoluto, então renomear sem mover deixava o rascunho órfão: o caminho
  antigo não existe mais e o novo não tem autosave — e na abertura seguinte ele
  era descartado. `DraftStore::rename` usa `UPDATE OR REPLACE` porque `path` é
  PRIMARY KEY: se o destino já tiver rascunho, o do arquivo vivo é o que vale.

## Fora desta fatia (com gatilho)

- Local History completo (histórico versionado por arquivo, estilo
  JetBrains) — mesma store SQLite, fatia própria depois.
- Migrar session/settings para SQLite (P4) — depois, quando a store provar.
- `fsync` do diretório após o rename (durabilidade do rename em power
  loss) — adicionar se aparecer necessidade real; o rename já protege
  contra crash de app.

## Plano / checklist

- [x] **P1** escrita atômica em `write_file` + teste.
- [x] **P2.1** dep `rusqlite` (bundled) em `kinein-core/Cargo.toml`.
- [x] **P2.2** módulo `db/` (open + WAL + migração v1 + `DraftStore`
      save/clear/list) + testes.
- [x] **P2.3** protocolo 0.40.0: `draft.save`/`draft.clear` (reusam
      `FsWriteParams`/`FsPathParams`), `DraftSaveResult`/`DraftInfo`;
      `drafts` na resposta de `workspace.open`.
- [x] **P2.4** core: `Core.drafts` + `Core::enable_persistence`, aberta na
      transição de workspace (`activate_workspace`), handlers `draft.*`,
      `fs.write` limpa o rascunho, `recover_drafts` filtra os que diferem do
      disco e descarta os de arquivo que sumiu.
- [x] **P2.5** UI: `autosaveDebounce` (1.5s) → `draft.save`; `draft.clear`
      no `closeTab`; `draftsRecovered` → `restoreDrafts` (overlay do buffer
      na aba, marcada modificada).
- [x] **P2.6** validação: gate/clippy/qmllint/smoke verdes.
      ⚠️ **Ressalva registrada em 2026-08-29.** Este item citava uma sonda e2e
      `sonda_drafts.py` (crash com SIGKILL → recupera; save limpa; escrita
      atômica sem temp solto). **Esse arquivo nunca foi commitado** —
      `git log --diff-filter=A` não devolve nada. A validação pode ter sido
      feita com um script descartável, mas **não é reproduzível hoje**: ninguém
      consegue re-rodar a prova do pilar 2. O que é reproduzível são os testes
      de `db/mod.rs` e, desde 2026-08-29, os de transição de workspace e de
      arquivo apagado em `tests/workspace.rs` e `tests/fs.rs`.
      **Pendente:** reescrever a sonda e commitá-la, ou rebaixar este item de
      `[x]` para `[ ]`. Um `[x]` que aponta para artefato ausente é a mesma
      mentira silenciosa que o `verificar-docs.sh` existe para impedir.
- [x] sync docs: 03 (contrato `draft.*` + 0.40.0), 18 (ponteiro), MANUAL
      (nota da rede de segurança), ContextoIA (registro S1).

## Estado

**FEITO (2026-07-11), com uma ressalva de 2026-08-29:** os dois pilares estão
entregues, mas a prova e2e do pilar 2 **não é reproduzível** — ver P2.6.
Protocolo 0.40.0. O aviso discreto ("N recuperados") ficou como a própria
marca de aba MODIFICADA (sem toast — primitiva de toast segue no radar).
Próximo, conforme o usuário: **dogfooding**.
