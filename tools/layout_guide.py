#!/usr/bin/env python3
"""Generate the Omakeys starter keymap layout guide (SVG + PNG).

The key data mirrors keyboard/penk/loremipsum36/keymaps/starter/keymap.c.
Geometry mirrors the flat 3x5+3 body in keyboard/penk/loremipsum36/keyboard.json
and the VIA layout in via/omakeys.v3.json (1.5u split gap, all 1u caps).

Cap kinds used in the drawing:
  base    light keycap, dark centred label (default)
  macro   keycap with a monospace label (Shot, rset, boot, RGB*)
  dual    keycap with a corner dot: tap-hold key (q, ent)
  accent  solid highlight: layer keys and firmware actions
  muted   solid shade, no shadow: plain held modifiers
  dead    empty cap with a dash: KC_NO
  ghost   faded cap: transparent on this layer, falls through to base

Usage:
    python3 tools/layout_guide.py                # write SVG + PNG for both variants
    python3 tools/layout_guide.py --svg-only
    python3 tools/layout_guide.py --variant dark
    python3 tools/layout_guide.py --check        # structure, palette, contrast, determinism
"""

from __future__ import annotations

import argparse
import math
import random
import shutil
import subprocess
import sys
from collections import namedtuple
from pathlib import Path
from xml.etree import ElementTree

ROOT = Path(__file__).resolve().parents[1]
DOCS = ROOT / "docs"

W, H = 3200, 1730
MARGIN = 72.0
GAP_U = 1.5
CONTENT_U = 11.5
ROWS_U = 4.0

U_S, PAD_S = 56.0, 32.0
U_H, PAD_H = 0.0, 48.0
PANEL_COL_X = (1672.0, 2420.0)
PANEL_ROW_Y = (370.0, 930.0)
HERO_X = MARGIN
HERO_W = PANEL_COL_X[0] - 56.0 - HERO_X
U_H = (HERO_W - 2 * PAD_H) / CONTENT_U

CAP_INSET = 0.05
CAP_SIZE = 0.9
CAP_EDGE = 0.45
CAP_R = 0.14

THUMB_U = (2.0, 3.0, 4.0, 6.5, 7.5, 8.5)

FONT_UI = "Helvetica Neue, Helvetica, Arial, sans-serif"
FONT_MONO = "Menlo, Monaco, monospace"

SURF_W = 2 * PAD_S + CONTENT_U * U_S
SURF_H = 2 * PAD_S + ROWS_U * U_S
BAND0 = PANEL_ROW_Y[0] + SURF_H + 28.0
BAND1 = PANEL_ROW_Y[1] + SURF_H + 28.0
HERO_BAND = 1000.0

CALLOUT_SIZE = 24.5
CALLOUT_TITLE_DY = 34.0
CALLOUT_LINE_DY = 70.0
CALLOUT_LINE_STEP = 30.0

Panel = namedtuple("Panel", "key x y u pad")
Cap = namedtuple("Cap", "label kind mono")
Layer = namedtuple("Layer", "name rows thumbs")
Leader = namedtuple("Leader", "anchor side via end")
Callout = namedtuple("Callout", "panel tx ty w title lines leaders")


def C(label="", kind="base", mono=False):
    return Cap(label, kind, mono)


PANELS = {
    "hero": Panel("hero", HERO_X, PANEL_ROW_Y[0], U_H, PAD_H),
    "sym": Panel("sym", PANEL_COL_X[0], PANEL_ROW_Y[0], U_S, PAD_S),
    "nav": Panel("nav", PANEL_COL_X[1], PANEL_ROW_Y[0], U_S, PAD_S),
    "fun": Panel("fun", PANEL_COL_X[0], PANEL_ROW_Y[1], U_S, PAD_S),
    "adj": Panel("adj", PANEL_COL_X[1], PANEL_ROW_Y[1], U_S, PAD_S),
}
P_HERO, P_SYM, P_NAV, P_FUN, P_ADJ = (PANELS[k] for k in ("hero", "sym", "nav", "fun", "adj"))

PANEL_TITLES = {
    "hero": "QWERTY · base layer",
    "sym": "SYMBOLS · hold left middle thumb",
    "nav": "NAVIGATION & NUMBERS · hold right middle thumb",
    "fun": "FUN · hold SYM + right middle thumb",
    "adj": "ADJUST · hold SYM + left inner thumb",
}

D = C("", "dead")
G = C("", "ghost")

BASE = Layer(
    "QWERTY",
    [
        [C("q", "dual"), C("w"), C("e"), C("r"), C("t"), C("y"), C("u"), C("i"), C("o"), C("p")],
        [C("a"), C("s"), C("d"), C("f"), C("g"), C("h"), C("j"), C("k"), C("l"), C("bksp")],
        [C("z"), C("x"), C("c"), C("v"), C("b"), C("n"), C("m"), C(","), C("."), C("ent", "dual")],
    ],
    [
        C("cmd", "muted"), C("SYM", "accent"), C("sft", "muted"),
        C("spc"), C("NAV", "accent"), C("opt", "muted"),
    ],
)

SYM = Layer(
    "Symbols",
    [
        [C("'"), C('"'), C("^"), C("?"), C("`"), C("["), C("<"), C("="), C(">"), C("]")],
        [C("!"), C("@"), C("#"), C("$"), C("%"), C("{"), C("("), C(":"), C(")"), C("}")],
        [C("\\"), C("~"), C("|"), C(";"), C("&"), C("/"), C("*"), C("-"), C("+"), C("_")],
    ],
    [
        C("cmd", "ghost"), C("SYM", "ghost"), C("ADJ", "accent"),
        C("spc", "ghost"), C("FUN", "accent"), C("opt", "ghost"),
    ],
)

NAV = Layer(
    "Navigate",
    [
        [C("Ctrl", "muted"), C("Cmd", "muted"), C("⇧Tab"), C("Tab"), C("Alt", "muted"),
         C(","), C("Home"), C("↑"), C("End"), C("Del")],
        [C("1"), C("2"), C("3"), C("4"), C("5"),
         C("."), C("←"), C("↓"), C("→"), C("Ent")],
        [C("6"), C("7"), C("8"), C("9"), C("0"),
         C("Esc"), C("PgUp"), C("PgDn"), C("Esc"), C("Ctrl", "muted")],
    ],
    [
        C("cmd", "ghost"), C("SYM", "ghost"), C("sft", "ghost"),
        C("spc", "ghost"), C("NAV", "ghost"), C("opt", "ghost"),
    ],
)

FUN = Layer(
    "Function",
    [
        [C("F11"), C("F12"), C("Shot", "macro"), C("Play"), C("Next"),
         C("Wh↑"), C("LMB"), C("M↑"), C("RMB"), C("Br+")],
        [C("F1"), C("F2"), C("F3"), C("F4"), C("F5"),
         C("Wh↓"), C("M←"), C("M↓"), C("M→"), C("Br-")],
        [C("F6"), C("F7"), C("F8"), C("F9"), C("F10"),
         D, C("Vol-"), C("Vol+"), D, C("spc", "ghost")],
    ],
    [
        C("cmd", "ghost"), C("SYM", "ghost"), C("sft", "ghost"),
        C("spc", "ghost"), D, D,
    ],
)

ADJ = Layer(
    "Adjust",
    [
        [D, D, D, D, D, C("Caps"), D, D, D, C("rset", "accent", mono=True)],
        [D, D, D, D, D, D, D, D, D, D],
        [D, D, D, D, D, C("RGB", "macro"), C("RGB+", "macro"), C("RGB-", "macro"),
         D, C("boot", "accent", mono=True)],
    ],
    [
        C("cmd", "ghost"), C("SYM", "ghost"), D,
        C("spc", "ghost"), C("NAV", "ghost"), C("opt", "ghost"),
    ],
)

LAYERS = {"hero": BASE, "sym": SYM, "nav": NAV, "fun": FUN, "adj": ADJ}


def cell_x(c):
    return c + GAP_U if c >= 5 else float(c)


def key_center(panel, r, c):
    return (
        panel.x + panel.pad + cell_x(c) * panel.u + 0.5 * panel.u,
        panel.y + panel.pad + r * panel.u + 0.5 * panel.u,
    )


def thumb_center(panel, c):
    return (
        panel.x + panel.pad + THUMB_U[c - 2] * panel.u + 0.5 * panel.u,
        panel.y + panel.pad + 3 * panel.u + 0.5 * panel.u,
    )


def kx(panel, r, c):
    return key_center(panel, r, c)[0]


def tx_of(panel, c):
    return thumb_center(panel, c)[0]


def gap_center(panel):
    return panel.x + panel.pad + 5 * panel.u + GAP_U * panel.u / 2


def anchor_point(panel, anchor, side):
    if anchor[0] == "K":
        _, r, c = anchor
        x, y = key_center(panel, r, c)
    else:
        _, c = anchor
        x, y = thumb_center(panel, c)
    e = CAP_EDGE * panel.u
    if side == "N":
        return x, y - e
    if side == "S":
        return x, y + e
    if side == "W":
        return x - e, y
    return x + e, y


CALLOUTS = [
    Callout(
        "hero", 1250, 1005, 360, "Dual-role Ctrl",
        ["Hold Q or Enter to send", "Ctrl. A tap types the key."],
        [
            Leader(
                ("K", 2, 9), "S",
                ((kx(P_HERO, 2, 9), 975.0),),
                (1430.0, HERO_BAND),
            )
        ],
    ),
    Callout(
        "hero", 524, 1005, 640, "Layer keys",
        ["Hold the middle thumbs: left for symbols,", "right for navigation."],
        [
            Leader(("T", 3), "S", (), (700.0, HERO_BAND)),
            Leader(("T", 6), "S", (), (988.0, HERO_BAND)),
        ],
    ),
    Callout(
        "sym", 2034, 690, 338, "Fun layer",
        ["Hold SYM, then this thumb for", "F-keys, media and screenshot."],
        [Leader(("T", 6), "S", (), (2203.0, BAND0))],
    ),
    Callout(
        "sym", 1680, 690, 338, "Adjust layer",
        ["Hold SYM, then this thumb for", "Caps, RGB, reset, flashing."],
        [Leader(("T", 4), "S", (), (1849.0, BAND0))],
    ),
    Callout(
        "nav", 2428, 690, 338, "Number pad",
        ["Digits 1-0 sit in two home", "rows under the left hand."],
        [Leader(("K", 2, 0), "S", (), (2597.0, BAND0))],
    ),
    Callout(
        "nav", 2782, 690, 338, "Arrows at home",
        ["Left, Down and Right sit on", "the home row; Up above."],
        [
            Leader(
                ("K", 0, 7), "N",
                (
                    (kx(P_NAV, 0, 7), P_NAV.y + 15.0),
                    (P_NAV.x + SURF_W - 23.0, P_NAV.y + 15.0),
                    (P_NAV.x + SURF_W - 23.0, BAND0),
                ),
                (2951.0, BAND0),
            )
        ],
    ),
    Callout(
        "fun", 1680, BAND1, 338, "Screenshot",
        ["Cmd+Shift+Ctrl+4 on macOS,", "Win+Shift+S on Windows."],
        [
            Leader(
                ("K", 0, 2), "N",
                (
                    (kx(P_FUN, 0, 2), P_FUN.y + 15.0),
                    (P_FUN.x + 18.0, P_FUN.y + 15.0),
                    (P_FUN.x + 18.0, BAND1),
                ),
                (1849.0, BAND1),
            )
        ],
    ),
    Callout(
        "adj", 2782, BAND1, 338, "Reset & update",
        ["rset wipes the stored keymap;", "boot enters firmware flash."],
        [
            Leader(
                ("K", 0, 9), "E",
                (
                    (P_ADJ.x + SURF_W - 18.0, key_center(P_ADJ, 0, 9)[1]),
                    (P_ADJ.x + SURF_W - 18.0, BAND1),
                ),
                (P_ADJ.x + SURF_W - 18.0, BAND1),
            ),
            Leader(("K", 2, 9), "S", (), (kx(P_ADJ, 2, 9), BAND1)),
        ],
    ),
    Callout(
        "adj", 2428, BAND1, 338, "Lighting",
        ["Toggle and step through the", "LED effects."],
        [Leader(("K", 2, 5), "W", ((gap_center(P_ADJ), key_center(P_ADJ, 2, 5)[1]), (gap_center(P_ADJ), BAND1)), (2597.0, BAND1))],
    ),
]

HILLS = (
    ((620.0, 330.0), (160.0, 290.0, 450.0, 640.0, 880.0, 1160.0), 20261001),
    ((2740.0, 1420.0), (180.0, 330.0, 530.0, 780.0, 1080.0), 20261002),
)

REQUIRED = (
    "bg contour surface surface_edge cap cap_top cap_bottom cap_text "
    "accent accent_text muted muted_text dead dead_text title subtitle "
    "panel_title callout callout_line rule shadow shadow_opacity ghost_opacity"
).split()

CONTRAST = (
    ("cap_text", "cap", 4.5),
    ("cap_text", "cap_bottom", 4.5),
    ("accent_text", "accent", 4.5),
    ("muted_text", "muted", 3.0),
    ("dead_text", "dead", 1.5),
    ("title", "bg", 3.0),
    ("panel_title", "bg", 3.0),
    ("subtitle", "bg", 3.0),
    ("callout", "bg", 3.0),
    ("contour", "bg", 1.2),
)

PALETTES = {
    "dark": {
        "bg": "#0B1220",
        "contour": "#152741",
        "surface": "#101B2D",
        "surface_edge": "#1E3350",
        "cap": "#E9EFF7",
        "cap_top": "#FFFFFF",
        "cap_bottom": "#D7E0EC",
        "cap_text": "#15202E",
        "accent": "#2EC4B6",
        "accent_text": "#062722",
        "muted": "#1F6F6B",
        "muted_text": "#D9F5F1",
        "dead": "#16233A",
        "dead_text": "#546A8C",
        "title": "#2EC4B6",
        "subtitle": "#8FA6C4",
        "panel_title": "#5AD9CB",
        "callout": "#F2B84B",
        "callout_line": "#F2B84B",
        "rule": "#24405F",
        "shadow": "#000000",
        "shadow_opacity": "0.28",
        "ghost_opacity": "0.45",
    },
    "light": {
        "bg": "#F7F4EC",
        "contour": "#DCD5C4",
        "surface": "#FFFFFF",
        "surface_edge": "#D8D2C4",
        "cap": "#FFFFFF",
        "cap_top": "#FFFFFF",
        "cap_bottom": "#EFE9DC",
        "cap_text": "#1B2A3A",
        "accent": "#0E7C74",
        "accent_text": "#FFFFFF",
        "muted": "#CFE7E3",
        "muted_text": "#0B4F4A",
        "dead": "#EFEBE0",
        "dead_text": "#A79F8D",
        "title": "#0E7C74",
        "subtitle": "#5B5646",
        "panel_title": "#0E7C74",
        "callout": "#A85400",
        "callout_line": "#9A6B22",
        "rule": "#D8D2C4",
        "shadow": "#000000",
        "shadow_opacity": "0.12",
        "ghost_opacity": "0.45",
    },
}


def esc(s):
    return (
        s.replace("&", "&amp;")
        .replace("<", "&lt;")
        .replace(">", "&gt;")
        .replace('"', "&quot;")
        .replace("'", "&#39;")
    )


def f(x):
    s = f"{x:.2f}".rstrip("0").rstrip(".")
    return "0" if s in ("", "-0") else s


def text(x, y, s, size, fill, anchor="start", family=FONT_UI, weight=500,
         letter_spacing=None, stroke=None, stroke_width=None, baseline=None):
    attrs = [
        f'x="{f(x)}"',
        f'y="{f(y)}"',
        f'font-family="{family}"',
        f'font-size="{f(size)}"',
        f'font-weight="{weight}"',
        f'text-anchor="{anchor}"',
    ]
    if stroke:
        attrs += ['fill="none"', f'stroke="{stroke}"', f'stroke-width="{f(stroke_width or 1)}"']
    else:
        attrs.append(f'fill="{fill}"')
    if letter_spacing is not None:
        attrs.append(f'letter-spacing="{f(letter_spacing)}"')
    if baseline:
        attrs.append(f'dominant-baseline="{baseline}"')
    return f'<text {" ".join(attrs)}>{esc(s)}</text>'


def closed_spline(points):
    n = len(points)
    parts = []
    for i in range(n):
        p0 = points[(i - 1) % n]
        p1 = points[i]
        p2 = points[(i + 1) % n]
        p3 = points[(i + 2) % n]
        c1 = (p1[0] + (p2[0] - p0[0]) / 6, p1[1] + (p2[1] - p0[1]) / 6)
        c2 = (p2[0] - (p3[0] - p1[0]) / 6, p2[1] - (p3[1] - p1[1]) / 6)
        if i == 0:
            parts.append(f"M {f(p1[0])} {f(p1[1])}")
        parts.append(
            f"C {f(c1[0])} {f(c1[1])} {f(c2[0])} {f(c2[1])} {f(p2[0])} {f(p2[1])}"
        )
    parts.append("Z")
    return " ".join(parts)


def contours(pal):
    out = []
    amps = (0.10, 0.06, 0.035, 0.02)
    for center, radii, seed in HILLS:
        rnd = random.Random(seed)
        phases = [rnd.uniform(0, math.tau) for _ in range(4)]
        for ri, r in enumerate(radii):
            pts = []
            for k in range(0, 72, 6):
                t = math.tau * k / 72
                rr = r * (
                    1
                    + sum(a * math.sin((j + 1) * t + ph) for j, (a, ph) in enumerate(zip(amps, phases)))
                )
                pts.append((center[0] + rr * math.cos(t), center[1] + rr * math.sin(t)))
            out.append(
                f'<path class="contour" d="{closed_spline(pts)}" fill="none" '
                f'stroke="{pal["contour"]}" stroke-width="2" '
                f'opacity="{f(0.28 + 0.05 * ri)}"/>'
            )
    return "\n".join(out)


def keycap(cx, cy, u, cap, pal):
    s = CAP_SIZE * u
    kx0 = cx - s / 2
    ky0 = cy - s / 2
    rx = CAP_R * u
    kind = cap.kind
    out = []
    if kind == "ghost":
        out.append(f'<g opacity="{pal["ghost_opacity"]}">')
    if kind in ("base", "macro", "dual", "accent"):
        sh = max(2.0, 0.04 * u)
        out.append(
            f'<rect x="{f(kx0)}" y="{f(ky0 + sh)}" width="{f(s)}" height="{f(s)}" '
            f'rx="{f(rx)}" fill="{pal["shadow"]}" opacity="{pal["shadow_opacity"]}"/>'
        )
    fill = {
        "base": "url(#capg)",
        "macro": "url(#capg)",
        "dual": "url(#capg)",
        "accent": pal["accent"],
        "muted": pal["muted"],
        "dead": pal["dead"],
        "ghost": pal["muted"],
    }[kind]
    out.append(
        f'<rect x="{f(kx0)}" y="{f(ky0)}" width="{f(s)}" height="{f(s)}" '
        f'rx="{f(rx)}" fill="{fill}"/>'
    )
    color = {
        "base": pal["cap_text"],
        "macro": pal["cap_text"],
        "dual": pal["cap_text"],
        "accent": pal["accent_text"],
        "muted": pal["muted_text"],
        "dead": pal["dead_text"],
        "ghost": pal["muted_text"],
    }[kind]
    label = cap.label if cap.label else ("–" if kind == "dead" else "")
    if label:
        n = len(label)
        if kind == "dead":
            size = 0.35 * u
        elif cap.mono:
            size = 0.30 * u if n <= 3 else 0.25 * u
        elif n <= 2:
            size = 0.40 * u
        elif n == 3:
            size = 0.33 * u
        elif n == 4:
            size = 0.27 * u
        else:
            size = 0.23 * u
        out.append(
            text(
                cx, cy, label, size, color,
                anchor="middle",
                family=FONT_MONO if cap.mono else FONT_UI,
                weight=600 if kind == "accent" else 500,
                baseline="central",
            )
        )
    if kind == "dual":
        out.append(
            f'<circle class="dual-dot" cx="{f(kx0 + s - 0.16 * u)}" cy="{f(ky0 + 0.16 * u)}" '
            f'r="{f(0.07 * u)}" fill="{pal["callout"]}"/>'
        )
    if kind == "ghost":
        out.append("</g>")
    return "\n".join(out)


def render_panel(key, pal):
    p = PANELS[key]
    layer = LAYERS[key]
    surf_h = 2 * p.pad + ROWS_U * p.u
    rx = 26 if key == "hero" else 18
    out = [f'<g class="panel" data-key="{key}">']
    surf_w = HERO_W if key == "hero" else SURF_W
    out.append(
        f'<rect x="{f(p.x)}" y="{f(p.y)}" width="{f(surf_w)}" height="{f(surf_h)}" '
        f'rx="{rx}" fill="{pal["surface"]}" stroke="{pal["surface_edge"]}" stroke-width="2"/>'
    )
    out.append(text(p.x, p.y - 16, PANEL_TITLES[key], 26, pal["panel_title"], weight=600))
    for r in range(3):
        for c in range(10):
            cx, cy = key_center(p, r, c)
            out.append(f'<g class="cap">{keycap(cx, cy, p.u, layer.rows[r][c], pal)}</g>')
    for i, cap in enumerate(layer.thumbs):
        cx, cy = thumb_center(p, i + 2)
        out.append(f'<g class="cap">{keycap(cx, cy, p.u, cap, pal)}</g>')
    out.append("</g>")
    return "\n".join(out)


def render_callout(co, pal):
    hero = co.panel == "hero"
    p = PANELS[co.panel]
    dot_r = 6.0 if hero else 4.5
    out = ['<g class="callout">']
    cx = co.tx + co.w / 2.0
    out.append(text(cx, co.ty + CALLOUT_TITLE_DY, co.title, CALLOUT_SIZE, pal["callout"], anchor="middle", weight=600))
    for i, line in enumerate(co.lines):
        out.append(text(cx, co.ty + CALLOUT_LINE_DY + i * CALLOUT_LINE_STEP, line, CALLOUT_SIZE, pal["callout"], anchor="middle", weight=400))
    out.append("</g>")
    seen_ends = set()
    for lead in co.leaders:
        p0 = anchor_point(p, lead.anchor, lead.side)
        end = (p0[0] if lead.end[0] is None else lead.end[0], lead.end[1])
        pts = [p0, *lead.via, end]
        d = " ".join(f"{f(x)},{f(y)}" for x, y in pts)
        out.append(
            f'<polyline class="leader" points="{d}" fill="none" stroke="{pal["callout_line"]}" '
            f'stroke-width="2.5" stroke-dasharray="3 9" stroke-linecap="round"/>'
        )
        out.append(f'<circle class="leader-dot" cx="{f(p0[0])}" cy="{f(p0[1])}" r="{f(dot_r)}" fill="{pal["callout_line"]}"/>')
        if end not in seen_ends:
            seen_ends.add(end)
            out.append(f'<circle class="leader-dot" cx="{f(end[0])}" cy="{f(end[1])}" r="{f(dot_r)}" fill="{pal["callout_line"]}"/>')
    return "\n".join(out)


def build_svg(variant, pal):
    out = []
    out.append(
        f'<svg xmlns="http://www.w3.org/2000/svg" width="{W}" height="{H}" '
        f'viewBox="0 0 {W} {H}">'
    )
    out.append("<title>Omakeys starter keymap layout guide</title>")
    out.append("<desc>Five-layer keymap guide for the Omakeys (LoremIpsum36) keyboard.</desc>")
    out.append("<defs>")
    out.append(
        f'<linearGradient id="capg" x1="0" y1="0" x2="0" y2="1">'
        f'<stop offset="0" stop-color="{pal["cap_top"]}"/>'
        f'<stop offset="1" stop-color="{pal["cap_bottom"]}"/></linearGradient>'
    )
    out.append(f'<clipPath id="canvas"><rect x="0" y="0" width="{W}" height="{H}"/></clipPath>')
    out.append("</defs>")
    out.append(f'<rect class="bg" width="{W}" height="{H}" fill="{pal["bg"]}"/>')
    out.append('<g clip-path="url(#canvas)">')
    out.append(contours(pal))
    out.append("</g>")
    out.append(
        text(MARGIN, 250, "omakeys", 168, None, weight=200, letter_spacing=8,
             stroke=pal["title"], stroke_width=2.5)
    )
    for key in ("hero", "sym", "nav", "fun", "adj"):
        out.append(render_panel(key, pal))
    for co in CALLOUTS:
        out.append(render_callout(co, pal))
    out.append(
        f'<line class="rule" x1="{MARGIN}" y1="1600" x2="{W - MARGIN}" y2="1600" '
        f'stroke="{pal["rule"]}" stroke-width="2"/>'
    )
    out.append(text(MARGIN, 1642, "10-column, 36-key split · QWERTY base", 22, pal["subtitle"], weight=400))
    out.append(text(W - MARGIN, 1642, "github.com/FarzadHayat/omakeys", 22, pal["subtitle"], anchor="end", weight=400))
    out.append("</svg>")
    return "\n".join(out) + "\n"


def _lum(color):
    c = color.lstrip("#")
    vals = [int(c[i:i + 2], 16) / 255 for i in (0, 2, 4)]

    def lin(v):
        return v / 12.92 if v <= 0.03928 else ((v + 0.055) / 1.055) ** 2.4

    r, g, b = (lin(v) for v in vals)
    return 0.2126 * r + 0.7152 * g + 0.0722 * b


def contrast(a, b):
    la, lb = _lum(a), _lum(b)
    hi, lo = max(la, lb), min(la, lb)
    return (hi + 0.05) / (lo + 0.05)


def _callout_box(co):
    top = co.ty + (CALLOUT_TITLE_DY - 28.0)
    bottom = co.ty + CALLOUT_LINE_DY + (len(co.lines) - 1) * CALLOUT_LINE_STEP + 10.0
    return (co.tx - 8.0, top, co.w + 16.0, bottom - top)


def check_callouts():
    fails = []
    by_panel = {}
    for co in CALLOUTS:
        by_panel.setdefault(co.panel, []).append(co)
        x, y, w, h = _callout_box(co)
        if x < 0 or y < 0 or x + w > W or y + h > H:
            fails.append(f"callout {co.title!r} leaves the canvas")
    for panel, cos in by_panel.items():
        for i in range(len(cos)):
            for j in range(i + 1, len(cos)):
                a = _callout_box(cos[i])
                b = _callout_box(cos[j])
                if a[0] < b[0] + b[2] and b[0] < a[0] + a[2] and a[1] < b[1] + b[3] and b[1] < a[1] + a[3]:
                    fails.append(f"callouts overlap on {panel}: {cos[i].title!r} / {cos[j].title!r}")
    return fails


def check(svgs):
    fails = []
    for variant, svg in svgs.items():
        try:
            root = ElementTree.fromstring(svg)
        except ElementTree.ParseError as exc:
            fails.append(f"{variant}: SVG does not parse: {exc}")
            continue
        panels = caps = 0
        for el in root.iter():
            cls = el.get("class")
            if cls == "panel":
                panels += 1
            elif cls == "cap":
                caps += 1
        if panels != 5:
            fails.append(f"{variant}: expected 5 panels, found {panels}")
        if caps != 180:
            fails.append(f"{variant}: expected 180 keycaps, found {caps}")
        pal = PALETTES[variant]
        for key in REQUIRED:
            if key not in pal:
                fails.append(f"{variant}: palette missing {key!r}")
        for a, b, minimum in CONTRAST:
            ratio = contrast(pal[a], pal[b])
            if ratio < minimum:
                fails.append(f"{variant}: contrast {a}/{b} = {ratio:.2f} < {minimum}")
        if build_svg(variant, pal) != svg:
            fails.append(f"{variant}: build is not deterministic")
    fails += check_callouts()
    return fails


def font_warning():
    exe = shutil.which("fc-match")
    if not exe:
        return None
    res = subprocess.run([exe, "Helvetica Neue"], capture_output=True, text=True)
    if "Helvetica" not in res.stdout:
        return f"fc-match 'Helvetica Neue' resolved to {res.stdout.strip()!r}; labels may use a fallback"
    return None


def rsvg_bin():
    for cand in (shutil.which("rsvg-convert"), "/opt/homebrew/bin/rsvg-convert"):
        if cand:
            return cand
    raise SystemExit("rsvg-convert not found; install librsvg (brew install librsvg)")


def render_pngs(out, variants):
    exe = rsvg_bin()
    for v in variants:
        src = out / f"layout-guide-{v}.svg"
        if v == "dark":
            dst = out / "layout-guide-dark.png"
            subprocess.run([exe, "-o", str(dst), str(src)], check=True)
        else:
            dst = out / "layout-guide-light-a3.png"
            subprocess.run(
                [
                    exe, "-w", "4961",
                    "--page-width", "4961",
                    "--page-height", "3508",
                    "--top", "413",
                    "--background-color", PALETTES["light"]["bg"],
                    "-o", str(dst), str(src),
                ],
                check=True,
            )
        print(f"png  {dst}")


def main(argv=None):
    ap = argparse.ArgumentParser(description="Generate the Omakeys layout guide.")
    ap.add_argument("--variant", choices=["dark", "light"], action="append")
    ap.add_argument("--svg-only", action="store_true")
    ap.add_argument("--check", action="store_true")
    ap.add_argument("--out-dir", type=Path, default=DOCS)
    args = ap.parse_args(argv)

    out = args.out_dir
    out.mkdir(parents=True, exist_ok=True)
    variants = args.variant or ["dark", "light"]

    svgs = {}
    for v in variants:
        svg = build_svg(v, PALETTES[v])
        svgs[v] = svg
        path = out / f"layout-guide-{v}.svg"
        path.write_text(svg, encoding="utf-8")
        print(f"svg  {path}")

    if args.check:
        warn = font_warning()
        if warn:
            print(f"warning: {warn}", file=sys.stderr)
        fails = check(svgs)
        if fails:
            for msg in fails:
                print(f"FAIL {msg}", file=sys.stderr)
            return 1
        print("check ok")

    if not args.svg_only:
        render_pngs(out, variants)
    return 0


if __name__ == "__main__":
    sys.exit(main())
