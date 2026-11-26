# Contributing to Kinein Vectis

The work happens on the core, in `core/`, on the single branch `main`. The project is in its
engineering phase: the CI, the gates, the architecture and the requirements come before any code
of the core ([README](README.md)). The contributions that help now:

- **Review of the documentation and of the requirements:** a gap, a contradiction, a limit with
  no measurement, a claim with no source.
- **The comparison of a requirement with its reference IDE:** what CLion, IntelliJ IDEA, VS Code
  or Zed does in the case, with the version and the source.
- **The CI and the build:** the `Core 0.1` workflow, the presets and the gates.

## Before a change

- **Read only the owner of the subject.** The map of the documentation is
  [docs/README.md](docs/README.md); each question has one file.
- **Behavior comes from the reference IDEs**, each cited with its version and its source, the code
  or the official documentation ([ADR-0012](docs/adr/ADR-0012-cpp23-backend.md)). A claim about a
  version, an API, a flag or a behavior carries a primary source or a measurement; anything else
  is stated as not verified.
- **Design first.** The algorithms, the concurrency and the protocol of the core are designed with
  the maintainer before any code is written. A new external dependency needs the maintainer's
  approval and a checked license.
- **Open an issue first** for anything beyond a small fix, with the
  [contribution task](.github/ISSUE_TEMPLATE/contribution_task.md) or the
  [bug report](.github/ISSUE_TEMPLATE/bug_report.md) template.

## Rules of a change

- **English** for everything a contributor reads or writes: identifiers, comments, messages, file
  names, documentation, commits and pull requests. The `ui/` directory is translated at the sync
  with the frontend.
- **Code:** the C++23 rules, the tool that enforces each one and the comment standard, C1 to C12,
  are in [the C++23 rules](docs/engineering/cpp23-rules.md).
- **Documentation:** the rules of [the documentation tree](docs/README.md#rules-of-this-tree).
- **Done** means the [definition of done](docs/engineering/definition-of-done.md): zero warnings,
  clean clang-tidy, the tests green in `dev`, `asan-ubsan` and `tsan`, and the list of what was not
  verified. The commands are in [Building the core](README.md#building-the-core).

## Commits and pull requests

- **Developer Certificate of Origin 1.1:** every commit carries a sign-off (`git commit -s`); no
  copyright assignment is required ([`DCO.txt`](DCO.txt)).
- **A pull request into `main`** follows the
  [pull request template](.github/pull_request_template.md): what changed, what was measured and
  what was not verified. The `Core 0.1` workflow runs in full on it and has to be green.

## License of a contribution

A contribution takes the license of the material it changes: the project's own code is under
`AGPL-3.0-only` ([`LICENSE`](LICENSE)) and its own documentation under CC BY-SA 4.0
([`LICENSE-CC-BY-SA-4.0.txt`](LICENSE-CC-BY-SA-4.0.txt)). The [README](README.md#license) lists
the exceptions.
