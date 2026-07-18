# apps/

Status: **Not yet implemented** — Phase 1 (architecture only). Core apps land in
Phase 4, Browser/Arcade in Phase 5
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §5–§6).

First-party Nova applications. Every app here consumes `sdk/*` **only** — never
`services/*` or `desktop/*` internals directly, and never another app's crate
([../docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md) §3 rule 1).
This is deliberate: first-party apps are held to the exact same SDK boundary as
third-party apps will be, so the SDK is proven by our own dogfooding, not bypassed by it.

| App | Purpose |
|---|---|
| `nova-files/` | File manager |
| `nova-terminal/` | Terminal emulator |
| `nova-notes/` | Text editor / notes |
| `nova-paint/` | Paint / raster editor |
| `nova-calculator/` | Calculator |
| `nova-monitor/` | System monitor (CPU/RAM/processes, boot-time metrics) |
| `nova-package-center/` | Package Center GUI client |
| `nova-browser/` | Web browser (the one app permitted to embed a third-party engine — [ADR-0005](../docs/decisions/ADR-0005-ui-toolkit.md) Consequences) |
| `nova-arcade/` | Chess, Snake, Sudoku, Minesweeper, Solitaire — sub-crates, first-class apps |

Each app ships a manifest declaring its permissions
([04-APPLICATION-FRAMEWORK-AND-SDK.md](../docs/04-APPLICATION-FRAMEWORK-AND-SDK.md) §4)
and is distributed as a signed `.novapkg`
([ADR-0007](../docs/decisions/ADR-0007-package-format.md)), even though these ship
pre-installed with the OS image.
