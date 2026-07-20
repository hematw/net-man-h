use std::cell::RefCell;
use std::rc::Rc;
use std::sync::Arc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Label, Orientation, Stack, StackSwitcher};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::nm::{NetworkService, NetworkSnapshot, NmError};
use crate::ui::pages::{self, Page, PageId};

pub struct AppWindow {
    window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    service: Arc<NetworkService>,
    snapshot: Rc<RefCell<NetworkSnapshot>>,
    pages: Vec<Page>,
    current_page: Rc<RefCell<PageId>>,
    status: Label,
}

impl AppWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("Aether Net")
            .default_width(920)
            .default_height(680)
            .build();

        let toast_overlay = adw::ToastOverlay::new();
        let service = Arc::new(NetworkService::new());
        let snapshot = Rc::new(RefCell::new(NetworkSnapshot::default()));
        let current_page = Rc::new(RefCell::new(PageId::Overview));

        let header = adw::HeaderBar::new();
        let refresh = gtk4::Button::from_icon_name("view-refresh-symbolic");
        refresh.set_tooltip_text(Some("Refresh"));
        header.pack_end(&refresh);

        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_hexpand(true);
        stack.set_vexpand(true);

        let overview = Page::overview();
        let wifi = Page::wifi();
        let ethernet = Page::ethernet();
        let vpn = Page::vpn();
        let hotspot = Page::hotspot();

        stack.add_named(overview.widget(), Some(PageId::Overview.as_str()));
        stack.add_named(wifi.widget(), Some(PageId::Wifi.as_str()));
        stack.add_named(ethernet.widget(), Some(PageId::Ethernet.as_str()));
        stack.add_named(vpn.widget(), Some(PageId::Vpn.as_str()));
        stack.add_named(hotspot.widget(), Some(PageId::Hotspot.as_str()));

        let switcher = StackSwitcher::new();
        switcher.set_stack(Some(&stack));
        header.set_title_widget(Some(&switcher));

        let status = Label::new(None);
        status.add_css_class("muted");

        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.append(&status);
        content.append(&stack);

        let root = GtkBox::new(Orientation::Vertical, 0);
        root.append(&header);
        root.append(&content);
        toast_overlay.set_child(Some(&root));
        window.set_content(Some(&toast_overlay));

        let this = Self {
            window,
            toast_overlay,
            service,
            snapshot,
            pages: vec![overview, wifi, ethernet, vpn, hotspot],
            current_page,
            status,
        };

        let win = this.clone_handle();
        refresh.connect_clicked(move |_| win.reload(false));

        stack.connect_visible_child_notify({
            let win = this.clone_handle();
            let current_page = this.current_page.clone();
            move |stack| {
                if let Some(name) = stack.visible_child_name() {
                    if let Some(id) = PageId::from_str(name.as_str()) {
                        *current_page.borrow_mut() = id;
                        win.reload(id == PageId::Wifi);
                    }
                }
            }
        });

        this.pages[1].set_wifi_handler({
            let win = this.clone_handle();
            move |action| win.handle_wifi_action(action)
        });
        this.pages[2].set_connection_handler({
            let win = this.clone_handle();
            move |action| win.handle_connection_action(action)
        });
        this.pages[3].set_connection_handler({
            let win = this.clone_handle();
            move |action| win.handle_connection_action(action)
        });
        this.pages[4].set_hotspot_handler({
            let win = this.clone_handle();
            move |action| win.handle_hotspot_action(action)
        });

        glib::timeout_add_seconds_local(20, {
            let win = this.clone_handle();
            let current_page = this.current_page.clone();
            move || {
                let rescan = *current_page.borrow() == PageId::Wifi;
                win.reload(rescan);
                glib::ControlFlow::Continue
            }
        });

        this.reload(false);
        this
    }

    pub fn present(&self) {
        self.window.present();
    }

    pub fn downgrade(&self) -> glib::WeakRef<adw::ApplicationWindow> {
        self.window.downgrade()
    }

    pub fn show_toast(&self, message: &str) {
        self.toast_overlay.add_toast(adw::Toast::new(message));
    }

    fn clone_handle(&self) -> WindowHandle {
        WindowHandle {
            window: self.window.clone(),
            toast_overlay: self.toast_overlay.clone(),
            service: self.service.clone(),
            snapshot: self.snapshot.clone(),
            pages: self.pages.iter().map(Page::clone_shallow).collect(),
            current_page: self.current_page.clone(),
            status: self.status.clone(),
        }
    }

    fn reload(&self, rescan_wifi: bool) {
        self.clone_handle().reload(rescan_wifi);
    }

    fn handle_wifi_action(&self, action: pages::WifiAction) {
        self.clone_handle().handle_wifi_action(action);
    }

    fn handle_connection_action(&self, action: pages::ConnectionAction) {
        self.clone_handle().handle_connection_action(action);
    }

    fn handle_hotspot_action(&self, action: pages::HotspotAction) {
        self.clone_handle().handle_hotspot_action(action);
    }
}

#[derive(Clone)]
struct WindowHandle {
    window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    service: Arc<NetworkService>,
    snapshot: Rc<RefCell<NetworkSnapshot>>,
    pages: Vec<Page>,
    current_page: Rc<RefCell<PageId>>,
    status: Label,
}

impl WindowHandle {
    fn reload(&self, rescan_wifi: bool) {
        let service = self.service.clone();
        let handle = self.clone();
        std::thread::spawn(move || {
            let result = service.snapshot(rescan_wifi);
            glib::MainContext::default().invoke(move || {
                handle.apply_snapshot(result);
            });
        });
    }

    fn apply_snapshot(&self, result: Result<NetworkSnapshot, NmError>) {
        match result {
            Ok(data) => {
                if data.connected {
                    self.status.set_text(&format!(
                        "{} · {} · {}",
                        data.connection_name, data.ip4, data.device
                    ));
                } else if data.wifi_enabled {
                    self.status.set_text("Not connected · WiFi on");
                } else {
                    self.status.set_text("Not connected · WiFi off");
                }
                for page in &self.pages {
                    page.render(&data);
                }
                *self.snapshot.borrow_mut() = data;
            }
            Err(err) => self.show_toast(&format!("Error: {err}")),
        }
    }

    fn show_toast(&self, message: &str) {
        self.toast_overlay.add_toast(adw::Toast::new(message));
    }

    fn handle_wifi_action(&self, action: pages::WifiAction) {
        let service = self.service.clone();
        let handle = self.clone();
        match action {
            pages::WifiAction::Toggle(enabled) => {
                std::thread::spawn(move || {
                    let result = service.set_wifi_enabled(enabled);
                    glib::MainContext::default().invoke(move || match result {
                        Ok(()) => handle.reload(false),
                        Err(err) => handle.show_toast(&err.to_string()),
                    });
                });
            }
            pages::WifiAction::Connect { ssid, password } => {
                std::thread::spawn(move || {
                    let result = service.connect_wifi(&ssid, password.as_deref());
                    glib::MainContext::default().invoke(move || match result {
                        Ok(()) => {
                            handle.show_toast(&format!("Connected to {ssid}"));
                            handle.reload(true);
                        }
                        Err(NmError::AuthRequired) => {
                            handle.show_toast("Enter password for this network");
                        }
                        Err(err) => handle.show_toast(&err.to_string()),
                    });
                });
            }
            pages::WifiAction::Disconnect => self.run_simple(|s| s.disconnect_wifi()),
            pages::WifiAction::Forget(ssid) => self.run_simple(move |s| s.forget_wifi(&ssid)),
            pages::WifiAction::Rescan => self.reload(true),
        }
    }

    fn handle_connection_action(&self, action: pages::ConnectionAction) {
        let service = self.service.clone();
        let handle = self.clone();
        match action {
            pages::ConnectionAction::Activate(uuid) => {
                std::thread::spawn(move || {
                    let result = service.activate_connection(&uuid);
                    glib::MainContext::default().invoke(move || match result {
                        Ok(()) => handle.reload(false),
                        Err(err) => handle.show_toast(&err.to_string()),
                    });
                });
            }
            pages::ConnectionAction::Deactivate(uuid) => {
                std::thread::spawn(move || {
                    let result = service.deactivate_connection(&uuid);
                    glib::MainContext::default().invoke(move || match result {
                        Ok(()) => handle.reload(false),
                        Err(err) => handle.show_toast(&err.to_string()),
                    });
                });
            }
            pages::ConnectionAction::Configure { uuid, config } => {
                std::thread::spawn(move || {
                    let result = service.save_ip_config(&uuid, &config);
                    glib::MainContext::default().invoke(move || match result {
                        Ok(()) => {
                            handle.show_toast("Settings saved");
                            handle.reload(false);
                        }
                        Err(err) => handle.show_toast(&err.to_string()),
                    });
                });
            }
        }
    }

    fn handle_hotspot_action(&self, action: pages::HotspotAction) {
        let service = self.service.clone();
        let handle = self.clone();
        match action {
            pages::HotspotAction::Start { ssid, password } => {
                std::thread::spawn(move || {
                    let result = service.create_hotspot(&ssid, &password);
                    glib::MainContext::default().invoke(move || match result {
                        Ok(()) => {
                            handle.show_toast("Hotspot started");
                            handle.reload(false);
                        }
                        Err(err) => handle.show_toast(&err.to_string()),
                    });
                });
            }
            pages::HotspotAction::Stop => self.run_simple(|s| s.stop_hotspot()),
        }
    }

    fn run_simple<F>(&self, action: F)
    where
        F: FnOnce(&NetworkService) -> Result<(), NmError> + Send + 'static,
    {
        let service = self.service.clone();
        let handle = self.clone();
        std::thread::spawn(move || {
            let result = action(&service);
            glib::MainContext::default().invoke(move || match result {
                Ok(()) => handle.reload(false),
                Err(err) => handle.show_toast(&err.to_string()),
            });
        });
    }
}
