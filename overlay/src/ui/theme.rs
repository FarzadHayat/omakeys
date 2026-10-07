//! Omarchy theme colors from `~/.local/state/omarchy/current/theme/colors.toml`.

use std::fs;
use std::path::PathBuf;

use serde::Deserialize;

#[derive(Debug, Clone, PartialEq)]
pub struct Theme {
    pub background: String,
    pub foreground: String,
    pub accent: String,
    pub muted: String,
    pub selection: String,
}

impl Default for Theme {
    fn default() -> Self {
        Self {
            background: "#121c2b".into(),
            foreground: "#e8d4b0".into(),
            accent: "#e8b84a".into(),
            muted: "#4d5e75".into(),
            selection: "#2b2418".into(),
        }
    }
}

#[derive(Debug, Deserialize)]
struct ColorsFile {
    #[serde(default)]
    accent: Option<String>,
    #[serde(default)]
    background: Option<String>,
    #[serde(default)]
    foreground: Option<String>,
    #[serde(default)]
    muted: Option<String>,
    #[serde(default)]
    selection: Option<String>,
}

pub fn theme_path() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/state/omarchy/current/theme/colors.toml")
}

pub fn theme_from_str(s: &str) -> Theme {
    let mut theme = Theme::default();
    let Ok(parsed) = toml::from_str::<ColorsFile>(s) else {
        return theme;
    };
    if let Some(v) = parsed.background {
        theme.background = v;
    }
    if let Some(v) = parsed.foreground {
        theme.foreground = v;
    }
    if let Some(v) = parsed.accent {
        theme.accent = v;
    }
    if let Some(v) = parsed.muted {
        theme.muted = v;
    }
    if let Some(v) = parsed.selection {
        theme.selection = v;
    }
    theme
}

pub fn load_omarchy_theme() -> Theme {
    match fs::read_to_string(theme_path()) {
        Ok(s) => theme_from_str(&s),
        Err(_) => Theme::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_colors_toml() {
        let s = r##"
accent = "#e8b84a"
background = "#121c2b"
foreground = "#e8d4b0"
muted = "#4d5e75"
selection = "#2b2418"
"##;
        let t = theme_from_str(s);
        assert_eq!(t.accent, "#e8b84a");
        assert_eq!(t.background, "#121c2b");
    }
}
