# RFC-0009: Update Service

Status: Accepted
Date: 2026-07-18
Owner: Chief Architect

## Purpose

`update-agent` discovers, downloads, verifies, and stages OS-level A/B updates — the
update domain distinct from app updates
([05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §1). State
machine in [16-STATE-MACHINES.md](../specs/16-STATE-MACHINES.md) §4.

## Responsibilities

- Periodically check the configured update channel for a newer signed root image.
- Download to the inactive A/B slot, verify signature/checksum.
- Coordinate the user-facing update prompt (via a notification,
  [RFC-0005](RFC-0005-notification-service.md)) or silent-apply-at-next-reboot per user
  preference.
- Trigger the bootloader slot switch and post-boot health check
  ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §5).

## Dependencies

`novabusd`, network access, the bootloader's slot-switch tooling
(`system/update/`, [../02-REPOSITORY-STRUCTURE.md](../02-REPOSITORY-STRUCTURE.md)), the
same trusted keyring `novapkg-agent` uses ([RFC-0004](RFC-0004-package-service.md)
Dependencies — shared verifier code, per
[../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §5).

## Public APIs

`nova.update.check {}` (manual "check now" trigger from Nova Settings),
`nova.update.apply {}` (user-confirmed "restart and update now").

## Events Published

`nova.update.available {version, size}`, `nova.update.download_progress {percent}`,
`nova.update.staged {version}` (ready, awaiting reboot), `nova.update.failed
{error_code}`.

## Events Consumed

None on its core path (periodic self-triggered check, not event-driven) — receives
`nova.update.check`/`apply` as direct calls (§Public APIs), not subscriptions.

## Configuration

Update channel (`stable`/`beta`, `/nova/config/system.toml`,
[20-CONFIGURATION-STRATEGY-SPEC.md](../specs/20-CONFIGURATION-STRATEGY-SPEC.md) §2),
check interval (default: every 12 hours, plus on-demand via `nova.update.check`),
silent-vs-prompted apply preference (Nova Settings, per-user).

## Startup Order

Not resident ([../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §3) — a
periodic wake (not a continuously running process), scheduled by a lightweight OpenRC
cron-equivalent, distinct from `novapkg-agent`'s on-demand wake pattern
([RFC-0004](RFC-0004-package-service.md) Startup Order) since update checks are
time-triggered rather than user-action-triggered.

## Failure Modes

- **Download/verification failure**: `nova.update.failed`, inactive slot left in a
  known-bad state, cleared before the next check attempt — never leaves a partially
  written slot that a subsequent slot-switch could accidentally activate.
- **Post-boot health check failure after applying** (the core A/B safety mechanism,
  [ADR-0008](../decisions/ADR-0008-filesystem-and-update-strategy.md)): bootloader
  reverts to the previous slot on the *next* boot automatically — this happens at the
  bootloader level, not inside `update-agent` itself (which isn't running yet at that
  point in boot), documented here because it's this service's responsibility to have
  correctly staged the health-check marker the bootloader reads.
- **Update available but user never confirms** (prompted mode): re-notified on a bounded
  schedule (once per day, not on every check interval) rather than nagging every 12
  hours.

## Recovery Strategy

A failed download/verification simply means "no update staged" — the system continues
running the current slot with no degradation (this is the entire point of the A/B
model, [ADR-0008](../decisions/ADR-0008-filesystem-and-update-strategy.md)). A failed
post-boot health check is self-recovering via the bootloader's automatic revert (§Failure
Modes) with no `update-agent` involvement required — by the time it matters,
`update-agent` hasn't started yet for that boot.

## Metrics

Check frequency/success rate, download duration, verification failure rate (shared
security-monitoring signal with [RFC-0004](RFC-0004-package-service.md) Metrics, since
both use the same verifier), time-to-apply after user confirmation.

## Logging

Every check attempt (info), download/verification failures (warn, security-relevant),
successful staging (info), slot-switch trigger (info, includes target version) — the
bootloader-level revert-on-failed-health-check is logged separately by the bootloader/
early-boot path, not by this service, and surfaced to Nova Monitor's boot history via the
boot ring buffer ([03-BOOT-TIMELINE.md](../specs/03-BOOT-TIMELINE.md) §4).

## Security Considerations

Shares its trust model entirely with
[../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §4 and
[RFC-0004](RFC-0004-package-service.md) Security Considerations — same signature
verification discipline, same "no bypass in the GUI" rule
([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §4). The one
addition specific to OS updates: because a compromised update has full-system blast
radius (unlike a compromised single app, which is sandboxed,
[ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md)), the trusted keyring entries
used for OS image verification are never the same keys used for `.novapkg` publisher
signatures (§Dependencies note "the same trusted keyring `novapkg-agent` uses" refers to
the *keyring mechanism/location*, not key reuse — the OS-image signing key and any given
app publisher's key are cryptographically distinct entries within that shared keyring
structure).

## Changelog

- 2026-07-18: Accepted.
