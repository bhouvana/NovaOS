# RFC-0007: Settings Service

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

"The Settings Service" is a **logical subsystem with no single owning process** — it is
the composition of `nova-settings-api` (client-side, in every app via the SDK), a
routing rule at the Nova Bus broker, and two backing stores split by data ownership:
`nova-themed` for appearance keys ([RFC-0006](RFC-0006-theme-service.md)) and
`nova-sessiond` for everything else system-scoped (network passthrough config, session
policy, security-relevant settings). Per-app settings
([06-NOVA-SDK-SPEC.md](../specs/06-NOVA-SDK-SPEC.md) §7) are not part of this service at
all — they live entirely in the requesting app's own `nova-storage` scope. This RFC
documents the split explicitly because it is the one core "service" in this document set
that is not a single binary, and that fact needs to be a documented decision, not a
discovered surprise.

## Responsibilities

- Provide one client API (`nova-settings-api`) regardless of which backing store a given
  key actually lives in — callers never need to know the split.
- Route each `nova.settings.write`/`read` call to the correct backing store based on a
  static key-prefix table (§Public APIs).
- Enforce that only Nova Settings (the app) can write system-scoped keys
  ([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2's `ipc_topics` grant model).

## Dependencies

`novabusd`, `nova-themed`, `nova-sessiond` (the two backing stores).

## Public APIs

`sdk/nova-settings-api`'s `get`/`get_system`/`set`
([06-NOVA-SDK-SPEC.md](../specs/06-NOVA-SDK-SPEC.md) §5) compile down to
`nova.settings.read`/`nova.settings.write {key, value}` Nova Bus calls. Routing table
(fixed, not dynamically extensible — a new system settings category requires a code
change to this table, not a runtime registration mechanism, keeping the security-
critical routing decision auditable in source):

| Key prefix | Routed to | Write-restricted to |
|---|---|---|
| `nova.theme.*` | `nova-themed` | `dev.novaos.settings` only |
| `nova.session.*` | `nova-sessiond` | `dev.novaos.settings` only |
| `nova.network.*` | `nova-sessiond` (thin wrapper over kernel networking config) | `dev.novaos.settings` only |
| anything else | rejected — `get_system`/system-scope `set` only recognizes the prefixes above | n/a |

## Events Published

None directly — publishes are the backing stores' responsibility
(`nova.theme.changed` from [RFC-0006](RFC-0006-theme-service.md); an analogous
`nova.session.settings_changed` from `nova-sessiond` for non-theme system keys).

## Events Consumed

None — this is a routing layer, not a subscriber.

## Configuration

The routing table (§Public APIs) is compiled into the Nova Bus broker's ACL logic, not
loaded from an external config file — see §Security Considerations for why this is
deliberate.

## Startup Order

No independent startup — functional as soon as `novabusd`, `nova-themed`, and
`nova-sessiond` are all up (the last of the "core four" to become available in the boot
sequence, [03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md) §1).

## Failure Modes

- **Routing table gap** (a key prefix an app expects isn't in the table): `get_system`/
  system `set` returns a clear `UNKNOWN_KEY` error, never silently no-ops — an app
  author discovers the gap immediately in testing rather than shipping a feature that
  quietly does nothing.
- **Backing store down** (`nova-themed` or `nova-sessiond` crashed): the relevant key
  range becomes temporarily unavailable; `nova-settings-api` surfaces this as a
  transient error, not a permanent one, since [RFC-0006](RFC-0006-theme-service.md)/
  [RFC-0008](RFC-0008-session-manager.md) both auto-restart.

## Recovery Strategy

Inherits the recovery strategy of whichever backing store owns the failed key range —
no separate recovery logic exists for "the settings service" as such, reinforcing that
it isn't a separate failure domain.

## Metrics

Read/write call volume by key prefix (useful for spotting an app hammering settings
reads it should be caching/subscribing to instead).

## Logging

System-scoped writes are logged at `info` with the writing `app_id` (always
`dev.novaos.settings` in a correctly functioning system — a system-scoped write logged
from any other `app_id` is a signal worth alerting on, §Security Considerations).

## Security Considerations

The write restriction ("Nova Settings only") is enforced at the Nova Bus broker ACL
using the same `ipc_topics` grant mechanism as any other permission
([../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2) — no app other than
`dev.novaos.settings` can ever be granted write access to `nova.settings.write.system`
in its manifest (Nova Store review policy, once external publishing exists post-v1,
must treat a request for this grant as an automatic rejection — noted here so the
constraint is visible to whoever eventually builds that review tooling,
[14-ECOSYSTEM-VISION.md](../specs/14-ECOSYSTEM-VISION.md)). The routing table being
compiled-in rather than runtime-configurable (§Configuration) means there is no config
file an attacker with filesystem write access to a lower-trust location could edit to
redirect system settings writes to an unintended handler.

## Changelog

- 2026-07-18: Accepted.
