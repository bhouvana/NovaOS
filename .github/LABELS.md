# Label Taxonomy

Documented here rather than created on GitHub directly, since this repository has no
GitHub remote yet. Once one exists, these can be created with `gh label create` (or the
repo settings UI) from this list — treat this file as the source of truth if the two
ever drift.

## Type

| Label | Color | Meaning |
|---|---|---|
| `bug` | `#d73a4a` | Something doesn't work as specified |
| `enhancement` | `#a2eeef` | A feature within an already-specified subsystem |
| `rfc` | `#5319e7` | Requires an RFC before implementation ([docs/rfcs/README.md](../docs/rfcs/README.md)) |
| `docs` | `#0075ca` | Documentation-only change |
| `chore` | `#cfd3d7` | Tooling, CI, dependency bumps — no user-visible behavior change |

## Subsystem (mirrors [docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md))

`area:system` · `area:services` · `area:desktop` · `area:sdk` · `area:apps` · `area:web`
· `area:tools` · `area:docs`

## Status

| Label | Meaning |
|---|---|
| `status:blocked` | Waiting on another issue/PR/decision |
| `status:blocked-on-environment` | Waiting on a Linux graphics toolchain (wlroots/QEMU) not yet set up — see [docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §4's Environment note |
| `status:needs-rfc` | Design not settled enough to implement |
| `good-first-issue` | Scoped, doesn't require deep context — not used until Phase 2.5+ per [CONTRIBUTING.md](../CONTRIBUTING.md) |

## Priority

`priority:critical` · `priority:high` · `priority:normal` · `priority:low`

## Milestones

GitHub milestones map 1:1 to the phases in
[docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md): `Phase 2 —
Vertical Slice`, `Phase 2.5 — Architecture Validation`, `Phase 3 — Core Desktop`,
`Phase 4 — Core Applications`, `Phase 5 — Browser, Games, Developer SDK`, `Phase 6 —
Browser Demo, Installer, Recovery, Release`. Created on GitHub once a remote exists;
until then, an issue's target phase is tracked via a `phase:N` label as a placeholder.
