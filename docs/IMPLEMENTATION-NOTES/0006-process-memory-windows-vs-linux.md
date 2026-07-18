# 0006: Process memory measured on Windows dev host, not the Linux target

Date: 2026-07-18
Status: Open — closes when a Linux build environment exists

## Architecture

[specs/02-MEMORY-BUDGET.md](../specs/02-MEMORY-BUDGET.md) §1 budgets `novabusd` at
2MB RSS and a simple app at 4–6MB RSS, measured as Linux RSS on a musl-linked binary
under Alpine ([ADR-0001](../decisions/ADR-0001-linux-base-distribution.md)).

## Reality

Measured on this Windows development host (`Get-Process`, release build):

```
                Working Set   Private Memory
novabusd.exe    6.74 MB       1.30 MB
hello.exe       6.81 MB       1.32 MB
```

## Reason

No Linux build environment exists in this development setup
(docs/12-ROADMAP-AND-MILESTONES.md §4's Environment note) — these are real,
measured numbers, not estimates, but they are Windows glibc-equivalent (MSVC CRT)
dynamically-linked binaries, not the musl-static Alpine binaries the budget describes.
Windows "Working Set" also fundamentally counts differently from Linux RSS: it includes
mapped-but-shared system DLLs (`ntdll.dll`, `kernel32.dll`, the Rust std runtime's
Windows syscall shims) that a statically-linked musl binary wouldn't have at all —
Working Set is not a fair proxy for what the same code's Linux RSS would be. "Private
Memory" (memory not shared with the OS/other processes) is a closer analog, and at
~1.3MB per process is already within the same order of magnitude as the 2MB/4-6MB
budget rows despite the platform mismatch — a weak positive signal, not proof.

## Decision

Recorded both numbers, with the caveat, rather than either (a) omitting a real
measurement because it isn't a perfect match for the target platform, or (b) presenting
Working Set as if it were comparable to the Linux RSS budget without the caveat — both
would misinform [specs/02-MEMORY-BUDGET.md](../specs/02-MEMORY-BUDGET.md)'s eventual
"real vs. budgeted" reconciliation more than reporting nothing would.

## Future Direction

Re-measure with `/proc/<pid>/status` `VmRSS` (the real target metric) once a Linux
build environment exists — WSL2 is the nearest-term path (already has gcc; needs Rust,
`protoc` reinstalled or symlinked, and the workspace rebuilt for the `x86_64-unknown-linux-gnu`
or eventual musl target) — and update
[specs/02-MEMORY-BUDGET.md](../specs/02-MEMORY-BUDGET.md)'s "estimate" column with a
real "measured" column at that point, per Phase 2.5's memory-budget question
([12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) §4a).
