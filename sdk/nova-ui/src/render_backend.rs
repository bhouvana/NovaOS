//! docs/specs/05-NOVA-UI-TOOLKIT-SPEC.md §5: one trait, two implementations
//! (GPU default via `wgpu`, software fallback) — every widget/app above this
//! boundary is unaware which one is active.

use crate::paint::PaintCommand;

/// Takes a display list (§4's paint-phase output) and produces a finished
/// frame. Implementations live outside this crate (in `nova-ui-wayland`, at
/// least for now — see that crate's `render` module) since they pull in
/// platform-specific dependencies (fontdue, wgpu, wayland) this crate
/// deliberately stays free of, keeping `nova-ui` itself headless/portable.
pub trait RenderBackend {
    /// (Re)size the target framebuffer. Called on window resize, before the
    /// next `render`.
    fn resize(&mut self, width: u32, height: u32);

    /// Render `commands` and return the finished frame as ARGB8888 pixels
    /// (row-major, one `u32` per pixel, little-endian) — the common
    /// denominator both the software and GPU backends can produce today for
    /// presentation via `wl_shm` (docs/rfcs/RFC-0003-nova-wm.md). A GPU
    /// backend that later presents via `zwp_linux_dmabuf` for a zero-copy
    /// path can add a second method rather than replacing this one.
    fn render(&mut self, commands: &[PaintCommand]) -> &[u8];
}
