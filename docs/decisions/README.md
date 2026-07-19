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
| [0001](ADR-0001-linux-base-distribution.md) | Base Linux distribution | Accepted (revised — see the ADR's own Revision section) |

ADRs 0002–0010 (init system, compositor, systems language, UI toolkit, IPC, package
format, filesystem/update strategy, browser boot emulator, app sandboxing model) were
written for the from-scratch desktop platform direction abandoned 2026-07-19 (see
[docs/00-VISION.md](../00-VISION.md)) and removed along with the code they governed.
Recoverable from git history if that direction is ever revisited.
