# Architecture Decision Records

ADRs capture significant, hard-to-reverse technical decisions: what was decided, what
alternatives were considered, and why. They are immutable once merged — a changed mind
gets a new ADR that supersedes the old one, not an edit.

## When to write one

Write an ADR when a decision:
- is expensive to reverse (language/runtime choice, on-disk format, IPC protocol, wire
  protocol), or
- crosses subsystem boundaries, or
- a reasonable engineer could disagree with and will ask "why not X" later.

Don't write one for reversible, local choices (a function name, a single file's internal
structure) — that's just code review.

## Template

```markdown
# ADR-NNNN: Title

Status: Proposed | Accepted | Superseded by ADR-XXXX | Deprecated
Date: YYYY-MM-DD
Deciders: <names/roles>

## Context
What problem are we solving? What constraints apply?

## Options Considered
1. Option A — pros/cons
2. Option B — pros/cons
3. Option C — pros/cons

## Decision
Which option, stated plainly.

## Rationale
Why this option wins against the goal-priority order in docs/00-VISION.md §6.

## Consequences
What this makes easier, what it makes harder, what it forecloses.

## Revisit Triggers
Concrete conditions under which this decision should be re-opened.
```

## Index

| ADR | Title | Status |
|---|---|---|
| [0001](ADR-0001-linux-base-distribution.md) | Base Linux distribution | Accepted |
| [0002](ADR-0002-init-and-service-supervision.md) | Init system & service supervision | Accepted |
| [0003](ADR-0003-compositor-and-display-protocol.md) | Compositor & display protocol | Accepted |
| [0004](ADR-0004-systems-language.md) | Primary systems language | Accepted |
| [0005](ADR-0005-ui-toolkit.md) | Native UI toolkit | Accepted |
| [0006](ADR-0006-ipc-mechanism.md) | IPC mechanism (Nova Bus) | Accepted |
| [0007](ADR-0007-package-format.md) | Package format & Package Center | Accepted |
| [0008](ADR-0008-filesystem-and-update-strategy.md) | Root filesystem & update strategy | Accepted |
| [0009](ADR-0009-browser-boot-emulator.md) | Browser boot emulator | Accepted |
| [0010](ADR-0010-app-sandboxing-model.md) | Application sandboxing model | Accepted |
