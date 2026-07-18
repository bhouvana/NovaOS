//! Layout primitives per docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §3.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub width: f32,
    pub height: f32,
}

impl Size {
    pub const ZERO: Size = Size {
        width: 0.0,
        height: 0.0,
    };
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Rect {
    pub origin: Point,
    pub size: Size,
}

impl Rect {
    pub fn contains(&self, p: Point) -> bool {
        p.x >= self.origin.x
            && p.x <= self.origin.x + self.size.width
            && p.y >= self.origin.y
            && p.y <= self.origin.y + self.size.height
    }
}

/// docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §2: what a parent offers a child
/// during the measure phase.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Constraints {
    pub min: Size,
    pub max: Size,
}

impl Constraints {
    pub fn tight(size: Size) -> Self {
        Constraints {
            min: size,
            max: size,
        }
    }

    pub fn loose(max: Size) -> Self {
        Constraints {
            min: Size::ZERO,
            max,
        }
    }

    /// Clamps a widget's desired size into these constraints — every
    /// `Widget::measure` implementation in this crate returns
    /// `constraints.clamp(desired)` rather than the raw desired size, so no
    /// widget can ever report a size its parent didn't allow for.
    pub fn clamp(&self, desired: Size) -> Size {
        Size {
            width: desired.width.clamp(self.min.width, self.max.width.max(self.min.width)),
            height: desired
                .height
                .clamp(self.min.height, self.max.height.max(self.min.height)),
        }
    }
}
