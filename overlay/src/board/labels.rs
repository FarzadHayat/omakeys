//! QMK keycode → short overlay legend (tap + optional hold).
//!
//! Icon style prefers compact Unicode / Omarchy glyphs. Text style uses short
//! English words. Omarchy Super (`U+E900`) needs the `omarchy` icon font.

use serde::{Deserialize, Serialize};

/// How special keys (mods, Enter, Space, …) are labeled on the map.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LabelStyle {
    /// Unicode / Omarchy glyphs (default).
    #[default]
    Icons,
    /// Short English words (Ctrl, Enter, Super, …).
    Text,
}

/// What to paint on a keycap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KeyLegend {
    /// Main legend (tap action for mod-tap / layer-tap).
    pub primary: String,
    /// Optional hold action (e.g. Ctrl on `LCTL_T(KC_Q)`), drawn smaller.
    pub hold: Option<String>,
}

impl KeyLegend {
    pub fn just(primary: impl Into<String>) -> Self {
        Self {
            primary: primary.into(),
            hold: None,
        }
    }

    pub fn dual(primary: impl Into<String>, hold: impl Into<String>) -> Self {
        Self {
            primary: primary.into(),
            hold: Some(hold.into()),
        }
    }
}

/// Omarchy logo (Super / GUI) — needs `omarchy` font at draw time.
pub const OMARCHY_SUPER: &str = "\u{E900}";

/// Short name for a LoremIpsum36 / rsta layer index (matches firmware enum).
pub fn layer_name(layer: u8) -> &'static str {
    match layer {
        0 => "BASE",
        1 => "QWE",
        2 => "SYM",
        3 => "NAV",
        4 => "FUN",
        5 => "ADJ",
        6 => "GAM",
        7 => "GMX",
        8 => "GMY",
        9 => "FV",
        10 => "FLIP",
        11 => "FVNAV",
        12 => "FVNUM",
        13 => "FVSYM",
        _ => "?",
    }
}

/// Owned label when a static name is enough, else `L{n}` fallback (unused
/// indices stay readable).
pub fn layer_label(layer: u8) -> String {
    let name = layer_name(layer);
    if name == "?" {
        format!("L{layer}")
    } else {
        name.to_string()
    }
}

fn basic_legend(kc: u8, style: LabelStyle) -> KeyLegend {
    match style {
        LabelStyle::Icons => basic_legend_icons(kc),
        LabelStyle::Text => basic_legend_text(kc),
    }
}

fn basic_legend_icons(kc: u8) -> KeyLegend {
    match kc {
        0x00 => KeyLegend::just("·"),
        0x01 => KeyLegend::just(""),
        0x04..=0x1D => KeyLegend::just(basic_letter(kc)),
        0x1E..=0x27 => KeyLegend::just(basic_digit(kc)),
        0x28 => KeyLegend::just("⏎"),
        0x29 => KeyLegend::just("⎋"),
        0x2A => KeyLegend::just("⌫"),
        0x2B => KeyLegend::just("⇥"),
        0x2C => KeyLegend::just("␣"),
        0x2D => KeyLegend::just("-"),
        0x2E => KeyLegend::just("="),
        0x2F => KeyLegend::just("["),
        0x30 => KeyLegend::just("]"),
        0x31 => KeyLegend::just("\\"),
        0x33 => KeyLegend::just(";"),
        0x34 => KeyLegend::just("'"),
        0x35 => KeyLegend::just("`"),
        0x36 => KeyLegend::just(","),
        0x37 => KeyLegend::just("."),
        0x38 => KeyLegend::just("/"),
        0x39 => KeyLegend::just("⇪"),
        0x3A..=0x45 => KeyLegend::just(format!("F{}", kc - 0x39)),
        0x46 => KeyLegend::just("⎙"),
        0x49 => KeyLegend::just("Ins"),
        0x4A => KeyLegend::just("Home"),
        0x4B => KeyLegend::just("⇞"),
        0x4C => KeyLegend::just("⌦"),
        0x4D => KeyLegend::just("End"),
        0x4E => KeyLegend::just("⇟"),
        0x4F => KeyLegend::just("→"),
        0x50 => KeyLegend::just("←"),
        0x51 => KeyLegend::just("↓"),
        0x52 => KeyLegend::just("↑"),
        0xA8 => KeyLegend::just("🔇"),
        0xA9 => KeyLegend::just("🔊"),
        0xAA => KeyLegend::just("🔉"),
        0xAB => KeyLegend::just("⏭"),
        0xAC => KeyLegend::just("⏮"),
        0xAE => KeyLegend::just("⏯"),
        0xBD => KeyLegend::just("☀"),
        0xBE => KeyLegend::just("☼"),
        0xCD => KeyLegend::just("M↑"),
        0xCE => KeyLegend::just("M↓"),
        0xCF => KeyLegend::just("M←"),
        0xD0 => KeyLegend::just("M→"),
        0xD1 => KeyLegend::just("LMB"),
        0xD2 => KeyLegend::just("RMB"),
        0xD3 => KeyLegend::just("MMB"),
        0xD9 => KeyLegend::just("⇈"),
        0xDA => KeyLegend::just("⇊"),
        0xE0 | 0xE4 => KeyLegend::just("⌃"),
        0xE1 | 0xE5 => KeyLegend::just("⇧"),
        0xE2 | 0xE6 => KeyLegend::just("⌥"),
        0xE3 | 0xE7 => KeyLegend::just(OMARCHY_SUPER),
        _ => KeyLegend::just(format!("0x{kc:02X}")),
    }
}

fn basic_legend_text(kc: u8) -> KeyLegend {
    match kc {
        0x00 => KeyLegend::just("·"),
        0x01 => KeyLegend::just(""),
        0x04..=0x1D => KeyLegend::just(basic_letter(kc)),
        0x1E..=0x27 => KeyLegend::just(basic_digit(kc)),
        0x28 => KeyLegend::just("Enter"),
        0x29 => KeyLegend::just("Esc"),
        0x2A => KeyLegend::just("Bksp"),
        0x2B => KeyLegend::just("Tab"),
        0x2C => KeyLegend::just("Spc"),
        0x2D => KeyLegend::just("-"),
        0x2E => KeyLegend::just("="),
        0x2F => KeyLegend::just("["),
        0x30 => KeyLegend::just("]"),
        0x31 => KeyLegend::just("\\"),
        0x33 => KeyLegend::just(";"),
        0x34 => KeyLegend::just("'"),
        0x35 => KeyLegend::just("`"),
        0x36 => KeyLegend::just(","),
        0x37 => KeyLegend::just("."),
        0x38 => KeyLegend::just("/"),
        0x39 => KeyLegend::just("Caps"),
        0x3A..=0x45 => KeyLegend::just(format!("F{}", kc - 0x39)),
        0x46 => KeyLegend::just("Prt"),
        0x49 => KeyLegend::just("Ins"),
        0x4A => KeyLegend::just("Home"),
        0x4B => KeyLegend::just("PgUp"),
        0x4C => KeyLegend::just("Del"),
        0x4D => KeyLegend::just("End"),
        0x4E => KeyLegend::just("PgDn"),
        0x4F => KeyLegend::just("Right"),
        0x50 => KeyLegend::just("Left"),
        0x51 => KeyLegend::just("Down"),
        0x52 => KeyLegend::just("Up"),
        0xA8 => KeyLegend::just("Mute"),
        0xA9 => KeyLegend::just("Vol+"),
        0xAA => KeyLegend::just("Vol-"),
        0xAB => KeyLegend::just("Next"),
        0xAC => KeyLegend::just("Prev"),
        0xAE => KeyLegend::just("Play"),
        0xBD => KeyLegend::just("Brt+"),
        0xBE => KeyLegend::just("Brt-"),
        0xCD => KeyLegend::just("MUp"),
        0xCE => KeyLegend::just("MDn"),
        0xCF => KeyLegend::just("MLt"),
        0xD0 => KeyLegend::just("MRt"),
        0xD1 => KeyLegend::just("LMB"),
        0xD2 => KeyLegend::just("RMB"),
        0xD3 => KeyLegend::just("MMB"),
        0xD9 => KeyLegend::just("WhUp"),
        0xDA => KeyLegend::just("WhDn"),
        0xE0 | 0xE4 => KeyLegend::just("Ctrl"),
        0xE1 | 0xE5 => KeyLegend::just("Shift"),
        0xE2 | 0xE6 => KeyLegend::just("Alt"),
        0xE3 | 0xE7 => KeyLegend::just("Super"),
        _ => KeyLegend::just(format!("0x{kc:02X}")),
    }
}

fn basic_letter(kc: u8) -> String {
    char::from(b'A' + (kc - 0x04)).to_string()
}

fn basic_digit(kc: u8) -> String {
    match kc {
        0x1E => "1".into(),
        0x1F => "2".into(),
        0x20 => "3".into(),
        0x21 => "4".into(),
        0x22 => "5".into(),
        0x23 => "6".into(),
        0x24 => "7".into(),
        0x25 => "8".into(),
        0x26 => "9".into(),
        0x27 => "0".into(),
        _ => "?".into(),
    }
}

fn shifted_symbol(basic: u8) -> Option<&'static str> {
    Some(match basic {
        0x1E => "!",
        0x1F => "@",
        0x20 => "#",
        0x21 => "$",
        0x22 => "%",
        0x23 => "^",
        0x24 => "&",
        0x25 => "*",
        0x26 => "(",
        0x27 => ")",
        0x2D => "_",
        0x2E => "+",
        0x2F => "{",
        0x30 => "}",
        0x31 => "|",
        0x33 => ":",
        0x34 => "\"",
        0x35 => "~",
        0x36 => "<",
        0x37 => ">",
        0x38 => "?",
        _ => return None,
    })
}

fn hold_mod(mods: u8, style: LabelStyle) -> String {
    match style {
        LabelStyle::Icons => hold_mod_icons(mods),
        LabelStyle::Text => hold_mod_text(mods),
    }
}

fn hold_mod_icons(mods: u8) -> String {
    match mods & 0x0F {
        0x01 => "⌃".into(),
        0x02 => "⇧".into(),
        0x04 => "⌥".into(),
        0x08 => OMARCHY_SUPER.into(),
        m => {
            let mut s = String::new();
            if m & 0x01 != 0 {
                s.push('⌃');
            }
            if m & 0x02 != 0 {
                s.push('⇧');
            }
            if m & 0x04 != 0 {
                s.push('⌥');
            }
            if m & 0x08 != 0 {
                s.push_str(OMARCHY_SUPER);
            }
            s
        }
    }
}

fn hold_mod_text(mods: u8) -> String {
    match mods & 0x0F {
        0x01 => "Ctrl".into(),
        0x02 => "Sft".into(),
        0x04 => "Alt".into(),
        0x08 => "Sup".into(),
        m => {
            let mut parts = Vec::new();
            if m & 0x01 != 0 {
                parts.push("Ctrl");
            }
            if m & 0x02 != 0 {
                parts.push("Sft");
            }
            if m & 0x04 != 0 {
                parts.push("Alt");
            }
            if m & 0x08 != 0 {
                parts.push("Sup");
            }
            parts.join("+")
        }
    }
}

/// Map a VIA/QMK keycode to a legend with sensible built-in defaults.
pub fn legend_for_keycode(kc: u16, style: LabelStyle) -> KeyLegend {
    match kc {
        0x0000 => KeyLegend::just("·"),
        0x0001 => KeyLegend::just(""),
        0x0002..=0x00FF => basic_legend(kc as u8, style),

        0x0100..=0x1FFF => {
            let mods = ((kc >> 8) & 0xFF) as u8;
            let basic = (kc & 0xFF) as u8;
            if mods == 0x02 {
                if let Some(sym) = shifted_symbol(basic) {
                    return KeyLegend::just(sym);
                }
            }
            let tap = basic_legend(basic, style).primary;
            let hold = hold_mod(mods, style);
            if hold.is_empty() {
                KeyLegend::just(tap)
            } else if tap.is_empty() {
                KeyLegend::just(hold)
            } else {
                match style {
                    LabelStyle::Icons => KeyLegend::just(format!("{hold}{tap}")),
                    LabelStyle::Text => KeyLegend::just(format!("{hold}+{tap}")),
                }
            }
        }

        0x2000..=0x3FFF => {
            let mods = ((kc >> 8) & 0x1F) as u8;
            let tap = (kc & 0xFF) as u8;
            KeyLegend::dual(basic_legend(tap, style).primary, hold_mod(mods, style))
        }

        0x4000..=0x4FFF => {
            let tap = (kc & 0xFF) as u8;
            let layer = ((kc >> 8) & 0x0F) as u8;
            KeyLegend::dual(basic_legend(tap, style).primary, layer_label(layer))
        }

        0x5200..=0x521F => KeyLegend::just(format!("TO({})", layer_name((kc - 0x5200) as u8))),
        0x5220..=0x523F => KeyLegend::just(layer_label((kc - 0x5220) as u8)),
        0x5240..=0x525F => KeyLegend::just(format!("TG({})", layer_name((kc - 0x5240) as u8))),

        0x7C00 => KeyLegend::just("BOOT"),
        0x7C03 => KeyLegend::just("CLR"),
        0x7842 => KeyLegend::just("RGB"),
        0x7843 => KeyLegend::just("RGB+"),
        0x7844 => KeyLegend::just("RGB-"),
        0x7E01 => KeyLegend::just(match style {
            LabelStyle::Icons => "🔇",
            LabelStyle::Text => "Mute",
        }),
        0x7E02 => KeyLegend::just(match style {
            LabelStyle::Icons => "📷",
            LabelStyle::Text => "Shot",
        }),

        _ => KeyLegend::just(format!("0x{kc:04X}")),
    }
}

/// Back-compat helper for tests / simple call sites (icon style).
pub fn label_for_keycode(kc: u16) -> String {
    let leg = legend_for_keycode(kc, LabelStyle::Icons);
    match leg.hold {
        Some(h) => format!("{}/{}", leg.primary, h),
        None => leg.primary,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_names_match_firmware() {
        assert_eq!(layer_name(0), "BASE");
        assert_eq!(layer_name(2), "SYM");
        assert_eq!(layer_name(3), "NAV");
        assert_eq!(layer_label(6), "GAM");
        assert_eq!(legend_for_keycode(0x5222, LabelStyle::Icons).primary, "SYM");
        assert_eq!(legend_for_keycode(0x5201, LabelStyle::Icons).primary, "TO(QWE)");
        assert_eq!(legend_for_keycode(0x5241, LabelStyle::Icons).primary, "TG(QWE)");
    }

    #[test]
    fn mod_tap_dual_icons() {
        let leg = legend_for_keycode(0x2114, LabelStyle::Icons);
        assert_eq!(leg.primary, "Q");
        assert_eq!(leg.hold.as_deref(), Some("⌃"));
        let ent = legend_for_keycode(0x2128, LabelStyle::Icons);
        assert_eq!(ent.primary, "⏎");
        assert_eq!(ent.hold.as_deref(), Some("⌃"));
    }

    #[test]
    fn mod_tap_dual_text() {
        let leg = legend_for_keycode(0x2114, LabelStyle::Text);
        assert_eq!(leg.primary, "Q");
        assert_eq!(leg.hold.as_deref(), Some("Ctrl"));
        let ent = legend_for_keycode(0x2128, LabelStyle::Text);
        assert_eq!(ent.primary, "Enter");
        assert_eq!(ent.hold.as_deref(), Some("Ctrl"));
    }

    #[test]
    fn gui_is_omarchy_super_icons() {
        let leg = legend_for_keycode(0x00E3, LabelStyle::Icons);
        assert_eq!(leg.primary, OMARCHY_SUPER);
    }

    #[test]
    fn gui_is_super_text() {
        let leg = legend_for_keycode(0x00E3, LabelStyle::Text);
        assert_eq!(leg.primary, "Super");
    }

    #[test]
    fn specials_icons() {
        assert_eq!(legend_for_keycode(0x002C, LabelStyle::Icons).primary, "␣");
        assert_eq!(legend_for_keycode(0x00E1, LabelStyle::Icons).primary, "⇧");
        assert_eq!(legend_for_keycode(0x002A, LabelStyle::Icons).primary, "⌫");
    }

    #[test]
    fn specials_text() {
        assert_eq!(legend_for_keycode(0x002C, LabelStyle::Text).primary, "Spc");
        assert_eq!(legend_for_keycode(0x00E1, LabelStyle::Text).primary, "Shift");
        assert_eq!(legend_for_keycode(0x002A, LabelStyle::Text).primary, "Bksp");
    }

    #[test]
    fn shifted_symbols() {
        assert_eq!(legend_for_keycode(0x021E, LabelStyle::Icons).primary, "!");
        assert_eq!(legend_for_keycode(0x021E, LabelStyle::Text).primary, "!");
    }
}
