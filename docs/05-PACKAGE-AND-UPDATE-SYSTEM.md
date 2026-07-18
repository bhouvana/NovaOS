# NovaOS — Package System & Update System

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Related: [ADR-0007](decisions/ADR-0007-package-format.md),
[ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md)

## 1. Two Independent Update Domains

NovaOS deliberately keeps two update mechanisms separate, because they have different
risk profiles and cadences:

| | OS Update | App Update |
|---|---|---|
| Unit | Whole A/B root image | Single `.novapkg` |
| Cadence | Infrequent, larger | Frequent, small |
| Failure blast radius | Whole system (mitigated by A/B rollback) | Single app |
| Owner component | `update-agent` (`system/update/`) | `novapkg` (`services/novapkg/`) |

Conflating these (as many traditional package-manager-based distros do, where a kernel
update and a text editor update go through the same mechanism) is exactly the failure
mode [ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md) avoids.

## 2. Nova Package Center — User Flow

1. **Discover**: browse/search a catalog fetched from the Nova Store index (signed
   manifest list, cached locally, refreshed periodically — no polling daemon, refresh is
   triggered on Package Center open or explicit user action).
2. **Install**: download `.novapkg`, verify signature against the trusted key set
   (§4), mount read-only under `/nova/apps/<id>/<version>/`, register with
   `nova-sessiond` and the Launcher index.
3. **Update**: same as install for a new version; old version retained until the new one
   passes first-launch health check, then garbage-collected (mirrors the OS A/B rollback
   philosophy at app scale, without needing two full slots per app — only the previous
   version's SquashFS, small by comparison to a full OS image).
4. **Remove**: unmount, delete version directories, delete `nova-storage` data only on
   explicit "remove data too" confirmation (never silently on uninstall).

## 3. `novapkg` CLI

A scriptable equivalent of Package Center for developers/power users and for CI
(installing test builds): `novapkg install|update|remove|search|list <args>`. Package
Center (GUI) and `novapkg` (CLI) both call the same `novapkg-agent` library — no logic
duplicated between the two front ends.

## 4. Trust & Signing

- Nova Store catalog is signed with a Nova-held key; individual packages are signed by
  their publisher's key, which must be registered with Nova Store before a package is
  listed (publisher identity verification is a Nova Store policy concern, not an OS
  mechanism concern — kept out of this architecture doc).
- `novapkg-agent` refuses to install a package whose signature doesn't chain to a trusted
  key. No "install anyway" bypass in the GUI; an explicit, scarier CLI flag
  (`--allow-unsigned`, logged, intended for local development builds only) exists for
  developers building their own apps before publishing.
- Detailed permission/sandbox mapping lives in
  [08-SECURITY-MODEL.md](08-SECURITY-MODEL.md).

## 5. OS Update Flow

1. `update-agent` checks the configured update channel (stable/beta) for a new signed
   root image manifest — a periodic, low-frequency wake (see
   [09-PERFORMANCE-STRATEGY.md](09-PERFORMANCE-STRATEGY.md) for the "no unnecessary
   background polling" budget), not a resident daemon.
2. On finding a newer version: download to the inactive A/B slot, verify signature and
   image checksum.
3. Prompt the user (via notification service,
   [03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §5) to restart, or apply
   silently at next reboot depending on user preference in Nova Settings.
4. On reboot: bootloader switches the active slot; a post-boot health check (does the
   compositor start, does `nova-sessiond` reach "ready") must pass within a bounded time
   window, or the bootloader automatically reverts to the previous slot on the *next*
   boot — no user intervention required to recover from a bad update.
5. Once the new slot is confirmed healthy (successful login), it becomes the sole
   candidate for the *next* update's inactive slot.

## 6. Offline & Air-Gapped Support

Both `novapkg` and `update-agent` support a local/offline mode: sideloading a
`.novapkg` or an update image from removable media, verified with the same signature
chain as the networked path — no reduced security guarantee for offline installs. This
matters for the browser-demo/VM audience who may not want the demo instance reaching the
network at all (see [07-BROWSER-DEPLOYMENT.md](07-BROWSER-DEPLOYMENT.md)).

## 7. What Is Explicitly Out of Scope (v1)

- Delta/binary-diff updates (full images/packages only in v1 — see
  [ADR-0008](decisions/ADR-0008-filesystem-and-update-strategy.md) Consequences).
- Multi-user package installs with per-user isolation (v1 is single-primary-user, see
  [08-SECURITY-MODEL.md](08-SECURITY-MODEL.md)).
- Self-hosted alternate Nova Store catalogs (architecture allows for it via the signed
  catalog format, but tooling to run your own isn't a v1 deliverable).
