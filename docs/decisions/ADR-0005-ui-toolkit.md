# ADR-0005: Native UI Toolkit

Status: Accepted
Date: 2026-07-18
Deciders: Chief Architect

## Context

Every Nova app (Files, Terminal, Settings, Paint, Calculator, Monitor, Package Center,
Arcade games) needs to render with one visual identity, one widget set, one theming
mechanism, one accessibility layer — see [06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md).
The toolkit choice determines the RAM cost of every running app and how much control we
have over the design system.

## Options Considered

1. **GTK4** — mature, accessible (a11y stack included), theming via CSS, large widget
   catalog, but it is GNOME's toolkit with GNOME's design assumptions baked in; adopting
   it means either fighting its defaults constantly or NovaOS visually becoming "a GNOME
   app," undermining the "own identity" goal. Also C-based with a Rust binding layer
   (gtk-rs) rather than Rust-native.
2. **Qt** — mature, cross-platform, powerful, but heavy (multi-tens-of-MB footprint per
   app family), licensing (LGPL/commercial split) adds friction, and C++-native with Rust
   bindings as a secondary citizen.
3. **Electron / web-based apps** — fastest to build UI in, but a Chromium runtime per app
   directly and severely violates the RAM budget; explicitly rejected except as an
   isolated exception for the Nova Browser's own rendering engine, which unavoidably
   embeds a real web engine.
4. **Custom immediate/retained-mode Rust toolkit ("Nova UI") on a GPU 2D renderer** —
   built specifically for NovaOS, using an existing low-level Rust 2D/GPU rendering crate
   (e.g. a `wgpu`- or `skia`-backed renderer) as the drawing layer, with NovaOS owning the
   widget set, theming, and layout on top. More upfront work; total control of footprint,
   look, and behavior; single language end-to-end with [ADR-0004](ADR-0004-systems-language.md).

## Decision

**Nova UI**: a custom, Rust-native GUI toolkit built on a lightweight GPU-accelerated 2D
rendering layer, speaking Wayland directly (via the same wlroots-adjacent client
libraries used elsewhere in the stack). Ships as a versioned crate (`nova-ui`) that is
the one and only supported way to build a NovaOS-native app UI, exposed through the SDK
(see [04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md)).

## Rationale

This is the only option under which NovaOS fully owns its design system rather than
inheriting someone else's, and the only option that keeps per-app RAM cost within the
overall 64–100 MB idle budget when multiple Nova apps may be resident. It also keeps the
entire stack in one language, reinforcing [ADR-0004](ADR-0004-systems-language.md)'s
consistency rationale. The upfront cost (building a toolkit instead of adopting one) is
accepted explicitly and is why NovaOS's roadmap treats the SDK/toolkit as core Phase 2–3
work, not an afterthought (see [12-ROADMAP-AND-MILESTONES.md](../12-ROADMAP-AND-MILESTONES.md)).

## Consequences

- Significant upfront investment before any app can be built — the roadmap sequences
  Nova UI directly after the compositor for exactly this reason.
- No inherited a11y stack — accessibility must be designed into Nova UI from its first
  widget, not retrofitted (tracked as a first-class requirement, not optional polish).
- No free ecosystem of pre-built GTK/Qt widgets — every widget in
  [06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md)'s catalog must be built by us.
- Nova Browser is the one app permitted to embed a heavier third-party engine (a
  WebKit-family engine, evaluated separately at implementation time) since re-implementing
  a web engine is out of scope by definition.

## Revisit Triggers

- If Nova UI's development cost measurably blocks the Phase 3/4 roadmap by more than one
  full milestone slip, re-evaluate a hybrid (Nova UI for chrome/shell, GTK4 escape hatch
  for a small number of complex apps).
