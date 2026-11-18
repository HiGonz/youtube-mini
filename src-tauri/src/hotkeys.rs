use crate::{commands, state::AppState, windows};
use std::str::FromStr;
use tauri::{AppHandle, Manager};
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut, ShortcutEvent, ShortcutState};

pub fn handler(app: &AppHandle, shortcut: &Shortcut, event: ShortcutEvent) {
    if event.state() != ShortcutState::Pressed { return; }
    let action = app.state::<AppState>().hotkey_map.lock().unwrap().get(&shortcut.id()).cloned();
    if let Some(a) = action { run(app, &a); }
}

fn run(app: &AppHandle, action: &str) {
    match action {
        "quick_launch" => windows::show_quick(app),
        "toggle_app" => windows::toggle_visibility(app),
        "toggle_mini" => windows::toggle_mini(app),
        "play_pause" | "media_play_pause" => { let _ = commands::run_player_action(app, "playpause"); }
        "next" | "media_next" => { let _ = commands::run_player_action(app, "next"); }
        "prev" | "media_prev" => { let _ = commands::run_player_action(app, "prev"); }
        "volume_up" => { let _ = commands::run_player_action(app, "volume_up"); }
        "volume_down" => { let _ = commands::run_player_action(app, "volume_down"); }
        "mute" => { let _ = commands::run_player_action(app, "mute"); }
        _ => {}
    }
}

/// (Re)registra todos los atajos. Los que fallen (ya usados por otra app, sintaxis inválida)
/// se reportan en `hotkey_errors` para mostrarlos en Ajustes; no impiden arrancar.
pub fn register_all(app: &AppHandle) {
    let st = app.state::<AppState>();
    let gs = app.global_shortcut();
    let _ = gs.unregister_all();
    let mut map = st.hotkey_map.lock().unwrap();
    let mut errors = st.hotkey_errors.lock().unwrap();
    map.clear();
    errors.clear();
    let hotkeys = st.settings.lock().unwrap().hotkeys.clone();
    for (action, accel) in hotkeys {
        if accel.is_empty() { continue; }
        match Shortcut::from_str(&accel) {
            Ok(sc) => match gs.register(sc) {
                Ok(_) => { map.insert(sc.id(), action); }
                Err(_) => errors.push(format!("{action}: «{accel}» está en uso por otra aplicación")),
            },
            Err(_) => errors.push(format!("{action}: «{accel}» no es un atajo válido")),
        }
    }
}
