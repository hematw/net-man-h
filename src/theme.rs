use gtk4::gdk::RGBA;
use gtk4::prelude::*;
use gtk4::{CssProvider, STYLE_PROVIDER_PRIORITY_APPLICATION};
use std::collections::HashMap;
use std::fs;

#[derive(Clone, Debug)]
pub struct Theme {
    pub accent: String,
    pub background: String,
    pub foreground: String,
    pub surface: String,
    pub surface2: String,
    pub muted: String,
    pub green: String,
    pub red: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            accent: "#6b8b80".into(),
            background: "#0D100E".into(),
            foreground: "#B1C2B1".into(),
            surface: "#252826".into(),
            surface2: "#0a0c0b".into(),
            muted: "#646a66".into(),
            green: "#b8cdb2".into(),
            red: "#9a9b82".into(),
        }
    }
}

pub fn load_theme() -> Theme {
    let path = dirs::home_dir()
        .unwrap_or_default()
        .join(".config/omarchy/current/theme/colors.toml");

    let Ok(content) = fs::read_to_string(path) else {
        return Theme::default();
    };

    let colors = parse_toml(&content);
    let pick = |key: &str, fallback: &str| {
        colors
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    };

    Theme {
        accent: pick("accent", "#6b8b80"),
        background: pick("background", "#0D100E"),
        foreground: pick("foreground", "#B1C2B1"),
        surface: pick("lighter_bg", "#252826"),
        surface2: pick("dark_bg", "#0a0c0b"),
        muted: pick("muted", "#646a66"),
        green: pick("green", "#b8cdb2"),
        red: pick("red", "#9a9b82"),
    }
}

fn parse_toml(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let value = value.trim().trim_matches('"').to_string();
            if !value.is_empty() {
                map.insert(key.trim().to_string(), value);
            }
        }
    }
    map
}

impl Theme {
    pub fn apply(&self) {
        let provider = CssProvider::new();
        let css = format!(
            r#"
            window {{
                background-color: {bg};
                color: {fg};
            }}
            .sidebar {{
                background-color: {surface2};
                border-right: 1px solid alpha({fg}, 0.08);
            }}
            .content-card {{
                background-color: alpha({surface}, 0.92);
                border-radius: 16px;
                padding: 16px;
            }}
            .muted {{
                color: {muted};
                opacity: 0.95;
            }}
            .status-pill {{
                background-color: alpha({green}, 0.22);
                color: {fg};
                border-radius: 999px;
                padding: 4px 10px;
            }}
            .network-row:hover {{
                background-color: alpha({accent}, 0.12);
            }}
            .accent-button {{
                background: {accent};
                color: {bg};
                border-radius: 12px;
                padding: 8px 14px;
            }}
            .error-banner {{
                background-color: alpha({red}, 0.18);
                color: {fg};
                border-radius: 12px;
                padding: 10px 12px;
            }}
            "#,
            bg = self.background,
            fg = self.foreground,
            surface = self.surface,
            surface2 = self.surface2,
            muted = self.muted,
            accent = self.accent,
            green = self.green,
            red = self.red,
        );
        provider.load_from_string(&css);
        if let Some(display) = gtk4::gdk::Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                &provider,
                STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }

        let settings = libadwaita::StyleManager::default();
        settings.set_color_scheme(libadwaita::ColorScheme::ForceDark);
        if let Ok(accent) = RGBA::parse(&self.accent) {
            settings.set_accent_color_rgba(&accent);
        }
    }
}
