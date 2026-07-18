# Spec 09 — Application Specifications

Status: Draft v0.1 · Last updated: 2026-07-18

Per-app design docs for every first-party app in `apps/`
([../02-REPOSITORY-STRUCTURE.md](../02-REPOSITORY-STRUCTURE.md)). Each section is scoped
to what's needed before implementation starts, not a full product spec — deeper feature
lists are refined during each app's Phase 4/5 milestone
([../12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md)).

## Nova Files

- **Purpose**: primary filesystem browser and file management.
- **Window model**: `multi_window = true` — supports several open windows sharing one
  process (§06-NOVA-SDK-SPEC §3), each an independent navigation context.
- **Views**: sidebar (Home/Downloads/Documents + pinned folders) + main pane
  (list/grid toggle) + optional preview pane.
- **Core interactions**: navigate, rename, delete (to a Trash concept backed by
  `nova-storage`, not immediate unlink), copy/move (drag-and-drop per
  [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §8), search (filename, in-scope only —
  no content indexing daemon in v1, consistent with the no-unnecessary-daemon
  principle), open-with (resolves to the app whose manifest declares a matching file-type
  handler extension, [../14-FUTURE-VISION.md](../14-FUTURE-VISION.md) §2).
- **Data model**: no local database — reads directly from its granted filesystem scopes
  (`home`, `downloads`) plus broker-mediated access to user-selected paths outside those.
- **Permissions**: `filesystem = ["home", "downloads"]`, `filesystem_user_selected = true`.
- **v1 scope**: local filesystem only. Network locations/cloud storage are
  [../14-FUTURE-VISION.md](../14-FUTURE-VISION.md)-class deferrals.

## Nova Terminal

- **Purpose**: shell access (BusyBox/standard POSIX shell inside the sandbox).
- **Window model**: single window, multi-tab (tabs are an in-app concept, not multiple
  OS-level windows — keeps window-manager interaction simple).
- **Core interactions**: standard terminal emulation (VT100-compatible escape sequences),
  tab management, copy/paste via [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §5.
- **Data model**: scrollback buffer per tab, bounded (configurable, default 10,000
  lines) — not persisted across restarts in v1.
- **Sandboxing note**: the shell process it spawns runs *inside Nova Terminal's own
  sandbox* (a child process, not a separate `nova-sessiond`-launched app) — so a shell
  command is bound by Terminal's granted filesystem scope, not an unbounded root shell.
  Developers needing broader access use an explicit, logged escape hatch (a "Developer
  Mode" toggle in Nova Settings that grants Terminal an expanded scope) rather than
  Terminal defaulting to unsandboxed.
- **Permissions**: `filesystem = ["home"]` by default; expanded under Developer Mode.

## Nova Browser

- **Purpose**: general web browsing — the one app permitted a heavier embedded engine
  ([ADR-0005](../decisions/ADR-0005-ui-toolkit.md) Consequences).
- **Window model**: `multi_window = true`, tabs within each window.
- **Engine**: embeds a WebKit-family engine as a declared `lib/` dependency inside its
  `.novapkg` ([07-PACKAGE-FORMAT-SPEC.md](07-PACKAGE-FORMAT-SPEC.md) §3) — evaluated at
  Phase 5 implementation time; chrome (tab bar, address bar, bookmarks) is built in Nova
  UI like any other app so it doesn't look like a foreign window inside NovaOS.
- **Permissions**: `network = true`, `filesystem = ["downloads"]`,
  `filesystem_user_selected = true` (file uploads).
- **v1 scope**: no extension/plugin ecosystem of its own in v1 (distinct from the
  system-level extension surface in
  [../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §6).

## Nova Settings

- **Purpose**: system configuration — display, theme, network, accounts, app
  permissions review, per-app settings (rendered generically from each installed app's
  schema, [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §7).
- **Window model**: single window, sidebar-categorized (Appearance, Network, Apps,
  Accounts, About).
- **Core interactions**: every write to a system-scoped setting goes through
  `nova-settings-api` to `nova-themed`/`nova-sessiond` per
  [01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §6 — Settings itself holds no
  special non-SDK privilege, only a manifest permission (`ipc_topics =
  ["nova.settings.write.system"]`) that no other app is granted, enforced at the Nova
  Bus broker ACL like any other permission.
- **Permissions**: the above `ipc_topics` grant, `filesystem = []` (no file access
  needed).

## Nova Monitor

- **Purpose**: system observability — CPU/RAM/process list, boot-time metrics (reads the
  boot ring buffer, [03-BOOT-TIMELINE.md](03-BOOT-TIMELINE.md) §4), per-app resource
  usage (cgroup accounting, [../03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md)
  §6).
- **Window model**: single window, tabbed views (Overview, Processes, Boot, Storage).
- **Data model**: polls `nova-sessiond`'s cgroup accounting + `/proc`-equivalent kernel
  interfaces at a fixed 1s interval while visible; stops polling when backgrounded
  (obeys its own `App::on_suspend`, [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §2 — even
  the monitoring tool respects the idle-CPU principle it's built to help enforce).
- **Permissions**: `ipc_topics = ["nova.session.stats"]` (a read-only stats topic
  `nova-sessiond` publishes), no filesystem access needed beyond its own storage.

## Nova Package Center

- **Purpose**: GUI client for install/update/remove/search
  ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md)).
- **Window model**: single window, catalog-browse + installed-apps views.
- **Data model**: renders the cached catalog fetched by `novapkg-agent`; itself holds no
  install logic (thin client over the agent, per
  [../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §3's
  "GUI and CLI both call the same agent" rule).
- **Permissions**: `ipc_topics = ["nova.package.*"]`, `network = true` (catalog/image
  fetch is delegated to the agent process, but Package Center itself needs network to
  fetch catalog *thumbnail images* directly from CDN for a responsive browse UI without
  round-tripping every asset through the agent).

## Nova Notes

- **Purpose**: plain-text and lightly-formatted note-taking/text editing.
- **Window model**: `multi_window = true`.
- **Core interactions**: create/edit/organize notes in folders, basic Markdown-style
  formatting rendered live, full-text search scoped to its own storage (no OS-wide
  search index dependency).
- **Data model**: notes stored as individual files in `nova-storage`'s `FileStore`
  ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §4), not a proprietary database — makes
  backup/export trivial (a user can find their notes as plain files if they ever need
  to, even though the app's own storage scope isn't directly user-Browse-able without
  going through Nova Files' broker-mediated access).
- **Permissions**: no filesystem grant beyond its own storage scope; export uses the
  broker-mediated save-file-picker path.

## Nova Paint

- **Purpose**: raster image editing.
- **Window model**: single window per open image (`multi_window = true`).
- **Core interactions**: the canvas area is a `Canvas` widget node
  ([05-NOVA-UI-TOOLKIT-SPEC.md](05-NOVA-UI-TOOLKIT-SPEC.md) §1) — immediate-mode drawing,
  the one deliberate exception to "every app is pure retained-mode Nova UI." Chrome
  (toolbar, color picker, layers panel) is ordinary Nova UI.
- **Extensibility**: filter plugins via `sdk/nova-plugin`'s app-defined `FilterPlugin`
  trait ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §9), sandboxed per
  [../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §6.
- **Permissions**: `filesystem = ["downloads"]`, `filesystem_user_selected = true`.

## Nova Calculator

- **Purpose**: standard + scientific calculator.
- **Window model**: single window, fixed-size (not resizable — a calculator's layout
  doesn't benefit from arbitrary resize; `resizable = false` in its manifest per
  [06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §3).
- **Core interactions**: standard/scientific mode toggle, history list.
- **Data model**: trivial — in-memory expression state, history persisted via
  `nova-storage`'s `KvStore`.
- **Permissions**: none beyond default sandbox (no filesystem, no network, no
  notifications) — the smallest-footprint app in the suite, and a useful "does the
  permission-less default path work at all" smoke test during Phase 4.

## Nova Arcade

Five sub-crates under `apps/nova-arcade/` ([../02-REPOSITORY-STRUCTURE.md](../02-REPOSITORY-STRUCTURE.md)),
each its own `.novapkg` (own manifest, own `app.id` like
`dev.novaos.arcade.chess`) rather than one monolithic "Arcade" binary — keeps
[../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §2's per-app launch
budget meaningful per game and lets a user install only the games they want via Package
Center, while `nova-arcade` as a folder groups them for shared scaffolding/build tooling.

| Game | Window model | Notable data model note |
|---|---|---|
| Chess | Single window, fixed aspect | Move-generation/search tables in-memory; `KvStore` for saved games |
| Snake | Single window, fixed aspect | Trivial grid-state model; high-score list in `KvStore` |
| Sudoku | Single window, fixed aspect | Puzzle generator + solver; puzzle library either bundled as an asset or generated on-demand |
| Minesweeper | Single window, fixed aspect | Trivial grid-state model; difficulty presets |
| Solitaire | Single window, fixed aspect | Card-game state machine; undo stack held in memory only |

All five: `resizable = false`, no filesystem/network/notification permissions beyond
`KvStore` access (implicit, not a manifest permission — every app gets its own storage
scope by default per [../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2's "unlisted
permissions denied, but own-storage access is not a listed permission, it's the
baseline"). Game boards render via a `Canvas` widget node like Nova Paint, chrome
(menus, score display, new-game button) is ordinary Nova UI.
