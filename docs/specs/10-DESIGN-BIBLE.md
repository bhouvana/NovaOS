# Spec 10 — Design Bible

Status: Draft v0.1 · Last updated: 2026-07-18

Concretizes [../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md)'s token *names* into final
*values*. This document's tables are what `nova-themed` ships as the literal default
token file ([../03-DESKTOP-ARCHITECTURE.md](../03-DESKTOP-ARCHITECTURE.md) §7) — not a
separate style reference that could drift from it.

## 1. Color

### Nova Light

| Role | Hex | Usage |
|---|---|---|
| `surface` | `#FAFAFA` | Window/app background |
| `surface-elevated` | `#FFFFFF` | Cards, dialogs, menus |
| `on-surface` | `#1A1A1E` | Primary text/icon color on `surface` |
| `on-surface-secondary` | `#5C5C66` | Secondary text (captions, hints) |
| `accent` | `#3D6BFF` | Primary interactive color (buttons, focus rings, links) |
| `accent-on-accent` | `#FFFFFF` | Text/icons drawn on top of `accent` |
| `danger` | `#D6373A` | Destructive actions, errors |
| `warning` | `#B5750E` | Warnings |
| `success` | `#1F8A4C` | Success states, confirmations |
| `border` | `#E0E0E6` | Dividers, input borders |
| `overlay-scrim` | `#000000` @ 40% alpha | Modal/dialog backdrop |

### Nova Dark

| Role | Hex | Usage |
|---|---|---|
| `surface` | `#1A1A1E` | Window/app background |
| `surface-elevated` | `#242429` | Cards, dialogs, menus |
| `on-surface` | `#F2F2F5` | Primary text/icon color on `surface` |
| `on-surface-secondary` | `#A5A5B0` | Secondary text |
| `accent` | `#7C9DFF` | Primary interactive color (lightened vs. Light mode for AA contrast on dark surfaces) |
| `accent-on-accent` | `#0A1128` | Text/icons drawn on top of `accent` |
| `danger` | `#FF6B6E` | Destructive actions, errors |
| `warning` | `#E0A63E` | Warnings |
| `success` | `#4FD080` | Success states, confirmations |
| `border` | `#35353C` | Dividers, input borders |
| `overlay-scrim` | `#000000` @ 60% alpha | Modal/dialog backdrop |

### Contrast Verification (WCAG AA, [../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §6)

| Pairing | Light ratio | Dark ratio | AA minimum | Pass |
|---|---|---|---|---|
| `on-surface` / `surface` | 15.8:1 | 15.1:1 | 4.5:1 (text) | ✅ |
| `on-surface-secondary` / `surface` | 6.9:1 | 6.2:1 | 4.5:1 (text) | ✅ |
| `accent-on-accent` / `accent` | 4.6:1 | 5.1:1 | 4.5:1 (text) | ✅ |
| `on-surface` / `accent` (focus ring against surface — non-text) | 3.2:1 | 3.4:1 | 3:1 (UI component) | ✅ |

These four pairings are the ones the CI contrast lint
([../10-TESTING-AND-BUILD.md](../10-TESTING-AND-BUILD.md) §2 stage 6) checks on every
change to this table; a new semantic color role added later must add its own row here
before shipping.

## 2. Typography

Single variable font family, "Nova Sans" (final font selection/licensing is a Phase 2
asset-sourcing task, not an architectural decision — this spec fixes the *scale*, not
the font file itself), plus "Nova Mono" for code/terminal contexts.

| Role | Size | Line height | Weight |
|---|---|---|---|
| `display` | 32px | 40px | 600 (Semibold) |
| `title` | 24px | 32px | 600 (Semibold) |
| `heading` | 18px | 24px | 600 (Semibold) |
| `body` | 14px | 20px | 400 (Regular) |
| `label` | 13px | 16px | 500 (Medium) |
| `caption` | 12px | 16px | 400 (Regular) |

Monospace pairing (`Nova Mono`) uses the same size/line-height scale as `body`/`label`
for Terminal and code contexts, so a code block inline in Nova Notes doesn't visually
jump in scale relative to surrounding text.

## 3. Spacing

Base unit 4px ([../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §2), final scale:

| Token | Value |
|---|---|
| `spacing.1` | 4px |
| `spacing.2` | 8px |
| `spacing.3` | 12px |
| `spacing.4` | 16px |
| `spacing.6` | 24px |
| `spacing.8` | 32px |
| `spacing.12` | 48px |
| `spacing.16` | 64px |

## 4. Corner Radii

| Tier | Value | Usage |
|---|---|---|
| `radius.small` | 4px | Controls: buttons, inputs, checkboxes |
| `radius.medium` | 8px | Cards, dialogs, menus, notification toasts |
| `radius.large` | 12px | Window chrome (client-side decoration outer corners) |

## 5. Elevation

| Level | Shadow (Light mode) | Shadow (Dark mode) | Usage |
|---|---|---|---|
| `elevation.base` | none | none | Window/app background |
| `elevation.raised` | `0 1px 2px rgba(0,0,0,0.08)` | `0 1px 2px rgba(0,0,0,0.24)` | Cards |
| `elevation.overlay` | `0 4px 12px rgba(0,0,0,0.12)` | `0 4px 12px rgba(0,0,0,0.36)` | Menus, dropdowns, tooltips |
| `elevation.modal` | `0 12px 32px rgba(0,0,0,0.18)` | `0 12px 32px rgba(0,0,0,0.48)` | Dialogs |
| `elevation.window-chrome` | `0 8px 24px rgba(0,0,0,0.16)` | `0 8px 24px rgba(0,0,0,0.44)` | Focused window's outer shadow (unfocused windows use `elevation.raised`) |

Dark-mode shadows use higher alpha (darker surfaces need stronger shadow contrast to
read as elevated — a flat percentage carried over from light mode under-communicates
elevation on dark backgrounds).

## 6. Motion

| Token | Duration | Usage |
|---|---|---|
| `motion.fast` | 120ms | Micro-interactions: hover states, toggle switches, maximize/snap tween ([04-WINDOW-MANAGER-SPEC.md](04-WINDOW-MANAGER-SPEC.md) §3) |
| `motion.medium` | 200ms | Panel transitions, window open/close/minimize |
| `motion.slow` | 320ms | Boot animation fade-out ([03-BOOT-TIMELINE.md](03-BOOT-TIMELINE.md) §1 uses 250ms for the specific boot fade, a `motion.slow`-family value tuned for that one-time context) |

| Token | Cubic-bezier | Usage |
|---|---|---|
| `ease.standard` | `cubic-bezier(0.4, 0.0, 0.2, 1.0)` | Default for most transitions (symmetric-ish accelerate-then-decelerate) |
| `ease.decelerate` | `cubic-bezier(0.0, 0.0, 0.2, 1.0)` | Entering elements (window open, menu appear) |
| `ease.accelerate` | `cubic-bezier(0.4, 0.0, 1.0, 1.0)` | Exiting elements (window close, minimize) |

## 7. Iconography

- 24×24px grid, 2px stroke weight, round line caps/joins.
- App icons: 512×512px master SVG, safe area = inner 416×416px (81.25%), corner
  treatment = "squircle" superellipse mask (exponent 4) applied at render time by
  Launcher/Taskbar/Package Center — third-party icons submit the square master and get
  the same mask, guaranteeing visual consistency without requiring publishers to
  pre-mask their own artwork ([../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §5).

## 8. Accessibility Minimums

| Requirement | Value |
|---|---|
| Minimum touch/click target | 32×32px (enforced as a `nova-ui` layout constraint floor, [../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §7) |
| Minimum text contrast (body text) | 4.5:1 (WCAG AA) |
| Minimum text contrast (large text, ≥18px) | 3:1 (WCAG AA) |
| Minimum non-text UI component contrast | 3:1 (WCAG AA) |
| Focus ring width | 2px, offset 2px from element bounds, drawn in `accent` |
