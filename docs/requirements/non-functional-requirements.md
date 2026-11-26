# Non-functional requirements of the core

Owner: the measured limit of each quality of the core, the measurement method and what enforces
it. A requirement with nothing that enforces it does not count as met.

State: draft of 2026-11-26. No requirement is met yet. A requirement that cites the maintainer's
decision was accepted on that date; one marked **proposal** waits for the pending decision it
cites ([pending decisions](../pending-decisions.md)); every other was accepted by the maintainer
on 2026-11-14 (DP-03).

## Measurement reference

- Machine: the reference machine of the project (16 cores, 21 GiB, Fedora 45).
- Sizes: C/C++ files of **85, 2,500, 20,000 and 40,000 lines**; the fixtures still have to be
  generated ([tests](../engineering/test-strategy.md#performance)).
- Every measurement states its date, binary, profile and method.

## Execution and protocol

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-EXE-01 | The core's loop does only state work; processes, the workspace's disk and the network never run on it | State request: p99 ≤ 10 ms, maximum ≤ 50 ms | Per-request record | A guard that fails I/O on the loop in the tests and in the debug build |
| RNF-EXE-02 | Every request has a deadline and can be cancelled | `core.ping` and the cancellation answer in ≤ 50 ms while a tool is hung | Ping and cancel while a fake tool is hung | Test with a fake tool hung on a lock, with no `sleep` |
| RNF-CON-01 | The contract has a single source | Changing a method or a field on one side only fails a test | Mutant of the contract | Contract test ([protocol](../architecture/protocol-and-contract.md)) |
| RNF-OBS-01 | Every request, process and job leaves a structured record, with no secret | 100% of requests recorded; zero secrets | Test with an in-memory collector; redaction test | The single entry point of a request |

## Editing and performance

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-EDI-01 | Editing does not cost the whole file | Per 1-character editing event in the 20,000-line file: the core's work p95 ≤ 16 ms; no message after the opening carries the whole document | Record; message sizes | Test at the reference sizes |
| RNF-PERF-01 | No path of the core grows faster than linearly with the size of the file or the workspace | From 20,000 to 40,000 lines, the time grows at most 2.5 times (**proposal**, P04) | Complexity test at the four sizes | Complexity test per path ([tests](../engineering/test-strategy.md#complexity)) |
| RNF-MEM-01 | Memory budget of the core while editing | Core RSS ≤ 60 MB after opening, indexing and editing the 20,000-line file | `/proc/<pid>/status` | Performance test with a verdict |
| RNF-ED-01 | The editor is as fast and as light as its component alone | At the four reference sizes: key to frame p95 ≤ 1.25 times and worst case ≤ 2 times the component alone, opening ≤ 1.25 times, RSS ≤ 1.5 times. IME and the AT-SPI tree at least as in the component alone (the maintainer's decision of 2026-11-25) | The method of the [test strategy](../engineering/test-strategy.md#performance); the reference is `TextEdit` alone, measured on the reference machine | Performance test with a verdict; IME and accessibility tests ([editing engine](../architecture/editing-engine.md#acceptance)) |
| RNF-ED-02 | A file beyond a size limit degrades in a declared way, never in silence and never by freezing the UI | Full tier up to the code insight limit; editing only up to the opening limit; not opened above it; the values come from the measurement (the maintainer's decisions of 2026-11-23 and 2026-11-25) | Files one byte below and above each limit | Tier test ([editing engine](../architecture/editing-engine.md#size-limits)) |

## Security and privacy

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-SEG-01 | Without trust, nothing the workspace defines runs | Zero marks in the hostile project for the opening of the workspace, for each automatic request of the UI and for the first file of each language | The paths of L05 as a fixture | The launcher and the hostile project test, with a mutant of the guard |
| RNF-SEG-02 | Every process states where it came from | Every launch recorded with program, origin and trust decision | Launcher record | The origin is a mandatory field of the launch request |
| RNF-SEG-03 | The core accepts no connection | Zero listening sockets of the core process, from its start to its shutdown (the maintainer's decision of 2026-11-26) | The sockets of the core listed from `/proc` after the opening of a workspace | Test with a mutant that opens a listening socket ([threat model](../architecture/threat-model.md#the-ui-and-the-core)) |
| RNF-PRIV-01 | Zero telemetry; nothing leaves the machine without a gesture of the person | Zero network connections started by the core without an explicit request | Test with the network denied | The same test |

## Data and durability

| ID | Requirement | Limit | Method | Enforced by |
| --- | --- | --- | --- | --- |
| RNF-DUR-01 | Typed text survives the crash of any process | At most **2 s** of continuous typing lost, whatever the length of the typing (the maintainer's decision of 2026-11-22) | `kill -9` on the UI and on the core after long typing with no pause (L04) | Crash test with a mutant that removes the maximum delay |
| RNF-DUR-02 | Writing a workspace file is atomic | No crash leaves a workspace file truncated or mixed (the maintainer's decision of 2026-11-26) | Crash injected between the write and the rename | Crash test on writing |
| RNF-DUR-03 | Opening and saving a file with no edit writes the same bytes | Zero bytes changed: the line ending, the BOM and every character are kept (the maintainer's decision of 2026-11-24) | Files with LF, CRLF, a BOM, U+00A0, a character outside the BMP, a combining mark, tabs and no final line break | Round-trip test ([text encoding](../architecture/text-encoding-and-positions.md#acceptance)) |
| RNF-DUR-04 | Losing the protection of typed text is never silent | While a draft cannot be written, a notice with the cause stays on screen, and it clears at the first write that succeeds (the maintainer's decision of 2026-11-26) | A fake file system that fails each write of a draft; a full tmpfs | Test with a mutant that only writes the failure to the log ([failures](../architecture/failures-by-boundary.md#the-file-system)) |
| RNF-DUR-05 | One core writes the state of a workspace | A second opening of an open workspace is refused with `WorkspaceInUse`; after a `kill -9` of the first core, with a child process of it still running, the next opening succeeds (the maintainer's decision of 2026-11-26) | Two cores on one workspace | Test with mutants without the lock and without close-on-exec ([failures](../architecture/failures-by-boundary.md#the-file-system)) |
| RNF-DADOS-05 | Personal state and cache do not live in the workspace tree | Zero per-person files under `.kinein`; opening without trust writes nothing in the workspace | Listing of the tree before and after | Opening test ([persistence](../architecture/persistence-and-durability.md)) |

## Engineering quality

| ID | Requirement | Limit | Enforced by |
| --- | --- | --- | --- |
| RNF-QUA-01 | Code with no warning | Zero warnings in the `dev` profile with GCC and in `lint` with Clang (the maintainer's decision of 2026-11-25) | `-Werror` in the development presets ([profiles](../engineering/build-profiles-and-hardening.md)) |
| RNF-QUA-02 | Clean static analysis | Zero clang-tidy findings (the maintainer's decision of 2026-11-25) | `lint` preset ([clang-tidy](../engineering/static-analysis-clang-tidy.md)) |
| RNF-QUA-03 | Tests green under the sanitizers | The whole suite passes in the `dev`, `asan-ubsan` and `tsan` profiles (the maintainer's decision of 2026-11-25) | `cmake --workflow` of each preset |
| RNF-QUA-04 | C++23 with no extensions, in every build of the project | `-std=c++23` in every compile command, never `-std=gnu++23` or another standard (the maintainer's decision of 2026-11-26) | `build.contract` test |
| RNF-ESC-01 | The core library depends neither on Qt nor on the periphery | `core/lib` builds without Qt installed | The `dev` preset does not look for Qt ([boundaries](../architecture/boundaries-and-responsibilities.md)) |
| RNF-POR-01 | The platforms of the core | Linux x86_64; another platform enters by a decision of its own, with a CI job of its own (the maintainer's decision of 2026-11-26) | The suite of the core on each platform | The CI, on Linux x86_64 |
