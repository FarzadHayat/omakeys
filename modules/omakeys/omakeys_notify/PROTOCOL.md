# omakeys_notify wire protocol

Version 1.0. The protocol runs over the existing VIA Raw HID endpoint
(usage page `0xFF60`, usage `0x61`) as 32-byte reports, so it needs no new
USB descriptors and shares the channel with VIA. Every frame starts with
the magic byte `0x4F` ("O") at byte 0 and an opcode at byte 1; bytes 2..31
carry the payload. Unused payload bytes are zero when sent and ignored
when received. All multi-byte fields are little-endian.

The firmware is silent unless spoken to: `HELLO`, `GET_STATE` and
`GET_KEYMAP` are answered on demand, and `LAYER`/`KEY` pushes only flow
while a host is subscribed.

## Frame layout

| Byte  | Field |
|------:|-------|
| 0     | magic, always `0x4F` |
| 1     | opcode |
| 2..31 | payload |

## Host → device

| Op     | Name        | Payload |
|-------:|-------------|---------|
| `0x01` | HELLO       | none |
| `0x02` | GET_STATE   | none |
| `0x03` | SUBSCRIBE   | none |
| `0x04` | UNSUBSCRIBE | none |
| `0x05` | GET_KEYMAP  | offset `u16` LE (bytes 2..3), len `u8` (byte 4, 1..27) |

- **HELLO** asks the board to identify itself. The `INFO` reply proves the
  protocol is present.
- **GET_KEYMAP** requests a byte range of QMK's dynamic keymap buffer.
  `offset` is the first byte and `len` the number requested; the firmware
  clamps `len` to 27.

## Device → host

| Op     | Name         | Payload |
|-------:|--------------|---------|
| `0x81` | INFO         | ver_major, ver_minor, rows, cols, layers, flags, keymap_bytes `u16` LE |
| `0x82` | STATE        | default_layer, reserved (0), layer_state `u32` LE |
| `0x83` | LAYER (push) | same shape as STATE |
| `0x84` | KEY (push)   | row, col, pressed (0/1) |
| `0x85` | KEYMAP       | offset `u16` LE, len, data (`len` bytes) |

### INFO (`0x81`)

| Byte | Field | Notes |
|-----:|-------|-------|
| 2 | `ver_major` | protocol major version, `1` |
| 3 | `ver_minor` | protocol minor version, `0` |
| 4 | `rows` | matrix rows (`MATRIX_ROWS`) |
| 5 | `cols` | matrix columns (`MATRIX_COLS`) |
| 6 | `layers` | `DYNAMIC_KEYMAP_LAYER_COUNT`, 0 when dynamic keymap is unavailable |
| 7 | `flags` | bit 0: dynamic keymap available |
| 8 | `keymap_bytes` low byte | `layers × rows × cols × 2`, LE |
| 9 | `keymap_bytes` high byte | |

### STATE / LAYER (`0x82` / `0x83`)

| Byte | Field | Notes |
|-----:|-------|-------|
| 2 | `default_layer` | low byte of QMK's `default_layer_state` bitmask (bit 0 = base) |
| 3 | reserved | 0 |
| 4 | `layer_state` byte 0 | low byte |
| 5 | `layer_state` byte 1 | |
| 6 | `layer_state` byte 2 | |
| 7 | `layer_state` byte 3 | high byte |

### KEY (`0x84`)

| Byte | Field | Notes |
|-----:|-------|-------|
| 2 | `row` | matrix row |
| 3 | `col` | matrix column |
| 4 | `pressed` | 1 on press, 0 on release |

### KEYMAP (`0x85`)

| Byte  | Field | Notes |
|------:|-------|-------|
| 2 | `offset` low byte | byte offset into the keymap buffer, LE |
| 3 | `offset` high byte | |
| 4 | `len` | number of data bytes, 0..27 |
| 5..31 | `data` | `len` bytes, zero-padded |

## Semantics

- **HELLO → INFO.** `INFO` carries matrix dimensions, the dynamic keymap
  layer count, and `keymap_bytes = layers × rows × cols × 2` (1120 for
  rsta). `flags` bit 0 reports whether `GET_KEYMAP` can return data.
- **Subscription.** `SUBSCRIBE` sets the subscribed flag, refreshes the
  keepalive timestamp, and on an inactive→active transition immediately
  pushes `STATE`. Repeated `SUBSCRIBE` frames are silent refreshes. After
  3000 ms without a `SUBSCRIBE` the subscription lapses and pushes stop;
  events during a lapse are not buffered. `SUBSCRIBE` doubles as the
  keepalive.
- **Pushes while subscribed, on change only.** `LAYER` on layer-state or
  default-layer change; `KEY` on every matrix press and release.
- **GET_STATE** answers regardless of subscription, so a host can resync on
  connect and after a keepalive lapse.
- **GET_KEYMAP** reads QMK's dynamic keymap buffer: layers concatenated,
  layer stride = `rows × cols × 2` bytes, 2 bytes per keycode. Cell bytes
  are QMK's raw dynamic keymap storage, keycodes big-endian (same order
  VIA's own GET_BUFFER returns).
  The firmware clamps `len` to 27 and to the end of the buffer; an offset
  at or past the end answers `len = 0`. Meaningful only when `flags` bit 0
  is set.
- **Versioning.** Hosts accept major 1 and ignore the minor version. A
  major mismatch means unsupported firmware; the host logs, keeps its last
  good state, and retries the probe on the next reconnect.
- **VIA coexistence.** VIA's command ids occupy `0x00`–`0x15` and never
  start with `0x4F`, so omakeys frames are unambiguous on the shared
  channel. Unknown `0x4F` opcodes are ignored. Keymaps chain
  `omakeys_notify_handle_command()` before VIA so non-magic frames fall
  through.
