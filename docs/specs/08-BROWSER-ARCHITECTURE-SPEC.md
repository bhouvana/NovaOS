# Spec 08 — Browser Architecture Specification

Status: Draft v0.1 · Last updated: 2026-07-18

The single most important spec in this tree per Staff Engineer review: concretizes
[../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) and
[ADR-0009](../decisions/ADR-0009-browser-boot-emulator.md) into the full client-side
stack and every I/O bridge between the browser tab and the emulated NovaOS guest.

## 1. Stack

```text
Browser tab (novaos.dev/demo)
   ↓
Site shell — React + TypeScript, Vite build (§1a: new decision, this doc)
   ↓
<canvas> element — v86's video output target
   ↓
v86 (WASM x86 emulator, ADR-0009), running in a Web Worker
   (kept off the main thread so emulation never janks the site UI
   or blocks on browser layout/paint)
   ↓
Virtual devices v86 exposes to the guest: PS/2 keyboard/mouse,
   virtio-blk (backs the guest disk image), a serial console,
   a 9p filesystem share (§4), optionally virtio-net (§7)
   ↓
Guest disk image: nova-browser-demo.img
   (SquashFS root, the browser-demo build profile from
   07-BROWSER-DEPLOYMENT.md §3 / 11-BUILD-PIPELINE-SPEC.md)
   ↓
NovaOS boots inside the emulated machine exactly as it would on
   real hardware or a real VM — same kernel, same OpenRC, same
   Nova Services, same compositor (software-rendering profile,
   02-MEMORY-BUDGET.md §3) — no browser-specific OS code path
```

### 1a. Site Frontend Framework (decision made in this doc)

Phase 1 didn't pick a frontend framework for `web/` — this is the one place Phase 1.5
had to make a call Phase 1 left open. **Decision: React + TypeScript, built with Vite,
deployed as a static SPA.** Rationale: React is the most broadly known choice for a
small team, has mature Web Worker/canvas integration patterns for exactly this kind of
"embed a heavy client-side runtime" use case, and Vite keeps the static-hosting story
(`web/` produces plain static assets, per
[../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) §5) simple. This is a
build-tooling choice, not an architecture-defining one — it doesn't warrant a numbered
ADR (no on-disk format, wire protocol, or resident-process implication per
[../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §8's ADR trigger list) but is
recorded here so it isn't re-litigated per PR.

### 1b. Strict Rendering Boundary (hard rule)

Per Staff Engineer review, this boundary is upgraded from implicit-in-the-architecture
to an explicit, binding rule that any future `web/` PR is checked against:

**React renders the website. It never renders anything that represents NovaOS's
desktop.** Concretely:

```text
React's rendering responsibility          NovaOS's rendering responsibility
────────────────────────────────          ──────────────────────────────────
Landing page, nav, docs, roadmap          Everything drawn inside <canvas>,
Loading screen (pre-boot only,             without exception, from the guest's
  §2) — a React component that             boot animation's first frame through
  disappears the instant the guest         every window, menu, cursor, and
  publishes its first framebuffer          dialog a visitor ever sees while
  frame, never reappears while the         interacting with the desktop
  guest is running
"Try it now" CTA, Install CTA link,       Nothing — these are React-rendered
  fullscreen toggle, save-state/           chrome buttons that call into the
  restart-demo buttons (page-level         Worker/canvas API (§6, §8) but never
  controls, 12-BROWSER-DEMO-               draw OS-looking UI themselves
  EXPERIENCE.md §2/§4)
```

The mechanical enforcement of this rule is that React has **no access to NovaOS's
design tokens, widget set, or any `nova-ui` code at all** — `web/` does not depend on
`sdk/nova-ui` or any other NovaOS source crate
([../02-REPOSITORY-STRUCTURE.md](../02-REPOSITORY-STRUCTURE.md) §3 rule 6 already
established "web/ depends only on build artifacts," which this rule is the rendering-
level consequence of). A React component literally cannot produce a NovaOS-styled
button, window, or icon because it has no way to reference
[10-DESIGN-BIBLE.md](10-DESIGN-BIBLE.md)'s tokens or `nova-ui`'s widgets — the
boundary is structural, not a convention a future contributor could accidentally cross
by importing the wrong thing.

Why this matters more than it might seem: the entire value proposition of the browser
demo is "this is the real OS, not a mockup"
([12-BROWSER-DEMO-EXPERIENCE.md](12-BROWSER-DEMO-EXPERIENCE.md) §1). The moment any
desktop-looking pixel comes from React instead of the guest's own compositor, that
claim becomes false for at least part of what a visitor sees, and NovaOS would be
committed to maintaining two desktop UI implementations in lockstep forever (one in
Rust/`nova-ui` for the real OS, one in React for the demo) — exactly the kind of
duplicated-implementation maintenance burden
[../00-VISION.md](../00-VISION.md) §6 rules out. Any future PR proposing a
React-rendered element that visually represents OS desktop content (a fake taskbar,
a React-drawn "loading your desktop" skeleton screen with app icons, etc.) is a
violation of this rule and requires an RFC to override it
([../rfcs/README.md](../rfcs/README.md)), not a quiet merge.

## 2. Boot Flow (browser-specific milestones layered onto 03-BOOT-TIMELINE.md)

```text
Page load: React shell mounts, shows branded loading screen
   ↓
Fetch v86 WASM binary + JS glue (small, cached aggressively via
   CDN + long-lived cache headers — changes only on v86 version bump)
   ↓
Fetch nova-browser-demo.img via HTTP range requests — v86 supports
   progressive/streamed disk reads, so full download completion is
   not a prerequisite for boot start (07-BROWSER-DEPLOYMENT.md §4)
   ↓
Spawn Web Worker, initialize v86 with: disk image (streamed),
   memory size (256MB guest RAM — generous relative to the
   64-100MB idle OS budget, leaving headroom for emulation
   overhead itself, which is host-side, not guest-visible RAM),
   boot device = virtio-blk
   ↓
v86 starts CPU emulation from the guest's BIOS/bootloader —
   from here, 03-BOOT-TIMELINE.md's sequence runs *inside* the
   emulated machine, unmodified
   ↓
Guest's boot animation DRM client renders → v86 captures the
   emulated framebuffer → Worker posts frame buffer to main
   thread → React shell blits to <canvas> (§3)
   ↓
Guest reaches "Desktop ready" (03-BOOT-TIMELINE.md) → React shell
   hides its own loading screen, hands input focus to <canvas>
   ↓
12-BROWSER-DEMO-EXPERIENCE.md's guided-tour flow begins
```

Target: page load → interactive desktop ≤ 15s
([../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §2), decomposed as:
≤3s asset fetch (WASM + enough of the disk image to start booting) + the guest's own
≤4.5s reference-hardware-class boot budget (v86 is slower than real hardware per-
instruction but the browser-demo image is smaller/lighter, roughly netting out — actual
number to be measured and fed back into this budget once Phase 2's browser-demo image
exists) + ≤2s worker/canvas setup overhead, with the remainder as margin.

## 3. Video Output

- v86 exposes the guest's emulated VGA/VBE framebuffer as a raw pixel buffer per frame.
- Worker thread posts changed-region buffers to the main thread via
  `postMessage`(transferable `ArrayBuffer`, zero-copy) at the guest's own repaint rate —
  which, per [04-WINDOW-MANAGER-SPEC.md](04-WINDOW-MANAGER-SPEC.md) §8's damage-driven
  scheduling, is only whenever the guest compositor actually has damage — the browser
  demo inherits the same "no animation, no render" idle-CPU property as real hardware.
- Main thread draws the buffer to `<canvas>` via `putImageData`/`ImageBitmap` — no WebGL
  required for this path since the guest is already using its own software-rendering
  profile ([02-MEMORY-BUDGET.md](02-MEMORY-BUDGET.md) §3); the canvas is purely a blit
  target, not a second rendering pipeline.

## 4. Keyboard

- Browser `keydown`/`keyup` events on the focused `<canvas>` are translated to PS/2
  scancodes and forwarded into the Worker → v86's emulated PS/2 controller.
- **Reserved-key handling**: `<canvas>` calls `event.preventDefault()` for keys the
  browser would otherwise intercept (F5 refresh, F11 fullscreen — handled instead by
  §6's in-page fullscreen control, Ctrl+W, Ctrl+T) while the canvas has focus, so typing
  inside the guest terminal or text editor behaves like a real keyboard. Escape is
  deliberately *not* captured — it remains the browser's/user's exit hatch if canvas
  focus-trapping ever misbehaves.
- Clicking outside the canvas releases keyboard capture; clicking back on it re-acquires
  — standard embedded-emulator UX, no custom focus-management protocol needed.

## 5. Mouse

- Uses the browser Pointer Lock API once the user clicks into the canvas, giving
  relative-motion deltas forwarded to v86's emulated PS/2 mouse (relative-motion mode,
  matching real hardware mice and the [04-WINDOW-MANAGER-SPEC.md](04-WINDOW-MANAGER-SPEC.md)
  §4 pointer model unmodified inside the guest).
- Exiting pointer lock (Escape, or browser-native UI) releases mouse capture; a visible
  "click to interact" overlay reappears, consistent with §4's canvas-focus model.
- Touch input (mobile visitors) is out of scope per
  [../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) §7's explicit non-goal —
  the canvas shows a "best viewed on desktop" notice rather than attempting a touch-to-
  mouse translation layer.

## 6. Fullscreen

Standard Fullscreen API wrapping the `<canvas>` element (`requestFullscreen()`), toggled
by an in-page button (not relying on the guest's own window manager, which has no
concept of the browser tab's chrome). Guest resolution is fixed at a single profile
(1280×720) rather than dynamically resizing with the browser window — dynamic resize
would require re-triggering the guest compositor's output reconfiguration on every
browser resize event, added complexity not justified for a demo experience; a fixed
resolution scaled via canvas CSS `object-fit: contain` for smaller viewports is simpler
and sufficient.

## 7. Clipboard & Downloads (host ↔ guest bridge)

Real hardware clipboard/download mechanisms don't exist inside an emulated x86 machine —
both are bridged through v86's **9p filesystem share** (a paravirtual filesystem
protocol v86 supports natively, requiring no custom guest kernel changes):

- A small NovaOS-side component, `nova-browser-bridge` (started only when booting under
  the v86 profile, detected via a kernel cmdline flag set by the browser-demo image
  build — never present in the real ISO), mounts the 9p share at
  `/nova/data/host-bridge/`.
- **Clipboard, guest → host**: an app calls `sdk/nova-clipboard`'s normal `write()` API
  ([06-NOVA-SDK-SPEC.md](06-NOVA-SDK-SPEC.md) §5); under the v86 profile,
  `nova-browser-bridge` additionally mirrors the written value to
  `host-bridge/clipboard.out`; the JS host side polls (or receives a 9p change
  notification) and calls the real browser `navigator.clipboard.writeText()`.
- **Clipboard, host → guest**: the reverse path — a JS-side paste action writes to
  `host-bridge/clipboard.in`, `nova-browser-bridge` publishes it onto
  `nova.clipboard.external_update`, and `sdk/nova-clipboard`'s `read()` surfaces it like
  any other clipboard write.
- **Downloads**: a virtual `Downloads` folder inside the guest is backed by the same 9p
  share; when a guest app writes a file there, `nova-browser-bridge` notifies the host
  JS, which triggers a real browser download (`Blob` + `<a download>`) — from the
  guest's perspective this is an ordinary filesystem write, no special download API
  exists in the SDK for it.

This bridge is the one NovaOS-side component that is aware it's running under emulation
— isolated to `nova-browser-bridge` alone, never leaking a "am I in a browser" branch
into the compositor, SDK, or any app, preserving
[../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) §6's "no component assumes
the browser environment" constraint at the granularity of "no component **other than
this one, explicitly optional, bridge**."

## 8. Persistence

- v86 supports serializing full machine state (CPU + RAM + device state) to a snapshot
  blob. On a "Save State" action (exposed in the
  [12-BROWSER-DEMO-EXPERIENCE.md](12-BROWSER-DEMO-EXPERIENCE.md) UI), the Worker
  produces this blob and the host JS persists it to `IndexedDB`, scoped to the origin
  (novaos.dev) — never uploaded anywhere, matching
  [../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) §6's local-only
  persistence and [../00-VISION.md](../00-VISION.md) §7's no-cloud-account requirement.
- "Restart Demo" discards the `IndexedDB` snapshot and re-boots from the pristine disk
  image — the explicit reset affordance called for in
  [12-BROWSER-DEMO-EXPERIENCE.md](12-BROWSER-DEMO-EXPERIENCE.md).
- No autosave-on-tab-close by default (avoids surprising a visitor with unexpectedly
  large `IndexedDB` usage); "Save State" is an explicit user action.

## 9. Networking

- **Default**: the guest's `network` capability is present in the manifest model but the
  virtio-net device is not attached by default — the demo boots and runs fully offline,
  matching [../07-BROWSER-DEPLOYMENT.md](../07-BROWSER-DEPLOYMENT.md) §6.
- **Optional "Connect" mode**: v86 supports a virtio-net device backed by a
  WebSocket-to-TCP relay. If enabled by explicit user action (a visible "Enable Network
  (Experimental)" toggle, off by default), the guest gets outbound network access
  through a public relay endpoint. This requires a small piece of backend
  infrastructure (the relay) that [ADR-0009](../decisions/ADR-0009-browser-boot-emulator.md)'s
  "no backend VM fleet" rationale does *not* forbid — a stateless WebSocket-to-TCP relay
  is a much smaller, cheaper, and more standard piece of infrastructure than a
  per-visitor VM, and remains entirely optional/off-by-default so the core "instantly
  accessible, static-hostable" demo experience is unaffected if the relay is unavailable
  or simply not enabled. Tracked as a Phase 6 stretch goal, not required for
  [../12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md) Phase 6 exit
  criteria.

## 10. Performance Notes

- The Worker-thread isolation (§1) is load-bearing for the ≤15s interactivity budget:
  keeping WASM execution off the main thread means the React shell's own paint/loading-
  screen work never competes with emulation for the main thread, and a slow-to-boot
  guest never produces a "page unresponsive" browser warning.
- `nova-compositor`'s software-rendering profile
  ([02-MEMORY-BUDGET.md](02-MEMORY-BUDGET.md) §3) was sized assuming v86-class emulated-
  CPU performance, not real hardware — this is the profile's primary intended
  consumer, with real low-end hardware as a secondary beneficiary
  ([../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md) §5).
