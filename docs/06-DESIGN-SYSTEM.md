# NovaOS — Design System

Status: Draft v0.1 · Owner: Chief Architect · Last updated: 2026-07-18

Related: [ADR-0005](decisions/ADR-0005-ui-toolkit.md)

## 1. Purpose

One visual and interaction language, implemented once in `sdk/nova-ui`, consumed by every
surface: desktop shell, settings, every app, every game. No app is permitted to hand-roll
widgets that bypass the design system — this doc plus `nova-ui`'s API *is* the design
system; there is no separate static style guide that can drift from the implementation.

## 2. Design Tokens

All visual values are tokens, not hardcoded per-widget values, published by
`nova-themed` ([03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §7) and consumed
live by `nova-ui`:

- **Color**: a semantic palette (`surface`, `surface-elevated`, `on-surface`, `accent`,
  `accent-on-accent`, `danger`, `warning`, `success`, `border`, `overlay-scrim`) — never
  raw hex values in app code. Both light and dark palettes are first-class, defined
  together, not "dark mode as an afterthought inversion."
- **Typography**: a single type scale (display, title, heading, body, label, caption),
  one system font shipped with the OS (a single well-hinted, permissively-licensed
  variable font covering the target script set) plus a monospace pairing for
  Terminal/code contexts.
- **Spacing**: a 4px base unit, scale `{1,2,3,4,6,8,12,16,24,32}` × base — every layout
  gap/padding value in `nova-ui` must come from this scale.
- **Radii**: three tiers (small: controls, medium: cards/dialogs, large: windows) —
  consistent corner language across window chrome and in-app surfaces.
- **Elevation**: expressed as a small set of shadow/blur presets tied to a z-index
  scale (base, raised, overlay, modal, window-chrome), not ad hoc per-component shadows.

## 3. Motion

- A single easing curve family (standard, decelerate, accelerate) and duration scale
  (fast: micro-interactions, medium: panel transitions, slow: window open/close) —
  defined once, referenced everywhere, so the whole OS *feels* like one rate of motion.
- Compositor-level animations (window open/close, workspace switch —
  [03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §2) use the same token values
  as in-app motion, even though they're implemented in a different process — tokens are
  published over the same theme channel both consume.
- Respect a system-wide "reduce motion" accessibility setting; every animation has a
  defined reduced/instant fallback, not just a global disable that breaks layout
  assumptions.

## 4. Component Catalog (v1 scope)

Buttons (primary/secondary/danger/icon) · Text input · Text area · Checkbox/Radio · Toggle
switch · Dropdown/Select · Slider · Progress (determinate/indeterminate) · Tabs · Menu/
Context menu · Dialog/Modal · Toast/Notification · Tooltip · List/Table · Card · Window
chrome (title bar, controls) · Sidebar/Nav · Breadcrumb · Badge/Tag · Avatar/Icon · File
picker (system-provided, broker-mediated per
[ADR-0010](decisions/ADR-0010-app-sandboxing-model.md) Consequences).

Each ships with: default state, hover/focus/active/disabled states, keyboard navigation
behavior, and an accessibility role — a component isn't "done" until all four exist (see
[11-CODING-STANDARDS.md](11-CODING-STANDARDS.md) §Definition of Done).

## 5. Iconography

A single icon set, drawn at a consistent stroke weight and grid (matching the radii/
spacing tokens above), shipped as vector data (not bitmap) so it scales cleanly across
HiDPI outputs and the browser-demo's software-rendering path
([ADR-0009](decisions/ADR-0009-browser-boot-emulator.md)). App icons (for Launcher/
Taskbar/Package Center) follow a documented template (safe area, corner radius, optional
"squircle" mask) so third-party app icons look native even though their artwork isn't
ours.

## 6. Theme Engine

- A theme is a signed token file (palette + optional font/icon-set override), not code —
  reinforced from [03-DESKTOP-ARCHITECTURE.md](03-DESKTOP-ARCHITECTURE.md) §7's security
  rationale.
- Ships with exactly two first-party themes at v1: **Nova Light** and **Nova Dark**,
  both meeting WCAG AA contrast at every semantic color pairing — contrast is checked in
  CI (see [10-TESTING-AND-BUILD.md](10-TESTING-AND-BUILD.md)), not just eyeballed.
- Third-party themes are a v1.x extensibility target (see
  [14-FUTURE-VISION.md](14-FUTURE-VISION.md)), not a v1.0 requirement — the token schema
  is designed to support it from day one even though the marketplace isn't built yet.

## 7. Accessibility

- Every `nova-ui` widget exposes a semantic role/label/state triple consumable by an
  eventual screen-reader service — accessibility is a `nova-ui` API contract, not a
  per-app opt-in (reinforces [ADR-0005](decisions/ADR-0005-ui-toolkit.md) Consequences).
- Full keyboard navigability is required for every component in §4 before it ships,
  independent of whether a screen reader exists yet — keyboard-only usability is
  useful on its own and de-risks the later screen-reader integration.
- Minimum touch target size and text contrast are enforced as `nova-ui` layout
  constraints, not style suggestions a widget author can skip.
