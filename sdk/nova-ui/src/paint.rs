//! `PaintContext` records the logical draw-command stream (the "display
//! list" of docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §4) a widget tree emits
//! during its paint phase — kept independent of *how* those commands become
//! pixels so `nova-ui` stays headless/portable. `render_backend::RenderBackend`
//! is the trait boundary that consumes this stream; see that module and
//! `sdk/nova-ui-wayland` for the software and GPU implementations.

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
        /// Font size in logical pixels. Widgets set this from
        /// `ThemeTokens::type_style` (docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md
        /// §6) rather than a canvas-wide default, so a `RenderBackend` never
        /// has to guess a size per command.
        font_size: f32,
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

    pub fn draw_text(
        &mut self,
        rect: crate::geometry::Rect,
        text: impl Into<String>,
        color: Color,
        font_size: f32,
    ) {
        self.commands.push(PaintCommand::Text {
            rect,
            text: text.into(),
            color,
            font_size,
        });
    }
}
