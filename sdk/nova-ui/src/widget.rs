//! The core `Widget` trait per docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §2.

use crate::geometry::{Constraints, Point, Rect, Size};
use crate::paint::PaintContext;

pub type WidgetId = u64;

#[derive(Debug, Clone)]
pub enum InputEvent {
    MouseDown { position: Point },
    MouseUp { position: Point },
    KeyDown { key: String },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EventResult {
    Handled,
    Bubble,
}

#[derive(Debug, Clone)]
pub struct AccessibilityNode {
    pub role: String,
    pub label: String,
}

/// docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §2's six-method contract. A widget
/// isn't "done" per docs/11-CODING-STANDARDS.md §7 until all six are real —
/// enforced here structurally: the trait has no default method bodies, so a
/// widget that skips one fails to compile rather than silently inheriting a
/// stub.
pub trait Widget {
    fn measure(&self, constraints: Constraints) -> Size;
    fn arrange(&mut self, rect: Rect);
    fn paint(&self, ctx: &mut PaintContext);
    fn hit_test(&self, point: Point) -> Option<WidgetId>;
    fn handle_event(&mut self, event: &InputEvent) -> EventResult;
    fn accessibility_node(&self) -> AccessibilityNode;
}
