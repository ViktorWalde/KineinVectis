# Lessons from 0.3

Owner: what the 0.3 series got wrong, with the evidence of origin and the mechanism
that prevents the repetition in the core. Citations follow the form of the
[README](README.md#how-to-cite-03).

The fundamental mistakes share one pattern: **the rule existed as an intention, and nothing
enforced it where the effect happens.** Trust was checked in some places, not where the process
is born. Durability was a QML `Timer`. The deadline was each handler's choice. The gate read
text, not structure. That is why each lesson below ends in an "Enforced by" column, not in a
recommendation.

## Mistakes that must not come back

| ID | What happened | Origin | What the core does | Enforced by |
| --- | --- | --- | --- | --- |
| L01 | The UI owned the text: each LSP request and each `syntaxTree.update` carried the whole file, and the reply brought 1 to 8 MiB per keystroke | ENG-55; 65:67-75; 64:211-232 | The core owns the text. The UI sends each edit as a delta with its base version, and no message after the opening carries the whole document | RNF-EDI-01 ([requirements](requirements/non-functional-requirements.md)) |
| L02 | Opening a 19,970-line file took 46.2 to 47.9 s: 8.1 times more text, 48.6 times more time | INF-25; 62:1371-1394 | Every path that depends on the file size has a complexity test | RNF-PERF-01 |
| L03 | The CPU time for typing 40 keys went from 2.31 s (85 lines) to 24.75 s (19,970 lines); the cause was not found | INF-27; 62:1426-1447 | The core's work per editing event does not depend on the file size | RNF-EDI-01 |
| L04 | The draft had a debounce with no maximum delay: typing without a 1.5 s pause wrote nothing, and a crash lost everything | INF-26; 62:1395-1425; `ui/qml/editor/EditorPersistenceController.qml:119-158` | Durability is a rule of the core, with a maximum delay, not a UI timer | RNF-DUR-01 |
| L05 | A workspace opened as untrusted ran its own code through five paths | INF-23; 64:107-130 | Trust is decided in the launcher, where the process is born, including for an argument that names a program (`--query-driver`) | RNF-SEG-01 ([processes](architecture/processes-and-trust.md)) |
| L06 | Processes were created in about 122 places, with no single policy | ENG-51; 64:297 | Only the launcher creates processes | clang-tidy, measured by `gate.launcher` ([gates](engineering/gates-and-effectiveness.md)) |
| L07 | The core recorded no duration, process or queue | ENG-52; 62:1994-1998 | One structured record per request, per process and per job, opened in a single place | RNF-OBS-01 |
| L08 | A slow `git` call held the loop for 18.0 s, and `core.ping` and `job.cancel` waited 17.8 s | ENG-50; 64:193-209 | The loop never blocks; every external call runs outside it, with a deadline and cancellation | RNF-EXE-01, RNF-EXE-02 |
| L09 | A response had no correlation with its request, and a request had no deadline or cancellation | ENG-53; 62:1999-2002 | Every request has an id, a deadline and cancellation; a stale response is discarded by its generation | RNF-EXE-02 ([execution](architecture/execution-and-concurrency.md)) |
| L10 | An old load of the compilation context landed on top of the new one | INF-22; 66:56-60 | Generations per workspace and per document: the result of an old generation is never applied | Ordering test ([tests](engineering/test-strategy.md)) |
| L11 | In Rust, a `Parser::new()` on each incremental analysis, and highlighting that walked the whole tree from the root | `crates/kinein-core/src/lang/service.rs:230`, `:292` and `:340` (read; no document records it) | Incremental state kept between calls; work proportional to the change, in the tree-sitter analysis of the editor and in every algorithm of the library | Complexity test (RNF-PERF-01) |
| L12 | The contract stalled: the schema stayed at protocol 0.62.0 while the code went to 0.223.0, and nothing read it | ENG-54; 64:304; `schemas/ipc.schema.json:4` | The contract has one source, and both sides are checked against it | RNF-CON-01 ([protocol](architecture/protocol-and-contract.md)) |
| L13 | Personal state (drafts, session) written inside the workspace tree, which two of the three templates did not ignore in git | ENG-56; 64:152-156 and 64:315-318 | State and cache live outside the workspace tree, in the XDG directories | RNF-DADOS-05 ([persistence](architecture/persistence-and-durability.md)) |
| L14 | Area QML talking directly to the bridge (8 files, 99 accesses) and a 791-line facade in `EditorController.qml` | ENG-22, ENG-24; 63:620-622 | QML decides presentation and intent; facts, rules, effective paths and trust belong to the core | Boundary rule ([boundaries](architecture/boundaries-and-responsibilities.md)) |
| L15 | Text-heuristic gates: 36 of 120 planted defects detected, and 14 of 15 correct forms rejected | ENG-41; 66:46-49 and 66:96-99 | Only deterministic gates enter. A project gate enters only with measured effectiveness | [gates](engineering/gates-and-effectiveness.md) |
| L16 | The suite failed under its own load, with no planted defect | INF-24; 62:1302-1370 | Tests do not synchronize on time; a deadline is a test parameter; flakiness is measured in batches | [tests](engineering/test-strategy.md) |
| L17 | A gate went green by exercising an old binary of the tree, not the one the run built | INF-31; 62:1544-1568 | A test checks the artifact of its own run (`build.elf_hardening` reads the ELF that was built) | `core/tests/build/` |
| L18 | An `exec` failure in a terminal session aborted the child and wrote a core dump | INF-29; 62:1480-1509 | The launcher returns an `exec` failure as a typed error, with no abort | Launcher test |
| L19 | An expired deadline was reported as "stale", telling the person to repeat a request that gives the same result | INF-30; 62:1510-1543 | Each failure has its own cause: an expired deadline is `DeadlineExceeded`, a cancellation is `RequestCancelled` | Deadline test |
| L20 | Documentation and ceremony growing faster than the product: 126 thousand lines of Markdown against 166 thousand of Rust and 74 thousand of QML and C++; of the last 400 commits, 99 were documentation and 5 refactoring | 64:319-320; measured on 2026-11-22 | A new document only on request; one fact, one owner; a rule a tool enforces is not repeated in text | [README](README.md#rules-of-this-tree) |
