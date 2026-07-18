# ADR-0009: Browser Boot Emulator

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

novaos.dev must boot the *real* NovaOS ISO in a browser tab, not a simulated/fake UI. This
requires an x86 (or other target-arch) machine emulator that runs in the browser, fast
enough to be usable, and licensed compatibly with an open-source project.

## Options Considered

1. **Build a custom WASM emulator from scratch** — full control, but reimplementing CPU
   emulation, virtual devices (disk, VGA/framebuffer, network, input), and BIOS/boot
   plumbing is a multi-year effort orthogonal to NovaOS's actual mission; explicitly out
   of scope.
2. **v86** (open-source x86 emulator, C compiled to WASM via Emscripten, with a JS
   wrapper) — actively maintained, specifically designed to boot real x86 Linux images
   in-browser (proven with various minimal Linux distros), reasonable performance for a
   lightweight OS, permissive open-source license, small enough to self-host alongside
   our own assets.
3. **JSLinux / other browser x86 emulators** — similar concept, smaller community/less
   active maintenance than v86 at time of writing.
4. **Server-side VM streamed to browser (noVNC-style)** — technically simpler emulation
   story (real QEMU on a server, video-streamed to the client), but requires a persistent
   server-side VM per visitor, which is expensive to host at any scale, adds network
   latency to every interaction, and contradicts "instantly accessible" (needs a backend
   fleet, not a static/CDN-hosted demo).

## Decision

**v86** as the in-browser x86 emulator, loading the actual NovaOS ISO (or a size-optimized
browser-demo variant image built from the same source tree, per
[ADR-0001](ADR-0001-linux-base-distribution.md)'s note on a Buildroot-produced variant)
as a virtual disk. The novaos.dev site is a static/CDN-hostable web app (works on Render
or equivalent static hosting) bundling the emulator, the ISO/image asset, and
documentation — no backend VM infrastructure required.

## Rationale

v86 is the only option that satisfies "real OS image, not a fake UI" without taking on
either a multi-year emulator-authorship project or a costly per-visitor server-side VM
fleet. Because it's a static client-side emulator, hosting cost and complexity stay low
(a CDN, not a VM orchestration platform), which matches "instantly accessible" and the
low-infrastructure-overhead spirit of the whole project.

## Consequences

- Emulated performance is necessarily slower than native/KVM — the browser-demo image is
  explicitly allowed to be a trimmed-down variant (fewer preloaded apps, smaller asset
  set) optimized for in-browser boot time and responsiveness, distinct from the full
  installable ISO, as long as it is a real build from the same source, not a mockup.
  Divergence between the two images is tracked in
  [07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md).
- No 3D-accelerated graphics in the browser demo (v86 provides framebuffer-level video) —
  Nova UI's rendering layer must have a software-rendering fallback path for this target,
  which also benefits low-end real hardware without a capable GPU.
- Persistence (saved files/settings) inside the browser demo is local to the browser tab
  (e.g., via v86's state-save features) — not synced anywhere, consistent with the
  no-cloud-account-required principle.

## Revisit Triggers

- If v86 maintenance stalls or a materially faster/better-licensed alternative emerges.
- If the software-rendering fallback path required for v86 compatibility becomes a
  disproportionate maintenance burden on Nova UI.
