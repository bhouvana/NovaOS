# system/

Status: **Not yet implemented** — Phase 1 (architecture only). Planned for Phase 2
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §3).

The base system layer: everything below Nova Services in
[../docs/01-SYSTEM-ARCHITECTURE.md](../docs/01-SYSTEM-ARCHITECTURE.md) §1. Owns the
boot sequence, init configuration, image build pipeline, and the A/B update mechanism.
Does not depend on anything else in this repository (see
[../docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md) §3).

| Folder | Purpose |
|---|---|
| `boot/` | Bootloader config (A/B slot selection) and the Nova boot animation client |
| `init/` | OpenRC service scripts that bring up Nova Services |
| `image/` | SquashFS root + A/B partition image build recipes (full ISO and browser-demo variants) |
| `update/` | `update-agent`: OS update discovery, download, signature verification, slot switching |

Related: [ADR-0001](../docs/decisions/ADR-0001-linux-base-distribution.md),
[ADR-0002](../docs/decisions/ADR-0002-init-and-service-supervision.md),
[ADR-0008](../docs/decisions/ADR-0008-filesystem-and-update-strategy.md).
