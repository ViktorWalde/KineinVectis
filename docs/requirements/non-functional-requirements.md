# Non-functional requirements of the C++ core

Owner: the measured limit of each quality of the core, the measurement method and what enforces
it. A requirement with nothing that enforces it does not count as met.

State: draft of 2026-11-22. No requirement is met yet. The IDs inherited from document 19
(19:114-123) keep their names, so that the lineage stays visible: `RF` and `RNF` are the 0.3
prefixes for functional and non-functional requirement. Their limits were accepted by the
maintainer in DP-03 of 64 (64:938). The requirements marked *(0.1)* do not come from 19, and
their value is a **proposal** until the decision.

## Measurement reference

- Machine: the project's measurement machine (16 cores, 21 GiB, Linux), the same as in 64 and 65.
- Sizes: C/C++ files of **85, 2,500, 20,000 and 40,000 lines**. The 0.3 baselines used Rust files
  of 85, 2,463 and 19,970 lines (65:670-673); the C/C++ fixtures still have to be generated and
  measured on the current product before the first comparison
  ([tests](../engineering/test-strategy.md#performance)).
- Every measurement states its date, binary, profile and method.

## Execution and protocol

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-EXE-01 | The core's loop does only state work; processes, the project's disk and the network never run on it | State request: p99 ≤ 10 ms, maximum ≤ 50 ms | Per-request record | A guard that fails I/O on the loop in the tests and in the debug build |
| RNF-EXE-02 | Every request has a deadline and can be cancelled | `core.ping` and the cancellation answer in ≤ 50 ms while a tool is hung | The experiment of 64 §2.3 as a test | Test with a fake tool hung on a lock, with no `sleep` |
| RNF-CON-01 | The contract has a single source | Changing a method or a field on one side only fails a test | Mutant of the contract | Contract test ([protocol](../architecture/protocol-and-contract.md)) |
| RNF-OBS-01 | Every request, process and job leaves a structured record, with no secret | 100% of requests recorded; zero secrets | Test with an in-memory collector; redaction test | The single entry point of a request |

## Editing and performance

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-EDI-01 | Editing does not cost the whole file | Per 1-character editing event in the 20,000-line file: the core's work p95 ≤ 16 ms; no message after the opening carries the whole document | Record; message sizes | Test at the reference sizes. Adapted from 19:121, which measured `syntaxTree.update`, a method that leaves |
| RNF-PERF-01 *(0.1)* | No path of the core grows faster than linearly with the size of the file or the project | From 20,000 to 40,000 lines, the time grows at most 2.5 times (**proposal**, P04) | Complexity test at the four sizes | Complexity test per path ([tests](../engineering/test-strategy.md#complexity)) |
| RNF-MEM-01 | Memory budget of the core while editing | Core RSS ≤ 60 MB after opening, indexing and editing the 20,000-line file | `/proc/<pid>/status` | Performance test with a verdict |
| RNF-ED-01 *(0.1)* | The conditions of DP-10 in the integrated product | R1 and R12 by the method of the proof, R10 with the whole document, real IME and Windows | 65 §20.3 and 65:874-879 | The phase D tests ([editing engine](../architecture/editing-engine.md)) |

## Security and privacy

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-SEG-01 | Without trust, nothing the project defines runs | Zero marks in the hostile project for `workspace.open`, for each automatic request of the UI and for the first file of each language | The vectors of 64 §2.1 and those of Neovim (65 §20.2, item 14) as a fixture | The launcher and the hostile project test, with a mutant of the guard |
| RNF-SEG-02 | Every process states where it came from | Every launch recorded with program, origin and trust decision | Launcher record | The origin is a mandatory field of the launch request |
| RNF-SEG-03 *(0.1)* | `nvim` runs no program | Zero `execve` from `nvim`; without Landlock, an untrusted project opens without the new editor | The 14 vectors of 65 §20.2, item 14 | Hostile test with a mutant that removes the Landlock rule |
| RNF-PRIV-01 | Zero telemetry; nothing leaves the machine without a gesture of the person | Zero network connections started by the core without an explicit request | Test with the network denied | RNF1 of 19 (19:66) |

## Data and durability

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-DUR-01 *(0.1)* | Typed text survives the crash of any process | At most **2 s** of continuous typing lost, whatever the length of the typing (the maintainer's decision of 2026-11-22; in the current product since the fix of INF-26 in QML) | `kill -9` on the UI, the core and `nvim` after long typing with no pause (INF-26) | Crash test with a mutant that removes the maximum delay |
| RNF-DUR-02 *(0.1)* | Writing a project file is atomic | No crash leaves a project file truncated or mixed | Crash injected between the write and the rename | Crash test on writing |
| RNF-DADOS-05 | Personal state and cache do not live in the project tree | Zero per-person files under `.kinein`; opening without trust writes nothing in the project | Listing of the tree before and after | Opening test ([persistence](../architecture/persistence-and-durability.md)) |

## Engineering quality

| ID | Requirement | Limit | Enforced by |
| --- | --- | --- | --- |
| RNF-QUA-01 *(0.1)* | Code with no warning | Zero warnings in the `dev` profile with GCC and in `lint` with Clang | `-Werror` in the development presets ([profiles](../engineering/build-profiles-and-hardening.md)) |
| RNF-QUA-02 *(0.1)* | Clean static analysis | Zero clang-tidy findings | `lint` preset ([clang-tidy](../engineering/static-analysis-clang-tidy.md)) |
| RNF-QUA-03 *(0.1)* | Tests green under the sanitizers | The whole suite passes in the `dev`, `asan-ubsan` and `tsan` profiles | `cmake --workflow` of each preset |
| RNF-QUA-04 *(0.1)* | C++23 with no extensions, and the code builds as C++26 | `cxx26-probe` green | `build.contract` test |
| RNF-ESC-01 | The core library depends neither on Qt nor on the periphery | `core/lib` builds without Qt installed | The `dev` preset does not look for Qt ([boundaries](../architecture/boundaries-and-responsibilities.md)) |
| RNF-POR-01 *(0.1)* | Linux x86_64 first; Windows is a condition of DP-10 | The tests of step 10 of 62 §3 on Windows | NOT VERIFIED: nothing of the new core has run on Windows |
