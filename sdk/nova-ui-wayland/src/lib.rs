//! A minimal real Wayland client window for NovaOS apps, built on
//! [smithay-client-toolkit] — the presentation half of the SDK
//! ([04-APPLICATION-FRAMEWORK-AND-SDK.md](../../../docs/04-APPLICATION-FRAMEWORK-AND-SDK.md)).
//! `nova-app`'s `Window` stayed a logical state machine through Phase 2
//! because there was no compositor to hand real surfaces to
//! ([[docs/rfcs/RFC-0003-nova-wm.md]] now exists); this crate is that
//! missing "commands -> real pixels on a real Wayland surface" backend for
//! the [`nova_ui::paint`] command stream, adapted from
//! smithay-client-toolkit's own `simple_window` example (v0.19.2).
//!
//! Scope is deliberately small: one fixed-size xdg_toplevel window, `wl_shm`
//! software rendering via [`Canvas`], pointer clicks (left/right) and
//! scroll, and named-key keyboard shortcuts (arrows, Enter, Backspace,
//! Escape, function keys) via [`Keysym`] — no free-text input, no resizing
//! policy, no multi-window apps. Real NovaOS apps beyond this Phase 3
//! milestone will need those; adding them is a matter of extending this
//! crate's handlers, not replacing its structure.

mod canvas;
pub use canvas::Canvas;
pub use nova_ui::paint::Color;
pub use smithay_client_toolkit::seat::keyboard::Keysym;

use std::time::Duration;

use smithay_client_toolkit::{
    compositor::{CompositorHandler, CompositorState},
    delegate_compositor, delegate_keyboard, delegate_output, delegate_pointer, delegate_registry,
    delegate_seat, delegate_shm, delegate_xdg_shell, delegate_xdg_window,
    output::{OutputHandler, OutputState},
    registry::{ProvidesRegistryState, RegistryState},
    registry_handlers,
    reexports::{calloop::EventLoop, calloop_wayland_source::WaylandSource},
    seat::{
        keyboard::{KeyEvent, KeyboardHandler, Modifiers},
        pointer::{PointerEvent, PointerEventKind, PointerHandler},
        Capability, SeatHandler, SeatState,
    },
    shell::{
        xdg::{
            window::{Window, WindowConfigure, WindowDecorations, WindowHandler},
            XdgShell,
        },
        WaylandSurface,
    },
    shm::{
        slot::{Buffer, SlotPool},
        Shm, ShmHandler,
    },
};
use wayland_client::{
    globals::registry_queue_init,
    protocol::{wl_keyboard, wl_output, wl_pointer, wl_seat, wl_shm, wl_surface},
    Connection, QueueHandle,
};

/// Implemented by a NovaOS app to describe what a window looks like and how
/// it reacts to input. `run` drives one of these to a real Wayland surface.
pub trait NovaWindowApp {
    /// Draw the current app state onto `canvas`. Called once per configure
    /// and once per compositor frame callback thereafter.
    fn paint(&mut self, canvas: &mut Canvas);

    /// The left pointer button was pressed at `(x, y)` in window-local
    /// pixels.
    fn on_click(&mut self, _x: i32, _y: i32) {}

    /// The right pointer button was pressed at `(x, y)` in window-local
    /// pixels — the standard "open a context menu" gesture.
    fn on_right_click(&mut self, _x: i32, _y: i32) {}

    /// A scroll/wheel axis event, in pixels (`dy` positive = scroll down).
    fn on_scroll(&mut self, _dx: f64, _dy: f64) {}

    /// A key was pressed. `keysym` is `xkeysym::Keysym` — match against its
    /// named associated constants (`Keysym::Up`, `Keysym::Return`, ...)
    /// rather than raw codes.
    fn on_key(&mut self, _keysym: Keysym) {}

    /// Checked after every event loop dispatch; return `true` to close the
    /// window and return from [`run`].
    fn should_exit(&self) -> bool {
        false
    }
}

pub fn run<A: NovaWindowApp + 'static>(title: &str, app_id: &str, width: u32, height: u32, app: A) {
    let conn = Connection::connect_to_env().expect("connect to WAYLAND_DISPLAY");

    let (globals, event_queue) = registry_queue_init(&conn).expect("registry_queue_init");
    let qh = event_queue.handle();
    let mut event_loop: EventLoop<AppState<A>> =
        EventLoop::try_new().expect("failed to initialize the event loop");
    let loop_handle = event_loop.handle();
    WaylandSource::new(conn.clone(), event_queue)
        .insert(loop_handle)
        .expect("insert wayland source into event loop");

    let compositor = CompositorState::bind(&globals, &qh).expect("wl_compositor not available");
    let xdg_shell = XdgShell::bind(&globals, &qh).expect("xdg shell not available");
    let shm = Shm::bind(&globals, &qh).expect("wl_shm not available");

    let surface = compositor.create_surface(&qh);
    let window = xdg_shell.create_window(surface, WindowDecorations::RequestServer, &qh);
    window.set_title(title);
    window.set_app_id(app_id);
    window.set_min_size(Some((width, height)));
    window.commit();

    let pool = SlotPool::new((width * height * 4) as usize, &shm).expect("failed to create shm pool");

    let mut state = AppState {
        registry_state: RegistryState::new(&globals),
        seat_state: SeatState::new(&globals, &qh),
        output_state: OutputState::new(&globals, &qh),
        shm,
        exit: false,
        first_configure: true,
        pool,
        width,
        height,
        buffer: None,
        window,
        keyboard: None,
        pointer: None,
        pointer_pos: (0, 0),
        canvas: Canvas::new(width, height),
        app,
    };

    loop {
        event_loop
            .dispatch(Duration::from_millis(16), &mut state)
            .expect("event loop dispatch");

        if state.exit || state.app.should_exit() {
            break;
        }
    }
}

struct AppState<A: NovaWindowApp> {
    registry_state: RegistryState,
    seat_state: SeatState,
    output_state: OutputState,
    shm: Shm,

    exit: bool,
    first_configure: bool,
    pool: SlotPool,
    width: u32,
    height: u32,
    buffer: Option<Buffer>,
    window: Window,
    keyboard: Option<wl_keyboard::WlKeyboard>,
    pointer: Option<wl_pointer::WlPointer>,
    pointer_pos: (i32, i32),
    canvas: Canvas,
    app: A,
}

impl<A: NovaWindowApp + 'static> AppState<A> {
    fn draw(&mut self, qh: &QueueHandle<Self>) {
        let width = self.width;
        let height = self.height;
        let stride = width as i32 * 4;

        let buffer = self.buffer.get_or_insert_with(|| {
            self.pool
                .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
                .expect("create buffer")
                .0
        });

        let canvas_slice = match self.pool.canvas(buffer) {
            Some(slice) => slice,
            None => {
                let (second_buffer, slice) = self
                    .pool
                    .create_buffer(width as i32, height as i32, stride, wl_shm::Format::Argb8888)
                    .expect("create buffer");
                *buffer = second_buffer;
                slice
            }
        };

        if self.canvas.width != width || self.canvas.height != height {
            self.canvas.resize(width, height);
        }
        self.app.paint(&mut self.canvas);
        canvas_slice.copy_from_slice(self.canvas.as_argb8888_bytes());

        self.window.wl_surface().damage_buffer(0, 0, width as i32, height as i32);
        self.window.wl_surface().frame(qh, self.window.wl_surface().clone());
        buffer.attach_to(self.window.wl_surface()).expect("buffer attach");
        self.window.commit();
    }
}

impl<A: NovaWindowApp + 'static> CompositorHandler for AppState<A> {
    fn scale_factor_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_factor: i32,
    ) {
    }

    fn transform_changed(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _new_transform: wl_output::Transform,
    ) {
    }

    fn frame(&mut self, _conn: &Connection, qh: &QueueHandle<Self>, _surface: &wl_surface::WlSurface, _time: u32) {
        self.draw(qh);
    }

    fn surface_enter(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }

    fn surface_leave(
        &mut self,
        _conn: &Connection,
        _qh: &QueueHandle<Self>,
        _surface: &wl_surface::WlSurface,
        _output: &wl_output::WlOutput,
    ) {
    }
}

impl<A: NovaWindowApp + 'static> OutputHandler for AppState<A> {
    fn output_state(&mut self) -> &mut OutputState {
        &mut self.output_state
    }
    fn new_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn update_output(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
    fn output_destroyed(&mut self, _conn: &Connection, _qh: &QueueHandle<Self>, _output: wl_output::WlOutput) {}
}

impl<A: NovaWindowApp + 'static> WindowHandler for AppState<A> {
    fn request_close(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &Window) {
        self.exit = true;
    }

    fn configure(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _window: &Window,
        configure: WindowConfigure,
        _serial: u32,
    ) {
        self.buffer = None;
        self.width = configure.new_size.0.map(|v| v.get()).unwrap_or(self.width);
        self.height = configure.new_size.1.map(|v| v.get()).unwrap_or(self.height);

        if self.first_configure {
            self.first_configure = false;
            self.draw(qh);
        }
    }
}

impl<A: NovaWindowApp + 'static> SeatHandler for AppState<A> {
    fn seat_state(&mut self) -> &mut SeatState {
        &mut self.seat_state
    }
    fn new_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}

    fn new_capability(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        seat: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard && self.keyboard.is_none() {
            self.keyboard = self.seat_state.get_keyboard(qh, &seat, None).ok();
        }
        if capability == Capability::Pointer && self.pointer.is_none() {
            self.pointer = self.seat_state.get_pointer(qh, &seat).ok();
        }
    }

    fn remove_capability(
        &mut self,
        _conn: &Connection,
        _: &QueueHandle<Self>,
        _: wl_seat::WlSeat,
        capability: Capability,
    ) {
        if capability == Capability::Keyboard {
            if let Some(k) = self.keyboard.take() {
                k.release();
            }
        }
        if capability == Capability::Pointer {
            if let Some(p) = self.pointer.take() {
                p.release();
            }
        }
    }

    fn remove_seat(&mut self, _: &Connection, _: &QueueHandle<Self>, _: wl_seat::WlSeat) {}
}

impl<A: NovaWindowApp + 'static> KeyboardHandler for AppState<A> {
    fn enter(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _surface: &wl_surface::WlSurface,
        _: u32,
        _: &[u32],
        _keysyms: &[Keysym],
    ) {
        tracing::debug!("keyboard enter (focus gained)");
    }
    fn leave(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _surface: &wl_surface::WlSurface,
        _: u32,
    ) {
    }
    fn press_key(&mut self, _: &Connection, qh: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, event: KeyEvent) {
        self.app.on_key(event.keysym);
        self.draw(qh);
    }
    fn release_key(&mut self, _: &Connection, _: &QueueHandle<Self>, _: &wl_keyboard::WlKeyboard, _: u32, _event: KeyEvent) {}
    fn update_modifiers(
        &mut self,
        _: &Connection,
        _: &QueueHandle<Self>,
        _: &wl_keyboard::WlKeyboard,
        _serial: u32,
        _modifiers: Modifiers,
        _layout: u32,
    ) {
    }
}

impl<A: NovaWindowApp + 'static> PointerHandler for AppState<A> {
    fn pointer_frame(
        &mut self,
        _conn: &Connection,
        qh: &QueueHandle<Self>,
        _pointer: &wl_pointer::WlPointer,
        events: &[PointerEvent],
    ) {
        use PointerEventKind::*;
        for event in events {
            if &event.surface != self.window.wl_surface() {
                continue;
            }
            match event.kind {
                Enter { .. } | Motion { .. } => {
                    self.pointer_pos = (event.position.0 as i32, event.position.1 as i32);
                }
                Press { button, .. } => {
                    let (x, y) = self.pointer_pos;
                    tracing::debug!(x, y, button, "pointer press");
                    match button {
                        smithay_client_toolkit::seat::pointer::BTN_LEFT => self.app.on_click(x, y),
                        smithay_client_toolkit::seat::pointer::BTN_RIGHT => self.app.on_right_click(x, y),
                        _ => {}
                    }
                    self.draw(qh);
                }
                Axis { horizontal, vertical, .. } => {
                    if !horizontal.is_none() || !vertical.is_none() {
                        self.app.on_scroll(horizontal.absolute, vertical.absolute);
                        self.draw(qh);
                    }
                }
                _ => {}
            }
        }
    }
}

impl<A: NovaWindowApp + 'static> ShmHandler for AppState<A> {
    fn shm_state(&mut self) -> &mut Shm {
        &mut self.shm
    }
}

impl<A: NovaWindowApp + 'static> ProvidesRegistryState for AppState<A> {
    fn registry(&mut self) -> &mut RegistryState {
        &mut self.registry_state
    }
    registry_handlers![OutputState, SeatState,];
}

delegate_compositor!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_output!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_shm!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_seat!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_keyboard!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_pointer!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_xdg_shell!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_xdg_window!(@<A: NovaWindowApp + 'static> AppState<A>);
delegate_registry!(@<A: NovaWindowApp + 'static> AppState<A>);
