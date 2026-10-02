/* Copyright 2026 Farzad
 * Based on rstacruz's qmk-base36 config (https://github.com/rstacruz/my_qmk_keymaps).
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 2 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 */

#pragma once

/* KeyPeek also wants via_command_kb; we own that hook for LayerLens + KeyPeek. */
#define KEYPEEK_DISABLE_RAW_HID_HANDLER

/* Note: this clone's WS2812 chain is on GP1, not GP0; see ws2812.pin in keyboard.json. */

/* RGB Matrix: classic left-to-right rainbow wave as boot default. The stock
 * CYCLE_LEFT_RIGHT effect spreads the full hue wheel because our LED positions
 * use the board's real 0..223 x coordinates (see keyboard.json); a few stock
 * effects stay enabled to cycle through on _ADJ. */
#define RGB_MATRIX_DEFAULT_MODE RGB_MATRIX_CYCLE_LEFT_RIGHT

#define ENABLE_RGB_MATRIX_CYCLE_LEFT_RIGHT
#define ENABLE_RGB_MATRIX_CYCLE_UP_DOWN
#define ENABLE_RGB_MATRIX_SOLID_COLOR
#define ENABLE_RGB_MATRIX_BREATHING
#define ENABLE_RGB_MATRIX_RAINBOW_MOVING_CHEVRON

/* Starter layers: base + SYM / NAV / FUN / ADJ */
#define DYNAMIC_KEYMAP_LAYER_COUNT 5

/* Tap-hold & timing tweaks (FORCE_NKRO dropped: board USB descriptor has nkro off) */
#define PERMISSIVE_HOLD
#define HOLD_ON_OTHER_KEY_PRESS_PER_KEY
#define TAPPING_TERM 125
#define TAPPING_TERM_PER_KEY

/* Mouse keys */
#undef MOUSEKEY_DELAY
#undef MOUSEKEY_INTERVAL
#undef MOUSEKEY_WHEEL_DELAY
#undef MOUSEKEY_MAX_SPEED
#undef MOUSEKEY_TIME_TO_MAX
#define MOUSEKEY_DELAY             0
#define MOUSEKEY_INTERVAL          16
#define MOUSEKEY_WHEEL_DELAY       0
#define MOUSEKEY_MAX_SPEED         6
#define MOUSEKEY_TIME_TO_MAX       32
#define MOUSEKEY_WHEEL_MAX_SPEED   8
#define MOUSEKEY_WHEEL_TIME_TO_MAX 8
