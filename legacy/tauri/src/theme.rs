use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ThemeColors {
    pub accent: String,
    pub background: String,
    pub foreground: String,
    pub lighter_bg: String,
    pub dark_bg: String,
    pub muted: String,
    pub green: String,
    pub red: String,
    pub yellow: String,
    pub blue: String,
    pub cyan: String,
}

impl Default for ThemeColors {
    fn default() -> Self {
        Self {
            accent: "#6b8b80".into(),
            background: "#0D100E".into(),
            foreground: "#B1C2B1".into(),
            lighter_bg: "#252826".into(),
            dark_bg: "#0a0c0b".into(),
            muted: "#646a66".into(),
            green: "#b8cdb2".into(),
            red: "#9a9b82".into(),
            yellow: "#e9fdd9".into(),
            blue: "#6b8b80".into(),
            cyan: "#c2e9cc".into(),
        }
    }
}

fn colors_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_default()
        .join(".config/omarchy/current/theme/colors.toml")
}

fn parse_toml_colors(content: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for line in content.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        if let Some((key, value)) = line.split_once('=') {
            let key = key.trim().to_string();
            let value = value.trim().trim_matches('"').to_string();
            if !value.is_empty() {
                map.insert(key, value);
            }
        }
    }
    map
}

pub fn load_theme() -> ThemeColors {
    let path = colors_path();
    let Ok(content) = fs::read_to_string(path) else {
        return ThemeColors::default();
    };

    let colors = parse_toml_colors(&content);
    let pick = |key: &str, fallback: &str| {
        colors
            .get(key)
            .cloned()
            .unwrap_or_else(|| fallback.to_string())
    };

    ThemeColors {
        accent: pick("accent", "#6b8b80"),
        background: pick("background", "#0D100E"),
        foreground: pick("foreground", "#B1C2B1"),
        lighter_bg: pick("lighter_bg", "#252826"),
        dark_bg: pick("dark_bg", "#0a0c0b"),
        muted: pick("muted", "#646a66"),
        green: pick("green", "#b8cdb2"),
        red: pick("red", "#9a9b82"),
        yellow: pick("yellow", "#e9fdd9"),
        blue: pick("blue", "#6b8b80"),
        cyan: pick("cyan", "#c2e9cc"),
    }
}
