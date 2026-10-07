//! `~/.config/omakeys/overrides.toml` — text/emoji label overrides.

use std::collections::HashMap;
use std::fs;

use serde::Deserialize;

use super::config::config_dir;

#[derive(Debug, Clone, Deserialize)]
pub struct OverrideEntry {
    pub layer: u8,
    pub row: u8,
    pub col: u8,
    pub label: String,
}

#[derive(Debug, Clone, Deserialize, Default)]
struct OverridesFile {
    #[serde(default, rename = "override")]
    entries: Vec<OverrideEntry>,
}

#[derive(Debug, Clone, Default)]
pub struct Overrides {
    pub map: HashMap<(u8, u8, u8), String>,
}

impl Overrides {
    pub fn get(&self, layer: u8, row: u8, col: u8) -> Option<&str> {
        self.map.get(&(layer, row, col)).map(|s| s.as_str())
    }

    pub fn from_entries(entries: impl IntoIterator<Item = OverrideEntry>) -> Self {
        let mut map = HashMap::new();
        for e in entries {
            if e.label.is_empty() {
                continue;
            }
            map.insert((e.layer, e.row, e.col), e.label);
        }
        Self { map }
    }
}

pub fn overrides_path() -> std::path::PathBuf {
    config_dir().join("overrides.toml")
}

pub fn load_overrides() -> Overrides {
    let path = overrides_path();
    match fs::read_to_string(&path) {
        Ok(s) => match toml::from_str::<OverridesFile>(&s) {
            Ok(file) => Overrides::from_entries(file.entries),
            Err(_) => Overrides::default(),
        },
        Err(_) => Overrides::default(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn override_wins() {
        let mut o = Overrides::default();
        o.map.insert((2, 0, 3), "‽".into());
        assert_eq!(o.get(2, 0, 3), Some("‽"));
        assert_eq!(o.get(2, 0, 4), None);
    }

    #[test]
    fn parse_toml_entries() {
        let s = r#"
[[override]]
layer = 2
row = 0
col = 3
label = "‽"
"#;
        let file: OverridesFile = toml::from_str(s).unwrap();
        let o = Overrides::from_entries(file.entries);
        assert_eq!(o.get(2, 0, 3), Some("‽"));
    }
}
