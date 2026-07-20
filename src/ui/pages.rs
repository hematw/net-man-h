use std::cell::{Cell, RefCell};
use std::rc::Rc;

use gtk4::glib;
use gtk4::prelude::*;
use gtk4::{
    Align, Box as GtkBox, Button, CheckButton, Entry, Label, ListBox, ListBoxRow, Orientation,
    PasswordEntry, ScrolledWindow, SelectionMode, Spinner,
};
use libadwaita as adw;
use libadwaita::prelude::*;

use crate::nm::{ConnectionInfo, IpConfig, NetworkSnapshot, WifiNetwork};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PageId {
    Overview,
    Wifi,
    Ethernet,
}

impl PageId {
    pub fn as_str(self) -> &'static str {
        match self {
            PageId::Overview => "overview",
            PageId::Wifi => "wifi",
            PageId::Ethernet => "ethernet",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            PageId::Overview => "Overview",
            PageId::Wifi => "WiFi",
            PageId::Ethernet => "Ethernet",
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
    AskPassword(String),
}

#[derive(Clone)]
pub enum ConnectionAction {
    Activate(String),
    Deactivate(String),
    Edit(String),
    Save { uuid: String, config: IpConfig },
}

type WifiHandler = Rc<RefCell<Option<Rc<dyn Fn(WifiAction)>>>>;
type ConnectionHandler = Rc<RefCell<Option<Rc<dyn Fn(ConnectionAction)>>>>;

#[derive(Clone)]
pub struct Page {
    root: ScrolledWindow,
    list: ListBox,
    kind: PageId,
    wifi_handler: WifiHandler,
    connection_handler: ConnectionHandler,
    wifi_toggle: Option<CheckButton>,
    suppressing_toggle: Rc<Cell<bool>>,
}

impl Page {
    pub fn overview() -> Self {
        Self::simple(PageId::Overview)
    }

    pub fn ethernet() -> Self {
        Self::simple(PageId::Ethernet)
    }

    pub fn wifi() -> Self {
        let (root, list) = Self::scaffold(PageId::Wifi);
        let toolbar = GtkBox::new(Orientation::Horizontal, 8);
        let wifi_toggle = CheckButton::with_label("WiFi on");
        let scan = Button::with_label("Scan");
        let disconnect = Button::with_label("Disconnect");
        toolbar.append(&wifi_toggle);
        toolbar.append(&scan);
        toolbar.append(&disconnect);

        let toolbar_row = ListBoxRow::new();
        toolbar_row.set_selectable(false);
        toolbar_row.set_activatable(false);
        toolbar_row.set_child(Some(&toolbar));
        list.append(&toolbar_row);

        let wifi_handler: WifiHandler = Rc::new(RefCell::new(None));
        let suppressing_toggle = Rc::new(Cell::new(false));
        {
            let handler = wifi_handler.clone();
            let suppressing = suppressing_toggle.clone();
            wifi_toggle.connect_toggled(move |btn| {
                if suppressing.get() {
                    return;
                }
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
            wifi_toggle: Some(wifi_toggle),
            suppressing_toggle,
        }
    }

    fn simple(kind: PageId) -> Self {
        let (root, list) = Self::scaffold(kind);
        Self {
            root,
            list,
            kind,
            wifi_handler: Rc::new(RefCell::new(None)),
            connection_handler: Rc::new(RefCell::new(None)),
            wifi_toggle: None,
            suppressing_toggle: Rc::new(Cell::new(false)),
        }
    }

    fn scaffold(kind: PageId) -> (ScrolledWindow, ListBox) {
        let root = ScrolledWindow::builder()
            .hscrollbar_policy(gtk4::PolicyType::Never)
            .vexpand(true)
            .build();
        let container = GtkBox::new(Orientation::Vertical, 12);
        let heading = Label::new(Some(kind.label()));
        heading.add_css_class("title-2");
        heading.set_halign(Align::Start);
        container.append(&heading);
        let list = ListBox::new();
        list.add_css_class("boxed-list");
        list.set_selection_mode(SelectionMode::None);
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

    pub fn render(&self, snapshot: &NetworkSnapshot, busy: Option<&str>) {
        if let Some(toggle) = &self.wifi_toggle {
            self.suppressing_toggle.set(true);
            toggle.set_active(snapshot.wifi_enabled);
            self.suppressing_toggle.set(false);
        }

        let keep = if self.kind == PageId::Wifi { 1 } else { 0 };
        while self.list.row_at_index(keep).is_some() {
            if let Some(row) = self.list.row_at_index(keep) {
                self.list.remove(&row);
            }
        }

        match self.kind {
            PageId::Overview => self.render_overview(snapshot),
            PageId::Wifi => self.render_wifi(snapshot, busy),
            PageId::Ethernet => self.render_connections(
                snapshot
                    .connections
                    .iter()
                    .filter(|c| {
                        c.connection_type == "802-3-ethernet" || c.connection_type == "ethernet"
                    })
                    .cloned()
                    .collect(),
                busy,
            ),
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
        self.list
            .append(&info_row("Connection", &or_dash(&snapshot.connection_name)));
        self.list.append(&info_row("IPv4", &or_dash(&snapshot.ip4)));
        self.list
            .append(&info_row("Gateway", &or_dash(&snapshot.gateway)));
        self.list
            .append(&info_row("Device", &or_dash(&snapshot.device)));

        for device in &snapshot.devices {
            self.list.append(&info_row(
                &device.name,
                &format!("{} · {}", device.device_type, device.state),
            ));
        }

        let active: Vec<_> = snapshot.connections.iter().filter(|c| c.active).collect();
        if !active.is_empty() {
            self.list.append(&section_label("Active profiles"));
            for conn in active {
                let handler = self.connection_handler.clone();
                let uuid = conn.uuid.clone();
                let row = action_row(&conn.name, &conn.connection_type, "Configure", move || {
                    if let Some(cb) = handler.borrow().as_ref() {
                        cb(ConnectionAction::Edit(uuid.clone()));
                    }
                });
                self.list.append(&row);
            }
        }
    }

    fn render_wifi(&self, snapshot: &NetworkSnapshot, busy: Option<&str>) {
        if !snapshot.wifi_enabled {
            self.list
                .append(&section_label("WiFi is off. Enable it to scan."));
            return;
        }

        let mut saved: Vec<&WifiNetwork> =
            snapshot.wifi_networks.iter().filter(|n| n.saved).collect();
        let mut other: Vec<&WifiNetwork> =
            snapshot.wifi_networks.iter().filter(|n| !n.saved).collect();
        saved.sort_by_key(|n| std::cmp::Reverse(n.signal));
        other.sort_by_key(|n| std::cmp::Reverse(n.signal));

        if !saved.is_empty() {
            self.list.append(&section_label("Saved"));
            for network in saved {
                self.list.append(&self.wifi_row(network, busy));
            }
        }
        if !other.is_empty() {
            self.list.append(&section_label("Available"));
            for network in other {
                self.list.append(&self.wifi_row(network, busy));
            }
        }
        if snapshot.wifi_networks.is_empty() {
            self.list
                .append(&section_label("No networks found. Try Scan."));
        }
    }

    fn wifi_row(&self, network: &WifiNetwork, busy: Option<&str>) -> ListBoxRow {
        let row = ListBoxRow::new();
        row.add_css_class("network-row");
        let row_box = GtkBox::new(Orientation::Horizontal, 12);
        row_box.set_margin_top(8);
        row_box.set_margin_bottom(8);
        row_box.set_margin_start(8);
        row_box.set_margin_end(8);

        let text = GtkBox::new(Orientation::Vertical, 2);
        text.set_hexpand(true);
        let title = Label::new(Some(&network.ssid));
        title.set_halign(Align::Start);
        let subtitle = Label::new(Some(&format!(
            "{} · {}%{}",
            network.security,
            network.signal,
            if network.saved { " · Saved" } else { "" }
        )));
        subtitle.add_css_class("muted");
        subtitle.set_halign(Align::Start);
        text.append(&title);
        text.append(&subtitle);

        let actions = GtkBox::new(Orientation::Horizontal, 6);
        let is_busy = busy == Some(network.ssid.as_str());
        let any_busy = busy.is_some();

        if network.in_use {
            let pill = Label::new(Some("Connected"));
            pill.add_css_class("status-pill");
            actions.append(&pill);
        } else if is_busy {
            let spinner = Spinner::new();
            spinner.set_spinning(true);
            let label = Label::new(Some("Connecting…"));
            label.add_css_class("muted");
            actions.append(&spinner);
            actions.append(&label);
        } else {
            let connect = Button::with_label("Connect");
            connect.add_css_class("accent-button");
            connect.set_cursor_from_name(Some("pointer"));
            connect.set_sensitive(!any_busy);
            let handler = self.wifi_handler.clone();
            let ssid = network.ssid.clone();
            let secured = network.security != "Open";
            let saved = network.saved;
            connect.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    if secured && !saved {
                        cb(WifiAction::AskPassword(ssid.clone()));
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

        if network.saved && !is_busy {
            let forget = Button::with_label("Forget");
            forget.set_cursor_from_name(Some("pointer"));
            forget.set_sensitive(!any_busy);
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

    fn render_connections(&self, connections: Vec<ConnectionInfo>, busy: Option<&str>) {
        if connections.is_empty() {
            self.list.append(&section_label("No ethernet profiles"));
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
            text.set_hexpand(true);
            let title = Label::new(Some(&conn.name));
            title.set_halign(Align::Start);
            let subtitle = Label::new(Some(&or_dash(&conn.device)));
            subtitle.add_css_class("muted");
            text.append(&title);
            text.append(&subtitle);

            let actions = GtkBox::new(Orientation::Horizontal, 6);
            let uuid = conn.uuid.clone();
            let is_busy = busy == Some(uuid.as_str()) || busy == Some(conn.name.as_str());
            let any_busy = busy.is_some();

            if is_busy {
                let spinner = Spinner::new();
                spinner.set_spinning(true);
                let label = Label::new(Some("Working…"));
                label.add_css_class("muted");
                actions.append(&spinner);
                actions.append(&label);
            } else if conn.active {
                let btn = Button::with_label("Disconnect");
                btn.set_cursor_from_name(Some("pointer"));
                btn.set_sensitive(!any_busy);
                let handler = self.connection_handler.clone();
                let uuid = uuid.clone();
                btn.connect_clicked(move |_| {
                    if let Some(cb) = handler.borrow().as_ref() {
                        cb(ConnectionAction::Deactivate(uuid.clone()));
                    }
                });
                actions.append(&btn);
            } else {
                let btn = Button::with_label("Connect");
                btn.add_css_class("accent-button");
                btn.set_cursor_from_name(Some("pointer"));
                btn.set_sensitive(!any_busy);
                let handler = self.connection_handler.clone();
                let uuid = uuid.clone();
                btn.connect_clicked(move |_| {
                    if let Some(cb) = handler.borrow().as_ref() {
                        cb(ConnectionAction::Activate(uuid.clone()));
                    }
                });
                actions.append(&btn);
            }

            let edit = Button::with_label("IP");
            edit.set_cursor_from_name(Some("pointer"));
            edit.set_sensitive(!any_busy);
            let handler = self.connection_handler.clone();
            edit.connect_clicked(move |_| {
                if let Some(cb) = handler.borrow().as_ref() {
                    cb(ConnectionAction::Edit(uuid.clone()));
                }
            });
            actions.append(&edit);

            row_box.append(&text);
            row_box.append(&actions);
            row.set_child(Some(&row_box));
            self.list.append(&row);
        }
    }
}

pub fn prompt_password(
    parent: &impl IsA<gtk4::Window>,
    ssid: &str,
    on_done: impl FnOnce(Option<String>) + 'static,
) {
    let dialog = adw::MessageDialog::new(
        Some(parent),
        Some(&format!("Connect to {ssid}")),
        Some("Enter the WiFi password."),
    );
    dialog.add_response("cancel", "Cancel");
    dialog.add_response("connect", "Connect");
    dialog.set_response_appearance("connect", adw::ResponseAppearance::Suggested);
    dialog.set_default_response(Some("connect"));
    dialog.set_close_response("cancel");

    let entry = PasswordEntry::new();
    entry.set_show_peek_icon(true);
    entry.set_hexpand(true);
    dialog.set_extra_child(Some(&entry));

    let on_done = std::cell::RefCell::new(Some(on_done));
    dialog.connect_response(None, move |dialog, response| {
        let Some(on_done) = on_done.borrow_mut().take() else {
            return;
        };
        if response == "connect" {
            let password = dialog
                .extra_child()
                .and_then(|child| child.downcast::<PasswordEntry>().ok())
                .map(|entry| entry.text().to_string())
                .filter(|value| !value.is_empty());
            on_done(password);
        } else {
            on_done(None);
        }
    });
    dialog.present();
}

pub fn show_ip_editor(
    parent: &impl IsA<gtk4::Window>,
    name: &str,
    uuid: String,
    config: IpConfig,
    on_save: impl Fn(ConnectionAction) + 'static,
) {
    let dialog = gtk4::Window::builder()
        .transient_for(parent)
        .modal(true)
        .title(format!("IP settings · {name}"))
        .default_width(420)
        .default_height(420)
        .build();

    let root = GtkBox::new(Orientation::Vertical, 12);
    root.set_margin_top(16);
    root.set_margin_bottom(16);
    root.set_margin_start(16);
    root.set_margin_end(16);

    let method = CheckButton::with_label("Use static IPv4");
    method.set_active(config.method == "manual");
    method.set_cursor_from_name(Some("pointer"));

    let static_fields = GtkBox::new(Orientation::Vertical, 12);
    let address = Entry::new();
    address.set_placeholder_text(Some("192.168.1.50/24"));
    address.set_text(&config.addresses);
    let gateway = Entry::new();
    gateway.set_placeholder_text(Some("192.168.1.1"));
    gateway.set_text(&config.gateway);
    let dns = Entry::new();
    dns.set_placeholder_text(Some("1.1.1.1 8.8.8.8"));
    dns.set_text(&config.dns);
    static_fields.append(&labeled("Address / prefix", &address));
    static_fields.append(&labeled("Gateway", &gateway));
    static_fields.append(&labeled("DNS", &dns));
    static_fields.set_visible(config.method == "manual");

    {
        let static_fields = static_fields.clone();
        method.connect_toggled(move |btn| {
            static_fields.set_visible(btn.is_active());
        });
    }

    let mtu = Entry::new();
    mtu.set_placeholder_text(Some("1500"));
    if let Some(value) = config.mtu {
        mtu.set_text(&value.to_string());
    }
    let autoconnect = CheckButton::with_label("Auto-connect");
    autoconnect.set_active(config.autoconnect);
    autoconnect.set_cursor_from_name(Some("pointer"));

    let dhcp_hint = Label::new(Some("DHCP will assign address, gateway, and DNS automatically."));
    dhcp_hint.add_css_class("muted");
    dhcp_hint.set_halign(Align::Start);
    dhcp_hint.set_wrap(true);
    dhcp_hint.set_visible(config.method != "manual");
    {
        let dhcp_hint = dhcp_hint.clone();
        method.connect_toggled(move |btn| {
            dhcp_hint.set_visible(!btn.is_active());
        });
    }

    root.append(&method);
    root.append(&dhcp_hint);
    root.append(&static_fields);
    root.append(&labeled("MTU", &mtu));
    root.append(&autoconnect);

    let actions = GtkBox::new(Orientation::Horizontal, 8);
    actions.set_halign(Align::End);
    let cancel = Button::with_label("Cancel");
    cancel.set_cursor_from_name(Some("pointer"));
    let save = Button::with_label("Save & apply");
    save.add_css_class("accent-button");
    save.set_cursor_from_name(Some("pointer"));
    actions.append(&cancel);
    actions.append(&save);
    root.append(&actions);
    dialog.set_child(Some(&root));

    {
        let dialog = dialog.clone();
        cancel.connect_clicked(move |_| dialog.close());
    }
    {
        let dialog = dialog.clone();
        save.connect_clicked(move |_| {
            let config = IpConfig {
                method: if method.is_active() {
                    "manual".into()
                } else {
                    "auto".into()
                },
                addresses: address.text().to_string(),
                gateway: gateway.text().to_string(),
                dns: dns.text().to_string(),
                mtu: mtu.text().parse().ok(),
                autoconnect: autoconnect.is_active(),
            };
            on_save(ConnectionAction::Save {
                uuid: uuid.clone(),
                config,
            });
            dialog.close();
        });
    }

    dialog.present();
}

fn labeled(title: &str, widget: &impl IsA<gtk4::Widget>) -> GtkBox {
    let box_ = GtkBox::new(Orientation::Vertical, 4);
    let label = Label::new(Some(title));
    label.set_halign(Align::Start);
    label.add_css_class("muted");
    box_.append(&label);
    box_.append(widget);
    box_
}

fn info_row(title: &str, value: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    let box_ = GtkBox::new(Orientation::Horizontal, 12);
    box_.set_margin_top(8);
    box_.set_margin_bottom(8);
    box_.set_margin_start(8);
    box_.set_margin_end(8);
    let left = Label::new(Some(title));
    left.set_halign(Align::Start);
    left.set_hexpand(true);
    let right = Label::new(Some(value));
    right.set_halign(Align::End);
    box_.append(&left);
    box_.append(&right);
    row.set_child(Some(&box_));
    row
}

fn section_label(text: &str) -> ListBoxRow {
    let row = ListBoxRow::new();
    row.set_selectable(false);
    row.set_activatable(false);
    let label = Label::new(Some(text));
    label.add_css_class("title-4");
    label.set_margin_top(8);
    label.set_margin_start(8);
    label.set_halign(Align::Start);
    row.set_child(Some(&label));
    row
}

fn action_row(
    title: &str,
    subtitle: &str,
    button_label: &str,
    on_click: impl Fn() + 'static,
) -> ListBoxRow {
    let row = ListBoxRow::new();
    let row_box = GtkBox::new(Orientation::Horizontal, 12);
    row_box.set_margin_top(8);
    row_box.set_margin_bottom(8);
    row_box.set_margin_start(8);
    row_box.set_margin_end(8);
    let text = GtkBox::new(Orientation::Vertical, 2);
    text.set_hexpand(true);
    let title_l = Label::new(Some(title));
    title_l.set_halign(Align::Start);
    let sub = Label::new(Some(subtitle));
    sub.add_css_class("muted");
    sub.set_halign(Align::Start);
    text.append(&title_l);
    text.append(&sub);
    let button = Button::with_label(button_label);
    button.connect_clicked(move |_| on_click());
    row_box.append(&text);
    row_box.append(&button);
    row.set_child(Some(&row_box));
    row
}

fn or_dash(value: &str) -> String {
    if value.is_empty() {
        "—".into()
    } else {
        value.to_string()
    }
}

// Keep glib available for dialog helpers that may use it.
#[allow(unused_imports)]
use glib as _;
