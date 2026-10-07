//! Pure overlay state: visibility, active layer, held keys, paint model.

use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::board::client::Keymap;
use crate::board::labels::legend_for_keycode;
use crate::board::profile::BoardProfile;
use crate::board::protocol::HostEvent;

use super::config::{Anchor, AppConfig, VisibilityMode};
use super::overrides::Overrides;

#[derive(Debug, Clone, PartialEq)]
pub struct KeyView {
    pub row: u8,
    pub col: u8,
    pub x: f64,
    pub y: f64,
    pub w: f64,
    pub h: f64,
    /// Main (tap) legend.
    pub label: String,
    /// Hold legend for mod-tap / layer-tap (corner).
    pub hold_label: Option<String>,
    pub held: bool,
    /// Modifier / layer / tap-hold key — idle fill uses accent so keys that
    /// can change mods or layers stand out. Plain typing keys stay muted.
    pub special: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Frame {
    pub visible: bool,
    pub opacity: f64,
    pub scale: f64,
    pub anchor: Anchor,
    pub layer: u8,
    pub keys: Vec<KeyView>,
}

/// Key held since press: origin layer + keycode frozen at press time.
/// Freezing matters for layer-tap: QMK activates the hold layer on press, so
/// the live map would otherwise show the hold-layer keycap instead of the
/// tap action from the layer the press started on.
#[derive(Debug, Clone, Copy)]
struct HeldKey {
    from_layer: u8,
    keycode: u16,
    /// After release, keep the key lit until this instant so quick taps
    /// (press+release in one poll) still paint at least one frame.
    flash_until: Option<Instant>,
}

/// How long a released key stays highlighted (covers same-batch tap events).
const TAP_FLASH: Duration = Duration::from_millis(90);

pub struct OverlayCore {
    profile: BoardProfile,
    config: AppConfig,
    overrides: Overrides,
    keymap: Keymap,
    /// QMK `default_layer_state` bitmask (usually bit 0 = base).
    default_layer: u32,
    /// QMK `layer_state` overlay bitmask (excludes default layer).
    layer_state: u32,
    held: HashMap<(u8, u8), HeldKey>,
    last_activity: Option<Instant>,
    /// Shared with IPC so `ping` can report Live vs Hidden.
    force_hidden: Arc<AtomicBool>,
}

impl OverlayCore {
    pub fn new(profile: BoardProfile, config: AppConfig, overrides: Overrides) -> Self {
        let layers = (profile.max_layer_inclusive as usize) + 1;
        let keymap = Keymap::empty(layers, profile.rows as usize, profile.cols as usize);
        Self {
            profile,
            config: config.normalized(),
            overrides,
            keymap,
            default_layer: 0b1,
            layer_state: 0,
            held: HashMap::new(),
            last_activity: None,
            force_hidden: Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn force_hidden_handle(&self) -> Arc<AtomicBool> {
        Arc::clone(&self.force_hidden)
    }

    pub fn is_force_hidden(&self) -> bool {
        self.force_hidden.load(Ordering::Relaxed)
    }

    pub fn set_keymap(&mut self, map: Keymap) {
        self.keymap = map;
    }

    pub fn set_config(&mut self, config: AppConfig) {
        self.config = config.normalized();
    }

    pub fn set_overrides(&mut self, o: Overrides) {
        self.overrides = o;
    }

    pub fn toggle_force_hidden(&mut self) {
        let next = !self.is_force_hidden();
        self.force_hidden.store(next, Ordering::Relaxed);
    }

    pub fn handle(&mut self, ev: HostEvent, now: Instant) {
        match ev {
            HostEvent::Layer {
                default_layer,
                layer_state,
            } => {
                // Layer polls (keepalive / LayerLens) must not count as
                // activity — they fire every ~1.5s and would pin Auto mode
                // visible forever when timeout is only a few seconds.
                if default_layer != 0 {
                    self.default_layer = default_layer;
                }
                self.layer_state = layer_state;
            }
            HostEvent::Key { row, col, pressed } => {
                if pressed {
                    // Only real presses (and releases of keys we were tracking)
                    // count as activity. Spurious F releases forged from
                    // LayerLens state=0 keepalive must not reset Auto timeout.
                    self.last_activity = Some(now);
                    let (from_layer, keycode) = resolve_press_keycode(
                        &self.keymap,
                        self.default_layer,
                        self.layer_state,
                        row,
                        col,
                    );
                    self.held.insert(
                        (row, col),
                        HeldKey {
                            from_layer,
                            keycode,
                            flash_until: None,
                        },
                    );
                } else if let Some(h) = self.held.get_mut(&(row, col)) {
                    self.last_activity = Some(now);
                    // Arm flash once. Re-arming on repeat releases (e.g. phantom
                    // LayerLens/F collisions) would keep the key lit forever.
                    if h.flash_until.is_none() {
                        h.flash_until = Some(now + TAP_FLASH);
                    }
                }
            }
        }
    }

    pub fn tick(&mut self, now: Instant) {
        self.held
            .retain(|_, h| h.flash_until.map(|t| now < t).unwrap_or(true));
    }

    pub fn active_layer(&self) -> Option<u8> {
        let layer = highest_layer_bit(self.layer_state);
        if layer > self.profile.max_layer_inclusive {
            return None;
        }
        Some(layer)
    }

    pub fn frame(&self) -> Frame {
        let layer = self.active_layer().unwrap_or(0);
        let skip = self.active_layer().is_none();
        let visible = if self.is_force_hidden() || skip {
            false
        } else {
            match self.config.mode {
                VisibilityMode::Hidden => false,
                VisibilityMode::Always => true,
                VisibilityMode::Auto => match self.last_activity {
                    Some(t) => t.elapsed() < Duration::from_millis(self.config.timeout_ms),
                    None => false,
                },
            }
        };

        let keys = if skip {
            Vec::new()
        } else {
            self.profile
                .keys
                .iter()
                .map(|k| {
                    let held = self.held.get(&(k.row, k.col)).copied();
                    let (label_layer, kc) = match held {
                        Some(h) => (h.from_layer, h.keycode),
                        None => (layer, self.keymap.get(layer, k.row, k.col)),
                    };
                    let (label, hold_label) =
                        if let Some(over) = self.overrides.get(label_layer, k.row, k.col) {
                            (over.to_string(), None)
                        } else {
                            let leg = legend_for_keycode(kc, self.config.label_style);
                            (leg.primary, leg.hold)
                        };
                    let special = is_modifier_or_layer_key(kc);
                    KeyView {
                        row: k.row,
                        col: k.col,
                        x: k.x,
                        y: k.y,
                        w: k.w,
                        h: k.h,
                        label,
                        hold_label,
                        held: held.is_some(),
                        special,
                    }
                })
                .collect()
        };

        Frame {
            visible,
            opacity: self.config.opacity,
            scale: self.config.scale,
            anchor: self.config.anchor,
            layer,
            keys,
        }
    }
}

fn highest_layer_bit(state: u32) -> u8 {
    if state == 0 {
        return 0;
    }
    31 - state.leading_zeros() as u8
}

fn default_layer_index(default_layer: u32) -> u8 {
    if default_layer == 0 {
        0
    } else {
        highest_layer_bit(default_layer)
    }
}

/// Pick the keycode to freeze on press.
///
/// QMK `layer_state` is overlay-only. When Layer arrives before Key (LT/MO
/// already active), the top layer's key at this cell is the *destination*
/// key — not the dual/MO that was pressed. Prefer the default-layer keycode
/// when it is an LT/MO that targets the active overlay layer.
fn resolve_press_keycode(
    map: &Keymap,
    default_layer: u32,
    layer_state: u32,
    row: u8,
    col: u8,
) -> (u8, u16) {
    let def = default_layer_index(default_layer);
    let active = if layer_state == 0 {
        def
    } else {
        highest_layer_bit(layer_state)
    };
    let kc_def = map.get(def, row, col);
    if let Some(target) = layer_switch_target(kc_def) {
        if target == active {
            return (def, kc_def);
        }
    }
    (active, map.get(active, row, col))
}

/// Layer index activated by LT / MO / TG / TO, if this keycode is one of those.
fn layer_switch_target(kc: u16) -> Option<u8> {
    match kc {
        0x4000..=0x4FFF => Some(((kc >> 8) & 0x0F) as u8), // LT
        0x5200..=0x521F => Some((kc - 0x5200) as u8),       // TO
        0x5220..=0x523F => Some((kc - 0x5220) as u8),       // MO
        0x5240..=0x525F => Some((kc - 0x5240) as u8),       // TG
        _ => None,
    }
}

/// Accent idle fill for keys that can change mods or layers: standalone
/// mods, mod-tap, and layer switches (MO / LT / TG / TO).
fn is_modifier_or_layer_key(kc: u16) -> bool {
    match kc {
        0x00E0..=0x00E7 => true, // KC_LCTL … KC_RGUI
        0x2000..=0x3FFF => true, // MT / mod-tap
        0x4000..=0x4FFF => true, // LT
        0x5200..=0x525F => true, // TO / MO / TG
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::board::profile::loremipsum36;

    #[test]
    fn auto_hides_after_timeout() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig::default(),
            Overrides::default(),
        );
        let t0 = Instant::now();
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 0,
                pressed: true,
            },
            t0,
        );
        // Simulate aged activity by rewriting last_activity.
        core.last_activity = Instant::now().checked_sub(Duration::from_millis(2500));
        assert!(!core.frame().visible);
    }

    #[test]
    fn auto_shows_on_key_not_layer_poll() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig::default(),
            Overrides::default(),
        );
        let t0 = Instant::now();
        core.handle(
            HostEvent::Layer {
                default_layer: 0,
                layer_state: 0b100,
            },
            t0,
        );
        assert!(!core.frame().visible, "layer poll alone must not show overlay");
        assert_eq!(core.frame().layer, 2);
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 0,
                pressed: true,
            },
            t0,
        );
        assert!(core.frame().visible);
        // Keepalive-style layer refresh must not extend the idle timer.
        core.last_activity = Instant::now().checked_sub(Duration::from_millis(2500));
        core.handle(
            HostEvent::Layer {
                default_layer: 0,
                layer_state: 0b100,
            },
            Instant::now(),
        );
        assert!(!core.frame().visible, "layer poll must not reset auto timeout");
    }

    #[test]
    fn auto_ignores_spurious_key_release() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig::default(),
            Overrides::default(),
        );
        let t0 = Instant::now();
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 0,
                pressed: true,
            },
            t0,
        );
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 0,
                pressed: false,
            },
            t0,
        );
        core.last_activity = Instant::now().checked_sub(Duration::from_millis(2500));
        // Phantom F release (LayerLens state=0) — not in held map.
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 2,
                pressed: false,
            },
            Instant::now(),
        );
        assert!(
            !core.frame().visible,
            "spurious release must not reset auto timeout"
        );
    }

    #[test]
    fn frame_carries_normalized_scale() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                scale: 0.74,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        assert!((core.frame().scale - 0.70).abs() < f64::EPSILON);
        core.set_config(AppConfig {
            scale: 1.16,
            ..AppConfig::default()
        });
        assert!((core.frame().scale - 1.20).abs() < f64::EPSILON);
    }

    #[test]
    fn highest_layer_wins() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                mode: VisibilityMode::Always,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        core.handle(
            HostEvent::Layer {
                default_layer: 0,
                layer_state: 0b1100,
            },
            Instant::now(),
        );
        assert_eq!(core.frame().layer, 3);
    }

    #[test]
    fn held_key_marks_view() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                mode: VisibilityMode::Always,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 1,
                pressed: true,
            },
            Instant::now(),
        );
        let frame = core.frame();
        let k = frame.keys.iter().find(|k| k.row == 0 && k.col == 1).unwrap();
        assert!(k.held);
    }

    #[test]
    fn quick_tap_flash_survives_same_batch_release() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                mode: VisibilityMode::Always,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        let mut map = Keymap::empty(6, 4, 10);
        map.set(0, 0, 0, 0x2114); // LCTL_T(Q)
        core.set_keymap(map);
        let t0 = Instant::now();
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 0,
                pressed: true,
            },
            t0,
        );
        core.handle(
            HostEvent::Key {
                row: 0,
                col: 0,
                pressed: false,
            },
            t0, // same poll batch as press
        );
        let k = core
            .frame()
            .keys
            .iter()
            .find(|k| k.row == 0 && k.col == 0)
            .unwrap()
            .clone();
        assert!(k.held, "quick tap must stay lit after same-batch release");
        assert_eq!(k.label, "Q");
        assert_eq!(k.hold_label.as_deref(), Some("⌃"));
        core.tick(t0 + Duration::from_millis(50));
        let mid = core.frame();
        assert!(
            mid.keys.iter().any(|k| k.row == 0 && k.col == 0 && k.held),
            "flash should still be lit mid-duration"
        );
        core.tick(t0 + Duration::from_millis(100));
        let frame = core.frame();
        let k = frame
            .keys
            .iter()
            .find(|k| k.row == 0 && k.col == 0)
            .unwrap();
        assert!(!k.held, "flash should end after TAP_FLASH");
    }

    #[test]
    fn layer_tap_hold_keeps_tap_legend() {
        // LT(2, Esc) on base at (3,2); hold layer same pos is KC_LEFT.
        // Real QMK layer_state is only the overlay bits (0b100), not base|overlay.
        // KeyPeek may deliver Layer before Key — held key must still show Esc.
        const LT_ESC_L2: u16 = 0x4200 | 0x29; // layer 2 + KC_ESC
        const KC_LEFT: u16 = 0x0050;
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                mode: VisibilityMode::Always,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        let mut map = Keymap::empty(6, 4, 10);
        map.set(0, 3, 2, LT_ESC_L2);
        map.set(2, 3, 2, KC_LEFT);
        core.set_keymap(map);

        core.handle(
            HostEvent::Layer {
                default_layer: 0b1,
                layer_state: 0b100, // overlay only — matches KeyPeek/QMK
            },
            Instant::now(),
        );
        core.handle(
            HostEvent::Key {
                row: 3,
                col: 2,
                pressed: true,
            },
            Instant::now(),
        );

        let frame = core.frame();
        assert_eq!(frame.layer, 2);
        let k = frame.keys.iter().find(|k| k.row == 3 && k.col == 2).unwrap();
        assert!(k.held);
        assert_eq!(k.label, "⎋");
        assert_eq!(k.hold_label.as_deref(), Some("SYM"));
    }

    #[test]
    fn mo_hold_keeps_layer_legend() {
        // MO(SYM) on base thumb; SYM layer same pos is something else.
        const MO_SYM: u16 = 0x5222;
        const KC_LEFT: u16 = 0x0050;
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                mode: VisibilityMode::Always,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        let mut map = Keymap::empty(6, 4, 10);
        map.set(0, 3, 3, MO_SYM);
        map.set(2, 3, 3, KC_LEFT);
        core.set_keymap(map);

        core.handle(
            HostEvent::Layer {
                default_layer: 0b1,
                layer_state: 0b100,
            },
            Instant::now(),
        );
        core.handle(
            HostEvent::Key {
                row: 3,
                col: 3,
                pressed: true,
            },
            Instant::now(),
        );

        let frame = core.frame();
        let k = frame.keys.iter().find(|k| k.row == 3 && k.col == 3).unwrap();
        assert!(k.held);
        assert_eq!(k.label, "SYM");
    }

    #[test]
    fn special_flags_mods_layers_and_tap_hold() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                mode: VisibilityMode::Always,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        let mut map = Keymap::empty(6, 4, 10);
        map.set(0, 0, 0, 0x0014); // KC_Q
        map.set(0, 0, 1, 0x2114); // LCTL_T(KC_Q) — mod-tap, accented
        map.set(0, 0, 2, 0x00E0); // KC_LCTL
        map.set(0, 0, 3, 0x0036); // KC_COMM
        map.set(0, 0, 4, 0x0028); // KC_ENT
        map.set(0, 0, 5, 0x5222); // MO(SYM)
        map.set(0, 0, 6, 0x4200 | 0x29); // LT(2, Esc)
        core.set_keymap(map);
        let frame = core.frame();
        let find = |c| frame.keys.iter().find(|k| k.row == 0 && k.col == c).unwrap();
        assert!(!find(0).special, "letter muted");
        assert!(find(1).special, "mod-tap accented");
        assert!(find(2).special, "modifier accented");
        assert!(!find(3).special, "comma muted");
        assert!(!find(4).special, "enter muted");
        assert!(find(5).special, "MO accented");
        assert!(find(6).special, "LT accented");
    }

    #[test]
    fn frogv_layer_hides() {
        let mut core = OverlayCore::new(
            loremipsum36(),
            AppConfig {
                mode: VisibilityMode::Always,
                ..AppConfig::default()
            },
            Overrides::default(),
        );
        core.handle(
            HostEvent::Layer {
                default_layer: 0,
                layer_state: 1 << 9,
            },
            Instant::now(),
        );
        assert!(!core.frame().visible);
    }
}
