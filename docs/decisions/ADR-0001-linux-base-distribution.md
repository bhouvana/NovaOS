# ADR-0001: Base Linux Distribution

Status: Accepted
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
([ADR-0007](ADR-0007-package-format.md)).

## Rationale

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
- We gain Alpine's security update cadence for the base image "for free."
- We inherit OpenRC as a natural init choice, reinforced independently in
  [ADR-0002](ADR-0002-init-and-service-supervision.md).
- The browser/VM demo image (small, fixed-purpose) may additionally use a trimmed
  Buildroot-produced variant if the Alpine-based ISO proves too large for comfortable
  web delivery; both paths share the same NovaOS userland source tree.

## Revisit Triggers

- If musl compatibility friction consumes disproportionate engineering time.
- If Alpine's kernel lag blocks hardware support NovaOS needs (e.g., recent laptop
  Wi-Fi/GPU).
- If idle RAM measurements (see [09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md))
  show the base layer itself, not our services, is the dominant cost.
