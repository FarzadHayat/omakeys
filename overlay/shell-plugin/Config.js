.pragma library

function defaultConfig() {
  return {
    mode: "auto",
    timeout_ms: 2000,
    anchor: "bottom_right",
    opacity: 0.92,
    scale: 1.0,
    toggle_chord: "SUPER+ALT+O",
    label_style: "icons",
    tray_visible: true
  }
}

function mergeParsed(text) {
  try {
    var o = JSON.parse(text)
    var d = defaultConfig()
    for (var k in d) {
      if (o[k] !== undefined) d[k] = o[k]
    }
    // Legacy mode "hidden" → Auto; temp hide is the overlay toggle.
    if (d.mode === "hidden") d.mode = "auto"
    return d
  } catch (e) {
    return defaultConfig()
  }
}

function toApplyJson(cfg) {
  return JSON.stringify(cfg)
}
