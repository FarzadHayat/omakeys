# omakeys (desktop app)

Live layer map for the Omakeys LoremIpsum36 keyboard on Omarchy / Hyprland.

Shows what each key does on the active QMK layer. Layout labels come from the keyboard over USB (VIA). Geometry is the built-in LoremIpsum36 profile.

## Requirements

- Omarchy / Hyprland (Wayland)
- GTK4 + gtk4-layer-shell
- udev rule so your user can open the board’s hidraw (installed by the install script; source: `tools/50-omakeys.rules` in the repo root)
- rsta firmware with KeyPeek notify + VIA/RAW enabled
- Do **not** run stock KeyPeek or LayerLens host apps at the same time (shared Raw HID)

## Install

```bash
./overlay/dist/install-desktop.sh
```

One command. It builds/installs the `omakeys` binary (`--skip-build` to skip),
then the **desktop entry** (app menu: **Omakeys**), icons, the Omarchy bar
plugin, its bar slot in `shell.json`, and a **systemd user service** that
starts the daemon at login and restarts it on crash. If the udev rule for
board HID access is missing, it installs that too with sudo (`--no-udev` to
skip). Re-run the same command to rebuild and restart the daemon.

Check or control the daemon:

```bash
systemctl --user status omakeys.service
systemctl --user stop omakeys.service    # start to bring it back
```

Hyprland binds (optional): copy lines from `overlay/dist/hypr-binds.conf` into your Hyprland config.

## Config

`~/.config/omakeys/config.toml` — visibility mode, timeout, position, opacity, tray, toggle chord label.

`~/.config/omakeys/overrides.toml` — per-key text/emoji overrides:

```toml
[[override]]
layer = 2
row = 0
col = 3
label = "‽"
```

## CLI

```bash
omakeys run        # daemon (default)
omakeys toggle     # show/hide
omakeys settings   # open settings
```

## Develop

```bash
cargo test --manifest-path overlay/Cargo.toml
cargo run --manifest-path overlay/Cargo.toml -- run
```
