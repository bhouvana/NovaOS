//! A real ARGB8888 pixel buffer plus the "software-rasterizer fallback"
//! 05-NOVA-UI-TOOLKIT-SPEC.md §5 calls for — the piece Phase 2's
//! `nova_ui::paint::PaintContext` deliberately left as a logical command
//! stream with no backend. `Canvas` is that backend: it turns
//! [`nova_ui::paint::PaintCommand`]s (or direct calls) into real pixels a
//! Wayland `wl_shm` buffer can present.

use nova_ui::paint::{Color, PaintCommand};
use nova_ui::RenderBackend;

static FONT_BYTES: &[u8] = include_bytes!("../assets/DejaVuSans.ttf");

pub struct Canvas {
    pub width: u32,
    pub height: u32,
    /// ARGB8888, one u32 per pixel, row-major.
    pixels: Vec<u32>,
    font: fontdue::Font,
}

impl Canvas {
    pub fn new(width: u32, height: u32) -> Self {
        let font = fontdue::Font::from_bytes(FONT_BYTES, fontdue::FontSettings::default())
            .expect("embedded DejaVu Sans font must parse");
        Self {
            width,
            height,
            pixels: vec![0; (width * height) as usize],
            font,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.width = width;
        self.height = height;
        self.pixels.clear();
        self.pixels.resize((width * height) as usize, 0);
    }

    /// Raw ARGB8888 bytes, little-endian per pixel — ready to copy into a
    /// `wl_shm` `Argb8888` buffer.
    pub fn as_argb8888_bytes(&self) -> &[u8] {
        bytemuck_cast_u32_slice_to_bytes(&self.pixels)
    }

    pub fn clear(&mut self, color: Color) {
        let packed = pack_argb(color);
        self.pixels.fill(packed);
    }

    pub fn fill_rect(&mut self, x: i32, y: i32, w: u32, h: u32, color: Color) {
        let packed = pack_argb(color);
        let x0 = x.max(0) as u32;
        let y0 = y.max(0) as u32;
        let x1 = ((x + w as i32).max(0) as u32).min(self.width);
        let y1 = ((y + h as i32).max(0) as u32).min(self.height);
        for py in y0..y1 {
            let row_start = (py * self.width) as usize;
            for px in x0..x1 {
                self.pixels[row_start + px as usize] = packed;
            }
        }
    }

    pub fn draw_text(&mut self, x: i32, y: i32, text: &str, color: Color, px_size: f32) {
        let mut pen_x = x as f32;
        for ch in text.chars() {
            let (metrics, coverage) = self.font.rasterize(ch, px_size);
            let glyph_x = pen_x as i32 + metrics.xmin;
            let glyph_y = y - metrics.ymin - metrics.height as i32 + px_size as i32;
            for gy in 0..metrics.height {
                for gx in 0..metrics.width {
                    let alpha = coverage[gy * metrics.width + gx];
                    if alpha == 0 {
                        continue;
                    }
                    let px = glyph_x + gx as i32;
                    let py = glyph_y + gy as i32;
                    self.blend_pixel(px, py, color, alpha);
                }
            }
            pen_x += metrics.advance_width;
        }
    }

    /// Text width in pixels at the given size — used for centering labels.
    pub fn measure_text(&self, text: &str, px_size: f32) -> f32 {
        text.chars()
            .map(|ch| self.font.metrics(ch, px_size).advance_width)
            .sum()
    }

    fn blend_pixel(&mut self, x: i32, y: i32, color: Color, coverage: u8) {
        if x < 0 || y < 0 || x as u32 >= self.width || y as u32 >= self.height {
            return;
        }
        let idx = (y as u32 * self.width + x as u32) as usize;
        let a = coverage as u32;
        let existing = self.pixels[idx];
        let er = (existing >> 16) & 0xFF;
        let eg = (existing >> 8) & 0xFF;
        let eb = existing & 0xFF;

        let r = (color.r as u32 * a + er * (255 - a)) / 255;
        let g = (color.g as u32 * a + eg * (255 - a)) / 255;
        let b = (color.b as u32 * a + eb * (255 - a)) / 255;

        self.pixels[idx] = (0xFF << 24) | (r << 16) | (g << 8) | b;
    }

    /// Replays a `nova_ui::paint::PaintContext`'s recorded commands onto
    /// this canvas — the "commands -> pixels" step 05-NOVA-UI-TOOLKIT-SPEC.md
    /// §5 left undone.
    pub fn replay(&mut self, commands: &[PaintCommand]) {
        for command in commands {
            match command {
                PaintCommand::Rect { rect, color } => {
                    self.fill_rect(
                        rect.origin.x as i32,
                        rect.origin.y as i32,
                        rect.size.width as u32,
                        rect.size.height as u32,
                        *color,
                    );
                }
                PaintCommand::Text { rect, text, color, font_size } => {
                    let baseline_y = rect.origin.y as i32 + rect.size.height as i32;
                    self.draw_text(rect.origin.x as i32, baseline_y, text, *color, *font_size);
                }
            }
        }
    }
}

/// The software `RenderBackend` (docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §5's
/// fallback path) — `Canvas`'s own CPU rasterizer, unmodified. Does not
/// clear the frame itself; the widget tree's own background (a `Panel` or
/// root `Window` fill) is expected to cover the surface, same as any other
/// `RenderBackend` implementation would require.
impl RenderBackend for Canvas {
    fn resize(&mut self, width: u32, height: u32) {
        Canvas::resize(self, width, height);
    }

    fn render(&mut self, commands: &[PaintCommand]) -> &[u8] {
        self.replay(commands);
        self.as_argb8888_bytes()
    }
}

fn pack_argb(color: Color) -> u32 {
    ((color.a as u32) << 24) | ((color.r as u32) << 16) | ((color.g as u32) << 8) | (color.b as u32)
}

fn bytemuck_cast_u32_slice_to_bytes(pixels: &[u32]) -> &[u8] {
    // Safety: u32 -> [u8; 4] reinterpretation is valid for any bit pattern,
    // and Vec<u32>'s backing allocation is at least as aligned as needed.
    unsafe {
        std::slice::from_raw_parts(pixels.as_ptr() as *const u8, std::mem::size_of_val(pixels))
    }
}
