# ADR-0008: Root Filesystem & Update Strategy

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

Updates to the base OS (not user apps — see [ADR-0007](ADR-0007-package-format.md)) must
be reliable on consumer hardware with no IT department: a failed or interrupted update
must never leave the machine unbootable.

## Options Considered

1. **Traditional in-place package upgrades (apk/apt-style, mutable root)** — familiar, but
   partial-update failure states are a real, historically common failure mode; no clean
   rollback without extra tooling.
2. **OSTree (image-based, git-like content-addressed root)** — proven (used by Fedora
   Silverblue/CoreOS), atomic and rollback-capable, but its object-store model and tooling
   are a sizeable dependency to adopt for a small team, and it's designed for a more
   general update-anything model than we need.
3. **A/B partition images (read-only root, dual slot, atomic slot-switch on update)** —
   the ChromeOS/SteamOS/Android model: download a full (or delta) new root image to the
   inactive slot, verify it, switch the boot flag, reboot; previous slot remains as
   instant rollback.

## Decision

**Read-only SquashFS root filesystem, A/B partition scheme, with an OverlayFS writable
layer for user/local state.** Updates download a new root image to the inactive slot,
verify its signature, mark it active, and reboot into it; the previous slot is retained
as an automatic rollback target if the new slot fails a post-boot health check.

## Rationale

This directly matches the proven, boring choice used by every cohesive consumer OS in our
reference set (ChromeOS, SteamOS, Android) and is the option that best serves reliability
without novel infrastructure: no content-addressed object store to build/operate, just two
partitions and a signature check. It is easier to reason about, test, and explain than
OSTree, which matters for a small team and for [00-VISION.md](../00-VISION.md) §6's
simplicity-first priority.

## Consequences

- Root filesystem is immutable at runtime; anything a user changes (settings, files,
  installed apps) lives in the OverlayFS upper layer / dedicated `/nova/data` partition,
  never in the A/B image itself.
- Full-image updates are simple but larger than a delta-only scheme; binary-diff delta
  updates between adjacent versions are a planned optimization, not a v1 requirement.
- Disk usage is roughly 2x the root image size (two slots) — an explicit, accepted
  tradeoff for update safety, and small in absolute terms given the RAM/size targets.
- Package Center app installs ([ADR-0007](ADR-0007-package-format.md)) are independent of
  A/B slot updates and are not affected by an OS rollback (apps live outside the root
  image, in `/nova/apps`).

## Revisit Triggers

- If real-world image sizes make the 2x-disk cost a genuine problem on low-end target
  hardware, prioritize delta updates earlier than planned.
