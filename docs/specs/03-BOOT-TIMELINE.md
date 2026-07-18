# Spec 03 — Boot Timeline

Status: Draft v0.1 · Last updated: 2026-07-18

Itemizes the budgets in [../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md)
§2 (≤5s reference VM, ≤8s reference hardware) against the milestone sequence in
[../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §2 and the message-level
detail in [01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §8. Firmware/POST time
before the bootloader gets control is hardware/vendor-dependent and outside NovaOS's
control; both timelines start counting at bootloader handoff.

## 1. Reference VM (QEMU/KVM, virtio-blk, virtio-gpu, OVMF)

| ms (start) | ms (end) | Stage | Detail |
|---|---|---|---|
| 0 | 60 | Bootloader (GRUB) | A/B slot selection, kernel + initramfs load from virtio-blk |
| 60 | 340 | Kernel decompress + early init | Console, memory init, early driver probes |
| 340 | 460 | Kernel late init + initramfs handoff | virtio-gpu/virtio-net/virtio-blk drivers attach |
| 460 | 540 | Root mount | SquashFS root + OverlayFS upper mount ([ADR-0008](../decisions/ADR-0008-filesystem-and-update-strategy.md)) |
| 540 | 560 | **Milestone: boot animation DRM client starts** | Minimal direct-DRM client (not the compositor) shows first branded frame — earliest possible visible feedback |
| 560 | 780 | OpenRC minimal runlevel | mdev, loopback + primary network interface up |
| 780 | 840 | `novabusd` starts | Unix socket bound, ready for connections |
| 840 | 920 | `nova-sessiond` starts | Connects to bus, publishes `nova.session.starting` |
| 920 | 1300 | `nova-compositor` starts | wlroots init, DRM/KMS modeset (virtio-gpu), GPU context — the single longest cold-start stage |
| 1300 | 1340 | **Milestone: compositor takes display** | Seamless flip from boot-animation client's buffer to compositor's first frame — no black frame |
| 1340 | 1620 | `nova-shell` starts | Connects to compositor + bus, builds initial widget tree (Taskbar, Launcher, Notification Center), font/icon atlas warm-up |
| 1620 | 1780 | `nova-themed` applies theme | Publishes token set, `nova-shell` re-themes if default differs from token defaults (usually a no-op first paint) |
| 1780 | 1850 | First desktop paint | `nova-compositor` composites `nova-shell`'s surfaces |
| 1850 | 2100 | Boot animation fade-out | 250 ms cross-fade per [10-DESIGN-BIBLE.md](10-DESIGN-BIBLE.md) motion scale |
| — | **2100** | **Milestone: Desktop ready** | Total: **2.1 s**, 2.9 s of headroom against the 5 s VM budget |

## 2. Reference Hardware (mid-range x86_64 laptop, NVMe, integrated GPU)

| ms (start) | ms (end) | Stage | Detail |
|---|---|---|---|
| 0 | 250 | Bootloader (GRUB) | Real disk I/O + firmware call overhead vs. virtio |
| 250 | 900 | Kernel decompress + early init | Slower than VM: real ACPI parsing, more driver probing |
| 900 | 1500 | Kernel late init + initramfs handoff | Real GPU/Wi-Fi/storage driver attach (the widest variance point across hardware) |
| 1500 | 1650 | Root mount | NVMe, faster than typical VM virtio-blk in practice |
| 1650 | 1680 | **Milestone: boot animation starts** | |
| 1680 | 2100 | OpenRC minimal runlevel | Wi-Fi/Ethernet bring-up included here on real hardware |
| 2100 | 2180 | `novabusd` starts | |
| 2180 | 2300 | `nova-sessiond` starts | |
| 2300 | 3400 | `nova-compositor` starts | Real GPU driver (Mesa/DRM) init is the dominant real-hardware cost — 1.1 s vs. VM's 0.38 s |
| 3400 | 3450 | **Milestone: compositor takes display** | |
| 3450 | 3900 | `nova-shell` starts | Real disk read latency for font/icon assets vs. VM's cached path |
| 3900 | 4150 | `nova-themed` + first paint | |
| 4150 | 4500 | Boot animation fade-out | |
| — | **4500** | **Milestone: Desktop ready** | Total: **4.5 s**, 3.5 s of headroom against the 8 s hardware budget |

## 3. Boot Animation Handoff Detail

The boot animation is not a fixed-duration splash — it is a real DRM/KMS client
([../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §2) that:

1. Opens `/dev/dri/card0` directly (no compositor yet exists) at the milestone in §1/§2
   row "boot animation DRM client starts."
2. Renders a looping brand animation, advancing frames based on wall-clock time, not
   milestone arrival — so it never looks "stuck" waiting on a slow stage.
3. On receiving `nova.wm.ready` (published by `nova-compositor` once it owns the
   display), performs a **buffer handoff**: the compositor's first composited frame is
   flipped in at the next vblank using the same CRTC/plane the animation client held —
   guaranteeing no intervening black frame, matching the "polished," non-jarring bar set
   in [../01-SYSTEM-ARCHITECTURE.md](../01-SYSTEM-ARCHITECTURE.md) §2.
4. Exits only after receiving `nova.session.ready` and completing its fade-out, so a
   slow `nova-shell` startup extends the animation rather than revealing an empty desktop.

## 4. Instrumentation

Every named milestone above emits a monotonic-clock timestamp to the fixed-size boot ring
buffer defined in
[../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §3. The Phase 2 CI
performance gate ([../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stage 5)
asserts both that milestones fire in the order listed here and that "Desktop ready"
lands within budget on the reference VM image — the reference-hardware timeline is
validated at the manual test pass
([../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §4), not in CI, since CI runs
on VM infrastructure.

## 5. Biggest Lever If a Budget Is Missed

`nova-compositor` startup (GPU context + modeset) is the single largest stage in both
timelines (38% of VM boot time, 24% of hardware boot time). Any future boot-time
regression investigation should look here first before micro-optimizing earlier stages.
