// Copyright 2026 FarzadHayat
// SPDX-License-Identifier: GPL-3.0-only

#pragma once

#include <stdbool.h>
#include <stdint.h>

// Returns true when `data` started with the omakeys magic byte (0x4F) and was
// consumed. Non-magic frames return false, so a keymap that owns
// via_command_kb / raw_hid_receive can chain VIA afterwards:
//   if (omakeys_notify_handle_command(data, length)) return true;
bool omakeys_notify_handle_command(uint8_t *data, uint8_t length);

// Pushes LAYER while subscribed. Community modules usually auto-call this;
// keymaps that own `layer_state_set_user` should chain it explicitly:
//   return layer_state_set_omakeys_notify(state);
layer_state_t layer_state_set_omakeys_notify(layer_state_t state);

// Pushes KEY on every press/release. Auto-called by the module hook.
bool process_record_omakeys_notify(uint16_t keycode, keyrecord_t *record);

// Pushes LAYER when the default layer changes. Auto-called by the
// `default_layer_state_set` module hook (community module API 1.1.0).
layer_state_t default_layer_state_set_omakeys_notify(layer_state_t state);
