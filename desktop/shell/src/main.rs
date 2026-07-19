//! Nova Shell — the desktop's visible chrome
//! ([RFC-0001](../../../docs/rfcs/RFC-0001-nova-shell.md)).
//!
//! Phase 3 Milestone 1/2/3 scope: an always-on-top bar with a fixed set of
//! Launcher buttons for the NovaOS apps that exist so far (`hello-gui`,
//! `nova-files`). None of RFC-0001's Nova Bus integration (Taskbar entries,
//! Notification Center, search-first Launcher index) is implemented yet —
//! those need `novabusd`/`nova-sessiond` running as real sibling processes,
//! past this milestone's "does a desktop exist at all" scope. What's here
//! is real, not a mock: a real Wayland window, rendered through the same
//! `nova-ui-wayland` pipeline every app uses (RFC-0001's "rendered through
//! Nova UI like any other app"), that really launches real child processes.

use std::path::PathBuf;
use std::process::Command;

use nova_ui_wayland::{Canvas, Color, NovaWindowApp};

const BAR_WIDTH: u32 = 1024;
const BAR_HEIGHT: u32 = 56;

const BUTTON_Y: i32 = 8;
const BUTTON_H: u32 = 40;
const BUTTON_W: u32 = 120;
const BUTTON_SPACING: i32 = 12;

struct LauncherButton {
    label: &'static str,
    bin_path: PathBuf,
    x: i32,
}

struct NovaShell {
    buttons: Vec<LauncherButton>,
    status: Option<String>,
}

impl NovaWindowApp for NovaShell {
    fn paint(&mut self, canvas: &mut Canvas) {
        canvas.clear(Color::rgb(18, 18, 22));

        for button in &self.buttons {
            canvas.fill_rect(button.x, BUTTON_Y, BUTTON_W, BUTTON_H, Color::rgb(60, 120, 220));
            let label_w = canvas.measure_text(button.label, 16.0);
            let label_x = button.x + (BUTTON_W as f32 - label_w) as i32 / 2;
            canvas.draw_text(label_x, BUTTON_Y + 12, button.label, Color::rgb(255, 255, 255), 16.0);
        }

        if let Some(status) = &self.status {
            let status_x = self.buttons.last().map(|b| b.x + BUTTON_W as i32 + 20).unwrap_or(12);
            canvas.draw_text(status_x, BUTTON_Y + 12, status, Color::rgb(180, 180, 190), 14.0);
        }
    }

    fn on_click(&mut self, x: i32, y: i32) {
        if y < BUTTON_Y || y > BUTTON_Y + BUTTON_H as i32 {
            return;
        }
        let Some(button) = self
            .buttons
            .iter()
            .find(|b| x >= b.x && x <= b.x + BUTTON_W as i32)
        else {
            return;
        };

        match Command::new(&button.bin_path).spawn() {
            Ok(_) => {
                tracing::info!(path = ?button.bin_path, "launched app");
                self.status = Some(format!("Launched {}", button.label));
            }
            Err(err) => {
                tracing::error!(path = ?button.bin_path, %err, "failed to launch app");
                self.status = Some(format!("Failed to launch {}", button.label));
            }
        }
    }
}

/// Sibling binaries live in the same Cargo target directory as `nova-shell`
/// in this dev/vertical-slice environment — a real installed NovaOS image
/// would resolve these through the app registry
/// (docs/specs/09-APPLICATION-SPECS.md), not a sibling-path lookup.
fn find_sibling_binary(name: &str, env_override: &str) -> PathBuf {
    if let Ok(path) = std::env::var(env_override) {
        return PathBuf::from(path);
    }
    let mut path = std::env::current_exe().expect("current_exe");
    path.set_file_name(name);
    path
}

fn main() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let mut x = 12;
    let mut buttons = Vec::new();
    for (label, bin_name, env_override) in [
        ("Hello", "hello-gui", "NOVA_HELLO_GUI_BIN"),
        ("Files", "nova-files", "NOVA_FILES_BIN"),
    ] {
        buttons.push(LauncherButton {
            label,
            bin_path: find_sibling_binary(bin_name, env_override),
            x,
        });
        x += BUTTON_W as i32 + BUTTON_SPACING;
    }

    nova_ui_wayland::run(
        "Nova Shell",
        "os.nova.shell",
        BAR_WIDTH,
        BAR_HEIGHT,
        NovaShell { buttons, status: None },
    );
}
