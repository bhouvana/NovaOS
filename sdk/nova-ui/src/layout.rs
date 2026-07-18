//! `Row`/`Column` layout containers per
//! docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §1, §3.
//!
//! Event routing simplification for this vertical slice: `handle_event`
//! broadcasts to children in order and stops at the first one that returns
//! `Handled`, rather than the full capture/target/bubble-phase model in
//! docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §8 (which needs real hit-testing
//! against the pointer position to find a single target first). Sufficient
//! to prove "mouse and keyboard reach the right widget" for the Hello app's
//! one button; tracked as a Phase 3 hardening item once more than one
//! interactive widget needs to coexist in the same container.

use crate::geometry::{Constraints, Point, Rect, Size};
use crate::paint::PaintContext;
use crate::widget::{AccessibilityNode, EventResult, InputEvent, Widget, WidgetId};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Axis {
    Horizontal,
    Vertical,
}

pub struct LinearLayout {
    id: WidgetId,
    axis: Axis,
    pub spacing: f32,
    children: Vec<Box<dyn Widget>>,
    rect: Rect,
}

impl LinearLayout {
    fn new(id: WidgetId, axis: Axis) -> Self {
        LinearLayout {
            id,
            axis,
            spacing: 8.0, // docs/specs/10-DESIGN-BIBLE.md §3 `spacing.2`
            children: Vec::new(),
            rect: Rect {
                origin: Point { x: 0.0, y: 0.0 },
                size: Size::ZERO,
            },
        }
    }

    /// Named `with_child` rather than `add` — `add` reads as a builder verb
    /// but collides with `std::ops::Add::add` in clippy's eyes; picking a
    /// name that can't be confused for operator overloading is cheaper than
    /// carrying a lint suppression forever.
    pub fn with_child(mut self, child: Box<dyn Widget>) -> Self {
        self.children.push(child);
        self
    }
}

/// `Row`/`Column` are factory namespaces, not `Self`-returning constructors
/// — `Row::new`/`Column::new` deliberately return the shared `LinearLayout`
/// type rather than a `Row`/`Column` value, since a `Row` and a `Column` are
/// the same underlying widget differing only in `Axis`.
pub struct Row;
impl Row {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: WidgetId) -> LinearLayout {
        LinearLayout::new(id, Axis::Horizontal)
    }
}

pub struct Column;
impl Column {
    #[allow(clippy::new_ret_no_self)]
    pub fn new(id: WidgetId) -> LinearLayout {
        LinearLayout::new(id, Axis::Vertical)
    }
}

impl Widget for LinearLayout {
    fn measure(&self, constraints: Constraints) -> Size {
        let child_constraints = match self.axis {
            Axis::Vertical => Constraints::loose(Size {
                width: constraints.max.width,
                height: f32::INFINITY,
            }),
            Axis::Horizontal => Constraints::loose(Size {
                width: f32::INFINITY,
                height: constraints.max.height,
            }),
        };

        let mut main_axis_total = 0.0;
        let mut cross_axis_max = 0.0f32;
        for (i, child) in self.children.iter().enumerate() {
            let size = child.measure(child_constraints);
            match self.axis {
                Axis::Vertical => {
                    main_axis_total += size.height;
                    cross_axis_max = cross_axis_max.max(size.width);
                }
                Axis::Horizontal => {
                    main_axis_total += size.width;
                    cross_axis_max = cross_axis_max.max(size.height);
                }
            }
            if i + 1 < self.children.len() {
                main_axis_total += self.spacing;
            }
        }

        let desired = match self.axis {
            Axis::Vertical => Size {
                width: cross_axis_max,
                height: main_axis_total,
            },
            Axis::Horizontal => Size {
                width: main_axis_total,
                height: cross_axis_max,
            },
        };
        constraints.clamp(desired)
    }

    fn arrange(&mut self, rect: Rect) {
        self.rect = rect;
        let child_constraints = match self.axis {
            Axis::Vertical => Constraints::loose(Size {
                width: rect.size.width,
                height: f32::INFINITY,
            }),
            Axis::Horizontal => Constraints::loose(Size {
                width: f32::INFINITY,
                height: rect.size.height,
            }),
        };

        let mut cursor = match self.axis {
            Axis::Vertical => rect.origin.y,
            Axis::Horizontal => rect.origin.x,
        };

        for child in self.children.iter_mut() {
            let size = child.measure(child_constraints);
            let child_rect = match self.axis {
                Axis::Vertical => Rect {
                    origin: Point {
                        x: rect.origin.x,
                        y: cursor,
                    },
                    size,
                },
                Axis::Horizontal => Rect {
                    origin: Point {
                        x: cursor,
                        y: rect.origin.y,
                    },
                    size,
                },
            };
            child.arrange(child_rect);
            cursor += match self.axis {
                Axis::Vertical => size.height,
                Axis::Horizontal => size.width,
            } + self.spacing;
        }
    }

    fn paint(&self, ctx: &mut PaintContext) {
        for child in &self.children {
            child.paint(ctx);
        }
    }

    fn hit_test(&self, point: Point) -> Option<WidgetId> {
        // Children first (they're painted on top); fall back to the
        // container's own id for a click that lands in the container's rect
        // but on none of its children (e.g. inter-widget spacing) — lets a
        // click still resolve to *something* rather than silently hitting
        // nothing.
        self.children
            .iter()
            .find_map(|c| c.hit_test(point))
            .or_else(|| self.rect.contains(point).then_some(self.id))
    }

    fn handle_event(&mut self, event: &InputEvent) -> EventResult {
        for child in self.children.iter_mut() {
            if child.handle_event(event) == EventResult::Handled {
                return EventResult::Handled;
            }
        }
        EventResult::Bubble
    }

    fn accessibility_node(&self) -> AccessibilityNode {
        AccessibilityNode {
            role: "layout".into(),
            label: String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::label::Label;

    #[test]
    fn column_stacks_children_vertically_with_spacing() {
        let mut col = Column::new(1)
            .with_child(Box::new(Label::new(2, "One")))
            .with_child(Box::new(Label::new(3, "Two")));
        col.arrange(Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 200.0,
                height: 200.0,
            },
        });

        // First label occupies y:[0,20] (LINE_HEIGHT); `spacing` (8) pushes
        // the second label to y:[28,48] — proves vertical stacking
        // including the gap, not just "both are hittable somewhere."
        assert_eq!(col.hit_test(Point { x: 5.0, y: 5.0 }), Some(2));
        assert_eq!(col.hit_test(Point { x: 5.0, y: 35.0 }), Some(3));
        // The spacing gap itself belongs to no child — falls back to the
        // container id (99 in the dedicated test below covers this
        // directly; here it's enough that it's NOT child 3).
        assert_eq!(col.hit_test(Point { x: 5.0, y: 25.0 }), Some(1));

        // Far outside the arranged rect: no hit at all, not even the
        // container fallback.
        assert_eq!(col.hit_test(Point { x: 500.0, y: 500.0 }), None);
    }

    #[test]
    fn container_click_between_children_falls_back_to_container_id() {
        let mut col = Column::new(99).with_child(Box::new(Label::new(2, "One")));
        col.arrange(Rect {
            origin: Point { x: 0.0, y: 0.0 },
            size: Size {
                width: 200.0,
                height: 200.0,
            },
        });
        // Below the (short) label but still inside the container's arranged
        // rect: no child claims it, so the container itself does.
        assert_eq!(col.hit_test(Point { x: 5.0, y: 150.0 }), Some(99));
    }

    #[test]
    fn row_measures_wider_than_a_single_child() {
        let row = Row::new(1)
            .with_child(Box::new(Label::new(2, "Hi")))
            .with_child(Box::new(Label::new(3, "There")));
        let single = Label::new(4, "Hi");

        let unconstrained = Constraints::loose(Size {
            width: 1000.0,
            height: 1000.0,
        });
        let row_size = row.measure(unconstrained);
        let single_size = single.measure(unconstrained);
        assert!(row_size.width > single_size.width);
    }
}
