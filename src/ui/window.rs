use std::cell::RefCell;
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{Box as GtkBox, Button, Label, Orientation, Spinner, Stack, ToggleButton};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::nm::{IpConfig, NetworkService, NetworkSnapshot, NmError};
use crate::ui::pages::{
    prompt_password, show_ip_editor, ConnectionAction, Page, PageId, WifiAction,
};

pub struct AppWindow {
    window: adw::ApplicationWindow,
}

impl AppWindow {
    pub fn new(app: &adw::Application) -> Self {
        let window = adw::ApplicationWindow::builder()
            .application(app)
            .title("net-man-h")
            .default_width(920)
            .default_height(680)
            .build();
        window.add_css_class("aether-window");

        let toast_overlay = adw::ToastOverlay::new();
        let service = NetworkService::new();
        let snapshot = Rc::new(RefCell::new(NetworkSnapshot::default()));
        let current_page = Rc::new(RefCell::new(PageId::Overview));
        let busy = Rc::new(RefCell::new(None::<String>));

        let header = adw::HeaderBar::new();
        let refresh = Button::from_icon_name("view-refresh-symbolic");
        refresh.set_tooltip_text(Some("Refresh"));
        refresh.set_cursor_from_name(Some("pointer"));
        header.pack_end(&refresh);

        let sidebar = GtkBox::new(Orientation::Vertical, 8);
        sidebar.add_css_class("sidebar");
        sidebar.set_size_request(200, -1);

        let brand = Label::new(Some("net-man-h"));
        brand.add_css_class("brand-title");
        brand.set_halign(gtk4::Align::Start);
        let brand_sub = Label::new(Some("NetworkManager"));
        brand_sub.add_css_class("muted");
        brand_sub.set_halign(gtk4::Align::Start);
        sidebar.append(&brand);
        sidebar.append(&brand_sub);

        let stack = Stack::new();
        stack.set_transition_type(gtk4::StackTransitionType::Crossfade);
        stack.set_hexpand(true);
        stack.set_vexpand(true);

        let overview = Page::overview();
        let wifi = Page::wifi();
        let ethernet = Page::ethernet();

        stack.add_named(overview.widget(), Some(PageId::Overview.as_str()));
        stack.add_named(wifi.widget(), Some(PageId::Wifi.as_str()));
        stack.add_named(ethernet.widget(), Some(PageId::Ethernet.as_str()));

        let mut nav_buttons = Vec::new();
        for id in [PageId::Overview, PageId::Wifi, PageId::Ethernet] {
            let button = ToggleButton::with_label(id.label());
            button.add_css_class("nav-button");
            button.set_halign(gtk4::Align::Fill);
            button.set_cursor_from_name(Some("pointer"));
            if id == PageId::Overview {
                button.set_active(true);
            }
            sidebar.append(&button);
            nav_buttons.push((id, button));
        }

        let shared: Vec<ToggleButton> = nav_buttons.iter().map(|(_, b)| b.clone()).collect();
        for (id, button) in &nav_buttons {
            let id = *id;
            let stack = stack.clone();
            let current_page = current_page.clone();
            let shared = shared.clone();
            button.connect_clicked(move |btn| {
                for other in &shared {
                    if other != btn {
                        other.set_active(false);
                    }
                }
                btn.set_active(true);
                stack.set_visible_child_name(id.as_str());
                *current_page.borrow_mut() = id;
            });
        }

        let credit = Label::new(Some("built with 🩵 by hematw"));
        credit.add_css_class("credit");
        credit.set_halign(gtk4::Align::Start);
        credit.set_margin_top(16);
        credit.set_vexpand(true);
        credit.set_valign(gtk4::Align::End);
        sidebar.append(&credit);

        let status_row = GtkBox::new(Orientation::Horizontal, 8);
        let spinner = Spinner::new();
        spinner.set_visible(false);
        let status = Label::new(None);
        status.add_css_class("muted");
        status.set_halign(gtk4::Align::Start);
        status.set_wrap(true);
        status.set_hexpand(true);
        status_row.append(&spinner);
        status_row.append(&status);

        let content = GtkBox::new(Orientation::Vertical, 12);
        content.set_margin_top(12);
        content.set_margin_bottom(12);
        content.set_margin_start(12);
        content.set_margin_end(12);
        content.append(&status_row);
        content.append(&stack);

        let body = GtkBox::new(Orientation::Horizontal, 0);
        body.append(&sidebar);
        body.append(&content);

        let root = GtkBox::new(Orientation::Vertical, 0);
        root.append(&header);
        root.append(&body);
        toast_overlay.set_child(Some(&root));
        window.set_content(Some(&toast_overlay));

        let handle = WindowHandle {
            window: window.clone(),
            toast_overlay,
            service,
            snapshot,
            pages: vec![overview, wifi, ethernet],
            current_page: current_page.clone(),
            status,
            spinner,
            busy,
        };

        {
            let handle = handle.clone();
            refresh.connect_clicked(move |_| {
                let page = *handle.current_page.borrow();
                handle.reload(page == PageId::Wifi, page == PageId::Wifi);
            });
        }

        handle.pages[0].set_connection_handler({
            let handle = handle.clone();
            move |action| handle.handle_connection_action(action)
        });
        handle.pages[1].set_wifi_handler({
            let handle = handle.clone();
            move |action| handle.handle_wifi_action(action)
        });
        handle.pages[2].set_connection_handler({
            let handle = handle.clone();
            move |action| handle.handle_connection_action(action)
        });

        for (_, button) in &nav_buttons {
            let handle = handle.clone();
            button.connect_clicked(move |_| {
                let page = *handle.current_page.borrow();
                handle.reload(page == PageId::Wifi, false);
            });
        }

        glib::timeout_add_local(std::time::Duration::from_secs(20), {
            let handle = handle.clone();
            move || {
                if handle.busy.borrow().is_some() {
                    return glib::ControlFlow::Continue;
                }
                let page = *handle.current_page.borrow();
                handle.reload(page == PageId::Wifi, false);
                glib::ControlFlow::Continue
            }
        });

        handle.reload(false, false);

        Self { window }
    }

    pub fn present(&self) {
        self.window.present();
    }
}

#[derive(Clone)]
struct WindowHandle {
    window: adw::ApplicationWindow,
    toast_overlay: adw::ToastOverlay,
    service: NetworkService,
    snapshot: Rc<RefCell<NetworkSnapshot>>,
    pages: Vec<Page>,
    current_page: Rc<RefCell<PageId>>,
    status: Label,
    spinner: Spinner,
    busy: Rc<RefCell<Option<String>>>,
}

enum WorkerMsg {
    Snapshot(Result<NetworkSnapshot, NmError>),
    Toast(String),
    Connected(String),
    AuthRequired(String),
    Saved,
    OpenEditor {
        name: String,
        uuid: String,
        config: IpConfig,
    },
}

impl WindowHandle {
    fn set_busy(&self, label: Option<String>) {
        *self.busy.borrow_mut() = label.clone();
        let spinning = label.is_some();
        self.spinner.set_visible(spinning);
        self.spinner.set_spinning(spinning);
        if let Some(label) = &label {
            self.status.set_text(&format!("Connecting to {label}…"));
        }
        self.rerender();
    }

    fn rerender(&self) {
        let snapshot = self.snapshot.borrow().clone();
        let busy = self.busy.borrow().clone();
        for page in &self.pages {
            page.render(&snapshot, busy.as_deref());
        }
    }

    fn spawn<F>(&self, work: F)
    where
        F: FnOnce() -> WorkerMsg + Send + 'static,
    {
        let (sender, receiver) = std::sync::mpsc::channel();
        std::thread::spawn(move || {
            let _ = sender.send(work());
        });

        let handle = self.clone();
        glib::timeout_add_local(std::time::Duration::from_millis(20), move || {
            match receiver.try_recv() {
                Ok(msg) => {
                    handle.on_worker_msg(msg);
                    glib::ControlFlow::Break
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => glib::ControlFlow::Continue,
                Err(std::sync::mpsc::TryRecvError::Disconnected) => glib::ControlFlow::Break,
            }
        });
    }

    fn reload(&self, include_wifi: bool, rescan_wifi: bool) {
        let service = self.service.clone();
        self.spawn(move || WorkerMsg::Snapshot(service.snapshot(include_wifi, rescan_wifi)));
    }

    fn on_worker_msg(&self, msg: WorkerMsg) {
        match msg {
            WorkerMsg::Snapshot(result) => {
                self.set_busy(None);
                self.apply_snapshot(result);
            }
            WorkerMsg::Toast(message) => {
                self.set_busy(None);
                self.show_toast(&message);
                self.rerender();
            }
            WorkerMsg::Connected(ssid) => {
                self.set_busy(None);
                self.show_toast(&format!("Connected to {ssid}"));
                self.reload(true, true);
            }
            WorkerMsg::AuthRequired(ssid) => {
                self.set_busy(None);
                self.handle_wifi_action(WifiAction::AskPassword(ssid));
            }
            WorkerMsg::Saved => {
                self.set_busy(None);
                self.show_toast("Settings saved");
                self.reload(false, false);
            }
            WorkerMsg::OpenEditor { name, uuid, config } => {
                self.set_busy(None);
                let handle = self.clone();
                show_ip_editor(&self.window, &name, uuid, config, move |action| {
                    handle.handle_connection_action(action);
                });
            }
        }
    }

    fn apply_snapshot(&self, result: Result<NetworkSnapshot, NmError>) {
        match result {
            Ok(data) => {
                if self.busy.borrow().is_none() {
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
                }
                *self.snapshot.borrow_mut() = data;
                self.rerender();
            }
            Err(err) => self.show_toast(&format!("Error: {err}")),
        }
    }

    fn show_toast(&self, message: &str) {
        self.toast_overlay.add_toast(adw::Toast::new(message));
    }

    fn handle_wifi_action(&self, action: WifiAction) {
        let service = self.service.clone();
        match action {
            WifiAction::Toggle(enabled) => {
                self.set_busy(Some("WiFi".into()));
                self.spawn(move || match service.set_wifi_enabled(enabled) {
                    Ok(()) => WorkerMsg::Snapshot(service.snapshot(true, false)),
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
            WifiAction::AskPassword(ssid) => {
                let handle = self.clone();
                let ssid_for_dialog = ssid.clone();
                prompt_password(&self.window, &ssid_for_dialog, move |password| {
                    if let Some(password) = password {
                        handle.handle_wifi_action(WifiAction::Connect {
                            ssid,
                            password: Some(password),
                        });
                    }
                });
            }
            WifiAction::Connect { ssid, password } => {
                self.set_busy(Some(ssid.clone()));
                self.show_toast(&format!("Connecting to {ssid}…"));
                self.spawn(move || match service.connect_wifi(&ssid, password.as_deref()) {
                    Ok(()) => WorkerMsg::Connected(ssid),
                    Err(NmError::AuthRequired) => WorkerMsg::AuthRequired(ssid),
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
            WifiAction::Disconnect => {
                self.set_busy(Some("disconnect".into()));
                self.spawn(move || match service.disconnect_wifi() {
                    Ok(()) => WorkerMsg::Snapshot(service.snapshot(true, false)),
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
            WifiAction::Forget(ssid) => {
                self.set_busy(Some(ssid.clone()));
                self.spawn(move || match service.forget_wifi(&ssid) {
                    Ok(()) => WorkerMsg::Snapshot(service.snapshot(true, false)),
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
            WifiAction::Rescan => {
                self.set_busy(Some("scan".into()));
                self.status.set_text("Scanning networks…");
                self.reload(true, true);
            }
        }
    }

    fn handle_connection_action(&self, action: ConnectionAction) {
        let service = self.service.clone();
        match action {
            ConnectionAction::Activate(uuid) => {
                self.set_busy(Some(uuid.clone()));
                self.spawn(move || match service.activate_connection(&uuid) {
                    Ok(()) => WorkerMsg::Snapshot(service.snapshot(false, false)),
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
            ConnectionAction::Deactivate(uuid) => {
                self.set_busy(Some(uuid.clone()));
                self.spawn(move || match service.deactivate_connection(&uuid) {
                    Ok(()) => WorkerMsg::Snapshot(service.snapshot(false, false)),
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
            ConnectionAction::Edit(uuid) => {
                let snapshot = self.snapshot.borrow().clone();
                let name = snapshot
                    .connections
                    .iter()
                    .find(|c| c.uuid == uuid)
                    .map(|c| c.name.clone())
                    .unwrap_or_else(|| uuid.clone());
                self.spawn(move || match service.read_ip_config(&uuid) {
                    Ok(config) => WorkerMsg::OpenEditor { name, uuid, config },
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
            ConnectionAction::Save { uuid, config } => {
                self.set_busy(Some("save".into()));
                self.spawn(move || match service.save_ip_config(&uuid, &config) {
                    Ok(()) => WorkerMsg::Saved,
                    Err(err) => WorkerMsg::Toast(err.to_string()),
                });
            }
        }
    }
}
