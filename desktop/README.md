# desktop/

Status: **Not yet implemented** — Phase 1 (architecture only). Skeleton in Phase 2,
full implementation in Phase 3
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §3–§4).

The desktop shell — the "Desktop Shell" layer in
[../docs/01-SYSTEM-ARCHITECTURE.md](../docs/01-SYSTEM-ARCHITECTURE.md) §1. Full design
in [../docs/03-DESKTOP-ARCHITECTURE.md](../docs/03-DESKTOP-ARCHITECTURE.md).

| Folder | Purpose |
|---|---|
| `compositor/` | `nova-compositor` — the Wayland compositor/window manager (wlroots-based) |
| `shell/` | `nova-shell` — Taskbar, Launcher, Notification Center (one process, three surfaces) |
| `settings/` | Nova Settings app — architecturally a normal sandboxed SDK app, pre-installed |

Related: [ADR-0003](../docs/decisions/ADR-0003-compositor-and-display-protocol.md). May
depend on `services/*` and `sdk/nova-ui`; never depended on by `apps/*`
([../docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md) §3 rule 3).
