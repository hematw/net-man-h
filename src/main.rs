mod app;
mod nm;
mod theme;
mod ui;

use gtk4::prelude::*;
use gtk4::{gio, glib};
use libadwaita as adw;
use libadwaita::prelude::*;

const APP_ID: &str = "io.github.hemat.AetherNet";

fn main() -> glib::ExitCode {
    adw::init().expect("Failed to initialize libadwaita");

    let app = adw::Application::builder().application_id(APP_ID).build();

    app.connect_activate(|app| {
        app::on_activate(app);
    });

    app.run()
}
