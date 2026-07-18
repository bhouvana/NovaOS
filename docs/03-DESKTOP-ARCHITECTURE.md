# NovaOS — Desktop Architecture

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Related: [ADR-0003](decisions/ADR-0003-compositor-and-display-protocol.md),
[ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)

## 1. Components

The desktop shell is deliberately two processes, not five:

- **`nova-compositor`** (`desktop/compositor/`) — the Wayland compositor/window manager
  itself: surface management, input routing, window decoration, workspaces, animations,
  multi-monitor/HiDPI, gesture handling.
- **`nova-shell`** (`desktop/shell/`) — a single process hosting three UI surfaces
  (Taskbar, Launcher, Notification Center) built with Nova UI like any other app, but
  granted shell-level permissions. Kept as one process rather than three because they
  share state (open windows, app list, notification queue) and splitting them would mean
  re-deriving that shared state over IPC for no benefit.

`nova-settings` (`desktop/settings/`) ships with the OS and is architecturally a regular
sandboxed app (uses only the SDK) that happens to be pre-installed and pinned to the
taskbar — it is not special-cased in the compositor or shell.

## 2. Window Manager Model

- **Compositing protocol**: Wayland core + a small set of Nova-specific protocol
  extensions (analogous to how Sway/GNOME add compositor-specific protocols) for things
  the core protocol doesn't cover: window snapping hints, workspace switching, the
  permission-prompt surface type.
- **Window model**: floating by default with snap zones (halves/quarters, ChromeOS/
  Windows-style), plus an optional tiling mode toggle per workspace — not two competing
  paradigms, one model with a per-workspace switch.
- **Decorations**: client-side decorations drawn by Nova UI (consistent title bar/controls
  across all apps, per the design system —
  [06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md)) rather than server-side, so every app's
  chrome is pixel-identical without per-app opt-in.
- **Animations**: compositor-owned (window open/close/minimize, workspace switch) —
  budgeted against the FPS/latency targets in
  [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md), never blocking input.
- **Multi-monitor & HiDPI**: per-output scale factors, Nova UI renders at the output's
  native scale (no bitmap upscaling blur) — a hard requirement, not a stretch goal, since
  it's cheap to get right from the start and expensive to retrofit.

## 3. Launcher

- Single search-first entry point (type-to-search across installed apps, Package Center
  results, and system actions/settings — one unified index, not separate "app search" and
  "settings search").
- Opened via a keybinding and a taskbar affordance; renders as a `nova-shell` surface.
- Backed by an in-memory index built from installed app manifests
  ([ADR-0007](decisions/ADR-0007-package-format.md)); no background indexing daemon —
  rebuilt on install/uninstall events delivered over Nova Bus.

## 4. Taskbar

- Shows: launcher affordance, running/pinned app list, workspace indicator, system tray
  (network/battery/volume via thin Nova Services wrappers), clock, notification center
  affordance.
- Running-app state comes from Nova Session Manager (§6), not from the compositor
  directly — the taskbar doesn't need to know window-manager internals, only "what apps
  are alive."

## 5. Notification Service

- A Nova Bus topic (`nova.notify`) that any app may publish to via `sdk/nova-notify`
  (subject to the sandboxing/permission model — apps must declare the notification
  capability).
- `nova-shell`'s Notification Center subscribes and renders: toast on arrival, persistent
  history list, do-not-disturb state (itself a Nova Bus-published setting other
  components can read, e.g., to suppress toasts during fullscreen apps/games).
- No separate notification daemon process — folded into `nova-shell` since it's purely a
  rendering/history concern once the bus delivers the message; this keeps the "always
  resident" process list in
  [01-SYSTEM-ARCHITECTURE.md](01-SYSTEM-ARCHITECTURE.md) §3 short.

## 6. Session Management

`nova-sessiond` (`services/nova-sessiond/`) is the single authority for:

- Launching an app: resolve its `.novapkg` mount, construct its sandbox (namespaces,
  seccomp, Landlock per [ADR-0010](decisions/ADR-0010-app-sandboxing-model.md)), start the
  process, register it with the compositor and taskbar.
- Tracking liveness: crash detection, restart-on-crash policy (bounded retries, then
  surface a notification — never silently loop-restart).
- Login/lock/logout/shutdown/suspend flow: a single state machine, exposed to
  `nova-shell` for the lock screen and power menu UI, but owned here so there is one
  place that knows "is the session locked."
- Resource accounting per app (cgroups-v2), feeding Nova Monitor
  ([04-APPLICATION-FRAMEWORK-AND-SDK.md](04-APPLICATION-FRAMEWORK-AND-SDK.md)).

## 7. Theming

`nova-themed` (`services/nova-themed/`) owns the active theme's token set (colors,
spacing, radii — see [06-DESIGN-SYSTEM.md](06-DESIGN-SYSTEM.md)) and publishes it over
Nova Bus; Nova UI subscribes and re-themes live, no app restart required for a light/dark
switch. Themes are data (a signed token file), not code — third-party themes are possible
without executing untrusted logic.

## 8. What Is Explicitly Not Here

- No separate "panel" vs "dock" vs "system tray" processes — consolidated into
  `nova-shell` per §1's rationale.
- No desktop-icons-on-wallpaper model — file management happens in Nova Files; the
  desktop background is not a drop target for files in v1 (revisit only if strong user
  demand emerges post-v1).
