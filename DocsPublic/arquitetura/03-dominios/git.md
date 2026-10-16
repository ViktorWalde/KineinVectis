# `git.*` — o domínio no protocolo IPC

> Parte do [03](../03-protocolo-ipc.md), que tem o formato base, as listas
> canônicas de métodos e eventos e as regras. Movido sem mudar o texto em
> 2026-10-16 ([61](../../roadmaps/61-organizacao-da-documentacao.md) §3.7, O6).

### Git (`git.status` e operações diárias)

Implementado no protocolo `0.30.0` (fatia M3.1). Orquestra o
binario `git` com saida ESTAVEL (`status --porcelain=v2 --branch
--untracked-files=all -z`); deteccao de repo por exit code
(`rev-parse --is-inside-work-tree`), nunca por mensagem (pode vir
localizada). Sincrono e stateless: a UI decide quando consultar
(open/save/fs-ops/manual).

```text
git.status {} → { repo: bool,
                  branch?, detached?, shortSha?,
                  upstream?, ahead?, behind?,
                  entries: [{ path, kind, staged }] }
kind ∈ modified|added|deleted|renamed|untracked|conflicted (o core
consolida o par XY do porcelain; a UI nunca decodifica porcelain).
```

- Paths das entries são relativos ao ROOT do workspace (convertidos do
  toplevel via `rev-parse --show-prefix`); entries fora do workspace e os
  metadados `.kinein/` são filtrados.
- Workspace sem git → `{ repo: false, entries: [] }` (nunca é erro).
  Erros reais: `NO_WORKSPACE`, `TOOL_NOT_FOUND` (git ausente),
  `INTERNAL_ERROR` (git falhou).

`git.fileDiff { path }` (protocolo `0.31.0`, fatia M3.2) → diff do
arquivo contra o HEAD, em duas granularidades na mesma resposta:

```text
git.fileDiff { path } → { path (canônico, ecoado p/ correlação),
                          repo, tracked,
                          hunks: [{ kind: added|modified|removed,
                                    startLine, lineCount }],
                          text }
```

- `hunks` vem de `--unified=0` (ranges exatos para a gutter; `removed` é
  ancorado na linha seguinte à remoção, lineCount 1); `text` vem de
  `--unified=3` (visão de diff). A UI nunca parseia diff.
- Untracked → `tracked: false` + um hunk `added` do arquivo inteiro e
  `text` vazio. `path` confinado ao workspace (`INVALID_PARAMS` fora).
- Diff é do arquivo EM DISCO (buffer não salvo não aparece — a gutter
  atualiza no save/troca de aba; attach de buffer é melhoria futura).

Mutações (protocolo `0.32.0`, fatia M3.3) — todas respondem o MESMO
shape do `git.status` (a UI atualiza tudo de uma vez):

```text
git.stage    { paths: [abs] } → GitStatusResult   (git add)
git.unstage  { paths: [abs] } → GitStatusResult   (git restore --staged)
git.discard  { paths: [abs] } → GitStatusResult   (DESTRUTIVO: restore
                                 p/ tracked, clean -f p/ untracked; a
                                 confirmação é responsabilidade da UI)
git.commit   { message, amend? } → GitStatusResult (commita SÓ o staged; `amend`
                                  reescreve o último commit — 0.126.0)
```

- Guarda do commit por exit code estável (`git diff --cached --quiet`):
  nada staged → `INVALID_REQUEST` com mensagem própria. `message` vazia
  e `paths` vazio/fora do root → `INVALID_PARAMS` (path deletado não
  canonicaliza: vale a checagem lexical dentro do root). Mutação em
  workspace sem git → `INVALID_REQUEST` (leitura responde `repo:false`).
  Falha real do git → `INTERNAL_ERROR` com o stderr.

Leitura de histórico (protocolo `0.33.0`, fatia M3.4) — todas com
formato estável (`--porcelain`, `%x1f`/NUL) e sem parsear saída
localizada:

```text
git.blame { path } → { path (ecoado), repo, tracked, groups: [
    { startLine, lineCount, sha, author, authorTime (epoch),
      summary, committed }] }
git.log { maxCount?, ref? } → { repo, entries: [       (ref: branch/tag, 0.127.0)
    { sha, shortSha, author, authorTime (epoch), summary,
      parents: [sha] (0.126.0; dois num merge), refs: ["HEAD -> main", "origin/main", "tag: v1"] }] }
git.commitDiff { sha } → { sha (ecoado), text (patch unificado) }
```

- `git.blame` usa `git blame --porcelain`; metadado de sha repetido é
  memoizado (o porcelain só o manda uma vez). Linha não commitada tem
  sha zerado → `committed:false` (detecção pelo sha, nunca pela string
  "Not Committed Yet"). Untracked/repo recém-init sem HEAD →
  `tracked:false` com `groups` vazio (não é erro). `path` confinado ao
  root e existente em disco → senão `INVALID_PARAMS`.
- `git.log` usa `--pretty=format:%H%x1f%h%x1f%an%x1f%at%x1f%s -z`;
  `maxCount` default 100, fora de `1..=500` → `INVALID_PARAMS`. Repo
  sem commits (sem HEAD, checado por `rev-parse --verify --quiet`) →
  `entries: []`. Fora de repo → `repo:false`.
- `git.commitDiff` usa `git show --pretty=format: --unified=3`; o `sha`
  é validado (4..=64 hex) ANTES de virar argv (barreira contra string
  arbitrária); merge trivial pode devolver `text` vazio. Fora de repo →
  `INVALID_REQUEST` (a chamada só nasce da lista do `git.log`).

Operações diárias adicionadas no protocolo `0.49.0`:

```text
git.branches {} → { repo, current?, detached, branches: [{ name, current }] }
git.checkout { branch } → GitStatusResult
git.branchCreate { name, checkout? } → GitStatusResult
git.pull {} → { jobId }
git.push {} → { jobId }
git.stash { action: "push|pop", message? } → GitStatusResult
```

- nomes de branch são validados antes de virar argumento; checkout/create e
  stash são recusados pela UI quando existem buffers sujos;
- pull/push são jobs canceláveis e publicam saída/resultado pelo Job System, e
  o desfecho sai em **`event.git.remoteFinished { operation, success, message }`**
  (documentado em 2026-08-29; o evento existia desde o `0.49.0` e era o único
  dos 32 eventos do core que não estava neste contrato). A UI usa `operation`
  para saber se foi `pull` ou `push` sem guardar o `jobId`;
- stash push inclui untracked, limita-se ao workspace e exclui `.kinein`;
  pop restaura o stash mais recente;
- todas as mutações síncronas devolvem o status inteiro para não duplicar
  estado derivado na UI.
