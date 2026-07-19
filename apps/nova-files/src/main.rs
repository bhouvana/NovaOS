//! Nova Files — Phase 3 Milestone 3's "first real app": a directory listing
//! that starts wherever it was launched from and can navigate. Deliberately
//! minimal per the milestone's own scope ("even if it only lists one
//! directory initially") but real: real `std::fs` reads, real icons (drawn
//! shapes, not loaded images — no asset pipeline yet), a real scrollable
//! list, a real right-click context menu, and real keyboard navigation.
//! No destructive filesystem actions (no delete/rename) are wired up yet —
//! deliberately out of scope until there's a confirmation UI to guard them.

use std::fs;
use std::path::PathBuf;

use nova_ui_wayland::{Canvas, Color, Keysym, NovaWindowApp};

const WINDOW_WIDTH: u32 = 480;
const WINDOW_HEIGHT: u32 = 400;
const PATH_BAR_HEIGHT: i32 = 32;
const ROW_HEIGHT: f32 = 28.0;
const ICON_SIZE: u32 = 16;

struct Entry {
    name: String,
    is_dir: bool,
}

struct ContextMenu {
    x: i32,
    y: i32,
    target_index: usize,
}

const MENU_ITEM_HEIGHT: i32 = 28;
const MENU_WIDTH: i32 = 140;
const MENU_ITEMS: &[&str] = &["Open", "Refresh"];

struct NovaFiles {
    cwd: PathBuf,
    entries: Vec<Entry>,
    selected: Option<usize>,
    scroll_offset: f32,
    context_menu: Option<ContextMenu>,
}

impl NovaFiles {
    fn new(start_dir: PathBuf) -> Self {
        let mut app = NovaFiles {
            cwd: start_dir,
            entries: Vec::new(),
            selected: None,
            scroll_offset: 0.0,
            context_menu: None,
        };
        app.reload();
        app
    }

    fn reload(&mut self) {
        let mut entries: Vec<Entry> = match fs::read_dir(&self.cwd) {
            Ok(iter) => iter
                .filter_map(|res| res.ok())
                .filter_map(|dirent| {
                    let name = dirent.file_name().to_string_lossy().into_owned();
                    let is_dir = dirent.file_type().map(|t| t.is_dir()).unwrap_or(false);
                    Some(Entry { name, is_dir })
                })
                .collect(),
            Err(err) => {
                tracing::warn!(path = ?self.cwd, %err, "failed to read directory");
                Vec::new()
            }
        };
        entries.sort_by(|a, b| match (a.is_dir, b.is_dir) {
            (true, false) => std::cmp::Ordering::Less,
            (false, true) => std::cmp::Ordering::Greater,
            _ => a.name.to_lowercase().cmp(&b.name.to_lowercase()),
        });
        self.entries = entries;
        self.selected = None;
        self.scroll_offset = 0.0;
        self.context_menu = None;
    }

    fn navigate_into(&mut self, index: usize) {
        let Some(entry) = self.entries.get(index) else {
            return;
        };
        if entry.is_dir {
            self.cwd.push(&entry.name);
            self.reload();
        }
    }

    fn navigate_up(&mut self) {
        if self.cwd.pop() {
            self.reload();
        }
    }

    fn list_viewport_height(&self) -> f32 {
        (WINDOW_HEIGHT as i32 - PATH_BAR_HEIGHT) as f32
    }

    fn max_scroll(&self) -> f32 {
        let content_height = self.entries.len() as f32 * ROW_HEIGHT;
        (content_height - self.list_viewport_height()).max(0.0)
    }

    fn ensure_selected_visible(&mut self) {
        let Some(selected) = self.selected else {
            return;
        };
        let row_top = selected as f32 * ROW_HEIGHT;
        let row_bottom = row_top + ROW_HEIGHT;
        let viewport_h = self.list_viewport_height();
        if row_top < self.scroll_offset {
            self.scroll_offset = row_top;
        } else if row_bottom > self.scroll_offset + viewport_h {
            self.scroll_offset = row_bottom - viewport_h;
        }
        self.scroll_offset = self.scroll_offset.clamp(0.0, self.max_scroll());
    }

    fn row_index_at(&self, x: i32, y: i32) -> Option<usize> {
        if y < PATH_BAR_HEIGHT || x < 0 || x as u32 > WINDOW_WIDTH {
            return None;
        }
        let list_y = (y - PATH_BAR_HEIGHT) as f32 + self.scroll_offset;
        let index = (list_y / ROW_HEIGHT) as usize;
        if index < self.entries.len() {
            Some(index)
        } else {
            None
        }
    }

    fn draw_folder_icon(canvas: &mut Canvas, x: i32, y: i32) {
        let color = Color::rgb(230, 180, 90);
        canvas.fill_rect(x, y + 3, ICON_SIZE, ICON_SIZE - 3, color);
        canvas.fill_rect(x, y + 1, ICON_SIZE / 2, 3, color);
    }

    fn draw_file_icon(canvas: &mut Canvas, x: i32, y: i32) {
        let color = Color::rgb(190, 195, 205);
        canvas.fill_rect(x, y, ICON_SIZE - 3, ICON_SIZE, color);
        canvas.fill_rect(x + 2, y + 4, ICON_SIZE - 8, 1, Color::rgb(60, 60, 66));
        canvas.fill_rect(x + 2, y + 8, ICON_SIZE - 8, 1, Color::rgb(60, 60, 66));
    }
}

impl NovaWindowApp for NovaFiles {
    fn paint(&mut self, canvas: &mut Canvas) {
        canvas.clear(Color::rgb(24, 24, 28));

        // Rows first, path bar painted on top — Canvas has no scissor/clip
        // region, so this paint order is how a scrolled row never visually
        // bleeds over the path bar instead of behind it.
        for (i, entry) in self.entries.iter().enumerate() {
            let row_top = PATH_BAR_HEIGHT as f32 + i as f32 * ROW_HEIGHT - self.scroll_offset;
            if row_top + ROW_HEIGHT < PATH_BAR_HEIGHT as f32 || row_top > WINDOW_HEIGHT as f32 {
                continue;
            }
            let row_top = row_top as i32;

            if self.selected == Some(i) {
                canvas.fill_rect(0, row_top, WINDOW_WIDTH, ROW_HEIGHT as u32, Color::rgb(45, 70, 110));
            }

            let icon_y = row_top + (ROW_HEIGHT as i32 - ICON_SIZE as i32) / 2;
            if entry.is_dir {
                Self::draw_folder_icon(canvas, 10, icon_y);
            } else {
                Self::draw_file_icon(canvas, 10, icon_y);
            }

            canvas.draw_text(
                34,
                row_top + ROW_HEIGHT as i32 - 8,
                &entry.name,
                Color::rgb(220, 220, 225),
                15.0,
            );
        }

        canvas.fill_rect(0, 0, WINDOW_WIDTH, PATH_BAR_HEIGHT as u32, Color::rgb(18, 18, 22));
        let path_text = self.cwd.to_string_lossy();
        canvas.draw_text(10, 22, &path_text, Color::rgb(200, 200, 210), 15.0);

        if let Some(menu) = &self.context_menu {
            let menu_height = MENU_ITEM_HEIGHT * MENU_ITEMS.len() as i32;
            canvas.fill_rect(menu.x, menu.y, MENU_WIDTH as u32, menu_height as u32, Color::rgb(40, 40, 46));
            for (i, label) in MENU_ITEMS.iter().enumerate() {
                let item_y = menu.y + i as i32 * MENU_ITEM_HEIGHT;
                canvas.draw_text(menu.x + 10, item_y + 19, label, Color::rgb(230, 230, 235), 14.0);
            }
        }
    }

    fn on_click(&mut self, x: i32, y: i32) {
        if let Some(menu) = &self.context_menu {
            let menu_height = MENU_ITEM_HEIGHT * MENU_ITEMS.len() as i32;
            let inside_menu = x >= menu.x
                && x <= menu.x + MENU_WIDTH
                && y >= menu.y
                && y <= menu.y + menu_height;
            if inside_menu {
                let item_index = ((y - menu.y) / MENU_ITEM_HEIGHT) as usize;
                let target_index = menu.target_index;
                self.context_menu = None;
                match MENU_ITEMS.get(item_index).copied() {
                    Some("Open") => {
                        self.selected = Some(target_index);
                        self.navigate_into(target_index);
                    }
                    Some("Refresh") => self.reload(),
                    _ => {}
                }
            } else {
                self.context_menu = None;
            }
            return;
        }

        match self.row_index_at(x, y) {
            Some(index) => {
                tracing::debug!(x, y, index, name = %self.entries[index].name, "row selected");
                self.selected = Some(index);
            }
            None => {
                if y >= PATH_BAR_HEIGHT {
                    self.selected = None;
                }
            }
        }
    }

    fn on_right_click(&mut self, x: i32, y: i32) {
        if let Some(index) = self.row_index_at(x, y) {
            self.selected = Some(index);
            self.context_menu = Some(ContextMenu { x, y, target_index: index });
        } else {
            self.context_menu = None;
        }
    }

    fn on_scroll(&mut self, _dx: f64, dy: f64) {
        self.scroll_offset = (self.scroll_offset + dy as f32).clamp(0.0, self.max_scroll());
    }

    fn on_key(&mut self, keysym: Keysym) {
        tracing::debug!(?keysym, "on_key");
        // Numpad arrow/enter variants (`KP_*`) arrive instead of the main
        // cluster's keysyms when NumLock is off on some keyboards/input
        // paths — treated identically to their non-numpad counterparts.
        match keysym {
            Keysym::Up | Keysym::KP_Up => {
                self.selected = Some(match self.selected {
                    Some(i) if i > 0 => i - 1,
                    Some(i) => i,
                    None => 0,
                });
                self.ensure_selected_visible();
            }
            Keysym::Down | Keysym::KP_Down => {
                if !self.entries.is_empty() {
                    self.selected = Some(match self.selected {
                        Some(i) if i + 1 < self.entries.len() => i + 1,
                        Some(i) => i,
                        None => 0,
                    });
                    self.ensure_selected_visible();
                }
            }
            Keysym::Return | Keysym::KP_Enter => {
                if let Some(index) = self.selected {
                    self.navigate_into(index);
                }
            }
            Keysym::BackSpace => self.navigate_up(),
            Keysym::F5 => self.reload(),
            Keysym::Escape => self.context_menu = None,
            _ => {}
        }
    }
}

fn main() {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let start_dir = std::env::var("NOVA_FILES_DIR")
        .map(PathBuf::from)
        .or_else(|_| std::env::current_dir())
        .unwrap_or_else(|_| PathBuf::from("/"));

    nova_ui_wayland::run(
        "Nova Files",
        "os.nova.files",
        WINDOW_WIDTH,
        WINDOW_HEIGHT,
        NovaFiles::new(start_dir),
    );
}
