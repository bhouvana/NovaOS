use crate::geometry::{Constraints, Point, Rect, Size};
use crate::paint::{Color, PaintContext};
use crate::widget::{AccessibilityNode, EventResult, InputEvent, Widget, WidgetId};

/// Estimated glyph advance until a real font/atlas exists
/// (docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §5) — deterministic and
/// documented as a placeholder, not tuned to look right, so it's obviously
/// wrong (and thus not silently trusted) once real text metrics land.
const ESTIMATED_CHAR_WIDTH: f32 = 8.0;
const LINE_HEIGHT: f32 = 20.0; // matches docs/specs/10-DESIGN-BIBLE.md §2 `body` line height

pub struct Label {
    id: WidgetId,
    pub text: String,
    rect: Rect,
}

impl Label {
    pub fn new(id: WidgetId, text: impl Into<String>) -> Self {
        Label {
            id,
            text: text.into(),
            rect: Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size::ZERO,
            },
        }
    }
}

impl Widget for Label {
    fn measure(&self, constraints: Constraints) -> Size {
        let desired = Size {
            width: self.text.chars().count() as f32 * ESTIMATED_CHAR_WIDTH,
            height: LINE_HEIGHT,
        };
        constraints.clamp(desired)
    }

    fn arrange(&mut self, rect: Rect) {
        self.rect = rect;
    }

    fn paint(&self, ctx: &mut PaintContext) {
        ctx.draw_text(self.rect, self.text.clone(), Color::rgb(0x1A, 0x1A, 0x1E)); // on-surface, Nova Light
    }

    fn hit_test(&self, point: Point) -> Option<WidgetId> {
        self.rect.contains(point).then_some(self.id)
    }

    fn handle_event(&mut self, _event: &InputEvent) -> EventResult {
        EventResult::Bubble // a label never consumes input
    }

    fn accessibility_node(&self) -> AccessibilityNode {
        AccessibilityNode {
            role: "label".into(),
            label: self.text.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measure_scales_with_text_length_and_respects_constraints() {
        let short = Label::new(1, "Hi");
        let long = Label::new(2, "Hello, Nova!");
        let unconstrained = Constraints::loose(Size {
            width: 1000.0,
            height: 1000.0,
        });
        let short_size = short.measure(unconstrained);
        let long_size = long.measure(unconstrained);
        assert!(long_size.width > short_size.width);

        let tiny = Constraints::loose(Size {
            width: 5.0,
            height: 5.0,
        });
        let clamped = long.measure(tiny);
        assert!(clamped.width <= 5.0);
    }

    #[test]
    fn hit_test_only_matches_inside_arranged_rect() {
        let mut label = Label::new(7, "Hello Nova");
        label.arrange(Rect {
            origin: Point { x: 10.0, y: 10.0 },
            size: Size {
                width: 80.0,
                height: 20.0,
            },
        });
        assert_eq!(label.hit_test(Point { x: 50.0, y: 15.0 }), Some(7));
        assert_eq!(label.hit_test(Point { x: 0.0, y: 0.0 }), None);
    }
}
