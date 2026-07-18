# RFC-0008: Session Manager

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

`nova-sessiond` is the single authority for app lifecycle, sandboxing, and session state
(login/lock/logout/shutdown/suspend) — the busiest and highest-trust Nova service after
Nova Bus itself. Full behavior in
[../03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md) §6; sandbox construction
in [ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md); session state machine in
[16-STATE-MACHINES.md](../specs/16-STATE-MACHINES.md) §3.

## Responsibilities

- Resolve, sandbox-construct, and launch apps ([01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md)
  §1).
- Track liveness, apply crash-restart policy.
- Own login/lock/logout/shutdown/suspend state.
- Own the permission grant store (in cooperation with `permission-broker`).
- Resource accounting (cgroup-v2) per app, feeding Nova Monitor.
- Register/unregister apps with the Launcher index on install/uninstall
  ([RFC-0004](RFC-0004-package-service.md) Events Published →
  [RFC-0001](RFC-0001-nova-shell.md) Events Consumed, with `nova-sessiond` as the
  intermediary that validates the manifest before forwarding the registration event).

## Dependencies

`novabusd`, `permission-broker`, the kernel sandboxing primitives (namespaces, seccomp,
Landlock, cgroups-v2 — [ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md)).

## Public APIs

`nova.session.launch {app_id}`, `nova.session.terminate {app_id}`,
`nova.session.lock {}` / `unlock {credential}`, `nova.session.shutdown {}` /
`suspend {}`, `nova.session.register_app {manifest}` /
`unregister_app {app_id}` — all request/response.

## Events Published

`nova.session.starting`, `nova.session.ready` (boot milestones,
[01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §8),
`nova.session.app_started` / `app_exited`, `nova.session.window_list_changed`,
`nova.session.locked` / `unlocked`, `nova.session.stats` (periodic, cgroup accounting —
consumed by [09-APPLICATION-SPECS.md](../specs/09-APPLICATION-SPECS.md) "Nova Monitor").

## Events Consumed

`nova.wm.window_mapped` / `window_unmapped` (from Nova WM,
[RFC-0003](RFC-0003-nova-wm.md) Events Published — feeds `window_list_changed`),
`nova.wm.ready` (boot sequencing, triggers `nova-shell` launch,
[01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §8), permission grant/deny
decisions from `permission-broker`.

## Configuration

Crash-restart retry count/window (compiled default: 3 attempts within 60s, overridable
in `/nova/config/system.toml` for development builds only — production images ship the
default fixed), session lock timeout (user-configurable via Nova Settings,
[RFC-0007](RFC-0007-settings-service.md)).

## Startup Order

Second Nova service to start, immediately after `novabusd`
([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md) §1) — everything downstream (Nova
WM, Nova Shell, every app) is launched *by* `nova-sessiond`, directly or transitively.

## Failure Modes

- **Crash**: catastrophic — no app can be launched, terminated, or crash-recovered while
  down; existing running apps keep running (they don't depend on `nova-sessiond` staying
  alive for their own operation, only for lifecycle *transitions*) but the desktop
  becomes unable to open anything new.
- **Sandbox construction failure for a specific app** (e.g., a Landlock rule the kernel
  rejects): that single launch fails with a clear error
  ([01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md) §1 Failure paths) —
  isolated to that app, never brings down `nova-sessiond` itself.
- **cgroup accounting failure** (rare kernel-level issue): degrades Nova Monitor's
  per-app stats to "unavailable" rather than failing app launch — resource accounting is
  observability, not a launch precondition.

## Recovery Strategy

OpenRC restarts `nova-sessiond` on crash (system-service-supervised, like
`novabusd` — [ADR-0002](../decisions/ADR-0002-init-and-service-supervision.md)). On
restart, it re-discovers currently-running app processes (via a process-table scan
correlated against its own last-known-state checkpoint, persisted to
`/nova/data/session-state/` — [19-FILESYSTEM-LAYOUT-SPEC.md](../specs/19-FILESYSTEM-LAYOUT-SPEC.md))
so already-open windows aren't orphaned by a `nova-sessiond` restart, unlike a Nova WM
restart which does lose them ([RFC-0003](RFC-0003-nova-wm.md) Recovery Strategy) — the
asymmetry is intentional: `nova-sessiond` doesn't hold the rendering surfaces, so it can
reattach to live processes without them noticing.

## Metrics

App launch latency (by stage: manifest resolve, sandbox construct, execve, first-frame
— [02-MEMORY-BUDGET.md](../specs/02-MEMORY-BUDGET.md)/[../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md)
§6 both depend on this breakdown existing), crash/restart counts per app, active session
count, per-app cgroup CPU/RAM.

## Logging

Every launch/terminate (info, with full stage timing), sandbox construction denials
(warn, includes the specific permission/rule that failed), crash-restart events (error,
includes retry count against the policy threshold), lock/unlock events (info — security-
relevant audit trail, never includes the credential itself).

## Security Considerations

The single highest-trust service after Nova Bus: it constructs every app's sandbox
([ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md)) and owns the permission
grant store, so a bug here can under-sandbox an app (weakening isolation) or
over-trust a permission request. The manifest-to-sandbox-rule mapping
(same one referenced in [ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md)
Consequences) is exercised by a dedicated CI test suite against every in-tree manifest
([../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stage 7) specifically
because this service's correctness *is* the security boundary between apps. Session
lock/unlock credential verification never happens client-side — `nova-sessiond` itself
verifies the credential against the secrets store
([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §5), never delegating that check to
a UI-layer component that could be bypassed by talking to Nova Bus directly.

## Changelog

- 2026-07-18: Accepted.
