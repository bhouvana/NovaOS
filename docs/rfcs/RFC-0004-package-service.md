# RFC-0004: Package Service

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

`novapkg-agent` is the only component that installs, updates, and removes `.novapkg`
apps. Package Center and the `novapkg` CLI are both thin clients over it
([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §3). Byte-level
format in [07-PACKAGE-FORMAT-SPEC.md](../specs/07-PACKAGE-FORMAT-SPEC.md); state machine
in [16-STATE-MACHINES.md](../specs/16-STATE-MACHINES.md) §2.

## Responsibilities

- Fetch and cache the Nova Store catalog.
- Download, verify (checksum + signature), mount, and register `.novapkg` packages.
- Track installed app versions and manage upgrade/removal.
- Garbage-collect superseded versions after a health-check grace period
  ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §2).

## Dependencies

`novabusd`, network access (catalog/package fetch — absent in offline/sideload mode,
[../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §6),
`nova-sessiond` (app registration, [01-INTERACTION-FLOWS.md](../specs/01-INTERACTION-FLOWS.md)
§5), the trusted keyring at `/nova/config/trusted-keys/`
([19-FILESYSTEM-LAYOUT-SPEC.md](../specs/19-FILESYSTEM-LAYOUT-SPEC.md)).

## Public APIs

Nova Bus method calls: `nova.package.install {app_id}`, `nova.package.update {app_id}`,
`nova.package.remove {app_id, keep_data: bool}`, `nova.package.search {query}`,
`nova.package.list_installed {}` — all request/response, all also exposed 1:1 as
`novapkg` CLI subcommands ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md)
§3).

## Events Published

| Topic | Payload | When |
|---|---|---|
| `nova.package.install_progress` | `{app_id, state, percent}` | Each state-machine transition ([16-STATE-MACHINES.md](../specs/16-STATE-MACHINES.md) §2) |
| `nova.package.install_complete` | `{app_id, version}` | Install/update finished successfully |
| `nova.package.install_failed` | `{app_id, error_code, reason}` | Verification or install failure |
| `nova.package.catalog_updated` | `{}` | Catalog cache refreshed |

## Events Consumed

None on the install/update path (it is invoked directly, not event-triggered) —
subscribes to nothing; it is purely a request/response service from other components'
perspective.

## Configuration

Update channel/catalog URL (`/nova/config/system.toml`,
[20-CONFIGURATION-STRATEGY-SPEC.md](../specs/20-CONFIGURATION-STRATEGY-SPEC.md) §2),
catalog cache location (`/nova/cache/package-catalog/`,
[19-FILESYSTEM-LAYOUT-SPEC.md](../specs/19-FILESYSTEM-LAYOUT-SPEC.md)), trusted keyring
path (fixed, not configurable — a configurable trust root would be a security
anti-pattern).

## Startup Order

Not resident ([../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §3) — started
on demand by the first `nova.package.*` call or Package Center launch, exits after an
idle timeout (5 minutes with no active operation and no open Package Center window).

## Failure Modes

- **Corrupted download**: caught by checksum verification before signature check
  ([07-PACKAGE-FORMAT-SPEC.md](../specs/07-PACKAGE-FORMAT-SPEC.md) §4) — reported as
  `install_failed` with `error_code: CHECKSUM_MISMATCH`, safe to retry.
- **Invalid signature**: `install_failed` with `error_code: SIGNATURE_INVALID` — never
  silently proceeds, no "install anyway" path in the GUI
  ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §4).
- **Disk full mid-install**: rolled back to pre-install state (partial SquashFS mount
  torn down, no half-installed app left registered).
- **Interrupted mid-download** (network loss, agent killed): resumable on retry via
  HTTP range requests against the same catalog-provided URL; no partial state left
  registered with `nova-sessiond` until install fully completes.

## Recovery Strategy

Every failure mode above rolls back to the pre-operation state — the state machine
([16-STATE-MACHINES.md](../specs/16-STATE-MACHINES.md) §2) has no state that leaves an
app half-registered. User-facing failures surface via
`nova.package.install_failed` → Package Center shows the error and offers Retry; no
automatic retry loop (avoids repeatedly hammering a failing download).

## Metrics

Install/update/remove counts and durations, catalog fetch latency, verification failure
rate (a spike is a potential Store-compromise signal, monitored the same way as Nova
Bus's ACL-denial spike, [RFC-0002](RFC-0002-nova-bus.md) §Metrics).

## Logging

Every install/update/remove attempt (info, includes `app_id` + version), verification
failures (warn — security-relevant), disk/network errors (error). Never logs the
signing key material itself, obviously, nor full package byte contents.

## Security Considerations

The trust boundary described in full in
[07-PACKAGE-FORMAT-SPEC.md](../specs/07-PACKAGE-FORMAT-SPEC.md) §4 and
[../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §4 — this service is the sole
enforcement point for "only signed, trusted-key packages get installed." The
`--allow-unsigned` developer escape hatch
([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §4) is CLI-only,
never exposed in Package Center's GUI, and every use is logged at `warn` with a
persistent on-disk marker so a system with any unsigned-installed app is visibly
distinguishable in Nova Monitor's security overview (not a silent, un-auditable bypass).

## Changelog

- 2026-07-18: Accepted.
