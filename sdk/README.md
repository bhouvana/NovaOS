# sdk/

Status: **Not yet implemented** — Phase 1 (architecture only). Skeleton (`nova-ui`
"hello world") in Phase 2, full v1.0.0 SDK by end of Phase 4
([../docs/12-ROADMAP-AND-MILESTONES.md](../docs/12-ROADMAP-AND-MILESTONES.md) §3–§5).

The Nova SDK — the **only** thing `apps/*` are allowed to depend on
([../docs/02-REPOSITORY-STRUCTURE.md](../docs/02-REPOSITORY-STRUCTURE.md) §3 rule 1).
Treated as a product with its own versioning and stability guarantees. Full design in
[../docs/04-APPLICATION-FRAMEWORK-AND-SDK.md](../docs/04-APPLICATION-FRAMEWORK-AND-SDK.md).

| Crate | Purpose |
|---|---|
| `nova-ui/` | Widgets, layout, theming, accessibility — see [06-DESIGN-SYSTEM.md](../docs/06-DESIGN-SYSTEM.md). [ADR-0005](../docs/decisions/ADR-0005-ui-toolkit.md) |
| `nova-app/` | App entrypoint trait, window/lifecycle API |
| `nova-storage/` | Per-app sandboxed storage (key-value, files, secrets) |
| `nova-notify/` | Notification publishing client |
| `nova-settings-api/` | Read/subscribe to system and own-app settings |
| `nova-clipboard/` | Clipboard + drag-and-drop |
| `nova-plugin/` | Plugin/extension hosting and authoring API |

All SDK crates are released in lockstep as one semver "SDK version"
([11-CODING-STANDARDS.md](../docs/11-CODING-STANDARDS.md) §10). Every public item must
carry a doc comment (`#![warn(missing_docs)]`, enforced in CI).
