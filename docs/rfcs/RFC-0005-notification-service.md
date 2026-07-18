# RFC-0005: Notification Service

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

The notification service is a **logical subsystem hosted inside `nova-shell`**, not a
separate process — deliberate, per
[../03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md) §5's rationale (a
standalone notification daemon would be one more entry in the "always resident" ledger,
[../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §3, for no behavior that
requires isolation from the Taskbar/Launcher it's already coupled to at the UI level).
This RFC documents it as its own contract anyway, per the Staff Engineer review's request
for a per-service RFC, precisely because "implemented inside another process" should not
mean "undocumented as a subsystem."

## Responsibilities

- Receive published notifications, apply Do-Not-Disturb filtering, render toasts.
- Maintain persistent notification history.
- Route notification action-button clicks back to the originating app.

## Dependencies

`novabusd` only (it's a subscriber/consumer within the `nova-shell` process, sharing
that process's compositor connection — see [RFC-0001](RFC-0001-nova-shell.md)
Dependencies).

## Public APIs

None beyond the SDK-level `sdk/nova-notify` client
([06-NOVA-SDK-SPEC.md](../specs/06-NOVA-SDK-SPEC.md) §5) any app uses to send a
notification — from a calling app's perspective, "the notification service" *is* the
`nova.notify.publish` topic; there's no separate query API (no "get my notification
history" call — history is Notification Center's own UI-local state, not queryable by
other apps, since a notification's content may be sender-app-private).

## Events Published

`nova.notify.dismiss` / `nova.notify.action` — see
[RFC-0001](RFC-0001-nova-shell.md) Events Published (same process, listed there since
they're published by the shared `nova-shell` binary, not a distinct one).

## Events Consumed

| Topic | Reaction |
|---|---|
| `nova.notify.publish` | ACL-checked at the Nova Bus broker (sender must have the `notifications` permission, [../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2) before delivery; on receipt, check Do-Not-Disturb state, render toast, append to history |

## Configuration

Do-Not-Disturb state and per-app notification-muting preferences — user settings,
surfaced in Nova Settings, stored via `nova-shell`'s own `nova-storage` scope
([RFC-0001](RFC-0001-nova-shell.md) Configuration).

## Startup Order

Live as soon as `nova-shell` is (§RFC-0001 Startup Order) — no separate initialization,
since it's not a separate process.

## Failure Modes

- **`nova-shell` crash**: notifications published during the outage are lost (not
  queued by Nova Bus, [RFC-0002](RFC-0002-nova-bus.md) Failure Modes' at-most-once
  semantics) — an app that needs guaranteed delivery of critical information should not
  rely solely on a notification (this is a documented, accepted limitation, not a gap to
  close with a durable queue, per the no-unnecessary-complexity principle).
- **Malformed notification payload** (e.g., missing required fields): dropped with a
  warn-level log including the sending `app_id`, never crashes the shell.

## Recovery Strategy

Covered by [RFC-0001](RFC-0001-nova-shell.md) Recovery Strategy (the whole process
restarts together) — notification history persisted via `nova-storage` survives a
`nova-shell` restart; only in-flight/undelivered notifications during the crash window
are lost.

## Metrics

Notifications delivered/sec, DND-suppressed count, history size, action-click rate —
folded into `nova-shell`'s overall metrics report
([RFC-0001](RFC-0001-nova-shell.md) Metrics), tagged by subsystem for Nova Monitor to
break out separately.

## Logging

Delivery events log metadata only (sender `app_id`, timestamp, whether DND-suppressed)
— never notification body text at `info` level, per
[RFC-0001](RFC-0001-nova-shell.md) Logging's same rule, restated here because it's the
specific subsystem the rule was written for.

## Security Considerations

The `notifications` permission ACL check happens at the Nova Bus broker
([RFC-0002](RFC-0002-nova-bus.md) Security Considerations), not inside `nova-shell` —
this subsystem trusts that anything arriving on `nova.notify.publish` already passed
that check, and does not re-validate sender permissions itself (defense-in-depth would
suggest a second check, but the broker is the sole enforcement point by design,
[../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2, to avoid every consumer needing
its own copy of the ACL logic). Notification action IDs are opaque tokens defined by the
sending app, never interpreted or executed by the notification service itself — clicking
an action only ever re-publishes an event back to the sender
([RFC-0001](RFC-0001-nova-shell.md) Events Published), never triggers privileged
behavior on the notification service's own authority.

## Changelog

- 2026-07-18: Accepted.
