# C++23 rules

Owner: the C++ methods of the core, the comment standard, and the mechanism that enforces each
one. The defense against each classic defect of the language is in
[known-cpp-defects-and-defenses.md](known-cpp-defects-and-defenses.md). The flags are in
[build-profiles-and-hardening.md](build-profiles-and-hardening.md). The file roles inside a module
are in [modules and layers](../architecture/modules-and-layers.md#inside-a-module-file-roles).

"Review" in the last column means that no tool enforces the rule: the reviewer checks it. Every
rule marked so is a candidate for a mechanism. "CG" cites the
[C++ Core Guidelines](https://isocpp.github.io/CppCoreGuidelines/CppCoreGuidelines) by rule ID
(the version of 2026-06-14).

## Language

| Rule | Why | Enforced by |
| --- | --- | --- |
| C++23 with no extensions (`CMAKE_CXX_EXTENSIONS OFF`) | Code portable between GCC, Clang and MSVC, which clangd understands | `build.contract` fails on `-std=gnu++` |
| C++26 only through an ADR, one feature at a time, once GCC, clangd and MSVC support it without the experimental label, guarded by a feature-test macro (`__cpp_*`) | C++26 is not published (the DIS ballot of 2026-10-23 failed); GCC 16 marks its support as experimental; the official Clang 23 has no reflection (P2996) | Review; the `cxx26-probe` preset only detects incompatibilities |
| English for everything a contributor reads: identifiers, comments, messages, file names, documentation | ENG-37 of 0.3: 14 thousand Portuguese identifiers became debt; contributors read English | Review |
| The warnings and protections of the profile, `-Werror` in development | The OpenSSF compiler hardening baseline, adapted per compiler ([profiles](build-profiles-and-hardening.md)) | The compiler; `build.contract`, `build.elf_hardening` |
| A Unicode bidirectional character in the code fails the build | Trojan Source (CVE-2021-42574) | `-Wbidi-chars=any` (GCC) |
| No `using namespace` in a header (CG SF.7) | It leaks into every file that includes the header | `-Wheader-hygiene` (Clang) |

## Resources and ownership

| Rule | Why | Enforced by |
| --- | --- | --- |
| RAII for every resource (memory, file, descriptor, pipe, process); no bare `new`, `delete`, `malloc` or `free` | Leaks and double frees | `cppcoreguidelines-owning-memory`, `cppcoreguidelines-no-malloc`; ASan and LSan |
| `T*` and `T&` never transfer ownership; ownership is a value or a `std::unique_ptr`. `T*` only observes and may be null; "not null" is a reference. No GSL (CG I.11, R.20) | The type says who frees | `cppcoreguidelines-owning-memory` |
| `std::shared_ptr` only with the reason for shared ownership written next to it (CG R.21) | Shared ownership hides the lifetime | Review |
| A `QObject` lives in the Qt parent tree; everything else in RAII; never both for the same object | Two owners for one object is a double free | Review; it exists only in `core/qt` |
| `std::string_view` and `std::span` only as parameters, never stored in a member or returned from a function that received a temporary. An array parameter is a `std::span` | Dangling references; array-to-pointer decay loses the size | `bugprone-dangling-handle`, `-Wdangling-reference` (GCC), `-Wdangling` (Clang), `cppcoreguidelines-pro-bounds-array-to-pointer-decay`; ASan `stack-use-after-return` |
| Rule of zero; if one special operation is declared, all five are (CG C.20, C.21) | Shallow copies and double frees | `cppcoreguidelines-special-member-functions` |

## Types and interfaces

| Rule | Why | Enforced by |
| --- | --- | --- |
| A domain value has a named type (`LineIndex`, `ByteOffset`) and an `enum class`; no bare `int` or `bool` with a meaning (CG I.4, Enum.3) | The compiler rejects a swapped argument | Review |
| A text position carries its unit in its type (`ByteOffset`, `Utf16Column`, `DisplayColumn`); only the conversions of `base` change the unit, at a boundary ([text encoding and positions](../architecture/text-encoding-and-positions.md)) | A column in the wrong unit moves an edit and corrupts the text with no error | Review; the property and conversion tests |
| Output by return value; several outputs are a `struct` with named members, not `pair` or `tuple` (CG F.20, F.21) | The name of each output is part of the interface | Review |
| `const` by default, on objects and on member functions; a template has a concept (CG Con.1, Con.2, T.10) | Mutation and template requirements are visible in the signature | Review |
| Every object is born initialized (`{}`); a constructor delivers a ready object, with no `init()` (CG ES.20, C.41) | No read of an indeterminate value, no half-built object | `cppcoreguidelines-pro-type-member-init`, `cppcoreguidelines-prefer-member-initializer` |
| A narrowing conversion only explicit and checked; only named casts (CG ES.46, ES.49) | Silent truncation and sign changes | `-Wconversion`, `-Wsign-conversion`, `-Wold-style-cast`, `-Wuseless-cast` |

## Errors

| Rule | Why | Enforced by |
| --- | --- | --- |
| `std::expected<T, E>` for an expected error (missing file, failed tool, expired deadline), with `E` naming the cause; the result is never discarded, not even with a cast to `void` | The error is part of the type and is not forgotten | `bugprone-unused-return-value` with `std::expected` (`core/.clang-tidy`) |
| An exception only for the unexpected (memory, broken invariant), caught before it crosses a thread, a Qt slot or a protocol handler: those entry points are `noexcept` (CG E.2, E.3) | An exception escaping a thread calls `std::terminate`; through the Qt loop, the behavior is undefined | `bugprone-exception-escape` on `noexcept` functions |
| Each failure cause has its own name (`DeadlineExceeded`, `RequestCancelled`, `WorkspaceUntrusted`) and its own test | L19: an expired deadline came out as "stale" | A test per failure path |

## Concurrency and execution

The invariants E1 to E10 are in [execution and concurrency](../architecture/execution-and-concurrency.md).
In terms of code:

| Rule | Enforced by |
| --- | --- |
| Threads only in `rt`, as `std::jthread` with `std::stop_token` | Review |
| A thread is never detached: no `detach()` (CG CP.26) | clang-tidy (`CustomFunctions`), `gate.detach` |
| One owner per piece of data; threads exchange messages | TSan; review |
| No non-constant global variable | `cppcoreguidelines-avoid-non-const-global-variables`; `-Wglobal-constructors` and `-Wexit-time-destructors` in `core/lib` (Clang) |
| A lock only through RAII (`std::scoped_lock`), with a name; never call unknown code (a callback, a Qt signal) while holding a lock (CG CP.20, CP.22) | `bugprone-unused-raii` for the unnamed lock; review for the unknown code |
| `wait` always with a predicate or a `stop_token`; `volatile` does not synchronize (CG CP.42, CP.8) | Review |
| A C library function that is not thread-safe is forbidden | `concurrency-mt-unsafe` |
| The loop never blocks; every external call has a deadline and cancellation | An I/O guard on the loop (to be designed) |

## Boundaries

| Rule | Enforced by |
| --- | --- |
| `core/lib` with no Qt, testable without an event loop | The `dev` preset does not look for Qt |
| moc never sees `core/lib` | AUTOMOC only on the targets of `core/qt` |
| Processes only through the launcher | clang-tidy (`CustomFunctions`), active and measured ([processes](../architecture/processes-and-trust.md)) |
| Dependencies between modules only toward a lower layer | `kinein_add_module`, proposed ([modules](../architecture/modules-and-layers.md)) |
| Nothing runs before trust in the workspace | Hostile project test (RNF-SEG-01) |
| A new external dependency only with the maintainer's approval and a checked license | Review |
| One responsibility, one owning module. Before a function is written, the behavior is searched for in the owner of the subject, not by name. A new variation enters as an adapter of an existing port (open/closed), never as an adjusted copy or a scattered `switch`. A port exists only where the variation exists: the second occurrence creates it | `gate.duplication` (T1 to T3); the same logic written another way (T4) in review |

## Suppressions

- `NOLINT` only in the form `NOLINT(<check>): <reason>`, on the line, never as a block.
- A compiler warning is not disabled per file; if a flag does not fit, it leaves the profile, with
  the reason in `core/cmake/`.
- The `-Werror` of the development presets is not disabled to get a build through.

## Comments

One standard covers every comment in `core/` and `.github/`: C++, CMake, Python, YAML and
`.clang-tidy`. It is the same in every file and for every author, and a change to it is a
decision of the maintainer. A comment that breaks it fails review, the same as a compiler
warning. No tool enforces these rules in the CI.

| # | Rule | Reason |
| --- | --- | --- |
| C1 | American English, complete sentences that end with a period, present tense, impersonal technical voice | One voice across the codebase; a reader never has to tell an opinion from a rule |
| C2 | ASCII only | Keeps the text readable in every terminal, diff and log, and leaves no room for look-alike characters |
| C3 | `//` in C++ (`/* */` only as corpus data); `#` in CMake, YAML, `.clang-tidy` and shell; docstrings in Python ([PEP 257](https://peps.python.org/pep-0257/)), with the function summary in the imperative (`Return ...`) | One syntax per language |
| C4 | At most 100 columns, the `ColumnLimit` of `.clang-format`. Only a URL, a path, a regular expression or a corpus line that cannot break may exceed it | The width of the code |
| C5 | A comment sits on its own line, above the code it explains. On a code line, only `NOLINT(<check>): <reason>`, `} // namespace <name>`, `#endif // <condition>`, a corpus mark, and `# vX.Y.Z` after an action pinned by SHA | Each allowed form is read by a tool or closes a block |
| C6 | Every file that accepts comments opens with the header below | The reader finds the owning document from any file |
| C7 | A comment states what the code cannot: the reason, the contract, the source, the limit. It never paraphrases the code | A paraphrase goes stale as soon as the code changes |
| C8 | State, not history: no "now", "new", "old", "still", "already", "changed", "used to" | History is in git |
| C9 | A technical claim (version, flag, API behavior) carries its source in one of three forms: `(<document title>, <section>)`; `Measured on YYYY-MM-DD with <tool> <version>.`; `Not verified: <reason>.` | A claim without a source cannot be checked |
| C10 | A decision is cited as `the maintainer's decision of YYYY-MM-DD (<ID>)`; a pending item by its ID in [pending-decisions.md](../pending-decisions.md). No `TODO`, `FIXME` or `XXX` | Pending work has one owner, not one per file |
| C11 | A file is cited by its path from the repository root (`docs/...`, `core/...`); a 0.3 lesson by its ID in [lessons-from-0.3.md](../lessons-from-0.3.md) | The path works from any directory |
| C12 | Never a person's name or user name ("the maintainer"), a mention of AI tools or assistants, or commented-out code outside a corpus | The code describes the product, not who wrote it |

The header (C6) uses `//` or `#`. In Python it is the module docstring with the same lines,
and its first line is in the imperative because it is also the `--help` text.

```text
# <One sentence: what the file is or does.>
#
# Documented in: <path from the repository root>[, section "<heading>"]
#
# <Optional paragraphs: rules, sources, limits.>
```

Corpus marks are `// reject: <what the defect is>` and `// accept: <what the form is>`, with no
final period. In the duplication corpus the mark is `// reject: <name of the original function>`.
Corpus code is data: `clang-format` does not reformat it, but its comments follow this standard.
The `Layout:` and role fields of the header ([file roles](../architecture/modules-and-layers.md#inside-a-module-file-roles))
enter C6 when the role gate is written.
