//! Nova Hello — the Phase 2 vertical-slice validation app
//! (docs/12-ROADMAP-AND-MILESTONES.md §4). Not part of the v1.0 first-party
//! app suite in docs/00-VISION.md §5; exists solely to prove the SDK's
//! `App` trait, `AppContext`, and Nova Bus IPC work end-to-end as real,
//! separate OS processes — see tests/vertical-slice for the harness that
//! launches this binary and `novabusd` as real child processes and drives
//! them through a full launch -> IPC -> shutdown cycle.

use nova_app::{App, AppContext, Window, WindowEvent};
use nova_ui::{Button, Column, Label, Widget};

struct HelloApp {
    window: Option<Window>,
    ui: Option<Box<dyn Widget>>,
}

impl App for HelloApp {
    fn new(_ctx: &AppContext) -> Self {
        HelloApp {
            window: None,
            ui: None,
        }
    }

    fn on_launch(&mut self, ctx: &mut AppContext) {
        let mut window = ctx.new_window("Hello Nova");
        window.map();
        tracing::info!(window_id = window.id, title = %window.title, "window mapped");

        // docs/specs/06-NOVA-SDK-SPEC.md §5 / §01-INTERACTION-FLOWS.md §1:
        // building the real widget tree an app would hand to the
        // compositor's Wayland surface, minus the compositor itself.
        let ui: Box<dyn Widget> = Box::new(
            Column::new(1)
                .with_child(Box::new(Label::new(2, "Hello, Nova!")))
                .with_child(Box::new(Button::new(3, "Close"))),
        );

        self.window = Some(window);
        self.ui = Some(ui);
    }

    fn on_window_event(&mut self, _ctx: &mut AppContext, event: WindowEvent) {
        tracing::debug!(?event, "window event");
    }

    fn on_suspend(&mut self, _ctx: &mut AppContext) {}
    fn on_resume(&mut self, _ctx: &mut AppContext) {}

    fn on_shutdown(&mut self, _ctx: &mut AppContext) {
        if let Some(w) = self.window.as_mut() {
            w.close();
            w.finish_close();
        }
        tracing::info!("hello app shutting down");
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt::init();

    let bus_addr =
        std::env::var("NOVA_BUS_ADDR").unwrap_or_else(|_| "127.0.0.1:7780".to_string());
    let app_id = std::env::var("NOVA_APP_ID").unwrap_or_else(|_| "dev.novaos.hello".to_string());

    let (mut app, mut ctx, timing) =
        nova_app::connect_and_launch::<HelloApp>(&bus_addr, &app_id).await?;
    tracing::info!(?timing, "hello app launched");

    let shutdown_topic = format!("nova.app.{app_id}.shutdown");
    let mut shutdown_calls = ctx.bus().register_handler(&shutdown_topic).await?;

    tracing::info!(app_id = %app_id, topic = %shutdown_topic, "hello app running, awaiting shutdown");

    tokio::select! {
        Some(call) = shutdown_calls.recv() => {
            call.respond(b"ok".to_vec());
            nova_app::shut_down(&mut app, &mut ctx);
        }
        _ = tokio::signal::ctrl_c() => {
            nova_app::shut_down(&mut app, &mut ctx);
        }
    }

    Ok(())
}
