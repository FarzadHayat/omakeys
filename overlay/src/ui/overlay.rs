//! GTK4 layer-shell overlay window.

use std::cell::RefCell;
use std::rc::Rc;

use cairo::Region;
use gtk::gdk::prelude::SurfaceExt;
use gtk::gdk::RGBA;
use gtk::prelude::*;
use gtk::{Application, ApplicationWindow, DrawingArea};
use gtk4_layer_shell::{Edge, KeyboardMode, Layer, LayerShell};
use pango::{Alignment, FontDescription, WrapMode};
use pangocairo::functions::{create_layout, show_layout};

use crate::board::{layer_label, OMARCHY_SUPER};
use crate::core::config::Anchor;
use crate::core::state::{Frame, KeyView};
use crate::ui::theme::Theme;

const UNIT_PX: f64 = 44.0;
const PAD: f64 = 12.0;
/// One family for letters + symbols so Pango does not mix Liberation / Nerd / Adwaita.
/// Super still switches to `omarchy` via markup. Matches Omarchy UI monospace.
const LEGEND_FONT: &str = "JetBrainsMono Nerd Font";

pub struct OverlayWindow {
    window: ApplicationWindow,
    area: DrawingArea,
    frame: Rc<RefCell<Frame>>,
    theme: Rc<RefCell<Theme>>,
}

impl OverlayWindow {
    pub fn new(app: &Application) -> Self {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Omakeys")
            .decorated(false)
            .resizable(false)
            .build();

        window.init_layer_shell();
        window.set_namespace("omakeys");
        window.set_layer(Layer::Overlay);
        window.set_keyboard_mode(KeyboardMode::None);
        for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
            window.set_margin(edge, 12);
        }

        let area = DrawingArea::new();
        let frame = Rc::new(RefCell::new(Frame {
            visible: false,
            opacity: 0.92,
            scale: 1.0,
            anchor: Anchor::BottomRight,
            layer: 0,
            keys: Vec::new(),
        }));
        let theme = Rc::new(RefCell::new(Theme::default()));

        {
            let frame_c = Rc::clone(&frame);
            let theme_c = Rc::clone(&theme);
            area.set_draw_func(move |_area, cr, _w, _h| {
                let frame = frame_c.borrow();
                let theme = theme_c.borrow();
                draw_keyboard(cr, &frame, &theme);
            });
        }

        window.set_child(Some(&area));

        // Overlay is visual-only: empty input region so pointer clicks pass through
        // to windows below. Re-apply on realize (surface exists) and after show/resize.
        window.connect_realize(|w| apply_click_through(w));

        Self {
            window,
            area,
            frame,
            theme,
        }
    }

    pub fn set_theme(&self, theme: Theme) {
        *self.theme.borrow_mut() = theme;
        self.area.queue_draw();
    }

    pub fn apply_frame(&self, frame: &Frame) {
        *self.frame.borrow_mut() = frame.clone();
        self.apply_anchor(frame.anchor);
        self.window.set_opacity(frame.opacity.clamp(0.05, 1.0));

        if frame.visible && !frame.keys.is_empty() {
            let (w, h) = content_size(&frame.keys, frame.scale);
            self.area.set_content_width(w as i32);
            self.area.set_content_height(h as i32);
            self.window.set_default_size(w as i32, h as i32);
            self.window.set_visible(true);
            apply_click_through(&self.window);
            self.area.queue_draw();
        } else {
            self.window.set_visible(false);
        }
    }

    fn apply_anchor(&self, anchor: Anchor) {
        let w = &self.window;
        for edge in [Edge::Left, Edge::Right, Edge::Top, Edge::Bottom] {
            w.set_anchor(edge, false);
        }
        match anchor {
            Anchor::TopLeft => {
                w.set_anchor(Edge::Top, true);
                w.set_anchor(Edge::Left, true);
            }
            Anchor::Top => {
                // Single edge only: opposite anchors stretch the surface.
                w.set_anchor(Edge::Top, true);
            }
            Anchor::TopRight => {
                w.set_anchor(Edge::Top, true);
                w.set_anchor(Edge::Right, true);
            }
            Anchor::Left => {
                w.set_anchor(Edge::Left, true);
            }
            Anchor::Right => {
                w.set_anchor(Edge::Right, true);
            }
            Anchor::BottomLeft => {
                w.set_anchor(Edge::Bottom, true);
                w.set_anchor(Edge::Left, true);
            }
            Anchor::Bottom => {
                w.set_anchor(Edge::Bottom, true);
            }
            Anchor::BottomRight => {
                w.set_anchor(Edge::Bottom, true);
                w.set_anchor(Edge::Right, true);
            }
        }
    }
}

/// Empty GDK input region → compositor ignores pointer on this surface.
fn apply_click_through(window: &ApplicationWindow) {
    let Some(surface) = window.surface() else {
        return;
    };
    surface.set_input_region(&Region::create());
}

fn content_size(keys: &[KeyView], scale: f64) -> (f64, f64) {
    let unit = UNIT_PX * scale;
    let pad = PAD * scale;
    let mut max_x: f64 = 0.0;
    let mut max_y: f64 = 0.0;
    for k in keys {
        max_x = max_x.max((k.x + k.w) * unit);
        max_y = max_y.max((k.y + k.h) * unit);
    }
    (max_x + pad * 2.0, max_y + pad * 2.0 + 18.0 * scale)
}

fn parse_rgba(hex: &str) -> RGBA {
    let h = hex.trim().trim_start_matches('#');
    if h.len() >= 6 {
        let r = u8::from_str_radix(&h[0..2], 16).unwrap_or(0) as f32 / 255.0;
        let g = u8::from_str_radix(&h[2..4], 16).unwrap_or(0) as f32 / 255.0;
        let b = u8::from_str_radix(&h[4..6], 16).unwrap_or(0) as f32 / 255.0;
        RGBA::new(r, g, b, 1.0)
    } else {
        RGBA::new(0.1, 0.1, 0.15, 1.0)
    }
}

fn escape_markup(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            _ => out.push(ch),
        }
    }
    out
}

/// Wrap Omarchy Super PUA glyph in the `omarchy` icon font; leave other text as Sans.
fn legend_markup(text: &str) -> String {
    let mut out = String::new();
    for ch in text.chars() {
        if ch == OMARCHY_SUPER.chars().next().unwrap() {
            out.push_str("<span font_family=\"omarchy\">");
            out.push(ch);
            out.push_str("</span>");
        } else {
            out.push_str(&escape_markup(&ch.to_string()));
        }
    }
    out
}

fn draw_legend(
    cr: &gtk::cairo::Context,
    text: &str,
    x: f64,
    y: f64,
    max_w: f64,
    font_pt: i32,
    align: Alignment,
) -> (i32, i32) {
    let layout = create_layout(cr);
    layout.set_font_description(Some(&FontDescription::from_string(&format!(
        "{LEGEND_FONT} {font_pt}"
    ))));
    layout.set_width((max_w * f64::from(pango::SCALE)) as i32);
    layout.set_alignment(align);
    layout.set_markup(&legend_markup(text));
    let (tw, th) = layout.pixel_size();
    cr.move_to(x, y);
    show_layout(cr, &layout);
    (tw, th)
}

/// Fixed-size tap legend: wrap inside the key; only shrink if height still overflows.
fn tap_legend_layout(
    cr: &gtk::cairo::Context,
    text: &str,
    max_w: f64,
    max_h: f64,
    scale: f64,
) -> (pango::Layout, i32) {
    const BASE_PT: f64 = 13.0;
    const MIN_PT: f64 = 8.0;
    let markup = legend_markup(text);
    let width = (max_w * f64::from(pango::SCALE)) as i32;
    let mut pt = (BASE_PT * scale).round().max(MIN_PT) as i32;
    let min_pt = MIN_PT.round().max(6.0) as i32;

    loop {
        let layout = create_layout(cr);
        layout.set_font_description(Some(&FontDescription::from_string(&format!(
            "{LEGEND_FONT} {pt}"
        ))));
        layout.set_width(width);
        layout.set_wrap(WrapMode::WordChar);
        layout.set_alignment(Alignment::Center);
        layout.set_markup(&markup);
        let (_tw, th) = layout.pixel_size();
        if f64::from(th) <= max_h || pt <= min_pt {
            return (layout, th);
        }
        pt -= 1;
    }
}

fn draw_keyboard(cr: &gtk::cairo::Context, frame: &Frame, theme: &Theme) {
    let bg = parse_rgba(&theme.background);
    let fg = parse_rgba(&theme.foreground);
    let accent = parse_rgba(&theme.accent);
    let muted = parse_rgba(&theme.muted);
    let scale = frame.scale.clamp(0.50, 1.50);
    let unit = UNIT_PX * scale;
    let pad = PAD * scale;
    let gap = 4.0 * scale;
    let header_h = 16.0 * scale;
    let radius = 6.0 * scale;

    let (w, h) = content_size(&frame.keys, scale);
    cr.set_source_rgba(
        f64::from(bg.red()),
        f64::from(bg.green()),
        f64::from(bg.blue()),
        0.92,
    );
    cr.rectangle(0.0, 0.0, w, h);
    let _ = cr.fill();

    cr.set_source_rgba(
        f64::from(fg.red()),
        f64::from(fg.green()),
        f64::from(fg.blue()),
        0.85,
    );
    let _ = draw_legend(
        cr,
        &layer_label(frame.layer),
        pad,
        pad - 2.0 * scale,
        80.0 * scale,
        (10.0 * scale).round().max(8.0) as i32,
        Alignment::Left,
    );

    for k in &frame.keys {
        let x = pad + k.x * unit;
        let y = pad + header_h + k.y * unit;
        let kw = k.w * unit - gap;
        let kh = k.h * unit - gap;

        if k.held {
            cr.set_source_rgba(
                f64::from(accent.red()),
                f64::from(accent.green()),
                f64::from(accent.blue()),
                0.95,
            );
        } else if k.special {
            // Accent-tinted idle fill so mods / layer keys stand out.
            cr.set_source_rgba(
                f64::from(accent.red()),
                f64::from(accent.green()),
                f64::from(accent.blue()),
                0.38,
            );
        } else {
            cr.set_source_rgba(
                f64::from(muted.red()),
                f64::from(muted.green()),
                f64::from(muted.blue()),
                0.55,
            );
        }
        rounded_rect(cr, x, y, kw, kh, radius);
        let _ = cr.fill();

        // Held = accent fill. Use bright fg for legends (dark-on-accent was
        // still hard to read). Idle uses normal fg.
        let (tr, tg, tb) = (
            f64::from(fg.red()),
            f64::from(fg.green()),
            f64::from(fg.blue()),
        );
        let has_hold = k
            .hold_label
            .as_deref()
            .is_some_and(|h| !h.is_empty());

        // Tap first (center). One base size for all keys; long labels wrap
        // instead of shrinking. Shrink only if still taller than the keycap.
        if !k.label.is_empty() {
            cr.set_source_rgba(tr, tg, tb, 1.0);
            let max_h = if has_hold {
                (kh - 6.0 * scale).max(8.0 * scale)
            } else {
                (kh - 2.0 * scale).max(8.0 * scale)
            };
            let (layout, th) = tap_legend_layout(cr, &k.label, kw, max_h, scale);
            let y_off = if has_hold {
                (kh - f64::from(th)) / 2.0 + 3.0 * scale
            } else {
                (kh - f64::from(th)) / 2.0
            };
            cr.move_to(x, y + y_off.max(0.0));
            show_layout(cr, &layout);
        }

        if let Some(hold) = k.hold_label.as_deref() {
            if !hold.is_empty() {
                cr.set_source_rgba(tr, tg, tb, if k.held { 1.0 } else { 0.75 });
                let hold_w = (kw * 0.45).max(14.0 * scale);
                let _ = draw_legend(
                    cr,
                    hold,
                    x + kw - hold_w - 2.0 * scale,
                    y + 1.0 * scale,
                    hold_w,
                    (10.0 * scale).round().max(8.0) as i32,
                    Alignment::Right,
                );
            }
        }
    }
}

fn rounded_rect(cr: &gtk::cairo::Context, x: f64, y: f64, w: f64, h: f64, r: f64) {
    let r = r.min(w / 2.0).min(h / 2.0);
    cr.new_sub_path();
    cr.arc(x + w - r, y + r, r, -std::f64::consts::FRAC_PI_2, 0.0);
    cr.arc(x + w - r, y + h - r, r, 0.0, std::f64::consts::FRAC_PI_2);
    cr.arc(
        x + r,
        y + h - r,
        r,
        std::f64::consts::FRAC_PI_2,
        std::f64::consts::PI,
    );
    cr.arc(
        x + r,
        y + r,
        r,
        std::f64::consts::PI,
        3.0 * std::f64::consts::FRAC_PI_2,
    );
    cr.close_path();
}
