# Static analysis: clang-tidy

Owner: what clang-tidy checks in the core, what is disabled and how it runs. The configuration is
`core/.clang-tidy`, independent of the root `.clang-tidy`, which keeps applying to `ui/`.

## How it runs

- **In the build:** the `lint` preset compiles with Clang and runs clang-tidy on each unit
  (`CMAKE_CXX_CLANG_TIDY`). Any finding fails the compilation (`WarningsAsErrors: '*'`).
- **In the editor:** clangd reads the same `core/.clang-tidy`.
- **Version:** clang-tidy 23.1.5. With the current file, 394 checks are active
  (`clang-tidy --list-checks`, 2026-11-22).

## Categories

`bugprone-`, `cert-`, `concurrency-`, `cppcoreguidelines-`, `modernize-` and `performance-`, the
categories of the plan, plus `clang-analyzer-`. The analyzer follows execution paths and catches
what a local check does not: null dereferences, use after free, leaks, use after `std::move` and
blocking inside a critical section ([known defects](known-cpp-defects-and-defenses.md)). It is
not in the requested categories; whether it stays is P13 of the
[pending decisions](../pending-decisions.md).

`readability-`, `misc-`, `hicpp-` and `portability-`, which the root configuration uses, stay
out: style is not a defect, and `hicpp-` repeats `cppcoreguidelines-`. The exceptions are the two
function-size checks below.

## Disabled, with the reason

All of them come from the 0.3 `.clang-tidy` without a new measurement, and they are evaluated
again with the first code of the core. The reason for each one is also a comment in
`core/.clang-tidy`.

| Check | Reason |
| --- | --- |
| `bugprone-easily-swappable-parameters` | High noise in 0.3; the planned defense is a named type for each domain parameter |
| `cppcoreguidelines-avoid-magic-numbers` | Style, with no associated defect |
| `cppcoreguidelines-non-private-member-variables-in-classes` | A data struct with public fields is the form of the Core Guidelines (C.2, C.131) |
| `modernize-use-trailing-return-type` | Style |

Disabling one more requires the concrete finding that motivated it, recorded in this table.

## Pinned options

- `bugprone-unused-return-value.CheckedReturnTypes` includes `::std::expected`. It is the default
  of clang-tidy 23, pinned so that it does not change with the version. With
  `AllowCastToVoid: false`, not even a cast to `void` discards the error.
- `bugprone-unsafe-functions.CustomFunctions` lists the process-creation functions, the gate
  "processes only through the launcher" ([processes](../architecture/processes-and-trust.md)),
  and `std::thread::detach` and `std::jthread::detach`, the gate "no detached thread". The
  default list of the check (`gets`, the Annex K functions) stays enabled.

Approved by the maintainer on 2026-11-23, not yet enabled: per function, cognitive complexity up
to 25 (`readability-function-cognitive-complexity`, whose default threshold in clang-tidy 23 is
25, measured on 2026-11-23) and nesting up to 4 (`readability-function-size.NestingThreshold`,
with no default). There is no line limit, per function or per file
([file roles](../architecture/modules-and-layers.md#inside-a-module-file-roles)).

## Gates on clang-tidy

A clang-tidy option used as a gate has a corpus in `core/tests/gates/` and a test in the `lint`
preset that measures its effectiveness on every build ([gates](gates-and-effectiveness.md)). The
test runs only the gate's check, with the options of `core/.clang-tidy`: changing the option
without updating the corpus fails.

A lost `// accept:` mark shrinks the number of correct forms, and the test still passes. Approved
on 2026-11-23, not yet in place: the expected number of defects and of correct forms of each gate
is pinned in CMake, so that a lost mark fails.

## Suppression

`NOLINT(<check>): <reason>`, on the line. Never a `NOLINT` without a check, and never
`NOLINTBEGIN`/`NOLINTEND`.
