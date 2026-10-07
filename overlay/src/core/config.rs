//! `~/.config/omakeys/config.toml`

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

use crate::board::labels::LabelStyle;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisibilityMode {
    /// Legacy: treated as [`Auto`] on load. Temp hide is the overlay toggle.
    Hidden,
    Always,
    #[default]
    Auto,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Anchor {
    TopLeft,
    Top,
    TopRight,
    Left,
    Right,
    BottomLeft,
    Bottom,
    #[default]
    BottomRight,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AppConfig {
    pub mode: VisibilityMode,
    pub timeout_ms: u64,
    pub anchor: Anchor,
    pub opacity: f64,
    /// Overlay paint scale (1.0 = current default key size). Clamped to
    /// 0.50..=1.50 and snapped to 0.10 steps.
    #[serde(default = "default_scale")]
    pub scale: f64,
    pub toggle_chord: String,
    /// Legacy field; ignored (Omarchy bar widget replaced the SNI tray).
    #[serde(default = "default_tray_visible")]
    pub tray_visible: bool,
    #[serde(default)]
    pub label_style: LabelStyle,
}

fn default_tray_visible() -> bool {
    true
}

fn default_scale() -> f64 {
    1.0
}

/// Clamp to 50%–150% and snap to 10% steps.
pub fn normalize_scale(scale: f64) -> f64 {
    if !scale.is_finite() {
        return default_scale();
    }
    let pct = (scale * 100.0).round().clamp(50.0, 150.0);
    let stepped = (pct / 10.0).round() * 10.0;
    stepped / 100.0
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            mode: VisibilityMode::Auto,
            timeout_ms: 2000,
            anchor: Anchor::BottomRight,
            opacity: 0.92,
            scale: default_scale(),
            toggle_chord: "SUPER+ALT+O".into(),
            tray_visible: default_tray_visible(),
            label_style: LabelStyle::Icons,
        }
    }
}

impl AppConfig {
    pub fn normalized(mut self) -> Self {
        // Mode Hidden removed from UI — temp mute is force_hidden / toggle.
        if self.mode == VisibilityMode::Hidden {
            self.mode = VisibilityMode::Auto;
        }
        self.scale = normalize_scale(self.scale);
        self.opacity = self.opacity.clamp(0.05, 1.0);
        self
    }
}

pub fn config_dir() -> PathBuf {
    dirs::config_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("omakeys")
}

pub fn config_path() -> PathBuf {
    config_dir().join("config.toml")
}

pub fn load_config() -> AppConfig {
    let path = config_path();
    match fs::read_to_string(&path) {
        Ok(s) => toml::from_str::<AppConfig>(&s)
            .unwrap_or_default()
            .normalized(),
        Err(_) => AppConfig::default(),
    }
}

pub fn save_config(cfg: &AppConfig) -> Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir).with_context(|| format!("mkdir {}", dir.display()))?;
    let s = toml::to_string_pretty(cfg).context("serialize config")?;
    fs::write(config_path(), s).context("write config.toml")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_round_trip_toml() {
        let cfg = AppConfig::default();
        let s = toml::to_string(&cfg).unwrap();
        let back: AppConfig = toml::from_str(&s).unwrap();
        assert_eq!(back, cfg);
    }

    #[test]
    fn config_json_round_trip() {
        let cfg = AppConfig::default();
        let s = serde_json::to_string(&cfg).unwrap();
        let back: AppConfig = serde_json::from_str(&s).unwrap();
        assert_eq!(back.mode, cfg.mode);
        assert_eq!(back.timeout_ms, cfg.timeout_ms);
        assert_eq!(back.anchor, cfg.anchor);
        assert!((back.opacity - cfg.opacity).abs() < f64::EPSILON);
        assert!((back.scale - cfg.scale).abs() < f64::EPSILON);
        assert_eq!(back.label_style, cfg.label_style);
    }

    #[test]
    fn normalize_scale_clamps_and_steps() {
        assert!((normalize_scale(1.0) - 1.0).abs() < f64::EPSILON);
        assert!((normalize_scale(0.74) - 0.70).abs() < f64::EPSILON);
        assert!((normalize_scale(0.76) - 0.80).abs() < f64::EPSILON);
        assert!((normalize_scale(1.6) - 1.50).abs() < f64::EPSILON);
        assert!((normalize_scale(0.1) - 0.50).abs() < f64::EPSILON);
        assert!((normalize_scale(f64::NAN) - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn normalize_migrates_hidden_mode_to_auto() {
        let cfg = AppConfig {
            mode: VisibilityMode::Hidden,
            ..AppConfig::default()
        }
        .normalized();
        assert_eq!(cfg.mode, VisibilityMode::Auto);
    }
}
