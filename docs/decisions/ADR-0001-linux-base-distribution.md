# ADR-0001: Base Linux Distribution

Status: Accepted (Revised)
Date: 2026-07-18
Deciders: Chief Architect

## Context

NovaOS runs on top of the Linux kernel but must not become "a desktop environment you
install on top of distro X." We need a base that provides: the kernel, a libc, a minimal
userland, and a driver/firmware story broad enough for real hardware — while being small
enough to hit a 64–100 MB idle RAM budget and simple enough that NovaOS fully owns the
user-visible layer.

## Options Considered

1. **Build from scratch (LFS-style)** — total control, but multi-year effort to reach
   hardware/driver parity; unsustainable for a small team.
2. **Buildroot** — purpose-built for minimal, fully custom embedded-style images; produces
   very small, fixed-purpose systems; weaker out-of-the-box breadth of desktop hardware
   drivers/firmware packages than a general distro; slower iteration loop (full image
   rebuilds) for a desktop-scale project with frequent app-layer changes.
3. **Debian/Ubuntu base (minimal/netboot)** — excellent hardware/driver coverage and
   package ecosystem to bootstrap against, but glibc + apt + dpkg + default services carry
   more baseline RAM/disk than our budget tolerates unless aggressively stripped, and
   "aggressively stripped Debian" tends to bit-rot as upstream changes.
4. **Alpine Linux** — musl libc, BusyBox userland, apk package manager, OpenRC init,
   designed from the start to be minimal (base image ~5 MB). Strong track record as a base
   for other minimal distros (postmarketOS). Uses a standard (if slightly older-kernel-lag)
   linux-lts kernel package with reasonable firmware coverage via `linux-firmware`.

## Decision

**Alpine Linux** as the base layer (kernel + musl libc + minimal userland), consumed as a
build-time dependency, not a runtime identity. NovaOS's own init, services, compositor,
and apps sit on top; `apk` is used only to pull base-system packages during image build,
not exposed to end users — end users only ever see Nova Package Center
([ADR-0007](ADR-0007-package-format.md)). **Superseded 2026-07-18, see Revision below.**

## Revision (2026-07-18): Alpine → Tiny Core Linux

The Chief Architect restated NovaOS's end goal explicitly: build the whole OS on
**Tiny Core Linux**, with the UI/UX robustness of macOS. This is a base-layer change, not
an app-layer one — evaluated with the same rigor as the wlroots→Smithay revision
([ADR-0003](ADR-0003-compositor-and-display-protocol.md)), since a base distro is far
harder to change later than a library dependency.

**What's actually different from Alpine**, checked against Tiny Core's real documented
architecture rather than assumed:

- **Size**: Alpine's base image (~5 MB) is smaller than Tiny Core's smallest bootable
  variant, "Core" (~17 MB, 28 MB RAM minimum; "Tiny Core" with a GUI toolkit is 23 MB).
  Tiny Core's win over Alpine isn't raw base-image size.
- **The real difference is the execution model**: Tiny Core is designed to run
  **entirely from RAM** by default — extensions (`.tcz` files, squashfs-based) are loaded
  as symlinks into a RAM-resident system rather than installed onto a normal persistent
  root filesystem the way Alpine (or a typical Linux install) works. This is a genuinely
  different boot/persistence paradigm, not just "an even smaller Alpine," and it's the
  part that needs real design work: how NovaOS's update model
  ([ADR-0007](ADR-0007-package-format.md)'s A/B slots) and
  [19-FILESYSTEM-LAYOUT-SPEC.md](../specs/19-FILESYSTEM-LAYOUT-SPEC.md) map onto a
  RAM-resident base — Tiny Core does have a more traditional "TCE/CopyFS" install mode as
  an alternative to its default RAM-resident mode, which is the more likely fit and needs
  to be the explicit target, not assumed compatible by default.
- **What transfers unchanged from the original Alpine decision**: Tiny Core is consumed
  the same way Alpine was going to be — "a build-time dependency, not a runtime
  identity." Tiny Core's own default desktop (FLTK/FLWM) is irrelevant, exactly as
  Alpine's bare userland was always going to be fully replaced by NovaOS's own
  init/compositor/UI stack. This part of the original rationale needed no rework.
- **Genuinely unverified, flagged rather than guessed**: Tiny Core's libc (glibc vs.
  musl) and how cleanly NovaOS's init/service-supervision model
  ([ADR-0002](ADR-0002-init-and-service-supervision.md)) sits on top of Tiny Core's boot
  sequence instead of Alpine's OpenRC-compatible one. Both need a real build/boot spike
  to answer, not a documentation guess — and that spike is currently
  `status:blocked-on-environment` (no QEMU/bootable-kernel testing available on the
  current dev machine, see [[novaos-build-env]]), so this ADR update is a decision
  record, not yet a proven implementation, unlike the Smithay revision which was proven
  the same day it was decided.

## Rationale (superseded, kept for history — see Revision above for the current decision)

Alpine gives the smallest credible baseline (musl + BusyBox) with the least amount of
"fighting the base distro to remove things," which directly serves the RAM budget and the
simplicity principle ranked above performance in [00-VISION.md](../00-VISION.md) §6.
Buildroot was close second and remains the fallback for the embedded/browser-demo image
variant (see Consequences) because its from-source, no-package-manager model produces the
smallest possible fixed image.

## Consequences

- musl instead of glibc means some upstream binary-only Linux software (rare for our own
  stack, which we control) may need musl compatibility shims or a compat runtime layer —
  acceptable since NovaOS's own components are built from source against musl.
- ~~We gain Alpine's security update cadence for the base image "for free."~~
- ~~We inherit OpenRC as a natural init choice, reinforced independently in
  [ADR-0002](ADR-0002-init-and-service-supervision.md).~~ ADR-0002 needs its own revisit
  once Tiny Core's boot/init model is spiked — tracked as a Revisit Trigger below, not
  silently assumed compatible.
- The browser/VM demo image (small, fixed-purpose) may additionally use a trimmed
  Buildroot-produced variant if the Tiny-Core-based ISO proves too large for comfortable
  web delivery; both paths share the same NovaOS userland source tree.

## Revisit Triggers

- If musl compatibility friction consumes disproportionate engineering time (pending
  confirmation of Tiny Core's libc).
- If Tiny Core's kernel/hardware support proves too narrow for NovaOS's real hardware
  targets — Tiny Core's driver/firmware breadth has not been evaluated against the same
  bar Alpine was held to in the original Options Considered below.
- If idle RAM measurements (see [09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md))
  show the base layer itself, not our services, is the dominant cost.
- **New**: if a real build/boot spike shows Tiny Core's RAM-resident default model
  fights NovaOS's update/persistence design more than the TCE/CopyFS install mode
  resolves — revisit whether Alpine (or Buildroot) was actually the better fit once this
  is provable rather than theoretical.
