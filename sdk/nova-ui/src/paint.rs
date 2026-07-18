//! Headless paint backend for this vertical slice.
//! docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §5 specifies a GPU backend and a
//! software-rasterizer fallback; neither exists yet without a compositor to
//! present pixels to (docs/12-ROADMAP-AND-MILESTONES.md §4's Environment
//! note). `PaintContext` here records the same *logical* draw-command
//! stream either real backend would consume, letting layout/paint logic be
//! written and tested for real now, with only the final
//! "commands -> pixels" step deferred.

#[derive(Debug, Clone, PartialEq)]
pub enum PaintCommand {
    Rect {
        rect: crate::geometry::Rect,
        color: Color,
    },
    Text {
        rect: crate::geometry::Rect,
        text: String,
        color: Color,
    },
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Color {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl Color {
    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Color { r, g, b, a: 255 }
    }
}

#[derive(Default)]
pub struct PaintContext {
    pub commands: Vec<PaintCommand>,
}

impl PaintContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn fill_rect(&mut self, rect: crate::geometry::Rect, color: Color) {
        self.commands.push(PaintCommand::Rect { rect, color });
    }

    pub fn draw_text(&mut self, rect: crate::geometry::Rect, text: impl Into<String>, color: Color) {
        self.commands.push(PaintCommand::Text {
            rect,
            text: text.into(),
            color,
        });
    }
}
