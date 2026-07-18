# Spec 05 — Nova UI Toolkit Specification

Status: Draft v0.1 · Last updated: 2026-07-18

Concretizes [ADR-0005](../decisions/ADR-0005-ui-toolkit.md) and
[../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) for `sdk/nova-ui`.

## 1. Widget Tree Hierarchy

```text
Window                 top-level surface, one per app window (03-DESKTOP-ARCHITECTURE §2)
 └─ Panel               a themed rectangular region (background, border, elevation)
     └─ Layout           arranges children: Row | Column | Grid | Stack
         └─ Widget        Button | TextInput | Checkbox | Slider | ... (06-DESIGN-SYSTEM §4)
             ├─ Icon       vector icon reference, themed color
             └─ Canvas     raw draw-call escape hatch (Paint app, Arcade games, charts)
     └─ Animation         a decorator node: wraps a subtree, applies a transform/opacity
                           tween driven by 10-DESIGN-BIBLE §3 tokens, transparent to
                           layout (child's layout size is unaffected by animation state)
```

`Canvas` is the one node type that opts an app out of the retained-mode widget model for
a subtree (immediate-mode draw calls each frame) — used by Nova Paint's drawing surface
and Nova Arcade's game boards, never by ordinary app chrome, keeping the "every app
shares the same design system" guarantee (§ [../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md)
§1) intact for everything except deliberately custom-rendered content.

## 2. Core Widget Trait

```rust
trait Widget {
    fn measure(&self, constraints: Constraints) -> Size;
    fn arrange(&mut self, rect: Rect);
    fn paint(&self, ctx: &mut PaintContext);
    fn hit_test(&self, point: Point) -> Option<WidgetId>;
    fn handle_event(&mut self, event: &InputEvent) -> EventResult; // Handled | Bubble
    fn accessibility_node(&self) -> AccessibilityNode; // role/label/state — 06-DESIGN-SYSTEM §7
}

struct Constraints { min: Size, max: Size }
```

Every widget in the [../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §4 catalog
implements this trait; a component is not "done" per
[../11-CODING-STANDARDS.md](../11-CODING-STANDARDS.md) §7 until all six methods have
real (not stub) implementations, including `accessibility_node`.

## 3. Layout Algorithm

Single-pass constraint-based layout (Flutter/Druid-family model, not a browser-style
multi-pass reflow — chosen for predictable, cheap-enough-for-live-resize performance,
[04-WINDOW-MANAGER-SPEC.md](04-WINDOW-MANAGER-SPEC.md) §6):

1. **Measure phase** (top-down): a parent offers `Constraints` (min/max size) to each
   child; each child returns its desired `Size` within those bounds. `Row`/`Column`
   distribute available main-axis space per child `flex` weight (0 = intrinsic size,
   >0 = proportional share of remaining space) — the same model as CSS flexbox's core
   case, deliberately not the full CSS flexbox spec (wrap, order, etc. are out of scope
   for v1's component catalog).
2. **Arrange phase** (top-down): parent assigns each child a final `Rect` based on the
   sizes computed in phase 1 and the layout's alignment rules (start/center/end/stretch
   per main/cross axis).
3. **Paint phase** (top-down, but only for damaged subtrees): each widget emits draw
   commands into a display list; unchanged subtrees reuse their cached display list
   from the previous frame (see §4).

All three phases run in one pass per frame — no iterative reflow-until-stable loop —
because layout inputs (constraints from the window, plus each widget's own declared
sizing) are fully determined before the pass starts; nothing in the model allows a
child's arrange result to retroactively change a sibling's measure result.

## 4. Rendering Pipeline

```text
App state change (event handled, timer fired, data updated)
   ↓
Widget tree diff: which widgets' inputs changed since last frame?
   (a widget is "dirty" if its measure/paint-relevant fields changed)
   ↓
Measure + Arrange (§3) — re-run only for dirty widgets and their
   ancestors up to the nearest layout boundary; unaffected siblings
   skip both phases entirely
   ↓
Paint: dirty widgets emit a new display list fragment; clean widgets'
   cached fragments are reused verbatim
   ↓
Display list → GPU draw call batching: adjacent fragments sharing a
   material (solid color, same font atlas page, same icon atlas page)
   are merged into one draw call — minimizes draw-call count, the
   dominant GPU-side cost for 2D UI at this scale
   ↓
Submit to Wayland surface (wl_surface.attach + damage + commit,
   damage region = union of dirty widgets' screen-space rects — feeds
   directly into 04-WINDOW-MANAGER-SPEC §7's compositor-side damage
   tracking, so a UI-only change never triggers a full-window
   recomposite)
```

## 5. Rendering Backend

Two backends behind one internal trait (`RenderBackend`), selected at app startup based
on GPU capability detection ([../04-APPLICATION-FRAMEWORK-AND-SDK.md](../04-APPLICATION-FRAMEWORK-AND-SDK.md)
§5):

- **GPU backend** (default): vector paint commands (rects, rounded rects, text runs,
  icon blits) translated to GPU draw calls via a `wgpu`-based renderer; font/icon atlases
  are GPU textures, rebuilt lazily as new glyphs/icons are first used.
- **Software backend** (fallback — old hardware, [08-BROWSER-ARCHITECTURE-SPEC.md](08-BROWSER-ARCHITECTURE-SPEC.md)):
  the same paint-command stream rasterized on the CPU into a plain pixel buffer, same
  widget/layout code above it unaware of which backend is active — the `RenderBackend`
  trait boundary is exactly at "take a display list, produce pixels," so switching
  backends never touches widget logic.

## 6. Theming API

```rust
trait ThemeTokens {
    fn color(&self, role: ColorRole) -> Color;      // 06-DESIGN-SYSTEM §2
    fn spacing(&self, step: SpacingStep) -> f32;
    fn radius(&self, tier: RadiusTier) -> f32;
    fn elevation(&self, level: ElevationLevel) -> ShadowSpec;
    fn type_style(&self, role: TypeRole) -> TypeStyle;
}
```

- `nova-ui` holds one `Arc<ThemeTokens>`, swapped atomically when `nova-themed` publishes
  `nova.theme.changed` ([01-INTERACTION-FLOWS.md](01-INTERACTION-FLOWS.md) §6). Every
  widget reads tokens through this handle at paint time — never caches a resolved color
  across frames — so a theme swap requires no widget-tree rebuild, only a repaint (all
  widgets marked dirty for one frame).
- Widgets never accept raw `Color`/`f32` style values from app code for anything covered
  by the token set (enforced by the `Widget` constructors' type signatures only
  accepting `ColorRole`/`SpacingStep`/etc., never a raw `Color`) — this is what makes
  [../06-DESIGN-SYSTEM.md](../06-DESIGN-SYSTEM.md) §1's "no app can bypass the design
  system" claim mechanically true rather than a style guideline.

## 7. Animation System

- Every `Animation` node (§1) is driven by a `Tween<T>` (linear interpolation between
  start/end values of type `T`: `f32`, `Color`, `Rect`) sampled once per frame against
  wall-clock elapsed time, using an easing curve from
  [10-DESIGN-BIBLE.md](10-DESIGN-BIBLE.md) §3.
- The animation scheduler only requests frames while at least one `Animation` node is
  active (registers with the compositor's frame callback,
  [04-WINDOW-MANAGER-SPEC.md](04-WINDOW-MANAGER-SPEC.md) §8) — an app with no running
  animation and no input pending renders zero frames, directly serving the idle-CPU goal
  behind [../09-PERFORMANCE-STRATEGY.md](../09-PERFORMANCE-STRATEGY.md) §6's suspended-
  app expectation.
- "Reduce motion" ([04-WINDOW-MANAGER-SPEC.md](04-WINDOW-MANAGER-SPEC.md) §3) is
  implemented once, here, as a global multiplier on every `Tween`'s duration (→0 when
  enabled) — not re-implemented per animation call site.

## 8. Input Event Routing

- Compositor delivers raw input (pointer motion/button, keyboard, touch) to the focused
  window's Wayland surface; `nova-app` deserializes into `InputEvent` and calls
  `Window::handle_event`.
- Routing within the widget tree: hit-test (§2's `hit_test`) from the root down to the
  deepest widget under the pointer for pointer events; keyboard events go to the
  currently-focused widget (a separate, in-window focus concept from
  [04-WINDOW-MANAGER-SPEC.md](04-WINDOW-MANAGER-SPEC.md) §4's window-level focus).
- **Capture phase → target → bubble phase**: matches the standard DOM-style model
  (familiar to any web-experienced contributor) — a parent may intercept in capture
  phase (rare; used for modal/dialog scrim click-outside-to-dismiss) before the target
  widget handles it, and unhandled events bubble upward through ancestors after the
  target returns `EventResult::Bubble`.

## 9. Accessibility Tree

Built in lockstep with the widget tree (§2's `accessibility_node`), not as a
post-hoc pass: every `arrange` call also updates that widget's position in the
accessibility tree, so the two trees are never observably out of sync. Exposed via a
dedicated Wayland protocol extension (accessibility-tree-v1, a NovaOS-specific
extension per [ADR-0003](../decisions/ADR-0003-compositor-and-display-protocol.md)
Consequences) that a future screen-reader service subscribes to — the service itself is
post-v1 ([../14-FUTURE-VISION.md](../14-FUTURE-VISION.md)), but the tree it would consume
is populated from day one.
