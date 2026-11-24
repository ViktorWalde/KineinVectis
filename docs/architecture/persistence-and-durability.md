# Persistence and durability

Owner: where each file the IDE owns lives, how it is written and how much text a crash can lose.
It closes lessons L04 and L13 ([lessons](../lessons-from-0.3.md)).

## Three classes, plus the build tree

From 64 §5.9 (64:603-672), decision DP-08 of the maintainer (64:943). The directory names are
English in the C++ core; 64 wrote `projetos/`, a name no code uses.

| Class | Where | What |
| --- | --- | --- |
| Project configuration, shareable | `<root>/.kinein/` | what a colleague should receive through git |
| The person's state in the project | `$XDG_STATE_HOME/kinein-vectis/projects/<id>/` | session, drafts, local choices |
| Project cache | `$XDG_CACHE_HOME/kinein-vectis/projects/<id>/` | derived and re-creatable |
| Build tree | `<root>/.kinein/build/` | build output, with its own `.gitignore` |

- `<id>` derives from the canonical path of the root, and the state follows a project renamed on
  the same volume through the identity of the directory (64:620-635).
- Opening a folder without trust writes nothing in it.
- Each own file has an owner, a class and a schema version in a single catalog.

## Writing rules

| Rule | Origin |
| --- | --- |
| Atomic write: temporary file in the same directory, `fsync`, rename, `fsync` of the directory | RNF-DUR-02 |
| Write with the expected content: if the disk changed since the read, the write is refused with the conflict | `fs.write` of 0.3 (65:83-84) |
| Never follow a link, never open what is not a regular file | R-DADOS-02 of 63 |
| Never overwrite a file that is not understood (a newer schema version): warn and preserve | R-DADOS-03 of 63 |
| Migration of the 0.3 `.kinein`: copy, read back, and only then remove | 64:660-672 |

```mermaid
sequenceDiagram
    participant S as storage / sys.fs
    participant D as directory of the target
    S->>D: create a temporary file in the same directory
    S->>D: write the content, then fsync the file
    S->>D: rename the temporary file over the target
    S->>D: fsync the directory
    Note over S,D: a crash at any step leaves the old file or the new one, never a mix
```

## Durability of typed text

- **The 0.3 problem:** a debounce with no maximum delay in a QML `Timer`. Whoever types without a
  1.5 s pause has nothing written, and the loss has no ceiling (INF-26).
- **The rule:** the durability policy belongs to the core. Every change of the buffer mirror
  schedules the write of the draft with a **maximum delay**; a pause in the typing may bring the
  write forward, never push it past the delay.
- **The limit:** RNF-DUR-01, 2 s, the maintainer's decision of 2026-11-22
  ([non-functional requirements](../requirements/non-functional-requirements.md)).
- **The path:** the draft comes from the mirror the core keeps through `nvim_buf_attach`, without
  the Neovim swap file (decision 3 of 65 §21, 65:895-896). The project file is written by the
  core, through `BufWriteCmd` (65:892-894).
- **Acceptance:** `kill -9` on the UI, the core and `nvim` after long typing with no pause, with
  the loss measured, and a mutant with no maximum delay that must fail.
