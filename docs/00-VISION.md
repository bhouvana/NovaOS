# NovaOS — Vision & Product Goals

Status: v0.2 — direction changed · Owner: Chief Architect · Last updated: 2026-07-19

> **This document replaces the v0.1 vision below.** The prior direction — a from-scratch
> desktop platform (custom compositor, custom UI toolkit, custom SDK, custom package
> format) running on top of an unmodified Linux base — has been abandoned. See §1 for why
> and §7 for what happens to the work already done under that direction.

## 1. What NovaOS Is Now

**NovaOS is Tiny Core Linux, booted, configured, and curated as a real, complete,
working desktop operating system.** It is not a from-scratch reimplementation of a
desktop OS. The engineering effort goes into building and curating a real, modified
Tiny Core boot image — kernel, drivers, the real X11/window-manager desktop stack, a
chosen set of real applications — not into writing a new compositor, toolkit, or SDK
to replace what Tiny Core (and Linux) already provide.

**Why the change**: the original plan called for building NovaOS's own desktop layer —
compositor, window manager, UI toolkit, SDK, app suite, package format — from scratch in
Rust, using Linux only as a kernel underneath. That work reached a real, proven
milestone (a working Wayland compositor, `desktop/compositor`, booting bare-metal under
QEMU/KVM via DRM/KMS, with its own shell rendering live). But a custom compositor with
two demo buttons is not "a whole Linux OS with all its biggest features" — real
terminal, real window manager, real application ecosystem, years of battle-testing.
Reproducing that from scratch was never a realistic goal for this project. Tiny Core
already has all of it, real and working. NovaOS's job is to make that a good, curated,
purpose-built system — not to reinvent it.

**Non-goals** (explicit, in addition to §7 below): forking or modifying the Linux
kernel's scheduler, memory manager, or network stack beyond driver-level changes needed
for boot; replacing Tiny Core's real window manager, terminal, or core utilities with
custom-built equivalents.

## 2. Why NovaOS Exists (revised)

Tiny Core Linux is real, tiny, fast, and complete — but it ships as a generic,
un-opinionated base meant to be assembled by whoever installs it. NovaOS exists to do
that assembly work once, well, and curated: a specific, coherent, purpose-built desktop
experience built on Tiny Core's real components, with a specific kernel configuration,
a specific application set, and specific defaults — not a blank base the user has to
configure themselves, and not a reimplementation of what already works.

## 3. Product Goals (ranked, revised)

1. **A real, working boot** — Tiny Core Linux, with whatever kernel/driver
   modifications are needed, booting reliably under QEMU/KVM and (eventually) real
   hardware. Proven for the first time 2026-07-19 (see [README.md](../README.md) for
   current status).
2. **Curation over construction** — a coherent, deliberately-chosen set of Tiny Core's
   real packages (window manager, terminal, applications), not everything available,
   and not custom-built replacements for any of it.
3. **Low resource footprint** — inherited directly from Tiny Core's own tiny footprint;
   no longer a target NovaOS has to engineer for separately.
4. **Honest, verifiable proof** — every milestone demonstrated by a real boot and a
   real screenshot, not a design document. This has not changed from the prior
   direction and remains the standing bar.

## 4. Target Environments

- Virtual machines: QEMU/KVM — primary development and proof target, real and working.
- Real hardware: x86_64, not yet attempted — the logical next step once the QEMU image
  is stable.
- Browser deployment (novaos.dev, WASM x86 emulation): deferred indefinitely — not a
  current goal.

## 5. Success Criteria (current definition of done)

- A modified Tiny Core boot image (kernel + initramfs) that boots reliably under
  QEMU/KVM into a real, usable desktop — done, see [README.md](../README.md) for
  current status.
- A curated, deliberate application and window-manager choice (not necessarily
  flwm/wbar/aterm long-term — those were the first real proof, not a final commitment).
- Boots on real hardware from the same or an equivalent image.
- Whatever branding/theming work NovaOS wants to layer on top of Tiny Core's real
  desktop (wallpaper, icons, defaults) — cosmetic curation, not new software.

## 6. Design Philosophy (unchanged)

Every decision is evaluated against, in order: **simplicity, maintainability,
consistency, performance, developer experience, low memory, beautiful UX, modularity.**
This didn't change with the direction shift — if anything, abandoning the from-scratch
desktop platform is this philosophy applied at the largest possible scale: prefer a
boring, well-understood, already-working technology (Tiny Core's real desktop stack)
over a novel one (a custom compositor/toolkit/SDK) that hadn't earned its complexity.

## 7. What Happened To The Prior Direction's Work

Removed from the working tree 2026-07-19, immediately after this vision was rewritten
— the user made clear this was a full direction change, not "keep both": `desktop/`
(nova-compositor, a real Smithay-based Wayland compositor with a proven bare-metal
DRM/KMS backend, and nova-shell), `sdk/` (nova-ui, nova-ui-wayland, nova-app),
`apps/` (hello, hello-gui, nova-files), `services/` (nova-bus, nova-bus-broker),
`tests/`, `tools/`, `web/`, and the full Phase 1/1.5 documentation set (`docs/01-*`
through `docs/14-*`, `docs/specs/`, `docs/rfcs/`, `docs/IMPLEMENTATION-NOTES/`, and
ADRs 0002–0010) are all gone from the working tree. None of it is hypothetical — it was
real, built, and proven (see the git history immediately before the removal commit for
the full record) — but it is not recoverable from the current working tree, only from
git history, if it's ever needed again. See [README.md](../README.md) for what
actually remains and what's next.
