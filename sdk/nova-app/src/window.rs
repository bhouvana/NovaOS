//! A logical window handle. See the module doc in `lib.rs` for why this is
//! not a real GPU surface in this vertical slice — matches the *data model*
//! of `Window` from docs/specs/04-WINDOW-MANAGER-SPEC.md §2 (id, geometry,
//! title) without the compositor-side rendering that struct assumes.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Size {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Clone)]
pub struct Window {
    pub id: u64,
    pub title: String,
    pub size: Size,
    state: WindowState,
}

/// The reachable subset of docs/specs/04-WINDOW-MANAGER-SPEC.md §1's state
/// machine that makes sense without a real compositor driving transitions
/// like Minimized/Maximized/Snapped (those require real screen geometry).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowState {
    Pending,
    Mapped,
    Closing,
    Destroyed,
}

#[derive(Debug, Clone)]
pub enum WindowEvent {
    Mapped,
    Focused,
    Unfocused,
    CloseRequested,
    MouseClick { x: i32, y: i32 },
    KeyPress { key: String },
}

impl Window {
    pub fn new(id: u64, title: String) -> Self {
        Window {
            id,
            title,
            size: Size {
                width: 960,
                height: 640,
            },
            state: WindowState::Pending,
        }
    }

    pub fn state(&self) -> WindowState {
        self.state
    }

    /// docs/specs/04-WINDOW-MANAGER-SPEC.md §1: Pending -> Mapped on first
    /// commit. Here: an app calling this is standing in for "the compositor
    /// accepted our first frame."
    pub fn map(&mut self) {
        assert_eq!(
            self.state,
            WindowState::Pending,
            "only a Pending window can be Mapped (docs/specs/04-WINDOW-MANAGER-SPEC.md §1)"
        );
        self.state = WindowState::Mapped;
    }

    pub fn close(&mut self) {
        assert_eq!(
            self.state,
            WindowState::Mapped,
            "only a Mapped window can transition to Closing"
        );
        self.state = WindowState::Closing;
    }

    pub fn finish_close(&mut self) {
        assert_eq!(self.state, WindowState::Closing);
        self.state = WindowState::Destroyed;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn window_lifecycle_follows_the_documented_transitions() {
        let mut w = Window::new(1, "Hello Nova".into());
        assert_eq!(w.state(), WindowState::Pending);
        w.map();
        assert_eq!(w.state(), WindowState::Mapped);
        w.close();
        assert_eq!(w.state(), WindowState::Closing);
        w.finish_close();
        assert_eq!(w.state(), WindowState::Destroyed);
    }

    #[test]
    #[should_panic(expected = "only a Pending window can be Mapped")]
    fn cannot_map_a_window_twice() {
        let mut w = Window::new(1, "Hello Nova".into());
        w.map();
        w.map();
    }

    #[test]
    #[should_panic(expected = "only a Mapped window can transition to Closing")]
    fn cannot_close_a_window_that_was_never_mapped() {
        let mut w = Window::new(1, "Hello Nova".into());
        w.close();
    }
}
