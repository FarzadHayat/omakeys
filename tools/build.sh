#!/usr/bin/env bash
set -euo pipefail

PROJECT="$(cd "$(dirname "$0")/.." && pwd)"
QMK_HOME="${QMK_FIRMWARE:-$HOME/qmk_firmware}"
SRC="$PROJECT/keyboard/penk/loremipsum36"
DEST="$QMK_HOME/keyboards/penk/loremipsum36"
KM="${1:-default}"

TOOLBIN="$PROJECT/tools/toolchain/Payload/bin"
if [ -d "$TOOLBIN" ]; then
    export PATH="$TOOLBIN:$PATH"
fi
export PATH="$HOME/.local/bin:$PATH"

mkdir -p "$(dirname "$DEST")"
rsync -a --delete "$SRC/" "$DEST/"

if [ -d "$PROJECT/modules" ]; then
    for vendor_dir in "$PROJECT/modules"/*/; do
        [ -d "$vendor_dir" ] || continue
        vendor="$(basename "$vendor_dir")"
        mkdir -p "$QMK_HOME/modules/$vendor"
        rsync -a --delete "$vendor_dir" "$QMK_HOME/modules/$vendor/"
    done
fi

qmk compile -kb penk/loremipsum36 -km "$KM"

UF2_SRC="$QMK_HOME/penk_loremipsum36_${KM}.uf2"
if [ -f "$UF2_SRC" ]; then
    mkdir -p "$PROJECT/firmware"
    cp -f "$UF2_SRC" "$PROJECT/firmware/"
fi

# KeyPeek needs layout metadata for plain QMK/VIA boards.
qmk info -kb penk/loremipsum36 -km "$KM" -m -f json \
    > "$PROJECT/via/keyboard_info.json"
