# Pending decisions

Owner: what is still the maintainer's decision, with the recommendation and the document that
becomes the owner once the decision is made. A decision that is made leaves this list and goes to
its owner, with the date.

State on 2026-11-23. The decisions already made are in the owner of each one, with the date, and
in the git history.

## Pending

| # | Decision | Recommendation | Owner afterwards |
| --- | --- | --- | --- |
| P01 | License of the algorithms library: LGPL-3.0, MPL-2.0 or Apache-2.0 | None: it is the maintainer's choice. Any choice amends ADR-0011 (AGPL-3.0-only, DCO with no copyright assignment). The history has a single human author | A new ADR |
| P04 | Value of RNF-PERF-01: the maximum growth from 20,000 to 40,000 lines | 2.5 times: linear, with a margin for the variation of the machine (65 §20.6) | [requirements](requirements/non-functional-requirements.md) |
| P07 | `NDEBUG` in the `production` profile | Do not define it: `assert` stays active, like the `_GLIBCXX_ASSERTIONS` the plan keeps | [profiles](engineering/build-profiles-and-hardening.md) |
| P09 | QML in the dogfooding scope: `qmlls` in Neovim | Edit QML as text in N0, and `qmlls` in N1: the UI is QML and dogfooding will need it | [requirements](requirements/functional-requirements.md) |
| P11 | Phase B: the inventory of the 89 `*Controller.qml` | Do it before touching the bridge; it does not block the core | A new file, if the maintainer asks for it |
| P12 | Phase D: the order of the first milestones and the done criteria of each one | After the approval of the [modules](architecture/modules-and-layers.md); the proposed order of the base is in the [functional requirements](requirements/functional-requirements.md#the-order-of-the-base) | [definition of done](engineering/definition-of-done.md) |
| P13 | `clang-analyzer-*` in `core/.clang-tidy`, beyond the six requested categories | Keep it: it catches use after free, leaks and null dereferences along execution paths | [clang-tidy](engineering/static-analysis-clang-tidy.md) |
| P14 | Move the UI to the modules of `core/cmake/` instead of `cmake/KineinStrictOptions.cmake` | After the first milestone of the core, in its own step, with the UI build green in the same profiles. Until then both coexist | [profiles](engineering/build-profiles-and-hardening.md) |
| P15 | The architecture of modules and layers, and the layer gate in CMake | Approve it as written in [modules and layers](architecture/modules-and-layers.md), or adjust it. The gate is written only after the approval | [modules](architecture/modules-and-layers.md) |
| P16 | The third-party dependencies of the base: JSON and msgpack | Decide before the `codec` module. JSON candidates: nlohmann/json (simple, slow), simdjson (read-only, fast), glaze (compile-time reflection). For msgpack, reuse the C++ of the proof (`origin/prova/eng-60`) | [modules](architecture/modules-and-layers.md) |
| P17 | With the 0.3 CI out of the branch (2026-11-23), two rules of 0.1 have no CI mechanism: English identifiers, and no reference to untracked local notes in `docs/` | English identifiers: review, because a text-heuristic gate on code does not enter (L15). Privacy of the documents: a project gate of 0.1, with a corpus and measured effectiveness, together with the one of P19 | [gates](engineering/gates-and-effectiveness.md) |
| P18 | The GitHub rules for the branch `reescrita/core-cpp23`, the only branch in focus (`main` stays out) | Now: actions only from GitHub and pinned by SHA, a read-only default token, approval for outside contributors, a branch that cannot be deleted or force-pushed. Requiring green checks rejects direct pushes ("commits must first be pushed to another ref where the checks pass", rulesets API documentation): only with pull requests | [gates](engineering/gates-and-effectiveness.md#the-ci) |
| P19 | Enforce rules 6 to 12 of the [README](README.md#rules-of-this-tree) by a mechanism: size, header, citation of 0.3, one question per file | Yes: a gate with a corpus and measured effectiveness, like the others, in the CI. Until then, they are conventions | [gates](engineering/gates-and-effectiveness.md) |
| P20 | T4: the same logic written another way, which no tool decides (Rice's theorem). The gate for T1 to T3 was approved and entered on 2026-11-23 | Discuss with the maintainer. Proposed defense: one owner per responsibility, open/closed (the second occurrence creates the port) and review | [gates](engineering/gates-and-effectiveness.md) |
