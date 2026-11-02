// Copyright 2026 FarzadHayat
// SPDX-License-Identifier: GPL-3.0-only

#include QMK_KEYBOARD_H

#include "omakeys_notify.h"
#include "raw_hid.h"
#include "usb_descriptor.h"
#include "timer.h"

#ifdef DYNAMIC_KEYMAP_ENABLE
#    include "dynamic_keymap.h"
#endif

#include <string.h>

ASSERT_COMMUNITY_MODULES_MIN_API_VERSION(1, 1, 0);

#define OMAKEYS_MAGIC 0x4F

#define OMAKEYS_OPC_HELLO 0x01
#define OMAKEYS_OPC_GET_STATE 0x02
#define OMAKEYS_OPC_SUBSCRIBE 0x03
#define OMAKEYS_OPC_UNSUBSCRIBE 0x04
#define OMAKEYS_OPC_GET_KEYMAP 0x05

#define OMAKEYS_OPC_INFO 0x81
#define OMAKEYS_OPC_STATE 0x82
#define OMAKEYS_OPC_LAYER 0x83
#define OMAKEYS_OPC_KEY 0x84
#define OMAKEYS_OPC_KEYMAP 0x85

#define OMAKEYS_VER_MAJOR 1
#define OMAKEYS_VER_MINOR 0

#define OMAKEYS_FLAG_DYNAMIC_KEYMAP 0x01

#define OMAKEYS_SUBSCRIBE_TIMEOUT_MS 3000
#define OMAKEYS_MAX_CHUNK 27

static bool     omakeys_subscribed = false;
static uint32_t omakeys_last_ping  = 0;

static bool omakeys_is_active(void) {
    return omakeys_subscribed && timer_elapsed32(omakeys_last_ping) < OMAKEYS_SUBSCRIBE_TIMEOUT_MS;
}

static uint8_t omakeys_layer_count(void) {
#ifdef DYNAMIC_KEYMAP_ENABLE
    return DYNAMIC_KEYMAP_LAYER_COUNT;
#else
    return 0;
#endif
}

static uint8_t omakeys_flags(void) {
#ifdef DYNAMIC_KEYMAP_ENABLE
    return OMAKEYS_FLAG_DYNAMIC_KEYMAP;
#else
    return 0;
#endif
}

static uint16_t omakeys_keymap_bytes(void) {
    return (uint16_t)((uint32_t)omakeys_layer_count() * MATRIX_ROWS * MATRIX_COLS * 2);
}

// Zero a full report, stamp magic + opcode, copy the payload, then send.
static void omakeys_send_frame(uint8_t opcode, const uint8_t *payload, uint8_t payload_len) {
#if defined(RAW_ENABLE)
    uint8_t buf[RAW_EPSIZE] = {0};
    buf[0]                  = OMAKEYS_MAGIC;
    buf[1]                  = opcode;
    if (payload != NULL && payload_len > 0) {
        memcpy(&buf[2], payload, payload_len);
    }
    raw_hid_send(buf, RAW_EPSIZE);
#else
    (void)opcode;
    (void)payload;
    (void)payload_len;
#endif
}

static void omakeys_send_info(void) {
    uint16_t      keymap_bytes = omakeys_keymap_bytes();
    const uint8_t payload[8]   = {
        OMAKEYS_VER_MAJOR,
        OMAKEYS_VER_MINOR,
        (uint8_t)MATRIX_ROWS,
        (uint8_t)MATRIX_COLS,
        omakeys_layer_count(),
        omakeys_flags(),
        (uint8_t)(keymap_bytes & 0xFF),
        (uint8_t)(keymap_bytes >> 8),
    };
    omakeys_send_frame(OMAKEYS_OPC_INFO, payload, sizeof(payload));
}

// Byte 2 is the low byte of QMK's default-layer bitmask passed by the caller
// (the global `default_layer_state` is not yet updated inside the
// default_layer_state_set module hook), byte 3 stays zero, bytes 4..7 are
// layer_state written explicitly little-endian.
static void omakeys_send_state(uint8_t opcode, uint8_t default_layer, layer_state_t state) {
    const uint8_t payload[6] = {
        default_layer,
        0,
        (uint8_t)(state & 0xFF),
        (uint8_t)((state >> 8) & 0xFF),
        (uint8_t)((state >> 16) & 0xFF),
        (uint8_t)((state >> 24) & 0xFF),
    };
    omakeys_send_frame(opcode, payload, sizeof(payload));
}

static void omakeys_send_key(uint8_t row, uint8_t col, bool pressed) {
    const uint8_t payload[3] = {row, col, pressed ? 1 : 0};
    omakeys_send_frame(OMAKEYS_OPC_KEY, payload, sizeof(payload));
}

static void omakeys_send_keymap(uint16_t offset, const uint8_t *data, uint8_t len) {
    uint8_t payload[3 + OMAKEYS_MAX_CHUNK];
    payload[0] = (uint8_t)(offset & 0xFF);
    payload[1] = (uint8_t)(offset >> 8);
    payload[2] = len;
    if (len > 0) {
        memcpy(&payload[3], data, len);
    }
    omakeys_send_frame(OMAKEYS_OPC_KEYMAP, payload, (uint8_t)(3 + len));
}

bool omakeys_notify_handle_command(uint8_t *data, uint8_t length) {
    if (length < 2 || data[0] != OMAKEYS_MAGIC) {
        return false;
    }

    switch (data[1]) {
        case OMAKEYS_OPC_HELLO:
            omakeys_send_info();
            break;

        case OMAKEYS_OPC_GET_STATE:
            omakeys_send_state(OMAKEYS_OPC_STATE, (uint8_t)(default_layer_state & 0xFF), layer_state);
            break;

        case OMAKEYS_OPC_SUBSCRIBE: {
            bool was_active    = omakeys_is_active();
            omakeys_subscribed = true;
            omakeys_last_ping  = timer_read32();
            if (!was_active) {
                omakeys_send_state(OMAKEYS_OPC_STATE, (uint8_t)(default_layer_state & 0xFF), layer_state);
            }
            break;
        }

        case OMAKEYS_OPC_UNSUBSCRIBE:
            omakeys_subscribed = false;
            break;

        case OMAKEYS_OPC_GET_KEYMAP: {
            if (length < 5) {
                break;
            }
            uint16_t offset = (uint16_t)data[2] | ((uint16_t)data[3] << 8);
            uint8_t  len    = data[4] > OMAKEYS_MAX_CHUNK ? OMAKEYS_MAX_CHUNK : data[4];
            uint16_t total  = omakeys_keymap_bytes();
            if (offset >= total) {
                omakeys_send_keymap(offset, NULL, 0);
                break;
            }
            if ((uint16_t)len > total - offset) {
                len = (uint8_t)(total - offset);
            }
#ifdef DYNAMIC_KEYMAP_ENABLE
            uint8_t chunk[OMAKEYS_MAX_CHUNK];
            dynamic_keymap_get_buffer(offset, len, chunk);
            omakeys_send_keymap(offset, chunk, len);
#else
            omakeys_send_keymap(offset, NULL, 0);
#endif
            break;
        }

        default:
            break;
    }

    return true;
}

layer_state_t layer_state_set_omakeys_notify(layer_state_t state) {
    if (omakeys_is_active()) {
        omakeys_send_state(OMAKEYS_OPC_LAYER, (uint8_t)(default_layer_state & 0xFF), state);
    }
    return state;
}

bool process_record_omakeys_notify(uint16_t keycode, keyrecord_t *record) {
    (void)keycode;
    if (omakeys_is_active()) {
        omakeys_send_key(record->event.key.row, record->event.key.col, record->event.pressed);
    }
    return true;
}

// Auto-called before the global `default_layer_state` is assigned, so the
// new default layer comes from the hook argument, not the global.
layer_state_t default_layer_state_set_omakeys_notify(layer_state_t state) {
    if (omakeys_is_active()) {
        omakeys_send_state(OMAKEYS_OPC_LAYER, (uint8_t)(state & 0xFF), layer_state);
    }
    return state;
}
