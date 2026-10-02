/* Copyright 2026 Farzad
 * Based on rstacruz's qmk-base36 keymap (https://github.com/rstacruz/my_qmk_keymaps).
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

/* Starter keymap: the retail default. QWERTY base, four helper layers,
 * no combos, no key overrides, no layer lock, no one-hand/game modes. */

#include QMK_KEYBOARD_H
/* Modern QMK defines the US symbol keycodes (KC_EXLM, KC_UNDS, ...) only via keymap_us.h. */
#include "keymap_us.h"
#include "layerlens_notify.h"
#include "keypeek_layer_notify.h"

/* Penk's LAYOUT takes the same 36 arguments in the same order as rsta's LAYOUT_36
 * (rows of 10, then thumbs at matrix (3,2)..(3,7) left to right). */
#define LAYOUT_36 LAYOUT

#ifdef OS_DETECTION_ENABLE
#  include "os_detection.h"
#endif

// OS-adaptive behavior: when host detection is off or unsure, act as macOS
// (primary machine) so nothing regresses there.
static bool host_is_mac(void) {
#ifdef OS_DETECTION_ENABLE
    os_variant_t os = detected_host_os();
    return os != OS_LINUX && os != OS_WINDOWS;
#else
    return true;
#endif
}

static bool host_is_windows(void) {
#ifdef OS_DETECTION_ENABLE
    return detected_host_os() == OS_WINDOWS;
#else
    return false;
#endif
}

/*
 * Keycode aliases {{{
 */

#define _v_     KC_TRNS
#define ___     KC_NO
#define x__ENT  LCTL_T(KC_ENT)     /* ctrl(hold) or enter(tap) */
#define x__Q    LCTL_T(KC_Q)       /* ctrl(hold) or q(tap) */
#define x__LMB  MS_BTN1         /* Left mouse button */
#define x__RMB  MS_BTN2         /* Right mouse button */

/* Macros and stuff. QK_KB_0 = 0x7E00, the base VIA maps customKeycodes entries
 * onto, in order; keep this order in sync with customKeycodes in the V3 definition. */
enum custom_keycodes {
  MC_SHOT = QK_KB_0
};
// }}}

/*
 * Layer definitions {{{
 */

enum layers {
  _BASE = 0, _SYM, _NAV, _FUN, _ADJ
};

const uint16_t PROGMEM keymaps[][MATRIX_ROWS][MATRIX_COLS] = {
  /*
   * _BASE / Qwerty ── {{{
   * ╭────┬────┬────┬────┬────╮     ╭────┬────┬────┬────┬────╮
   * │ q ^│ w  │ e  │ r  │ t  │     │ y  │ u  │ i  │ o  │ p  │
   * ├────┼────┼────┼────┼────┤     ├────┼────┼────┼────┼────┤
   * │ a  │ s  │ d  │ f  │ g  │     │ h  │ j  │ k  │ l  │bks │
   * ├────┼────┼────┼────┼────┤     ├────┼────┼────┼────┼────┤
   * │ z  │ x  │ c  │ v  │ b  │     │ n  │ m  │ ,  │ .  │ent^│
   * ╰────┴────┴────┴────┴────┴─╮ ╭─┴────┴────┴────┴────┴────╯
   *           │cmd │SYM │ sft  │ │ spc  │NAV │opt │
   *           ╰────┴────┴──────╯ ╰──────┴────┴────╯ */

  [_BASE] = LAYOUT_36(
    x__Q, KC_W, KC_E, KC_R, KC_T, /**/ KC_Y, KC_U, KC_I,    KC_O,   KC_P,
    KC_A, KC_S, KC_D, KC_F, KC_G, /**/ KC_H, KC_J, KC_K,    KC_L,   KC_BSPC,
    KC_Z, KC_X, KC_C, KC_V, KC_B, /**/ KC_N, KC_M, KC_COMM, KC_DOT, x__ENT,
    /**/  /**/  KC_LGUI, MO(_SYM), KC_LSFT, /**/ KC_SPC, MO(_NAV), KC_LALT  /**/     /**/
  ),
  // }}}

  /*
   * _SYM / Symbols ── {{{
   * ╭────┬────┬────┬────┬────╮     ╭────┬────┬────┬────┬────╮
   * │ '  │ "  │ ^  │ ?  │ `  │     │ [  │ <  │ =  │ >  │ ]  │
   * ├────┼────┼────┼────┼────┤     ├────┼────┼────┼────┼────┤
   * │ !  │ @  │ #  │ $  │ %  │     │ {  │ (  │ :  │ )  │ }  │
   * ├────┼────┼────┼────┼────┤     ├────┼────┼────┼────┼────┤
   * │ \  │ ~  │ |  │ ;  │ &  │     │ /  │ *  │ -  │ +  │ _  │
   * ╰────┴────┴────┴────┴────┴─╮ ╭─┴────┴────┴────┴────┴────╯
   *           │    │ ▓▓ │ ADJ  │ │      │FUN │    │
   *           ╰────┴────┴──────╯ ╰──────┴────┴────╯ */

  [_SYM] = LAYOUT_36(
    KC_QUOT, KC_DQUO, KC_CIRC, KC_QUES, KC_GRV,   /**/ KC_LBRC, KC_LT,    KC_EQL,  KC_GT,   KC_RBRC,
    KC_EXLM, KC_AT,   KC_HASH, KC_DLR,  KC_PERC,  /**/ KC_LCBR, KC_LPRN,  KC_COLN, KC_RPRN, KC_RCBR,
    KC_BSLS, KC_TILD, KC_PIPE, KC_SCLN, KC_AMPR,  /**/ KC_SLSH, KC_ASTR,  KC_MINS, KC_PLUS, KC_UNDS,
    /**/     /**/     _v_,     _v_,     MO(_ADJ), /**/ _v_,     MO(_FUN), _v_      /**/     /**/
  ),
  // }}}

  /*
   * _NAV / Navigate ── {{{
   * ╭────┬────┬────┬────┬────╮     ╭────╭────┬────┬────╮┄───╮
   * │ctl │cmd │ ⇧↹ │ ↹  │opt │     │ ,  │home│ ▲  │end │del │
   * ├────┼────┼────┼────┼────┤     ├────├────┼────┼────┤┄───┤
   * │ 1  │ 2  │ 3  │ 4  │ 5  │     │ .  │ ◀  │ ▼  │ ▶  │ent │
   * ├────┼────┼────┼────┼────┤     ├────╰────┴────┴────╯┄───┤
   * │ 6  │ 7  │ 8  │ 9  │ 0  │     │    │ p↑ │ p↓ │esc │ctl │
   * ╰────┴────┴────┴────┴────┴─╮ ╭─┴────┴────┴────┴────┴────╯
   *           │cmd │SYM │ sft  │ │      │ ▓▓ │    │
   *           ╰────┴────┴──────╯ ╰──────┴────┴────╯ */

  [_NAV] = LAYOUT_36(
    KC_LCTL, KC_RGUI, S(KC_TAB), KC_TAB,   KC_RALT, /**/ KC_COMM, KC_HOME, KC_UP,   KC_END,  KC_DEL,
    KC_1,    KC_2,    KC_3,      KC_4,     KC_5,    /**/ KC_DOT,  KC_LEFT, KC_DOWN, KC_RGHT, KC_ENT,
    KC_6,    KC_7,    KC_8,      KC_9,     KC_0,    /**/ KC_ESC,  KC_PGUP, KC_PGDN, KC_ESC,  KC_LCTL,
    /**/     /**/     _v_,       _v_,      _v_,     /**/ _v_,     _v_,     _v_      /**/     /**/
  ),
  // }}}

  /*
   * _FUN / Function ── {{{
   * ╭────┬────┬────┐┄───┬────╮     ╭───┄┌────┬────┬────┐┄───╮
   * │f11 │f12 │shot│play│next│     │ w↑ │ L  │ ▲  │ R  │ b+ │
   * ├────┼────┼────┼────┼────┐     ├───┄├────┼────┼────┤┄───┤
   * │ f1 │ f2 │ f3 │ f4 │ f5 │     │ w↓ │ ◀  │ ▼  │ ▶  │ b- │
   * ├────┼────┼────┼────┼────┤     ├───┄└────┴────┴────┘┄───┤
   * │ f6 │ f7 │ f8 │ f9 │f10 │     │    │ v- │ v+ │    │    │
   * ╰────┴────┴────┴────┴────┴─╮ ╭─┴────┴────┴────┴────┴────╯
   *           │    │SYM │      │ │      │    │    │
   *           ╰────┴────┴──────╯ ╰──────┴────┴────╯ */

  [_FUN] = LAYOUT_36(
    KC_F11, KC_F12, MC_SHOT, KC_MPLY, KC_MNXT, /**/ MS_WHLU, x__LMB,  MS_UP, x__RMB,  KC_BRIU,
    KC_F1,  KC_F2,  KC_F3,   KC_F4,   KC_F5,   /**/ MS_WHLD, MS_LEFT, MS_DOWN, MS_RGHT, KC_BRID,
    KC_F6,  KC_F7,  KC_F8,   KC_F9,   KC_F10,  /**/ ___,     KC_VOLD, KC_VOLU, ___,     _v_,
    /**/    /**/    _v_,     _v_,     _v_,     /**/ _v_,     ___,     ___      /**/     /**/
  ),
  // }}}

  /*
   * _ADJ / Adjust ── {{{
   * ╭────┬────┬────┬────┬────╮     ╭────┬────┬────┬────┬────╮
   * │    │    │    │    │    │     │caps│    │    │    │rset│
   * ├────┼────┼────┼────┼────┤     ├────┼────┼────┼────┼────┤
   * │    │    │    │    │    │     │    │    │    │    │    │
   * ├────┼────┼────┼────┼────┤     ├────┼────┼────┼────┼────┤
   * │    │    │    │    │    │     │RGB │RGB+│RGB-│    │boot│
   * ╰────┴────┴────┴────┴────┴─╮ ╭─┴────┴────┴────┴────┴────╯
   *           │    │SYM │      │ │      │    │    │
   *           ╰────┴────┴──────╯ ╰──────┴────┴────╯ */

  [_ADJ] = LAYOUT_36(
    ___,      ___,    ___,     ___,     ___,     /**/ KC_CAPS, ___,     ___,     ___,     QK_CLEAR_EEPROM,
    ___,      ___,    ___,     ___,     ___,     /**/ ___,     ___,     ___,     ___,     ___,
    ___,      ___,    ___,     ___,     ___,     /**/ QK_RGB_MATRIX_TOGGLE, QK_RGB_MATRIX_MODE_NEXT, QK_RGB_MATRIX_MODE_PREVIOUS, ___, QK_BOOTLOADER,
    /**/      /**/    _v_,     _v_,     ___,     /**/ _v_,     _v_,     _v_      /**/     /**/
  ),
  // }}}
};

/*
 * Hold on other keypress stuff {{{
 */

// https://beta.docs.qmk.fm/using-qmk/software-features/tap_hold#ignore-mod-tap-interrupt
bool get_hold_on_other_key_press(uint16_t keycode, keyrecord_t *record) {
  switch (keycode) {
    case x__Q:
    case x__ENT:
      return false;
    default:
      return true;
  }
}

/* Give the Q and Enter dual keys a longer tap window than the rest, so
 * slow presses and rolls (into the next key) still emit the tap instead of
 * resolving to the hold. */
uint16_t get_tapping_term(uint16_t keycode, keyrecord_t *record) {
  switch (keycode) {
    case x__Q:
    case x__ENT:
      return 200;
    default:
      return TAPPING_TERM;
  }
}
// }}}

/*
 * Live layer overlay {{{
 * LayerLens (macOS, poll) + KeyPeek (Win/Linux/macOS, subscribe/push).
 * Run one overlay app at a time; both share Raw HID with VIA.
 */

layer_state_t layer_state_set_user(layer_state_t state) {
  return layer_state_set_layerlens_notify(state);
}

bool via_command_kb(uint8_t *data, uint8_t length) {
  if (keypeek_handle_command(data, length)) {
    return true;
  }
  return layerlens_notify_handle_command(data, length);
}
// }}}

/*
 * Macro definitions {{{
 */

bool process_record_user(uint16_t keycode, keyrecord_t *record) {
  switch (keycode) {
  case MC_SHOT: // Take screenshot (Mac Cmd+Shift+Ctrl+4, Win Win+Shift+S, Linux PrtSc)
    if (record->event.pressed) {
      if (host_is_mac()) { SEND_STRING(SS_LSFT(SS_LCTL(SS_LGUI("4")))); }
      else if (host_is_windows()) { SEND_STRING(SS_LGUI(SS_LSFT("s"))); }
      else { tap_code16(KC_PSCR); }
    }
    break;
  }

  return true;
};
// }}}

// vim:fdm=marker:fmr={{{,}}}
