//! Nova UI toolkit. Implements the widget tree, layout algorithm, and event
//! model from docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §1–§3, §8, minus GPU/
//! software rendering (see `paint` module doc — no compositor exists yet to
//! present pixels to, docs/12-ROADMAP-AND-MILESTONES.md §4).
//!
//! Theming note: real theme tokens (docs/specs/10-DESIGN-BIBLE.md) are not
//! yet wired to a live `nova-themed` subscription (docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md
//! §6) — `label.rs`/`button.rs` hardcode the Nova Light palette's literal
//! values as constants instead. Tracked as a Phase 3 item once `nova-themed`
//! exists to subscribe to.

pub mod button;
pub mod geometry;
pub mod label;
pub mod layout;
pub mod paint;
pub mod widget;

pub use button::Button;
pub use geometry::{Constraints, Point, Rect, Size};
pub use label::Label;
pub use layout::{Column, Row};
pub use paint::{Color, PaintContext};
pub use widget::{AccessibilityNode, EventResult, InputEvent, Widget, WidgetId};
