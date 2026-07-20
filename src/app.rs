use gtk4::prelude::*;
use libadwaita as adw;

use crate::theme::load_theme;
use crate::ui::AppWindow;

pub fn on_activate(app: &adw::Application) {
    if let Some(window) = app.active_window() {
        window.present();
        return;
    }

    // Also check windows() in case focus is elsewhere.
    if let Some(window) = app.windows().into_iter().next() {
        window.present();
        return;
    }

    load_theme().apply();
    let window = AppWindow::new(app);
    window.present();
}
