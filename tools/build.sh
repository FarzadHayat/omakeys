#!/usr/bin/env bash
set -euo pipefail

PROJECT="$(cd "$(dirname "$0")/.." && pwd)"
QMK_HOME="${QMK_HOME:-${QMK_FIRMWARE:-$HOME/qmk_firmware}}"
# qmk CLI resolves the firmware tree from QMK_HOME, so it must be exported.
export QMK_HOME
SRC="$PROJECT/keyboard/penk/loremipsum36"
DEST="$QMK_HOME/keyboards/penk/loremipsum36"
KM="${1:-default}"

# The bundled ARM toolchain in tools/toolchain is a macOS (Mach-O) build; on
# Linux use the system one (Arch: pacman -S arm-none-eabi-gcc arm-none-eabi-newlib).
TOOLBIN="$PROJECT/tools/toolchain/Payload/bin"
if [ "$(uname -s)" = "Darwin" ] && [ -d "$TOOLBIN" ]; then
    export PATH="$TOOLBIN:$PATH"
fi
export PATH="$HOME/.local/bin:$PATH"

need() {
    command -v "$1" >/dev/null 2>&1 || { echo "build: $1 not found. $2" >&2; exit 1; }
}
need qmk "Install the QMK CLI (pipx install qmk, or uv tool install qmk)."
need rsync "Install rsync."
need arm-none-eabi-gcc "Install the ARM toolchain (Arch: pacman -S arm-none-eabi-gcc arm-none-eabi-newlib; macOS: see tools/toolchain)."
[ -d "$QMK_HOME" ] || { echo "build: $QMK_HOME not found. Clone qmk_firmware there or set QMK_FIRMWARE." >&2; exit 1; }

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
