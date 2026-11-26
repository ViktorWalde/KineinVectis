# Pending decisions

Owner: what is still the maintainer's decision, with the recommendation and the document that
becomes the owner once the decision is made. A decision that is made leaves this list and goes to
its owner, with the date.

State on 2026-11-26. The decisions already made are in the owner of each one, with the date, and
in the git history.

## Pending

| # | Decision | Recommendation | Owner afterwards |
| --- | --- | --- | --- |
| P01 | License of the algorithms library: LGPL-3.0, MPL-2.0 or Apache-2.0 | None: it is the maintainer's choice. Any choice amends ADR-0011 (AGPL-3.0-only, DCO with no copyright assignment). The history has a single human author | A new ADR |
| P04 | Value of RNF-PERF-01: the maximum growth from 20,000 to 40,000 lines | 2.5 times: linear, with a margin for the variation of the machine (65 §20.6) | [requirements](requirements/non-functional-requirements.md) |
| P07 | `NDEBUG` in the `production` profile | Do not define it: `assert` stays active, like the `_GLIBCXX_ASSERTIONS` the plan keeps | [profiles](engineering/build-profiles-and-hardening.md) |
| P09 | QML in the dogfooding scope: `qmlls` through the LSP client of the core | Edit QML as text in N0, and `qmlls` in N1: the UI is QML and dogfooding will need it | [requirements](requirements/functional-requirements.md) |
| P11 | Phase B: the inventory of the 89 `*Controller.qml` | Do it before touching the bridge; it does not block the core | A new file, if the maintainer asks for it |
| P12 | Phase D: the order of the first milestones and the done criteria of each one | After the approval of the [modules](architecture/modules-and-layers.md); the proposed order of the base is in the [functional requirements](requirements/functional-requirements.md#the-order-of-the-base) | [definition of done](engineering/definition-of-done.md) |
| P13 | `clang-analyzer-*` in `core/.clang-tidy`, beyond the six requested categories | Keep it: it catches use after free, leaks and null dereferences along execution paths | [clang-tidy](engineering/static-analysis-clang-tidy.md) |
| P14 | Move the UI to the modules of `core/cmake/` instead of `cmake/KineinStrictOptions.cmake` | After the first milestone of the core, in its own step, with the UI build green in the same profiles. Until then both coexist | [profiles](engineering/build-profiles-and-hardening.md) |
| P16 | The third-party dependency of the base: JSON | Decide before the `codec` module. Candidates: nlohmann/json (simple, slow), simdjson (read-only, fast), glaze (compile-time reflection) | [modules](architecture/modules-and-layers.md) |
| P21 | The library of the LSP client of the core; tree-sitter with the C and C++ grammars is decided in the [editing engine](architecture/editing-engine.md) | No C++ library is broadly adopted (searched on 2026-11-23). Two candidates: kotatsu `ipc` (C++23, Apache-2.0, the LSP 3.18 model, peers with cancellation and timeouts, a CI with GCC 14, Clang 20 and the three sanitizers, a build without exceptions; first commit in 2025-12, no release, one main author with 261 of 284 commits, its own coroutine loop) and lsp-framework (C++20, MIT, no dependency, LSP 3.17 and 3.18 types; one main author with 415 of 428 commits; errors by exception, `std::future`, its own thread pool and process class). Both are evaluated when the code of the core starts (the maintainer's decision of 2026-11-25), with a measured prototype outside `core/` against the rules of `rt`, `launch` and errors; adopting the loop of kotatsu is a design question of `rt`. Own code over the JSON of P16 only if neither fits | [editing engine](architecture/editing-engine.md) |
| P22 | The editing protocol, by ADR, before the `editor` module | Deltas in the order typed, in the units of [text encoding and positions](architecture/text-encoding-and-positions.md); versions numbered by the core; a proposal applied by the UI only on its base version; a hash of the text in each acknowledgment, to detect a divergence; after a crash of the core, the UI replays the deltas the draft does not have | A new ADR |
| P24 | The contract between the UI and the C++ core, by ADR, before the `protocol` module: the framing, the methods, the single source of the shapes and the protocol version; the editing protocol (P22) is part of it | None: the design of the protocol belongs to the maintainer. What already holds is in the [protocol](architecture/protocol-and-contract.md) | A new ADR and the [protocol](architecture/protocol-and-contract.md) |
