//! Nova Hello (GUI) — the first real graphical NovaOS app: a window, a
//! label, a button, and a counter, rendered as a real Wayland surface via
//! `nova-ui-wayland`. This is deliberately the smallest possible thing that
//! proves the whole chain (window creation, event handling, layout,
//! rendering, click -> state -> redraw) rather than a feature-complete app.

use nova_ui_wayland::{Canvas, Color, NovaWindowApp};

const WINDOW_WIDTH: u32 = 400;
const WINDOW_HEIGHT: u32 = 300;

const BUTTON_X: i32 = 40;
const BUTTON_Y: i32 = 130;
const BUTTON_W: u32 = 120;
const BUTTON_H: u32 = 48;

struct HelloApp {
    count: u32,
}

impl NovaWindowApp for HelloApp {
    fn paint(&mut self, canvas: &mut Canvas) {
        canvas.clear(Color::rgb(24, 24, 28));

        canvas.draw_text(40, 40, "Hello, Nova!", Color::rgb(240, 240, 245), 28.0);

        canvas.fill_rect(BUTTON_X, BUTTON_Y, BUTTON_W, BUTTON_H, Color::rgb(60, 120, 220));
        let label = "+1";
        let label_w = canvas.measure_text(label, 20.0);
        let label_x = BUTTON_X + (BUTTON_W as f32 - label_w) as i32 / 2;
        canvas.draw_text(label_x, BUTTON_Y + 14, label, Color::rgb(255, 255, 255), 20.0);

        canvas.draw_text(
            40,
            BUTTON_Y + BUTTON_H as i32 + 40,
            &format!("Count: {}", self.count),
            Color::rgb(200, 200, 210),
            22.0,
        );
    }

    fn on_click(&mut self, x: i32, y: i32) {
        let in_button = x >= BUTTON_X
            && x <= BUTTON_X + BUTTON_W as i32
            && y >= BUTTON_Y
            && y <= BUTTON_Y + BUTTON_H as i32;
        if in_button {
            self.count += 1;
        }
    }
}

fn main() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    nova_ui_wayland::run(
        "Nova Hello",
        "os.nova.hello",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        HelloApp { count: 0 },
    );
}
