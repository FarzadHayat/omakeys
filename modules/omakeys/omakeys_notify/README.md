# omakeys_notify (QMK module)

A QMK community module that exposes live layer and key events plus a
chunked dynamic-keymap read to the omakeys overlay over Raw HID. One
owned protocol replaces the vendored LayerLens and KeyPeek modules on the
rsta board; see [PROTOCOL.md](./PROTOCOL.md) for the full wire format.

GPL-3.0-only.

## What it does

- Answers `HELLO` with `INFO`: protocol version, matrix dimensions,
  dynamic keymap layer count, and keymap buffer size.
- Answers `GET_STATE` with the current default layer and layer bitmask,
  and `GET_KEYMAP` with 27-byte chunks of QMK's dynamic keymap buffer.
- After `SUBSCRIBE`, pushes `LAYER` on layer/default-layer changes and
  `KEY` on every press/release. The subscription lapses after 3 s without
  a `SUBSCRIBE`; `SUBSCRIBE` doubles as the keepalive.
- Stays silent otherwise, so VIA keeps the Raw HID channel to itself.

## Requirements

- QMK with Community Modules (2025q1 / 0.29.0 or later).
- `DYNAMIC_KEYMAP_ENABLE` for keymap reads (rsta gets it from VIA).
  Without it, `INFO` advertises `flags = 0` and `GET_KEYMAP` answers empty.
- Raw HID enabled (`RAW_ENABLE=yes`; `VIA_ENABLE` implies it). The module
  declares the `raw` feature, which emits `RAW_ENABLE=yes`, so the build
  enables the Raw HID endpoint for you.

## Install

Copy this directory to `modules/omakeys/omakeys_notify/` (or add this repo
as a submodule under `modules/`), then list it in your keymap's
`keymap.json`:

```json
{
    "modules": [
        "omakeys/omakeys_notify"
    ]
}
```

## Wiring

Include the header and chain the layer hook from `layer_state_set_user`:

```c
#include "omakeys_notify.h"

layer_state_t layer_state_set_user(layer_state_t state) {
    return layer_state_set_omakeys_notify(state);
}
```

Handle omakeys frames in `via_command_kb`; non-magic frames fall through
to VIA:

```c
bool via_command_kb(uint8_t *data, uint8_t length) {
    return omakeys_notify_handle_command(data, length);
}
```

If you have your own commands too, chain instead of replace:

```c
bool via_command_kb(uint8_t *data, uint8_t length) {
    if (omakeys_notify_handle_command(data, length)) return true;
    // ... your own handling ...
    return false;
}
```

Press/release pushes and default-layer (`DF`, `set_single_default_layer`)
pushes run through the auto-called `process_record` and
`default_layer_state_set` module hooks (community module API 1.1.0), so
nothing else needs wiring.

## Build

From this repository (needs QMK at `~/qmk_firmware`, or `QMK_FIRMWARE` set):

```sh
bash tools/build.sh rsta
```

Standalone, after copying the module into your QMK tree:

```sh
qmk compile -kb penk/loremipsum36 -km rsta
```
