# Editing engine

Owner: the editing component, who owns the text of a document, how an edit travels between the
UI and the core, how a large file degrades, what the component does not bring and how the editor
is accepted.

## The component

The Qt Quick `TextEdit`, with its `QQuickTextDocument`, inside a `Flickable`
([ADR-0012](../adr/ADR-0012-cpp23-backend.md)). It is Qt's own component: keyboard and mouse
input, selection, the clipboard, undo, input methods and accessibility come with it. A solid ready
component is used wherever one exists; what no component brings is built, with the reason written
in this file.

| Need | Comes from |
| --- | --- |
| Input, selection, clipboard, undo, IME, accessibility | `TextEdit` |
| Highlighting | tree-sitter in the core, with the C and C++ grammars (all three MIT, as GitHub reported on 2026-11-25; decided on 2026-11-25); the UI applies the ranges the core sends through a `QSyntaxHighlighter` on the `QQuickTextDocument` |
| Completion, diagnostics, navigation, rename, code actions | clangd, through the LSP client of the core (P21) |
| Line numbers, diagnostic marks, folding, multiple cursors, split view | Not in `TextEdit`. Each one enters only with a requirement, as QML over the component, fed by the core |

Inside a `Flickable`, `TextEdit` creates scene graph nodes only for the blocks in the viewport
([Text editing improvements in Qt Quick](https://www.qt.io/blog/text-editing-improvements-in-qt-quick)).
The layout, however, covers the whole document: that is the risk the first measurement checks.

## Size limits

The method of the JetBrains IDEs: each file falls in a tier by its size, and the person is told
when a file leaves the full tier, with the limit and the actual size
([CLion: File size and length limits](https://www.jetbrains.com/help/clion/configuring-file-size-limit.html)).
The core decides the tier from the size on disk and the limits in the settings (RF-SET-01); the
UI shows the notice and lets the person raise the limit.

```mermaid
flowchart TD
    OPEN["open a file"] --> SIZE{"size on disk"}
    SIZE -->|"up to the code insight limit"| FULL["full tier<br/>editing, highlighting, LSP"]
    SIZE -->|"up to the opening limit"| PLAIN["editing only<br/>no highlighting, no LSP,<br/>a notice with the limit and the size"]
    SIZE -->|"above the opening limit"| NONE["not opened in the editor<br/>a notice with the limit and the size"]
```

| Limit | JetBrains default (CLion 2026.3) | Kinein |
| --- | --- | --- |
| Opening | 20,000 KB (`idea.max.content.load.filesize`) | Set by the measurement of the component |
| Code insight | 2,500 KB (`idea.max.intellisense.filesize`); for a C/C++ file, 500,000 characters | Set by the measurement; the full tier covers at least the 40,000-line reference file |

- The method is adopted; the values come from the measurement of `TextEdit` and of the core. The
  500,000 characters of CLion are 25 characters per line at 20,000 lines: copied as they are, they
  would take highlighting away from the reference sizes of the requirements.
- In every tier where the file opens, the versions, the draft and the guarded write hold.
- Before the size, the content of the file decides whether it opens for editing
  ([text encoding and positions](text-encoding-and-positions.md#file-content)).

## Who owns what

| Responsibility | Owner | The others |
| --- | --- | --- |
| Text and version of the document | The core | The UI holds the copy it shows and never decides a version |
| A keystroke | The UI applies it to its copy at once and sends the delta with its base version, in line and UTF-16 column ([units](text-encoding-and-positions.md#between-the-ui-and-the-core)) | The core applies the deltas in order; a typed delta is never refused |
| A text change of the core (format, rename, fix) | The core proposes it with its base version | The UI applies it only on that base, and the change reaches the core as an ordinary delta; on another base it refuses, and the core computes again |
| A divergence between the copy and the document | The core: its text and version replace the copy | none |
| Writing the file | The core, with the guarded write of [persistence](persistence-and-durability.md) | The UI sends the intent |
| Draft and recovery | The core, from its document, with a maximum delay (RNF-DUR-01) | none |
| Highlighting, diagnostics, completion | The core, for a named version | The UI discards a result for an older version |
| The LSP session and the document sync | The core: one session per workspace root, and each open document synchronized in it by incremental `didChange` from the deltas | none |
| Cursor, selection, scroll | The UI (presentation) | The core receives the position when a request needs it |

The core's document changes only through the stream of deltas, so it has a single writer.

## An edit

```mermaid
sequenceDiagram
    participant P as person
    participant UI as UI, TextEdit
    participant C as core, editor module
    participant L as clangd
    P->>UI: keystroke
    UI->>UI: apply to the copy, draw the frame
    UI->>C: delta on base version n
    C->>C: apply, version n+1, schedule the draft
    C-->>UI: acknowledgment of version n+1
    C->>L: incremental didChange, version n+1
    C-->>UI: highlighting and diagnostics of version n+1
```

## A text change of the core

```mermaid
sequenceDiagram
    participant UI as UI, TextEdit
    participant C as core
    C->>UI: proposal on base version n
    alt the copy is on version n
        UI->>UI: apply to the copy
        UI->>C: delta on base version n
    else the copy is on a later version
        UI-->>C: refused, stale base
        C->>C: compute again on the current version
    end
```

- No message after the opening carries the whole document (RNF-EDI-01). The exception is the
  resynchronization after a divergence, which is recorded as a defect.
- The units of a delta are in [text encoding and positions](text-encoding-and-positions.md). Its shape, the
  detection of a divergence and the replay of the deltas the draft does not have after a crash of
  the core belong to the editing protocol, by ADR (P22).

## The life of a document

The core runs this machine for each open document. It follows IntelliJ IDEA 2026.3 (branch `263`):
a document with no unsaved edit reloads in silence when its file changes, and one with unsaved
edits gets a conflict, with the choice of the disk or the memory version and their diff
(`FileDocumentManagerImpl.contentsChanged`, `MemoryDiskConflictResolver`); the file is checked
before each write (`doSaveDocument`); closing an editor neither saves nor asks; the editors of a
deleted file close (`FileEditorManagerImpl.beforeFileDeletion`). The draft is the core's own
(RNF-DUR-01). A cell with "—" keeps the state and records the event.

```mermaid
stateDiagram-v2
    [*] --> Clean: no newer draft
    [*] --> Modified: newer draft, file unchanged since its base
    [*] --> Conflict: newer draft, file changed since its base
    Clean --> Modified: edit
    Modified --> Saving: save trigger
    Saving --> Clean: written, no later edit
    Saving --> Modified: written with later edits, or write failed
    Modified --> Conflict: disk changed
    Saving --> Conflict: disk changed
    Conflict --> Clean: load disk
    Conflict --> Modified: keep memory
    Clean --> Closed: editor closed, or deleted
    Modified --> Closed: deleted, draft kept
    Saving --> Closed: deleted, draft kept
    Conflict --> Closed: deleted, draft kept
    Closed --> [*]
```

| State | edit | save trigger | written | write failed | disk changed | load disk | keep memory | deleted | editor closed |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Clean | Modified | — | — | — | Clean, reloaded in silence | — | — | Closed | Closed |
| Modified | Modified | Saving | — | — | Conflict | — | — | Closed, draft kept | Modified |
| Saving | Saving | — | Clean, or Modified with later edits | Modified, with the error | Conflict, nothing written | — | — | Closed, draft kept | Saving |
| Conflict | Conflict | Conflict, nothing written | — | — | Conflict, with the new disk version | Clean, draft discarded | Modified, on the disk version as the base | Closed, draft kept | Conflict |
| Closed | — | — | — | — | — | — | — | — | — |

- **Save triggers,** as in IntelliJ: the window loses focus, before a build, run, test or
  configure, closing the workspace, and exit (`GeneralSettings.autoSaveFiles`,
  `SaveAndSyncHandlerImpl`); saving after inactivity is a setting, off by default, of 15 s
  (`autoSaveIfInactive`, `inactiveTimeout`). The person's save writes every modified document.
- **deleted** is any removal of the file that the IDE did not do; the draft of a document with
  unsaved edits stays, with the offer to write the file again from it. A rename or a move by the
  IDE changes the path and keeps the state. A document whose editor is closed closes at Clean.
- **Opening** with a draft newer than the file gives Modified when the file did not change since
  the base of the draft, and Conflict when it did. A read-only document (RF-ED-05) refuses every
  edit and stays Clean.
- **What the person sees:** the unsaved mark, the conflict with the diff of the two versions, a
  write error, and the offer to write a deleted file again from its draft.
- **Acceptance:** a table test with a fake file system drives every cell; mutants that write in
  Conflict, skip the check before the write, or drop the draft of a deleted document fail it.

## Acceptance

1. **The component alone**, before the integration: `TextEdit` with no IPC, offscreen at
   1280x800, at the four reference sizes and at the opening limit, by the method of the
   [test strategy](../engineering/test-strategy.md#performance). Its measurement on the reference
   machine is the reference of RNF-ED-01.
2. **The integrated product**, by the same method, within the limits of RNF-ED-01. A missed limit
   goes back to the maintainer with the measurement.
3. **IME** with fcitx5 or ibus: the preedit at the cursor and the right committed text.
4. **Accessibility** through AT-SPI: the whole document, the cursor and the selection.
5. **A property test:** after random deltas and proposals, the copy of the UI and the document of
   the core are equal, and a proposal on a stale base is refused.
6. **Crash:** RNF-DUR-01, with `kill -9` on the UI and on the core.
7. **Tiers:** a file one byte below and one byte above each limit falls in the right tier, and the
   notice states the limit and the size (RNF-ED-02).

## NOT VERIFIED

- `TextEdit` alone at the four reference sizes: never measured, so RNF-ED-01 has no reference
  number yet.
- A report on Qt 6.6 measured 5 s to redraw 21,000 lines, growing linearly
  ([Qt Forum](https://forum.qt.io/post/819626)). It was not reproduced with Qt 6.12.
- Folding and multiple cursors on `QQuickTextDocument`.
- What `TextEdit` holds at 2,500 KB and at 20,000 KB: the limits of Kinein come from that
  measurement.
