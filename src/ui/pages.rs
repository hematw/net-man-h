use std::cell::RefCell;
use std::rc::Rc;

use gtk4::prelude::*;
use gtk4::{
    Box as GtkBox, Button, CheckButton, Entry, Label, ListBox, ListBoxRow, Orientation,
    ScrolledWindow, Separator,
};

use crate::nm::{ConnectionInfo, IpConfig, NetworkSnapshot, WifiNetwork};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Overview,
    Wifi,
    Ethernet,
    Vpn,
    Hotspot,
}

impl PageId {
    pub fn as_str(self) -> &'static str {
        match self {
            PageId::Overview => "overview",
            PageId::Wifi => "wifi",
            PageId::Ethernet => "ethernet",
            PageId::Vpn => "vpn",
            PageId::Hotspot => "hotspot",
        }
    }

    pub fn from_str(value: &str) -> Option<Self> {
        match value {
            "overview" => Some(PageId::Overview),
            "wifi" => Some(PageId::Wifi),
            "ethernet" => Some(PageId::Ethernet),
            "vpn" => Some(PageId::Vpn),
            "hotspot" => Some(PageId::Hotspot),
            _ => None,
        }
    }
}

#[derive(Clone)]
pub enum WifiAction {
    Toggle(bool),
    Connect { ssid: String, password: Option<String> },
    Disconnect,
    Forget(String),
    Rescan,
}

#[derive(Clone)]
pub enum ConnectionAction {
    Activate(String),
    Deactivate(String),
    Configure { uuid: String, config: IpConfig },
}

#[derive(Clone)]
pub enum HotspotAction {
    Start { ssid: String, password: String },
    Stop,
}

type WifiHandler = Rc<RefCell<Option<Rc<dyn Fn(WifiAction)>>>>;
type ConnectionHandler = Rc<RefCell<Option<Rc<dyn Fn(ConnectionAction)>>>>;
type HotspotHandler = Rc<RefCell<Option<Rc<dyn Fn(HotspotAction)>>>>;

#[derive(Clone)]
pub struct Page {
    root: ScrolledWindow,
    list: ListBox,
    kind: PageId,
    wifi_handler: WifiHandler,
    connection_handler: ConnectionHandler,
    hotspot_handler: HotspotHandler,
    wifi_toggle: Option<CheckButton>,
    hotspot_ssid: Option<Entry>,
    hotspot_password: Option<Entry>,
}

impl Page {
    pub fn overview() -> Self {
        Self::simple(PageId::Overview, "Overview")
    }

    pub fn ethernet() -> Self {
        Self::simple(PageId::Ethernet, "Ethernet profiles")
    }

    pub fn vpn() -> Self {
        Self::simple(PageId::Vpn, "VPN profiles")
    }

    pub fn wifi() -> Self {
        let (root, list) = Self::scaffold("WiFi");
        let toolbar = GtkBox::new(Orientation::Horizontal, 8);
        let wifi_toggle = CheckButton::with_label("WiFi on");
        let scan = Button::with_label("Scan");
        let disconnect = Button::with_label("Disconnect");
        toolbar.append(&wifi_toggle);
        toolbar.append(&scan);
        toolbar.append(&disconnect);
        list.prepend(&toolbar);

        let wifi_handler: WifiHandler = Rc::new(RefCell::new(None));
        {
            let handler = wifi_handler.clone();
            wifi_toggle.connect_toggled(move |btn| {
                if let Some(cb) = handler.borrow().as_ref() {
                    cb(WifiAction::Toggle(btn.is_active()));
                }
            });
        }
        {
            let handler = wifi_handler.clone();
            scan.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    cb(WifiAction::Rescan);
                }
            });
        }
        {
            let handler = wifi_handler.clone();
            disconnect.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    cb(WifiAction::Disconnect);
                }
            });
        }

        Self {
            root,
            list,
            kind: PageId::Wifi,
            wifi_handler,
            connection_handler: Rc::new(RefCell::new(None)),
            hotspot_handler: Rc::new(RefCell::new(None)),
            wifi_toggle: Some(wifi_toggle),
            hotspot_ssid: None,
            hotspot_password: None,
        }
    }

    pub fn hotspot() -> Self {
        let (root, list) = Self::scaffold("Hotspot");
        let ssid = Entry::new();
        ssid.set_text("Aether Hotspot");
        ssid.set_placeholder_text(Some("Network name"));
        let password = Entry::new();
        password.set_visibility(false);
        password.set_placeholder_text(Some("Password (8+ chars)"));
        let start = Button::with_label("Start hotspot");
        let stop = Button::with_label("Stop hotspot");

        let box_ = GtkBox::new(Orientation::Vertical, 8);
        box_.append(&Label::new(Some("SSID")));
        box_.append(&ssid);
        box_.append(&Label::new(Some("Password")));
        box_.append(&password);
        let actions = GtkBox::new(Orientation::Horizontal, 8);
        actions.append(&start);
        actions.append(&stop);
        box_.append(&actions);
        list.append(&box_);

        let hotspot_handler: HotspotHandler = Rc::new(RefCell::new(None));
        {
            let handler = hotspot_handler.clone();
            let ssid = ssid.clone();
            let password = password.clone();
            start.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    cb(HotspotAction::Start {
                        ssid: ssid.text().to_string(),
                        password: password.text().to_string(),
                    });
                }
            });
        }
        {
            let handler = hotspot_handler.clone();
            stop.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    cb(HotspotAction::Stop);
                }
            });
        }

        Self {
            root,
            list,
            kind: PageId::Hotspot,
            wifi_handler: Rc::new(RefCell::new(None)),
            connection_handler: Rc::new(RefCell::new(None)),
            hotspot_handler,
            wifi_toggle: None,
            hotspot_ssid: Some(ssid),
            hotspot_password: Some(password),
        }
    }

    fn simple(kind: PageId, title: &str) -> Self {
        let (root, list) = Self::scaffold(title);
        Self {
            root,
            list,
            kind,
            wifi_handler: Rc::new(RefCell::new(None)),
            connection_handler: Rc::new(RefCell::new(None)),
            hotspot_handler: Rc::new(RefCell::new(None)),
            wifi_toggle: None,
            hotspot_ssid: None,
            hotspot_password: None,
        }
    }

    fn scaffold(title: &str) -> (ScrolledWindow, ListBox) {
        let root = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();
        let container = GtkBox::new(Orientation::Vertical, 12);
        let heading = Label::new(Some(title));
        heading.add_css_class("title-2");
        container.append(&heading);
        let list = ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(gtk4::SelectionMode::None);
        container.append(&list);
        root.set_child(Some(&container));
        (root, list)
    }

    pub fn widget(&self) -> &ScrolledWindow {
        &self.root
    }

    pub fn clone_shallow(&self) -> Self {
        self.clone()
    }

    pub fn set_wifi_handler<F: Fn(WifiAction) + 'static>(&self, handler: F) {
        *self.wifi_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn set_connection_handler<F: Fn(ConnectionAction) + 'static>(&self, handler: F) {
        *self.connection_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn set_hotspot_handler<F: Fn(HotspotAction) + 'static>(&self, handler: F) {
        *self.hotspot_handler.borrow_mut() = Some(Rc::new(handler));
    }

    pub fn render(&self, snapshot: &NetworkSnapshot) {
        if let Some(toggle) = &self.wifi_toggle {
            toggle.block_signal_handlers();
            toggle.set_active(snapshot.wifi_enabled);
            toggle.unblock_signal_handlers();
        }

        let keep = if self.kind == PageId::Wifi || self.kind == PageId::Hotspot {
            1
        } else {
            0
        };
        while self.list.row_at_index(keep).is_some() {
            if let Some(row) = self.list.row_at_index(keep) {
                self.list.remove(&row);
            }
        }

        match self.kind {
            PageId::Overview => self.render_overview(snapshot),
            PageId::Wifi => self.render_wifi(snapshot),
            PageId::Ethernet => self.render_connections(
                snapshot
                    .connections
                    .iter()
                    .filter(|c| c.connection_type == "802-3-ethernet")
                    .cloned()
                    .collect(),
            ),
            PageId::Vpn => self.render_connections(
                snapshot
                    .connections
                    .iter()
                    .filter(|c| c.connection_type == "vpn")
                    .cloned()
                    .collect(),
            ),
            PageId::Hotspot => {
                if snapshot.hotspot_active {
                    self.list.append(&section_label(&format!(
                        "Hotspot active: {}",
                        snapshot.hotspot_ssid
                    )));
                }
            }
        }
    }

    fn render_overview(&self, snapshot: &NetworkSnapshot) {
        self.list.append(&info_row(
            "Status",
            if snapshot.connected {
                "Online"
            } else {
                "Offline"
            },
        ));
        self.list.append(&info_row("IPv4", &snapshot.ip4));
        self.list.append(&info_row("Gateway", &snapshot.gateway));
        self.list.append(&Separator::new());
        for device in &snapshot.devices {
            self.list.append(&info_row(
                &device.name,
                &format!("{} · {}", device.device_type, device.state),
            ));
        }
    }

    fn render_wifi(&self, snapshot: &NetworkSnapshot) {
        let mut saved: Vec<&WifiNetwork> = snapshot.wifi_networks.iter().filter(|n| n.saved).collect();
        let mut other: Vec<&WifiNetwork> = snapshot.wifi_networks.iter().filter(|n| !n.saved).collect();
        saved.sort_by_key(|n| std::cmp::Reverse(n.signal));
        other.sort_by_key(|n| std::cmp::Reverse(n.signal));

        if !saved.is_empty() {
            self.list.append(&section_label("Saved"));
            for network in saved {
                self.list.append(&self.wifi_row(network));
            }
        }
        if !other.is_empty() {
            self.list.append(&section_label("Available"));
            for network in other {
                self.list.append(&self.wifi_row(network));
            }
        }
        if snapshot.wifi_networks.is_empty() {
            self.list.append(&section_label("No networks found. Try Scan."));
        }
    }

    fn wifi_row(&self, network: &WifiNetwork) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("network-row");
        let row_box = GtkBox::new(Orientation::Horizontal, 12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);

        let text = GtkBox::new(Orientation::Vertical, 2);
        let title = Label::new(Some(&network.ssid));
        title.set_halign(gtk4::Align::Start);
        let subtitle = Label::new(Some(&format!(
            "{} · {}%{}",
            network.security,
            network.signal,
            if network.saved { " · Saved" } else { "" }
        )));
        subtitle.add_css_class("muted");
        subtitle.set_halign(gtk4::Align::Start);
        text.append(&title);
        text.append(&subtitle);

        let actions = GtkBox::new(Orientation::Horizontal, 6);
        if network.in_use {
            let pill = Label::new(Some("Connected"));
            pill.add_css_class("status-pill");
            actions.append(&pill);
        } else {
            let connect = Button::with_label("Connect");
            connect.add_css_class("accent-button");
            let handler = self.wifi_handler.clone();
            let ssid = network.ssid.clone();
            let secured = network.security != "Open";
            let saved = network.saved;
            connect.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    if secured && !saved {
                        cb(WifiAction::Connect {
                            ssid: ssid.clone(),
                            password: None,
                        });
                    } else {
                        cb(WifiAction::Connect {
                            ssid: ssid.clone(),
                            password: None,
                        });
                    }
                }
            });
            actions.append(&connect);
        }

        if network.saved {
            let forget = Button::with_label("Forget");
            let handler = self.wifi_handler.clone();
            let ssid = network.ssid.clone();
            forget.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    cb(WifiAction::Forget(ssid.clone()));
                }
            });
            actions.append(&forget);
        }

        row_box.append(&text);
        row_box.append(&actions);
        row.set_child(Some(&row_box));
        row
    }

    fn render_connections(&self, connections: Vec<ConnectionInfo>) {
        if connections.is_empty() {
            self.list.append(&section_label("No profiles found"));
            return;
        }
        for conn in connections {
            let row = ListBoxRow::new();
            let row_box = GtkBox::new(Orientation::Horizontal, 12);
            row_box.set_margin_top(8);
            row_box.set_margin_bottom(8);
            row_box.set_margin_start(8);
            row_box.set_margin_end(8);

            let text = GtkBox::new(Orientation::Vertical, 2);
            let title = Label::new(Some(&conn.name));
            title.set_halign(gtk4::Align::Start);
            let subtitle = Label::new(Some(&conn.connection_type));
            subtitle.add_css_class("muted");
            text.append(&title);
            text.append(&subtitle);

            let actions = GtkBox::new(Orientation::Horizontal, 6);
            let handler = self.connection_handler.clone();
            let uuid = conn.uuid.clone();
            if conn.active {
                let btn = Button::with_label("Disconnect");
                btn.connect_clicked(move |_| {
                    if let Some(cb) = handler.borrow().as_ref() {
                        cb(ConnectionAction::Deactivate(uuid.clone()));
                    }
                });
                actions.append(&btn);
            } else {
                let btn = Button::with_label("Connect");
                btn.add_css_class("accent-button");
                let handler = self.connection_handler.clone();
                let uuid = conn.uuid.clone();
                btn.connect_clicked(move |_| {
                    if let Some(cb) = handler.borrow().as_ref() {
                        cb(ConnectionAction::Activate(uuid.clone()));
                    }
                });
                actions.append(&btn);
            }

            row_box.append(&text);
            row_box.append(&actions);
            row.set_child(Some(&row_box));
            self.list.append(&row);
        }
    }
}

fn info_row(title: &str, value: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    let box_ = GtkBox::new(Orientation::Horizontal, 12);
    box_.set_margin_top(8);
    box_.set_margin_bottom(8);
    box_.set_margin_start(8);
    box_.set_margin_end(8);
    let left = Label::new(Some(title));
    left.set_halign(gtk4::Align::Start);
    left.set_hexpand(true);
    let right = Label::new(Some(value));
    right.set_halign(gtk4::Align::End);
    box_.append(&left);
    box_.append(&right);
    row.set_child(Some(&box_));
    row
}

fn section_label(text: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_selectable(false);
    let label = Label::new(Some(text));
    label.add_css_class("title-4");
    label.set_margin_top(8);
    label.set_margin_start(8);
    row.set_child(Some(&label));
    row
}
