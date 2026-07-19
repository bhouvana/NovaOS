# ADR-0003: Compositor & Display Protocol

Status: Accepted (Revised)
Date: 2026-07-18
Deciders: Chief Architect

## Context

The window manager/compositor is the single most identity-defining piece of NovaOS — it
is what makes the OS feel like one thing rather than a Linux box running a WM. It must be
GPU-efficient, support modern input (touch, multi-monitor, HiDPI), and be small enough to
build and maintain with a small team.

## Options Considered

1. **X11 (Xorg) + custom WM** — huge legacy compatibility, but X11's architecture
   (server does far less, clients do more, no atomic modesetting model, decades of
   accreted extensions) actively fights the "modern, small, coherent" goal. Declining
   upstream investment industry-wide.
2. **Wayland, from-scratch compositor** — full control, but reimplementing DRM/KMS
   buffer management, input handling, and the Wayland core protocol correctly is a
   multi-year effort we don't need to redo — this exact problem is already solved.
3. **Wayland via wlroots** — a modular compositor library (used by Sway, river, Hyprland,
   COSMIC) that implements the hard parts (DRM/KMS, input, buffer/output management,
   protocol plumbing) and exposes a scene-graph/API for building a custom compositor on
   top with a fraction of the code of option 2.
4. **Embed an existing full desktop compositor (KWin/Mutter)** — fast to bootstrap, but
   these are tightly coupled to their parent desktop environments (KDE/GNOME) and carry
   dependency and design baggage (their own IPC, settings, theming assumptions) that
   directly conflicts with NovaOS owning its own design system end-to-end.

## Decision

**Wayland**, with a custom compositor ("**Nova Compositor**") built on **wlroots**.

## Rationale

wlroots gives us the correct, battle-tested low-level implementation (KMS/DRM, input,
protocol handling) while leaving 100% of the user-visible behavior — window management
policy, animations, theming, workspace model, gesture handling — as NovaOS's own code.
This is the direct architectural analog of ChromeOS/SteamOS building their own compositor
rather than adopting a stock desktop environment's, and it is the only option that
satisfies both "small enough to build and maintain" and "fully own the desktop identity."

## Consequences

- We do not get legacy X11 app compatibility natively. A narrow **XWayland** compatibility
  layer is included as an optional, lazily-started component (only launched on first X11
  client connection) so it costs nothing at idle.
- Every native Nova app must speak Wayland (via the Nova UI toolkit, see
  [ADR-0005](ADR-0005-ui-toolkit.md)) — this is consistent with "everything on-screen
  belongs to NovaOS."
- Nova Compositor owns window decorations, snapping, workspaces, and animations — see
  [03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md).
- wlroots is a C library; our compositor code binds to it from Rust
  ([ADR-0004](ADR-0004-systems-language.md)) via existing Rust bindings, isolated behind
  an internal `nova-compositor-core` crate boundary so the FFI surface is small and
  auditable. **Superseded 2026-07-18, see Revision below.**

## Revision (2026-07-18): wlroots bindings → Smithay

At Phase 3 implementation start, the Rust bindings this ADR assumed (`wlroots`/
`wlroots-rs`, `wlroots-sys`) turned out to be effectively abandoned — `wlroots-rs` is
stuck at v0.4.0 targeting a years-old wlroots API, and `wlroots-sys` v0.16.0 is one
minor version behind the wlroots (0.17.1) available on target systems, with no ABI
stability guarantee across minor versions. This is precisely the first Revisit Trigger
below, hit immediately rather than over time.

**Nova Compositor is built on [Smithay](https://smithay.github.io/) instead** — an
actively maintained, pure-Rust Wayland compositor library (no wlroots/C dependency,
no FFI surface) used by production compositors (COSMIC, niri). It provides the same
category of building blocks wlroots did (DRM/KMS backend, input, protocol plumbing,
scene graph) with 100% of window-management policy still NovaOS's own code — the core
rationale of this ADR is unchanged, only the underlying library is.

Everywhere "wlroots" appears elsewhere in the docs (RFC-0003, 03-DESKTOP-ARCHITECTURE.md,
04-WINDOW-MANAGER-SPEC.md) should be read as "Smithay" — behavioral contracts
(responsibilities, events, failure modes) are unaffected since those were written at the
policy level, not the library-API level.

## Revisit Triggers

- ~~If wlroots' maintenance trajectory stalls or its API churns faster than we can
  track.~~ **Triggered and resolved — see Revision above.**
- If XWayland-on-demand proves to be a meaningfully large chunk of idle RAM/complexity in
  practice, in which case it may be dropped entirely rather than kept "optional."
- If Smithay's maintenance trajectory stalls or its API churns faster than we can track.
