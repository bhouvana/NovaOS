# RFC-0003: Nova WM (Compositor)

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

`nova-compositor` is the Wayland compositor and window manager: it owns the display,
composites every app's surface, applies window-management policy, and renders the
system-trusted permission-prompt overlay. Full behavioral spec in
[04-WINDOW-MANAGER-SPEC.md](../specs/04-WINDOW-MANAGER-SPEC.md); this RFC is the
service-operational contract.

## Responsibilities

- DRM/KMS display ownership and modesetting.
- Wayland protocol server (core + Nova-specific extensions,
  [ADR-0003](../decisions/ADR-0003-compositor-and-display-protocol.md) Consequences).
- Window lifecycle, focus, z-order, damage tracking, frame scheduling
  ([04-WINDOW-MANAGER-SPEC.md](../specs/04-WINDOW-MANAGER-SPEC.md) §1–§8).
- Rendering the permission-prompt trusted surface
  ([01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §4).
- Lazily starting XWayland on first X11 client connection
  ([ADR-0003](../decisions/ADR-0003-compositor-and-display-protocol.md) Consequences).

## Dependencies

`novabusd` (for publishing window events and receiving focus requests), kernel DRM/KMS
subsystem, wlroots (linked library, not a separate process).

## Public APIs

Wayland protocol sockets (one per running app, handed out by `nova-sessiond` at launch,
[01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §1) — not a Nova Bus topic;
apps talk to the compositor directly over Wayland for surface/buffer management, and
over Nova Bus only for window-management *intents* that don't fit the Wayland protocol
model (§Events Consumed).

## Events Published

| Topic | Payload | When |
|---|---|---|
| `nova.wm.ready` | `{}` | Display ownership acquired, first frame composited ([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md) §1) |
| `nova.wm.window_mapped` / `window_unmapped` | `{app_id, window_id}` | Window lifecycle transitions ([01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §2) |
| `nova.wm.focus_changed` | `{window_id}` | Focus stack top changes |

## Events Consumed

| Topic | Reaction |
|---|---|
| `nova.wm.focus_request` | Move requested window to focus stack top ([04-WINDOW-MANAGER-SPEC.md](../specs/04-WINDOW-MANAGER-SPEC.md) §4) |
| `nova.wm.create_trusted_surface` | Render the permission-prompt overlay ([01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §4) — the only caller permitted is `permission-broker`, enforced at the Nova Bus ACL |
| `nova.theme.changed` | Re-render window-chrome elements that use theme tokens (title bar colors, etc.) |

## Configuration

Output configuration (resolution, scale factor, arrangement for multi-monitor) —
persisted via `nova-storage` under the compositor's own scope, written through Nova
Settings' Display page ([09-APPLICATION-SPECS.md](../specs/09-APPLICATION-SPECS.md)
"Nova Settings"), read at compositor startup and on `nova.settings.write` events for
live changes.

## Startup Order

Starts after `nova-sessiond` ([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md) §1),
publishes `nova.wm.ready` once display ownership is established — every other
UI-rendering component (`nova-shell`, then apps) blocks on this event.

## Failure Modes

- **Crash**: catastrophic for the visible desktop — every app loses its rendering
  surface simultaneously (analogous blast radius to Nova Bus, §RFC-0002 Failure Modes,
  but for pixels instead of messages).
- **GPU driver failure/hang**: detected via a bounded frame-completion timeout
  ([04-WINDOW-MANAGER-SPEC.md](../specs/04-WINDOW-MANAGER-SPEC.md) §8); falls back to
  the software-rendering path ([02-MEMORY-BUDGET.md](../specs/02-MEMORY-BUDGET.md) §3)
  rather than hanging indefinitely, with a logged warning.
- **Client (app) sends a malformed Wayland request**: that client's connection is
  terminated (app crash from the app's perspective, handled by
  `nova-sessiond`'s normal crash policy,
  [../03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md) §6) — never brings down
  the compositor itself.

## Recovery Strategy

`nova-sessiond` restarts a crashed compositor (bounded retry, same 3-attempt policy as
[RFC-0001](RFC-0001-nova-shell.md)); on restart, all app windows are lost (Wayland
surfaces don't survive a compositor restart) — apps receive a disconnect and are
relaunched fresh by `nova-sessiond` rather than attempting session/window restoration,
which is out of v1 scope. On exhausting retries: boot fails to reach "Desktop ready,"
surfaced via the boot ring buffer ([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md)
§4) as a hard boot failure — no degraded desktop is possible without a compositor.

## Metrics

Frame time (p50/p99), damage region count per frame, dropped-frame count, GPU/software
render-path indicator, per-output refresh rate, window count.

## Logging

Modeset events (info), client connect/disconnect (info), frame-budget overruns (warn,
[../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §5), GPU driver errors
(error), malformed-client-request terminations (warn, includes the offending `app_id`
for correlation with `nova-sessiond`'s crash log).

## Security Considerations

Owns the one surface type (permission prompts) no app can spoof
([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §3) — the `nova.wm.create_trusted_surface`
ACL restriction to `permission-broker` alone is the load-bearing security property of
the entire permission-prompt flow; a bug allowing any other caller to trigger this
surface type would let a malicious app render a fake "Allow" prompt. Input routing
(§04-WINDOW-MANAGER-SPEC §4) must never deliver input events to a window other than the
one currently focused, especially during a trusted-surface prompt (§04-WINDOW-MANAGER-SPEC
§4's forced-focus behavior) — an input-routing bug here is a clickjacking-class
vulnerability.

## Changelog

- 2026-07-18: Accepted.
