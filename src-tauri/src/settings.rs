//! Ajustes y datos persistentes (JSON en el directorio de configuración de la app).
//! No se guardan contraseñas, cookies ni tokens: eso lo gestiona WebView2 en su propio perfil.
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fs, path::Path};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub x: i32,
    pub y: i32,
    pub w: u32,
    pub h: u32,
    #[serde(default)]
    pub maximized: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
#[serde(default)]
pub struct Settings {
    pub close_behavior: String, // "tray" | "quit"
    pub start_with_windows: bool,
    pub remember_window: bool,
    pub hotkeys: BTreeMap<String, String>,
    pub mini_size: String, // "mini" | "compact" | "standard"
    pub mini_always_on_top: bool,
    pub auto_hide_controls: bool,
    pub theme: String, // "dark" | "oled" | "system"
    pub accent: String, // #rrggbb
    pub reduce_motion: bool,
    pub window: Option<Bounds>,
    pub mini_window: Option<Bounds>,
}

pub const HOTKEY_ACTIONS: &[(&str, &str)] = &[
    ("quick_launch", "Ctrl+Shift+Y"),
    ("toggle_app", "Ctrl+Alt+Y"),
    ("toggle_mini", "Ctrl+Shift+M"),
    ("play_pause", "Ctrl+Shift+Space"),
    ("next", "Ctrl+Shift+Right"),
    ("prev", "Ctrl+Shift+Left"),
    ("volume_up", "Ctrl+Shift+Up"),
    ("volume_down", "Ctrl+Shift+Down"),
    ("mute", "Ctrl+Shift+End"),
    // Teclas multimedia físicas. Vacío = desactivado.
    ("media_play_pause", "MediaPlayPause"),
    ("media_next", "MediaTrackNext"),
    ("media_prev", "MediaTrackPrevious"),
];

impl Default for Settings {
    fn default() -> Self {
        Self {
            close_behavior: "tray".into(),
            start_with_windows: false,
            remember_window: true,
            hotkeys: HOTKEY_ACTIONS.iter().map(|(a, k)| (a.to_string(), k.to_string())).collect(),
            mini_size: "compact".into(),
            mini_always_on_top: true,
            auto_hide_controls: true,
            theme: "dark".into(),
            accent: "#ff3b30".into(),
            reduce_motion: false,
            window: None,
            mini_window: None,
        }
    }
}

fn is_hex_color(s: &str) -> bool {
    s.len() == 7 && s.starts_with('#') && s[1..].chars().all(|c| c.is_ascii_hexdigit())
}

impl Settings {
    /// Normaliza valores inválidos (el JSON puede venir editado a mano o de otra versión).
    pub fn sanitize(&mut self) {
        if !matches!(self.close_behavior.as_str(), "tray" | "quit") { self.close_behavior = "tray".into(); }
        if !matches!(self.mini_size.as_str(), "mini" | "compact" | "standard") { self.mini_size = "compact".into(); }
        if !matches!(self.theme.as_str(), "dark" | "oled" | "system") { self.theme = "dark".into(); }
        if !is_hex_color(&self.accent) { self.accent = "#ff3b30".into(); }
        // Solo acciones conocidas; las que falten reciben su valor por defecto.
        let mut clean = BTreeMap::new();
        for (action, default) in HOTKEY_ACTIONS {
            let v = self.hotkeys.get(*action).cloned().unwrap_or_else(|| default.to_string());
            clean.insert(action.to_string(), v.trim().chars().take(48).collect());
        }
        self.hotkeys = clean;
    }
}

pub fn mini_dims(size: &str) -> (u32, u32) {
    match size {
        "mini" => (320, 180),
        "standard" => (560, 315),
        _ => (420, 236),
    }
}

pub fn load<T: for<'de> Deserialize<'de> + Default>(path: &Path) -> T {
    fs::read_to_string(path).ok().and_then(|s| serde_json::from_str(&s).ok()).unwrap_or_default()
}

/// Escritura atómica (tmp + rename) para no corromper el archivo si se cierra a mitad.
pub fn save<T: Serialize>(path: &Path, value: &T) {
    if let Some(dir) = path.parent() { let _ = fs::create_dir_all(dir); }
    let tmp = path.with_extension("tmp");
    if let Ok(s) = serde_json::to_string_pretty(value) {
        if fs::write(&tmp, s).is_ok() { let _ = fs::rename(&tmp, path); }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_fixes_garbage() {
        let mut s = Settings::default();
        s.theme = "neon".into();
        s.accent = "red".into();
        s.mini_size = "huge".into();
        s.hotkeys.insert("evil_action".into(), "Ctrl+X".into());
        s.hotkeys.remove("mute");
        s.sanitize();
        assert_eq!(s.theme, "dark");
        assert_eq!(s.accent, "#ff3b30");
        assert_eq!(s.mini_size, "compact");
        assert!(!s.hotkeys.contains_key("evil_action"));
        assert_eq!(s.hotkeys["mute"], "Ctrl+Shift+End");
    }

    #[test]
    fn partial_json_gets_defaults() {
        let s: Settings = serde_json::from_str(r#"{"theme":"oled"}"#).unwrap();
        assert_eq!(s.theme, "oled");
        assert_eq!(s.close_behavior, "tray");
    }

    #[test]
    fn default_hotkeys_are_unique() {
        let mut seen = std::collections::HashSet::new();
        for (_, k) in HOTKEY_ACTIONS { assert!(seen.insert(*k), "duplicado {k}"); }
    }
}
