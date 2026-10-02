# Omakeys: quick start

## Plug in

USB-C cable in, any port. The LEDs run a rainbow wave and the keyboard types
QWERTY. Nothing to install.

## The thumbs

Left side, from the outside in: **Cmd** (Windows key on Windows), **SYM** (hold),
**Shift**. Right side: **Space**, **NAV** (hold), **Option/Alt**.

- Hold **SYM** for symbols: the shifted punctuation row lives there.
- Hold **NAV** for arrows, numbers, Home/End/PgUp/PgDn, Tab and Esc.
- Hold **SYM**, then tap the right middle thumb (the NAV thumb) for **FUN**:
  F1-F12, media, mouse keys, and a screenshot key (top row, next to F12;
  takes a screenshot the way your OS expects).
- Hold **SYM**, then tap the left inner thumb for **ADJ**: Caps Lock, RGB
  controls, factory reset, firmware update.

## Ctrl

There is no separate Ctrl key: **hold Q or hold Enter**. Tap them normally for
Q or Enter. A quick tap always types the letter; only a deliberate hold
(200 ms) turns into Ctrl.

## Remap keys with VIA

1. Open [usevia.app](https://usevia.app) in Chrome or Edge.
2. Settings (gear) -> enable **Show Design tab**.
3. Design tab -> **Load Draft Definition** -> pick
   `omakeys.v3.json` (provided with your purchase).
4. Authorize the device when the browser asks. Your keymap appears; drag keys
   to remap. Changes are stored on the board.

A **Screenshot** macro is available under VIA's custom keys: on macOS it sends
Cmd+Shift+Ctrl+4, on Windows Win+Shift+S, on Linux PrtSc.

## See the layers live

Layer overlays show which layer is active while you type. Both are optional:

- **LayerLens** (macOS 15 or newer)
- **KeyPeek** (Windows, Linux, macOS; 0.6.0 or newer)

Run one overlay at a time. Either can stay open next to VIA.

## Reset to the factory layout

1. Hold **SYM**, press the left inner thumb (**ADJ**).
2. Tap the top-right key, `rset`.

Nothing visible happens; the stored keymap is wiped and the factory layout
comes back. If you ever flash new firmware and the layout looks unchanged,
this is why: do the reset above after flashing.

## Update the firmware

1. Unplug the keyboard.
2. Hold the **top-left key** and plug the cable back in. A drive called
   `RPI-RP2` appears.
3. Drag `Omakeys.uf2` onto that drive.
4. The board reboots by itself. Tap `rset` (see above) to load the factory
   layout.

## LED colors

Boot look is the left-to-right rainbow wave. On **ADJ**: `RGB` toggles the
lights, `RGB+` / `RGB-` step through effects, and VIA can set your own.
