# Definition of done

Owner: when a change to the core and when a milestone may be called done. "Done" without every
item is not done: it is "partial", with the list of what is missing.

## Every change

1. Zero warnings in the `dev` preset (GCC) and in `lint` (Clang).
2. A clean clang-tidy in `lint`.
3. Tests green in the `dev`, `asan-ubsan` and `tsan` presets
   (`cmake --workflow --preset <name>`, from `core/`).
4. A test for the added behavior, and a mutant for each added guard.
5. Performance measured before and after at the four sizes, when the change touches a path that
   depends on the size of the file or of the workspace.
6. The owning document of the subject updated in the same commit, when the change alters one of
   its facts. A thread enters the table of
   [execution](../architecture/execution-and-concurrency.md#threads-and-owners).
7. **The explicit list of what was not verified:** environment, platform or case the change did
   not exercise.
8. A commit with a DCO sign-off (`git commit -s`, ADR-0011) whose message states what was measured
   and what was not verified.

## Every milestone

- The done criteria of the milestone are written **before** the code, with a number, a method and
  the test that enforces each one.
- The milestone closes with the measurement of each criterion, or with an explicit NOT PROVEN.
- A data-loss or security defect found on the way jumps the queue.

## Criteria per milestone

Not defined yet. The proposed order of the milestones and the criteria of each one are Phase D
(P12 in the [pending decisions](../pending-decisions.md)).
