# RFC-0001: Nova Shell

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

Nova Shell is the desktop's visible chrome: Taskbar, Launcher, and Notification Center,
hosted in a single process. It does not manage windows (that's Nova WM,
[RFC-0003](RFC-0003-nova-wm.md)) and does not run app logic — it is a consumer of state
published by other services, rendered through Nova UI like any other app
([05-NOVA-UI-TOOLKIT-SPEC.md](../specs/05-NOVA-UI-TOOLKIT-SPEC.md)).

## Responsibilities

- Render the Taskbar (running/pinned apps, workspace indicator, system tray, clock).
- Render the Launcher (search-first app/settings launcher,
  [../03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md) §3).
- Render the Notification Center (toasts + history,
  [RFC-0005](RFC-0005-notification-service.md)).
- Maintain the Launcher's in-memory app index, rebuilt on install/uninstall events.

## Dependencies

`novabusd` (must be reachable before Nova Shell can do anything — every responsibility
above is bus-driven), `nova-compositor` (needs a Wayland socket to render into),
`nova-sessiond` (source of running-app state), `nova-themed` (source of theme tokens).

## Public APIs

No Nova Bus topics *served* by Nova Shell for other services to call (it is a
consumer, not a service other components depend on) — it exposes nothing over Nova Bus
except publishing on `nova.notify.dismiss`/`nova.notify.action` when a user interacts
with a notification ([RFC-0005](RFC-0005-notification-service.md) Public APIs).

## Events Published

| Topic | Payload | When |
|---|---|---|
| `nova.notify.dismiss` | `{notification_id}` | User dismisses a toast/history entry |
| `nova.notify.action` | `{notification_id, action_id}` | User clicks a notification action button |
| `nova.session.focus_request` | `{window_id}` | User clicks a Taskbar entry |
| `nova.session.launch` | `{app_id}` | User clicks a Launcher result |

## Events Consumed

| Topic | Reaction |
|---|---|
| `nova.session.window_list_changed` | Rebuild Taskbar entries |
| `nova.session.app_started` / `app_exited` | Update Taskbar running-app state |
| `nova.notify.publish` | Show toast, append to history ([RFC-0005](RFC-0005-notification-service.md)) |
| `nova.theme.changed` | Re-theme all three surfaces ([RFC-0006](RFC-0006-theme-service.md)) |
| `nova.session.register_app` / `unregister_app` | Rebuild Launcher index |

## Configuration

Taskbar pin list and Launcher's recently-used ranking are user preferences, stored via
Nova Shell's own `nova-storage` scope (Nova Shell is architecturally a privileged
session process, not a sandboxed app, but still uses the same storage API for
consistency — [20-CONFIGURATION-STRATEGY-SPEC.md](../specs/20-CONFIGURATION-STRATEGY-SPEC.md)
§3). No system-wide config file — nothing here is relevant to any process but Nova Shell
itself.

## Startup Order

Started by `nova-sessiond` immediately after receiving `nova.wm.ready`
([01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §8) — after Nova Bus,
Session Manager, and the compositor, before any user-launched app.

## Failure Modes

- **Crash**: the desktop has no Taskbar/Launcher/notifications, but existing app windows
  remain usable (Nova WM doesn't depend on Nova Shell staying alive) — a degraded but
  not total-failure state.
- **Slow start**: extends the boot animation ([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md)
  §3) rather than showing an incomplete desktop.
- **Launcher index corruption** (e.g., a malformed app manifest): that one app is
  skipped from the index with a logged warning, never a Launcher-wide failure.

## Recovery Strategy

`nova-sessiond` restarts a crashed Nova Shell automatically, bounded retry (3 attempts,
matching [../03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md) §6's general app
crash policy applied to this privileged process too) — on exhausting retries, a
minimal fallback: the compositor shows a plain background with no chrome and logs a
critical error, rather than looping forever.

## Metrics

Launcher index size, notification queue depth, Taskbar entry count, render frame time
for the three surfaces — published on `nova.monitor.metrics` per
[21-OBSERVABILITY-SPEC.md](../specs/21-OBSERVABILITY-SPEC.md) §2.

## Logging

App launch/focus requests (info), notification delivery (info), Launcher index rebuild
failures (warn), crashes (error, with the standard context fields from
[../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §5). Never logs notification
*body content* verbatim at info level (may contain user data) — only metadata (sender
app_id, timestamp) unless debug logging is explicitly enabled.

## Security Considerations

Nova Shell is a privileged session process (not sandboxed like a regular app,
[../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §7 notes first-party apps are
sandboxed identically to third-party — Nova Shell is the one deliberate,
documented exception, because it must be able to render the Taskbar/Launcher/
Notification Center for *all* apps, which an ordinary per-app sandbox scope cannot
express). Because of this, Nova Shell's own code is held to the same review bar as
`nova-sessiond` and the compositor — a bug here has full-desktop-visibility blast
radius, even though it cannot escalate to kernel/filesystem access beyond what its own
process privileges allow.

## Changelog

- 2026-07-18: Accepted.
