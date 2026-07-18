# Spec 19 — Filesystem Layout Specification

Status: Draft v0.1 · Last updated: 2026-07-18

The complete on-disk directory structure, consolidating paths referenced piecemeal
throughout this doc tree into one authoritative reference, per Staff Engineer review.

## 1. Top-Level Layout

```text
/                      Read-only SquashFS root (A/B slot, ADR-0008) — the base
                        system + all compiled Nova service/compositor/shell/
                        pre-installed-app binaries. Never written to at runtime.
/nova/                 Writable, persists across OS updates (lives outside the
                        A/B root image entirely — a separate data partition)
  apps/                 Installed .novapkg mounts
    <app_id>/
      <version>/         Read-only SquashFS mount, one per installed version
      current -> <version>/   Symlink to the active version
  data/                 Per-app writable storage
    <app_id>/
      kv.db               nova-storage KvStore backing file
      files/               nova-storage FileStore root
      secrets.enc          nova-storage SecretStore, encrypted at rest
      plugins/              App plugin directories (18-PLUGIN-ARCHITECTURE-SPEC §2)
    session-state/          nova-sessiond's crash-recovery checkpoint
                            (RFC-0008 Recovery Strategy)
    host-bridge/            9p share mount point, browser-demo profile only
                            (08-BROWSER-ARCHITECTURE-SPEC §7)
  config/                 System configuration (20-CONFIGURATION-STRATEGY-SPEC)
    system.toml             Global config: update channel, hostname, etc.
    theme.toml               Active theme selection (RFC-0006)
    trusted-keys/             Ed25519 public keys for package/update signature
                              verification (08-SECURITY-MODEL.md §4)
  cache/                  Reclaimable, safe to delete entirely, regenerated
                          on demand
    package-catalog/         Cached Nova Store catalog (RFC-0004)
    font-atlas/               Rasterized font/icon atlas cache (nova-ui)
  logs/                   Structured logs (21-OBSERVABILITY-SPEC), rotated
/home/<user>/            User files — the filesystem scope granted to apps
                          via the `home` permission (08-SECURITY-MODEL.md §2)
/run/nova/               tmpfs, cleared every boot
  bus.sock                 Nova Bus broker socket (15-NOVA-BUS-PROTOCOL-SPEC §1)
  wayland-0                 Compositor's Wayland socket
/boot/                  Bootloader + kernel + initramfs for both A/B slots
                        (ADR-0008)
```

## 2. Ownership & Access Rules

| Path | Owner (writer) | Readers | Persistence |
|---|---|---|---|
| `/` | Image builder only (build-time), never a runtime writer | All processes | Replaced wholesale on OS update, never patched in place |
| `/nova/apps/<id>/<version>/` | `novapkg-agent` (mount/unmount only — contents are read-only even to the owning app) | The owning app (read-only), Nova Shell (manifest read for Launcher index) | Survives OS updates; removed on app uninstall |
| `/nova/data/<app_id>/` | The owning app exclusively (enforced by the sandbox's mount namespace — no other app's sandbox even has this path mounted) | Owning app only | Survives app updates; removed on uninstall unless "keep data" confirmed ([../05-PACKAGE-AND-UPDATE-SYSTEM.md](../05-PACKAGE-AND-UPDATE-SYSTEM.md) §2) |
| `/nova/config/` | `nova-sessiond`/`nova-themed`/Nova Settings only (§RFC-0007's write-restriction model) | All Nova services (read) | Survives OS updates |
| `/nova/config/trusted-keys/` | Image builder (initial keys) + signed OS updates only (§08-SECURITY-MODEL §4) — never writable by any runtime app or even by `novapkg-agent`/`update-agent` themselves | `novapkg-agent`, `update-agent` (read-only) | Survives OS updates; rotated only via signed update |
| `/nova/cache/` | The service that owns each subdirectory | Same service | May be cleared at any time with no data loss — a service must regenerate on cache-miss, never treat cache absence as an error |
| `/nova/logs/` | Each service writes its own log file | Nova Monitor, log rotation tooling | Rotated per [21-OBSERVABILITY-SPEC.md](21-OBSERVABILITY-SPEC.md) §1 retention policy |
| `/home/<user>/` | The user (via apps granted the `home` permission) | Apps with `home` or `filesystem_user_selected` grants | Survives everything except explicit user deletion |
| `/run/nova/` | `novabusd`, `nova-compositor` (socket creation) | All Nova processes (connect only) | Cleared every boot — nothing here is ever assumed to persist |

## 3. Why `/nova/` Instead of Scattering Across FHS Paths

A traditional Linux distro spreads equivalent state across `/etc`, `/var`, `/usr`,
`/home` following the Filesystem Hierarchy Standard. NovaOS deliberately consolidates
everything it owns under one `/nova/` root instead:

- **One mount point to reason about** for what survives an OS update (everything under
  `/nova/` plus `/home/`) vs. what's replaced wholesale (everything under `/`) —
  directly serves [ADR-0008](../decisions/ADR-0008-filesystem-and-update-strategy.md)'s
  A/B model, where "is this path part of the image or part of the data partition" must
  be an instant, unambiguous answer.
- **One tree to back up/inspect** for a user or Nova Monitor wanting to see "everything
  NovaOS-specific on this machine."
- FHS compliance is deliberately not a goal — NovaOS does not aim for scripts written
  against a standard Linux FHS layout to work unmodified, consistent with
  [../00-VISION.md](../00-VISION.md) §7's non-goal that NovaOS isn't binary/script-
  compatible with generic Linux desktop tooling by design.

## 4. Sandboxed App View

An app's mount namespace ([ADR-0010](../decisions/ADR-0010-app-sandboxing-model.md))
never sees the full tree above — only:

```text
/nova/data/<own_app_id>/    read-write
/nova/apps/<own_app_id>/current/   read-only (its own installed files)
/home/<user>/                       read-write, ONLY if `home` permission granted
/home/<user>/Downloads/              read-write, ONLY if `downloads` permission granted
(user-selected paths)                read-only or read-write per grant, mounted
                                      in on-demand by the broker at picker-selection
                                      time, never a static mount
```

Everything else in §1 — other apps' `/nova/data/`, `/nova/config/trusted-keys/`, `/boot/`
— is simply not present in the mount namespace at all, not merely permission-denied.
This is the concrete filesystem-level mechanism behind
[../08-SECURITY-MODEL.md](../08-SECURITY-MODEL.md) §2's permission taxonomy.

## 5. Browser-Demo Profile Note

Under the `browser-demo` build profile
([11-BUILD-PIPELINE-SPEC.md](11-BUILD-PIPELINE-SPEC.md) §2), `/nova/data/` and
`/home/` live on a RAM-backed filesystem inside the v86 guest (there is no real disk to
persist to inside the emulator beyond the read-only disk image) — state surviving across
a demo session is exclusively via the host-bridge/v86-snapshot mechanism in
[08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md) §8, not via anything
written to `/nova/data/` persisting independently of that snapshot.
