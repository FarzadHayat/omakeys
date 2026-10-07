# Omakeys

A 36-key keyboard for first-time users, running beginner-friendly QMK/VIA
firmware: QWERTY out of the box, five clear layers, VIA remapping, and
optional live layer overlays. Built on the penk **LoremIpsum36** kit.

- Quick start: [docs/starter-quickstart.md](docs/starter-quickstart.md)
- Layout guide: [docs/layout-guide-dark.png](docs/layout-guide-dark.png)
- Print layout guide: [docs/layout-guide-light-a3.png](docs/layout-guide-light-a3.png)
- Build: `./tools/build.sh starter` (needs QMK + the ARM toolchain)

## Layout

| Layer | Hold | What it adds |
| --- | --- | --- |
| QWERTY | default | Plain QWERTY, arrows and F-keys on layers |
| SYM | left SYM thumb | Punctuation pairs and symbols |
| NAV | right middle thumb | Arrows, Home/End/PgUp/PgDn, numbers, Tab, Esc |
| FUN | SYM + right middle thumb | F1-F12, media, volume, mouse |
| ADJUST | SYM + left inner thumb | Caps Lock, RGB controls, factory reset, bootloader |

Ctrl lives on dual-role keys: hold **Q** or hold **Enter**. A quick tap always
types the letter; holding 200 ms sends Ctrl.

## Flashing

1. Unplug the keyboard.
2. Hold the **top-left key** while plugging the cable back in. An `RPI-RP2`
   drive appears.
3. Drag `Omakeys.uf2` onto the drive.
4. After it reboots, tap `rset` (hold **SYM**, tap the left inner thumb, tap
   the top-right key). Flashing alone keeps the stored keymap; the reset loads
   the factory layout.

## VIA

Open [usevia.app](https://usevia.app) in Chrome or Edge, enable the Design tab,
load `via/omakeys.v3.json`, and authorize the device. Remaps are stored on the
board.

## Layer overlays

On Omarchy/Hyprland, use the in-repo **Omakeys** app (`omakeys` CLI): [`overlay/README.md`](overlay/README.md).

Also: [LayerLens](https://github.com/FireBall1725/LayerLens) (macOS 15+) and
[KeyPeek](https://github.com/srwi/KeyPeek) (0.6.0+, Windows/Linux/macOS).
Run one overlay at a time; they share Raw HID with each other and with VIA.

## Firmware files

- `Omakeys.uf2` with `Omakeys.sha256`, in `release/v1.0.0/` and on the
  [releases page](https://github.com/FarzadHayat/omakeys/releases)
- `./tools/build.sh starter` builds the same firmware into `firmware/`

## License

GPL-3.0. The vendored layer-notify modules keep their own licenses
(KeyPeek: GPL-2.0-or-later, LayerLens: GPL-3.0-only), so distributed firmware
is effectively GPL-3.0. Keymap inspired by rstacruz's qmk-base36. The vendored
keyboard design and board files from penk's LoremIpsum36 are MIT (see
`keyboard/penk/loremipsum36/LICENSE`).
