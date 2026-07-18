use crate::geometry::{Constraints, Point, Rect, Size};
use crate::paint::{Color, PaintContext};
use crate::widget::{AccessibilityNode, EventResult, InputEvent, Widget, WidgetId};

const PADDING: f32 = 12.0; // docs/specs/10-DESIGN-BIBLE.md §3 `spacing.3`
const HEIGHT: f32 = 32.0; // docs/specs/10-DESIGN-BIBLE.md §8 minimum touch target
const ESTIMATED_CHAR_WIDTH: f32 = 8.0;

pub struct Button {
    id: WidgetId,
    pub label: String,
    rect: Rect,
    pressed: bool,
    clicked_this_frame: bool,
}

impl Button {
    pub fn new(id: WidgetId, label: impl Into<String>) -> Self {
        Button {
            id,
            label: label.into(),
            rect: Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size::ZERO,
            },
            pressed: false,
            clicked_this_frame: false,
        }
    }

    /// Consumed by the app's event loop after a frame — mirrors how a real
    /// retained-mode toolkit clears one-shot "was clicked" flags between
    /// frames rather than the app polling raw mouse state.
    pub fn take_click(&mut self) -> bool {
        std::mem::take(&mut self.clicked_this_frame)
    }
}

impl Widget for Button {
    fn measure(&self, constraints: Constraints) -> Size {
        let desired = Size {
            width: self.label.chars().count() as f32 * ESTIMATED_CHAR_WIDTH + PADDING * 2.0,
            height: HEIGHT,
        };
        constraints.clamp(desired)
    }

    fn arrange(&mut self, rect: Rect) {
        self.rect = rect;
    }

    fn paint(&self, ctx: &mut PaintContext) {
        let bg = if self.pressed {
            Color::rgb(0x2E, 0x54, 0xD1) // accent, darkened for pressed state
        } else {
            Color::rgb(0x3D, 0x6B, 0xFF) // docs/specs/10-DESIGN-BIBLE.md §1 `accent`
        };
        ctx.fill_rect(self.rect, bg);
        ctx.draw_text(self.rect, self.label.clone(), Color::rgb(0xFF, 0xFF, 0xFF));
    }

    fn hit_test(&self, point: Point) -> Option<WidgetId> {
        self.rect.contains(point).then_some(self.id)
    }

    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        match event {
            InputEvent::MouseDown { position } if self.rect.contains(*position) => {
                self.pressed = true;
                EventResult::Handled
            }
            InputEvent::MouseUp { position } if self.pressed => {
                self.pressed = false;
                if self.rect.contains(*position) {
                    self.clicked_this_frame = true;
                }
                EventResult::Handled
            }
            _ => EventResult::Bubble,
        }
    }

    fn accessibility_node(&self) -> AccessibilityNode {
        AccessibilityNode {
            role: "button".into(),
            label: self.label.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arranged_button(label: &str) -> Button {
        let mut b = Button::new(1, label);
        b.arrange(Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 100.0,
                height: HEIGHT,
            },
        });
        b
    }

    #[test]
    fn click_requires_mouse_down_and_up_inside_the_button() {
        let mut b = arranged_button("Close");
        assert!(!b.take_click());

        b.handle_event(&InputEvent::MouseDown {
            position: Point { x: 10.0, y: 10.0 },
        });
        b.handle_event(&InputEvent::MouseUp {
            position: Point { x: 10.0, y: 10.0 },
        });
        assert!(b.take_click(), "mouse down+up inside bounds should register a click");
        assert!(!b.take_click(), "take_click should clear the flag");
    }

    #[test]
    fn drag_off_button_before_release_does_not_click() {
        let mut b = arranged_button("Close");
        b.handle_event(&InputEvent::MouseDown {
            position: Point { x: 10.0, y: 10.0 },
        });
        b.handle_event(&InputEvent::MouseUp {
            position: Point { x: 500.0, y: 500.0 },
        });
        assert!(!b.take_click(), "release outside bounds should not click");
    }

    #[test]
    fn mouse_down_outside_bounds_bubbles_instead_of_handled() {
        let mut b = arranged_button("Close");
        let result = b.handle_event(&InputEvent::MouseDown {
            position: Point { x: 500.0, y: 500.0 },
        });
        assert_eq!(result, EventResult::Bubble);
    }
}
