#!/usr/bin/env bash
# Build + install: daemon binary, systemd user service, launcher, Omarchy bar
# plugin, and (if missing) the udev rule for board HID access.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
REPO_ROOT="$(cd "$ROOT/.." && pwd)"

DO_BUILD=1
DO_UDEV=1
for arg in "$@"; do
  case "$arg" in
    --skip-build) DO_BUILD=0 ;;
    --no-udev) DO_UDEV=0 ;;
    -h|--help)
      cat <<EOF
usage: $(basename "$0") [--skip-build] [--no-udev]

  --skip-build  do not run cargo install
  --no-udev     do not install the board hidraw udev rule (needs sudo)
EOF
      exit 0
      ;;
    *)
      echo "unknown option: $arg (see --help)" >&2
      exit 2
      ;;
  esac
done

DESKTOP_SRC="$ROOT/dist/omakeys.desktop"
ICON_SRC="$ROOT/dist/omakeys.svg"
ICON_SYM_SRC="$ROOT/dist/omakeys-symbolic.svg"
PLUGIN_SRC="$ROOT/shell-plugin"
APP_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/applications"
ICON_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/scalable/apps"
ICON_SYM_DIR="${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor/symbolic/apps"
PLUGIN_DST="${XDG_CONFIG_HOME:-$HOME/.config}/omarchy/plugins/omakeys"
SHELL_JSON="${XDG_CONFIG_HOME:-$HOME/.config}/omarchy/shell.json"
UNIT_DIR="${XDG_CONFIG_HOME:-$HOME/.config}/systemd/user"

if [[ "$DO_BUILD" == 1 ]]; then
  if command -v cargo >/dev/null 2>&1; then
    echo "Building omakeys (cargo install --path $ROOT)..."
    cargo install --path "$ROOT"
  else
    echo "warning: cargo not found; build manually: cargo install --path $ROOT" >&2
  fi
fi

BIN="$(command -v omakeys || true)"
if [[ -z "$BIN" && -x "$HOME/.cargo/bin/omakeys" ]]; then
  BIN="$HOME/.cargo/bin/omakeys"
fi
if [[ -z "$BIN" ]]; then
  echo "omakeys not found. Run: cargo install --path $ROOT" >&2
  exit 1
fi

mkdir -p "$APP_DIR" "$ICON_DIR" "$ICON_SYM_DIR"

install -m 644 "$ICON_SRC" "$ICON_DIR/omakeys.svg"
if [[ -f "$ICON_SYM_SRC" ]]; then
  install -m 644 "$ICON_SYM_SRC" "$ICON_SYM_DIR/omakeys-symbolic.svg"
fi
# Drop short-lived omakeys-v name if present
rm -f "$ICON_DIR/omakeys-v.svg" "$ICON_SYM_DIR/omakeys-v-symbolic.svg"
if command -v gtk-update-icon-cache >/dev/null 2>&1; then
  gtk-update-icon-cache -f "${XDG_DATA_HOME:-$HOME/.local/share}/icons/hicolor" 2>/dev/null || true
fi

# Rewrite Exec to absolute path so the menu launcher works even if PATH is thin.
tmp="$(mktemp)"
sed "s|^Exec=.*|Exec=$BIN run|" "$DESKTOP_SRC" >"$tmp"
install -m 644 "$tmp" "$APP_DIR/omakeys.desktop"
rm -f "$tmp"

if command -v update-desktop-database >/dev/null 2>&1; then
  update-desktop-database "$APP_DIR" 2>/dev/null || true
fi

# Omarchy bar-widget plugin (native panel).
if [[ -d "$PLUGIN_SRC" ]]; then
  mkdir -p "$PLUGIN_DST"
  # Replace a prior dev symlink with a real install tree.
  if [[ -L "$PLUGIN_DST" ]]; then
    rm -f "$PLUGIN_DST"
    mkdir -p "$PLUGIN_DST"
  fi
  cp -a "$PLUGIN_SRC"/. "$PLUGIN_DST/"
  echo "Installed Omarchy plugin: $PLUGIN_DST"
fi

# Ensure bar layout includes omakeys; unpin from SNI tray list if present.
if [[ -f "$SHELL_JSON" ]]; then
  SHELL_JSON="$SHELL_JSON" python3 - <<'PY'
import json, os, sys
path = os.environ["SHELL_JSON"]
with open(path, encoding="utf-8") as f:
    data = json.load(f)
bar = data.setdefault("bar", {})
layout = bar.setdefault("layout", {})
right = layout.setdefault("right", [])
if not isinstance(right, list):
    print("shell.json bar.layout.right is not a list; skip bar slot", file=sys.stderr)
    sys.exit(0)

def entry_id(entry):
    if isinstance(entry, str):
        return entry
    if isinstance(entry, dict):
        return entry.get("id")
    return None

if not any(entry_id(e) == "omakeys" for e in right):
    right.append({"id": "omakeys"})

for entry in right:
    if not isinstance(entry, dict):
        continue
    if entry.get("id") != "omarchy.tray":
        continue
    pinned = entry.get("pinned")
    if isinstance(pinned, list) and "omakeys" in pinned:
        entry["pinned"] = [p for p in pinned if p != "omakeys"]

with open(path, "w", encoding="utf-8") as f:
    json.dump(data, f, indent=2)
    f.write("\n")
print(f"Updated bar layout: {path}")
PY
else
  echo "No $SHELL_JSON — add {\"id\": \"omakeys\"} to bar.layout.right manually" >&2
fi

if command -v omarchy-shell >/dev/null 2>&1; then
  omarchy-shell -q shell rescanPlugins || true
fi

# Systemd user service: starts the daemon at login, restarts it on crash.
if command -v systemctl >/dev/null 2>&1; then
  mkdir -p "$UNIT_DIR"
  tmp_unit="$(mktemp)"
  sed "s|^ExecStart=.*|ExecStart=$BIN run|" "$ROOT/dist/omakeys.service" >"$tmp_unit"
  install -m 644 "$tmp_unit" "$UNIT_DIR/omakeys.service"
  rm -f "$tmp_unit"
  systemctl --user daemon-reload
  # Replace a running non-systemd instance so the service owns the daemon.
  if "$BIN" ping >/dev/null 2>&1 && ! systemctl --user is-active --quiet omakeys.service; then
    "$BIN" quit >/dev/null 2>&1 || true
    for _ in $(seq 1 20); do
      "$BIN" ping >/dev/null 2>&1 || break
      sleep 0.1
    done
  fi
  systemctl --user enable omakeys.service
  systemctl --user restart omakeys.service
  # Wait for the restarted daemon's IPC socket so a ping right after install works.
  ready=0
  for _ in $(seq 1 20); do
    if "$BIN" ping >/dev/null 2>&1; then
      ready=1
      break
    fi
    sleep 0.1
  done
  if [[ "$ready" == 1 ]]; then
    echo "Enabled systemd user service: omakeys.service"
  else
    echo "warning: daemon not answering; check: systemctl --user status omakeys.service" >&2
  fi
else
  echo "warning: systemctl not found; run the daemon manually: $BIN run &" >&2
fi

# Udev rule: lets a normal user open the board's hidraw (live key data).
UDEV_SRC="$REPO_ROOT/tools/50-omakeys.rules"
UDEV_DST="/etc/udev/rules.d/50-omakeys.rules"
if [[ "$DO_UDEV" == 1 ]]; then
  if [[ -f "$UDEV_DST" ]]; then
    echo "udev rule already installed: $UDEV_DST"
  elif [[ ! -f "$UDEV_SRC" ]]; then
    echo "warning: udev rule missing from checkout: $UDEV_SRC" >&2
  elif command -v sudo >/dev/null 2>&1; then
    echo "Installing udev rule (sudo): $UDEV_DST"
    sudo install -m 644 "$UDEV_SRC" "$UDEV_DST"
    sudo udevadm control --reload-rules
    sudo udevadm trigger
  else
    echo "warning: install udev rule manually: sudo cp $UDEV_SRC $UDEV_DST" >&2
  fi
fi

echo "Installed icon: $ICON_DIR/omakeys.svg"
echo "Installed launcher: $APP_DIR/omakeys.desktop"
echo "Daemon: systemctl --user status omakeys.service"
echo "Left-click the Omakeys bar icon for settings"
