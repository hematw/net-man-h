use gtk4::{glib, prelude::*};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::theme::load_theme;
use crate::ui::window::AppWindow;

static WINDOW: std::sync::OnceLock<glib::WeakRef<adw::ApplicationWindow>> =
    std::sync::OnceLock::new();

pub fn on_activate(app: &adw::Application) {
    if let Some(window) = WINDOW.get().and_then(|w| w.upgrade()) {
        window.present();
        return;
    }

    load_theme().apply();

    let window = AppWindow::new(app);
    let _ = WINDOW.set(window.downgrade());
    window.present();
}
