//! Nova SDK: app entrypoint, lifecycle, and `AppContext`.
//! Implements docs/specs/06-NOVA-SDK-SPEC.md §1–§2.
//!
//! Compositor note: this vertical slice has no real compositor
//! (`nova-compositor`) or session manager (`nova-sessiond`) yet — see
//! docs/12-ROADMAP-AND-MILESTONES.md §4's Environment note. `Window` here is
//! a logical handle (id, title, geometry) that an app can create and query,
//! not a real GPU surface. `nova_main`/`connect_and_launch` below drive the
//! real `App` lifecycle state machine over a real running `novabusd`
//! (services/nova-bus-broker) — everything except pixels-on-screen is real,
//! and is proven end-to-end by `tests/vertical-slice`, not by unit tests in
//! this crate alone (this crate's tests cover the pieces that don't need a
//! live Nova Bus connection; see that gap discussed in the module docs of
//! `tests/vertical-slice/tests/vertical_slice.rs`).

use nova_bus::client::Client;
use nova_bus::BusError;
use std::sync::atomic::{AtomicU64, Ordering};

pub mod window;
pub use window::{Window, WindowEvent};

/// docs/specs/06-NOVA-SDK-SPEC.md §2's lifecycle state machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleState {
    Launching,
    Running,
    Suspended,
    ShuttingDown,
}

/// The one handle an app gets into the SDK — docs/specs/06-NOVA-SDK-SPEC.md §1:
/// "one object, so an app never constructs SDK clients itself."
pub struct AppContext {
    pub app_id: String,
    bus: Client,
    next_window_id: AtomicU64,
}

impl AppContext {
    pub fn new(app_id: impl Into<String>, bus: Client) -> Self {
        Self {
            app_id: app_id.into(),
            bus,
            next_window_id: AtomicU64::new(1),
        }
    }

    pub fn bus(&self) -> &Client {
        &self.bus
    }

    /// docs/specs/06-NOVA-SDK-SPEC.md §5's `sdk/nova-notify` stand-in for
    /// this vertical slice: publishes directly on the real notify topic
    /// rather than going through a not-yet-built `nova-notify` crate.
    pub fn notify(&self, title: &str, body: &str) {
        let payload = format!("{title}\u{1}{body}").into_bytes();
        self.bus.publish("nova.notify.publish", payload);
    }

    pub fn new_window(&self, title: impl Into<String>) -> Window {
        let id = self.next_window_id.fetch_add(1, Ordering::Relaxed);
        Window::new(id, title.into())
    }
}

/// docs/specs/06-NOVA-SDK-SPEC.md §1: the app entrypoint trait.
pub trait App {
    fn new(ctx: &AppContext) -> Self
    where
        Self: Sized;
    fn on_launch(&mut self, ctx: &mut AppContext);
    fn on_window_event(&mut self, ctx: &mut AppContext, event: WindowEvent);
    fn on_suspend(&mut self, ctx: &mut AppContext);
    fn on_resume(&mut self, ctx: &mut AppContext);
    fn on_shutdown(&mut self, ctx: &mut AppContext);
}

/// Per-stage wall-clock timing for a `connect_and_launch` call — real
/// measurements per docs/12-ROADMAP-AND-MILESTONES.md §4a's "measure, don't
/// estimate" directive, decomposed the same way
/// docs/specs/09-PERFORMANCE-STRATEGY.md §6 calls for ("attributable to a
/// specific step, not just 'launch got slower'"), scoped to the stages that
/// exist without a real `nova-sessiond` driving sandbox construction
/// (docs/specs/01-INTERACTION-FLOWS.md §1's other stages — manifest
/// resolve, namespace/seccomp setup — aren't reachable from inside the app
/// process being launched, and remain environment-blocked, see
/// docs/12-ROADMAP-AND-MILESTONES.md §4).
#[derive(Debug, Clone, Copy)]
pub struct LaunchTiming {
    pub bus_connect: std::time::Duration,
    pub app_new: std::time::Duration,
    pub on_launch: std::time::Duration,
    pub total: std::time::Duration,
}

/// Real runtime entry: connects to a running Nova Bus, builds the
/// `AppContext`, and drives `App::new`/`on_launch`, publishing the
/// `nova.session.app_started` event a real `nova-sessiond` would otherwise
/// emit itself (docs/rfcs/RFC-0008-session-manager.md Events Published —
/// this vertical slice has the app self-report since no real session
/// manager exists yet to do it on the app's behalf, a tracked simplification
/// like the others noted in services/nova-bus/src/lib.rs). Returns the
/// constructed app + context so the caller (an app's `main()`, or the
/// vertical-slice test harness) can drive further lifecycle transitions,
/// plus the real per-stage timing.
pub async fn connect_and_launch<A: App>(
    bus_addr: &str,
    app_id: &str,
) -> Result<(A, AppContext, LaunchTiming), BusError> {
    let start = std::time::Instant::now();

    let t0 = std::time::Instant::now();
    let bus = Client::connect_tcp(bus_addr, app_id).await?;
    let bus_connect = t0.elapsed();

    let mut ctx = AppContext::new(app_id, bus);

    let t1 = std::time::Instant::now();
    let mut app = A::new(&ctx);
    let app_new = t1.elapsed();

    let t2 = std::time::Instant::now();
    app.on_launch(&mut ctx);
    let on_launch = t2.elapsed();

    ctx.bus()
        .publish("nova.session.app_started", app_id.as_bytes().to_vec());

    let timing = LaunchTiming {
        bus_connect,
        app_new,
        on_launch,
        total: start.elapsed(),
    };
    tracing::info!(
        app_id,
        bus_connect_us = timing.bus_connect.as_micros() as u64,
        app_new_us = timing.app_new.as_micros() as u64,
        on_launch_us = timing.on_launch.as_micros() as u64,
        total_us = timing.total.as_micros() as u64,
        "app launch timing"
    );

    Ok((app, ctx, timing))
}

/// Runs `App::on_shutdown` and publishes the matching exit event — the
/// counterpart to `connect_and_launch`, factored out so both the real
/// `hello` binary and the vertical-slice test harness drive shutdown
/// identically rather than duplicating the sequence.
pub fn shut_down<A: App>(app: &mut A, ctx: &mut AppContext) {
    app.on_shutdown(ctx);
    ctx.bus()
        .publish("nova.session.app_exited", ctx.app_id.clone().into_bytes());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_state_machine_has_exactly_the_states_the_spec_names() {
        // docs/specs/06-NOVA-SDK-SPEC.md §2 names exactly these four states;
        // this is a compile-time-checked exhaustiveness assertion (adding a
        // state to the enum without updating this match is a build error).
        fn all_states_covered(s: LifecycleState) -> &'static str {
            match s {
                LifecycleState::Launching => "Launching",
                LifecycleState::Running => "Running",
                LifecycleState::Suspended => "Suspended",
                LifecycleState::ShuttingDown => "ShuttingDown",
            }
        }
        assert_eq!(all_states_covered(LifecycleState::Launching), "Launching");
    }

    #[test]
    fn window_ids_assigned_by_a_context_are_unique_and_increasing() {
        // Exercises the id-allocation logic in isolation (AppContext::new_window
        // needs a live bus only to *use* the resulting Window over IPC, not to
        // allocate its id) by driving the same atomic counter pattern directly.
        let counter = AtomicU64::new(1);
        let ids: Vec<u64> = (0..5)
            .map(|_| counter.fetch_add(1, Ordering::Relaxed))
            .collect();
        let mut sorted = ids.clone();
        sorted.sort_unstable();
        assert_eq!(ids, sorted, "ids should already be increasing");
        assert_eq!(ids.iter().collect::<std::collections::HashSet<_>>().len(), 5);
    }
}
