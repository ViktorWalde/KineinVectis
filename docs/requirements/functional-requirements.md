# Functional requirements of the core

Owner: what the core does, at which priority level, which reference IDE each capability follows,
which test accepts it, and the delivery order of the base. The limits of time, memory and security
belong to [non-functional-requirements.md](non-functional-requirements.md).

State: draft of 2026-11-26. No requirement is implemented. The acceptance tests cited here do not
exist yet.

## Scope

- **Languages:** C and C++. Rust and Python come back once the core is solid (ADR-0012).
- **Outside the core:** databases, Grafana, remote work over SSH, containers, Cargo and
  Python.
- **Reference:** each capability of N0 names the IDE whose behavior it follows, with the version
  and the source, in the order of [ADR-0012](../adr/ADR-0012-cpp23-backend.md) (item 5). Its
  acceptance test checks that behavior as an observable result on a fixture
  ([test strategy](../engineering/test-strategy.md#acceptance-against-the-reference-ide)). Where no
  reference IDE has the behavior, the test proves the expected behavior and how each failure is
  handled. The reference of each N1 and N2 capability is named when its level is planned.

## Levels

| Level | Entry criterion |
| --- | --- |
| **N0** | Without it, Kinein cannot be developed in Kinein itself (dogfooding) |
| **N1** | The rest of the daily C and C++ work: git, debugging, assisted CMake editing, formatting and templates |
| **N2** | The differentiator: C/C++ tool setup, embedded and the IDE's own semantic analysis |

The dogfooding journey that defines N0, for C and C++: open the repository, edit, use clangd,
build and test with CMake, use the terminal, restart without losing a change, and handle a missing
or hung tool.

## N0: dogfooding

"CLion" is CLion 2026.3 and its help; "IntelliJ" is the source of IntelliJ IDEA 2026.3
(`JetBrains/intellij-community`, branch `263`); "VS Code" is VS Code 1.148.0.

| ID | Capability | Reference | Acceptance |
| --- | --- | --- | --- |
| RF-CORE-01 | Life cycle: answer the ping, report the protocol version, shut down cleanly | VS Code: the extension host, a separate process restarted after a crash (`ExtensionHostCrashTracker`); the [core process](../architecture/protocol-and-contract.md#the-core-process) | Shut down with no orphan process; the states and the restart rule of the core process |
| RF-CORE-02 | Every request can be cancelled and has a deadline; a long operation is a job with progress | CLion: the processes manager lists the background processes, and each can be cancelled ([processes][clion-processes]) | Test with a fake, hung tool (RNF-EXE-02): cancelling ends the job with `RequestCancelled` and kills its process group; an expired deadline ends it with `DeadlineExceeded` |
| RF-WS-01 | Open, close and query the workspace; list of recent ones | CLion: *Open* on a folder, *Recent Projects* and *Close Project*, one window per project ([opening and closing][clion-open]); one workspace per core process | Opening a folder without trust writes nothing in it (RNF-DADOS-05); opening from the recent list reaches the same state as opening the folder |
| RF-WS-02 | Trust in the workspace: state, grant and the restricted mode | CLion: *Trust Project*, safe mode and trusted locations ([project security][clion-security]); IntelliJ: trust goes only from untrusted to trusted while the workspace is open (`TrustedProjectsListener`) | Fixture of the hostile project with zero marks (RNF-SEG-01); granting trust restarts the LSP session; a removed trust applies at the next opening |
| RF-FS-01 | Read, write (with the expected content and an atomic write), create, rename, copy, move to the trash | CLion: *Copy* ([copy][clion-copy]) and the setting that moves a deleted file to the trash ([system settings][clion-system]); IntelliJ: the file is refreshed before each write (`FileDocumentManagerImpl.doSaveDocument`) | A crash in the middle of a write leaves no partial file (RNF-DUR-02); a write over a file changed on disk since it was read becomes a conflict, not an overwrite; a deleted file is in the trash of the [freedesktop.org specification][trash] and can be restored |
| RF-FS-02 | Search in the workspace and replace | CLion: *Find in Files* and *Replace in Files*, with match case, words, regular expression, directory, scope and file mask ([find and replace][clion-find]) | Each option has its own test on a fixture; replace changes only the matches found; complexity test (RNF-PERF-01) |
| RF-FS-03 | An external change on disk becomes an event | CLion: external changes are synchronized when the window regains focus or an editor tab opens ([system settings][clion-system]); IntelliJ: a document with no unsaved edit reloads in silence, and one with unsaved edits gets a conflict (`FileDocumentManagerImpl.contentsChanged`) | Test with a file changed from outside, with and without unsaved edits in its document |
| RF-TC-01 | Toolchains: detect the installed ones, keep one as the default, and apply the environment script of a toolchain | CLion: a toolchain groups CMake, the build tool, the compilers, the debugger and the environment; one is the default; the environment script applies to CMake generation, build and launch ([toolchains][clion-toolchains]) | The detected toolchain names each tool it found; the environment of the script reaches configure, build and run; a compiler defined by the workspace never runs without trust |
| RF-CMK-01 | CMake: the CMake profiles, with the configure and build presets loaded read-only; configure, reload and reset of the cache; the targets; `compile_commands.json` of the active profile | CLion: [CMake profiles][clion-profile] (the model through the CMake File API), [CMake presets][clion-presets] (`CMakePresets.json` and `CMakeUserPresets.json`) and [reload][clion-reload] | Configuring without trust is refused; a configure preset becomes a read-only profile; the targets come from the reply of the CMake File API, not from parsing `CMakeLists.txt`; changing the language standard of the profile changes `compile_commands.json` |
| RF-BLD-01 | Build, run and test the build target, with output and diagnostics | CLion: running builds the target first ([application configuration][clion-runconfig]); *Stop* sends SIGTERM and *Kill Process* SIGKILL ([run tool window][clion-run]); the CTest tests are found on opening and shown with status and duration ([CTest][clion-ctest]); build errors in the *Problems* tool window ([problems][clion-problems]) | Running builds first; cancelling kills the process group, SIGTERM then SIGKILL (E8); the CTest tests of the profile are found, each with its status and duration; a compiler error becomes a diagnostic at its position |
| RF-TERM-01 | Terminal: open, write, resize, scroll, select, close | CLion: the person's default shell, started in the root of the project ([terminal][clion-terminal]) | The shell is the person's default shell, in the root of the workspace; an `exec` failure becomes a typed error (L18) |
| RF-ED-01 | Keep the document of each open file: apply the deltas of the UI in order, number the versions, propose the core's text changes with a base version, resynchronize the UI after a divergence | No reference IDE splits the text this way ([ADR-0012](../adr/ADR-0012-cpp23-backend.md), item 3): own behavior | Property test: after random deltas and proposals, the copy of the UI equals the document; a proposal on a stale base is refused |
| RF-ED-02 | Write the editor's file with the guarded write and recover the text after a crash from the draft | IntelliJ: the save triggers (focus loss, before build and run, closing the workspace, exit; `GeneralSettings`, `SaveAndSyncHandlerImpl`). VS Code: a backup of each modified document 1,000 ms after the last change, postponed while typing (`WorkingCopyBackupTracker`); the draft adds the maximum delay that the backup lacks (L04) | A crash with zero keystrokes lost beyond the limit (RNF-DUR-01); each save trigger writes every document with unsaved edits |
| RF-ED-03 | Highlight with tree-sitter, incrementally, for a named version | Zed: "uses the Tree-sitter parsing library to provide built-in language-specific features" ([language extensions][zed-lang]) | RNF-EDI-01 at the reference sizes; complexity test (RNF-PERF-01) |
| RF-ED-04 | Place each file in its size tier (full, editing only, not opened), tell the person the limit and the size, and take the limits from the settings | The method of the JetBrains IDEs ([editing engine](../architecture/editing-engine.md#size-limits)) | Files one byte below and above each limit; raising a limit in the settings changes the tier |
| RF-ED-05 | Check the content of each file on opening (NUL byte, valid UTF-8 with or without a BOM, one line ending, U+2028 and U+2029), open it read only with the reason, and offer the gesture that converts it where one exists | The sources of each case in [text encoding](../architecture/text-encoding-and-positions.md#file-content) | One file per case reaches its state with its notice; the gesture makes it editable |
| RF-LSP-01 | One clangd session per root, created by the launcher, with the core's context (`compile_commands.json`, `--query-driver` only for a compiler detected without trust); each open document is synchronized in that session | IntelliJ: the LSP client of the platform; after an unexpected stop the server is shown as terminated and restarts only by hand (`LspClientManagerImpl.handleServerStop`, `RestartLspServerAction`) | One clangd process per root during the journey; changing the language standard of the profile changes the diagnostics; no `didChange` carries the whole document; a crash of clangd leaves the session terminated until a restart by hand |
| RF-SET-01 | Settings of the IDE and of the workspace | CLion: settings of the IDE and of the project, the latter in the project directory ([project and IDE settings][clion-settings]); VS Code: "Workspace settings override user settings" ([settings][vscode-settings]) | A value of the workspace takes precedence over the same value of the person; personal state never lives in the tree of the workspace (RNF-DADOS-05) |

## N1: the daily C and C++ work

| ID | Capability | Acceptance |
| --- | --- | --- |
| RF-GIT-01 | Git: status, diff, log, blame, stage, commit, branches; off without trust | The `fsmonitor` of the workspace never runs (L05) |
| RF-DBG-01 | Debugging over DAP: start, stop, steps, stack, variables, evaluate, memory, disassembly | Defined with N1 |
| RF-CMK-02 | Assisted editing of CMake: targets, libraries, language standard, presets, sanitizers | A preview before writing |
| RF-CA-01 | Configuration recipes with preview and apply | Applying writes exactly the preview |
| RF-PRJ-01 | Create a C/C++ project from a template | The new template ignores the IDE's state in git (L13) |
| RF-SETUP-01 | Installation guide for the tools | No `--version` runs on the loop (L08) |
| RF-FMT-01 | Format with the workspace's `clang-format`, with trust | The result equals `clang-format` run on the same file with the same `.clang-format` |

## N2: the differentiator

The order and the design of N2 belong to the maintainer, in the library with no Qt.

| ID | Capability |
| --- | --- |
| RF-EMB-01 | Flashing and probes |
| RF-EMB-02 | Serial |
| RF-EMB-03 | SDKs: ESP-IDF, Zephyr (`west`), PlatformIO |
| RF-EMB-04 | Memory and registers |
| RF-SEM-01 | The IDE's own semantic analysis: local index, definition, reference and scope |
| RF-SEM-02 | Quality and coverage |

## The order of the base

Proposal for Phase D (P12), with the done criterion of each step written before the code:

1. `base`, `sys.fs` and `codec` ([modules](../architecture/modules-and-layers.md)), after the
   decision on the libraries (P16).
2. `rt` and `protocol` (designed by the maintainer, after P24), plus `app` and `core/qt` with only
   the ping and the initialization. A test client over the contract proves the path from the wire
   to the library, the CI and the sanitizers early; dogfooding stays on the prototype of the tag `v0.3-rust-final` until the
   sync with the frontend ([ADR-0012](../adr/ADR-0012-cpp23-backend.md), item 6).
3. `storage`, `sys.process` and `launch`, with the hostile project test.
4. `workspace`, and then the N0 domain.

```mermaid
flowchart LR
    S1["1. base, sys.fs, codec<br/>after P16"] --> S2["2. rt, protocol, app, core/qt<br/>ping and initialization"]
    S2 --> S3["3. storage, sys.process, launch<br/>hostile project test"]
    S3 --> S4["4. workspace<br/>then the N0 domain"]
```

[clion-processes]: https://www.jetbrains.com/help/clion/working-with-processes.html
[clion-open]: https://www.jetbrains.com/help/clion/opening-reopening-and-closing-projects.html
[clion-security]: https://www.jetbrains.com/help/clion/project-security.html
[clion-copy]: https://www.jetbrains.com/help/clion/copy-refactoring.html
[clion-system]: https://www.jetbrains.com/help/clion/system-settings.html
[trash]: https://specifications.freedesktop.org/trash/latest/
[clion-find]: https://www.jetbrains.com/help/clion/finding-and-replacing-text-in-project.html
[clion-toolchains]: https://www.jetbrains.com/help/clion/how-to-create-toolchain-in-clion.html
[clion-profile]: https://www.jetbrains.com/help/clion/cmake-profile.html
[clion-presets]: https://www.jetbrains.com/help/clion/cmake-presets.html
[clion-reload]: https://www.jetbrains.com/help/clion/reloading-project.html
[clion-runconfig]: https://www.jetbrains.com/help/clion/run-debug-configuration-application.html
[clion-run]: https://www.jetbrains.com/help/clion/run-tool-window.html
[clion-ctest]: https://www.jetbrains.com/help/clion/ctest-support.html
[clion-problems]: https://www.jetbrains.com/help/clion/problems-tool-window.html
[clion-terminal]: https://www.jetbrains.com/help/clion/terminal-emulator.html
[zed-lang]: https://zed.dev/docs/extensions/languages
[clion-settings]: https://www.jetbrains.com/help/clion/configuring-project-and-ide-settings.html
[vscode-settings]: https://code.visualstudio.com/docs/configure/settings
