//! Comandos IPC. Dos superficies separadas:
//!  - UI local: control completo de la app.
//!  - Contenido remoto (YouTube): solo `yt_cmd` y `yt_report`, con entrada validada.
use crate::{
    hotkeys,
    settings::{Bounds, Settings},
    state::{AppState, NowPlaying},
    url_guard, windows,
};
use serde::{Deserialize, Serialize};
use std::sync::atomic::Ordering;
use tauri::{AppHandle, Emitter, Manager, Webview};
use tauri_plugin_autostart::ManagerExt;

type R<T> = Result<T, String>;

#[derive(Serialize)]
pub struct Snapshot {
    settings: Settings,
    now: NowPlaying,
    mini: bool,
    always_on_top: bool,
    hotkey_errors: Vec<String>,
}

fn snapshot(app: &AppHandle) -> Snapshot {
    let st = app.state::<AppState>();
    let settings = st.settings.lock().unwrap().clone();
    let now = st.now.lock().unwrap().clone();
    let hotkey_errors = st.hotkey_errors.lock().unwrap().clone();
    Snapshot {
        settings,
        now,
        mini: st.mini.load(Ordering::Relaxed),
        always_on_top: windows::main_window(app).and_then(|w| w.is_always_on_top().ok()).unwrap_or(false),
        hotkey_errors,
    }
}

#[tauri::command]
pub fn get_state(app: AppHandle) -> Snapshot { snapshot(&app) }

#[tauri::command]
pub fn get_settings(app: AppHandle) -> Settings { snapshot(&app).settings }

#[tauri::command]
pub fn set_settings(app: AppHandle, mut settings: Settings) -> R<Snapshot> {
    settings.sanitize();
    let st = app.state::<AppState>();
    {
        let mut cur = st.settings.lock().unwrap();
        // Los límites de ventana los gestiona el backend, no la UI.
        let (w, mw): (Option<Bounds>, Option<Bounds>) = (cur.window, cur.mini_window);
        let autostart_changed = cur.start_with_windows != settings.start_with_windows;
        *cur = settings;
        cur.window = w;
        cur.mini_window = mw;
        if autostart_changed {
            let want = cur.start_with_windows;
            let al = app.autolaunch();
            let _ = if want { al.enable() } else { al.disable() };
        }
    }
    st.save_settings();
    hotkeys::register_all(&app);
    let _ = app.emit("settings-changed", ());
    // Reaplica opciones del overlay del mini (auto-ocultar, acento...).
    if st.mini.load(Ordering::Relaxed) { windows::set_mini(&app, false); windows::set_mini(&app, true); }
    Ok(snapshot(&app))
}

#[tauri::command]
pub fn get_recent(app: AppHandle) -> serde_json::Value {
    let r = app.state::<AppState>().recent.lock().unwrap().clone();
    serde_json::json!({ "items": r.items, "last": r.last })
}

/// `action`: back | forward | home | music | reload | resume | go (con `input`).
#[tauri::command]
pub fn navigate(app: AppHandle, action: String, input: Option<String>, music: Option<bool>) -> R<()> {
    let st = app.state::<AppState>();
    match action.as_str() {
        "back" => windows::eval_yt(&app, "history.back()"),
        "forward" => windows::eval_yt(&app, "history.forward()"),
        "reload" => windows::eval_yt(&app, "location.reload()"),
        "home" => windows::navigate(&app, &url::Url::parse(url_guard::HOME).unwrap()),
        "music" => windows::navigate(&app, &url::Url::parse(url_guard::MUSIC_HOME).unwrap()),
        "resume" => {
            let last = st.recent.lock().unwrap().last.clone().ok_or("nada que reanudar")?;
            let mut u = url::Url::parse(&last.url).map_err(|e| e.to_string())?;
            if last.time > 5.0 && !last.music && u.query_pairs().all(|(k, _)| k != "t") {
                u.query_pairs_mut().append_pair("t", &format!("{}s", last.time as u64));
            }
            windows::navigate(&app, &u);
        }
        "go" => {
            let text = input.unwrap_or_default();
            if text.trim().is_empty() { return Ok(()); }
            let u = url_guard::resolve_input(&text, music.unwrap_or(false));
            if u.path().starts_with("/results") || u.path().starts_with("/search") {
                st.push_recent("search", text.trim(), u.as_str());
                st.save_recent();
            }
            windows::navigate(&app, &u);
        }
        _ => return Err("acción desconocida".into()),
    }
    Ok(())
}

#[tauri::command]
pub fn toggle_mini(app: AppHandle) { windows::toggle_mini(&app); }

#[tauri::command]
pub fn set_mini_size(app: AppHandle, size: String) {
    let st = app.state::<AppState>();
    if matches!(size.as_str(), "mini" | "compact" | "standard") {
        let mut s = st.settings.lock().unwrap();
        s.mini_size = size;
        s.mini_window = None; // fuerza el tamaño del preset la próxima vez
    }
    st.save_settings();
    if st.mini.load(Ordering::Relaxed) { windows::set_mini(&app, false); windows::set_mini(&app, true); }
}

#[tauri::command]
pub fn toggle_pip(app: AppHandle) { windows::eval_yt(&app, "window.__ytm&&window.__ytm.cmd('pip')"); }

#[tauri::command]
pub fn toggle_always_on_top(app: AppHandle) -> bool { windows::toggle_always_on_top(&app) }

#[tauri::command]
pub fn open_settings(app: AppHandle) { windows::open_settings(&app); }

#[tauri::command]
pub fn hide_quick(app: AppHandle) { windows::hide_quick(&app); }

#[tauri::command]
pub fn window_ctl(app: AppHandle, action: String) {
    let Some(w) = windows::main_window(&app) else { return };
    match action.as_str() {
        "minimize" => { let _ = w.minimize(); }
        "maximize" => { if w.is_maximized().unwrap_or(false) { let _ = w.unmaximize(); } else { let _ = w.maximize(); } }
        "close" => windows::request_close(&app),
        "drag" => { let _ = w.start_dragging(); }
        _ => {}
    }
}

/// Quick Launch: `go` (búsqueda/URL) o `resume`; `mini` opcional.
#[tauri::command]
pub fn quick_submit(app: AppHandle, action: String, input: Option<String>, music: Option<bool>, mini: Option<bool>) -> R<()> {
    windows::hide_quick(&app);
    windows::show_main(&app);
    navigate(app.clone(), action, input, music)?;
    if mini.unwrap_or(false) { windows::set_mini(&app, true); }
    Ok(())
}

const PLAYER_ACTIONS: &[&str] = &["playpause", "next", "prev", "volume_up", "volume_down", "mute"];

/// Acciones de reproducción desde la UI local (se ejecutan dentro del webview de YouTube).
#[tauri::command]
pub fn player_cmd(app: AppHandle, action: String) -> R<()> {
    run_player_action(&app, &action)
}

pub fn run_player_action(app: &AppHandle, action: &str) -> R<()> {
    if !PLAYER_ACTIONS.contains(&action) { return Err("acción no permitida".into()); }
    windows::eval_yt(app, &format!("window.__ytm&&window.__ytm.cmd('{action}')"));
    Ok(())
}

// ---------- Superficie remota (YouTube) ----------

/// Solo se acepta desde el webview "yt" y con URL de YouTube.
fn from_youtube(wv: &Webview) -> R<()> {
    let ok = wv.label() == "yt"
        && wv.url().ok().and_then(|u| u.host_str().map(url_guard::is_youtube_host)).unwrap_or(false);
    if ok { Ok(()) } else { Err("origen no permitido".into()) }
}

#[tauri::command]
pub fn yt_cmd(webview: Webview, app: AppHandle, action: String) -> R<()> {
    from_youtube(&webview)?;
    match action.as_str() {
        "expand" => windows::set_mini(&app, false),
        "close" => windows::request_close(&app),
        "drag" => { if let Some(w) = windows::main_window(&app) { let _ = w.start_dragging(); } }
        _ => return Err("acción desconocida".into()),
    }
    Ok(())
}

#[derive(Deserialize)]
pub struct YtState {
    playing: bool,
    title: String,
    artist: String,
    artwork: String,
    url: String,
    time: f64,
    duration: f64,
    music: bool,
    has_video: bool,
}

#[tauri::command]
pub fn yt_report(webview: Webview, app: AppHandle, state: YtState) -> R<()> {
    from_youtube(&webview)?;
    let url = url::Url::parse(&state.url).map_err(|e| e.to_string())?;
    if !url.host_str().is_some_and(url_guard::is_youtube_host) { return Err("url no permitida".into()); }
    let cut = |s: &str, n: usize| s.chars().take(n).collect::<String>();
    let num = |f: f64| if f.is_finite() && f >= 0.0 { f } else { 0.0 };
    let now = NowPlaying {
        playing: state.playing,
        title: cut(&state.title, 200),
        artist: cut(&state.artist, 120),
        artwork: if url_guard::is_allowed_artwork(&state.artwork) { state.artwork } else { String::new() },
        url: url.to_string(),
        time: num(state.time),
        duration: num(state.duration),
        music: state.music,
        has_video: state.has_video,
    };
    let st = app.state::<AppState>();
    let is_watch = now.has_video && (url.path() == "/watch" || now.music);
    if is_watch && !now.title.is_empty() {
        st.push_recent("video", &now.title, &now.url);
        st.recent.lock().unwrap().last = Some(now.clone());
        st.save_recent();
    }
    *st.now.lock().unwrap() = now.clone();
    if let Some(tray) = app.tray_by_id("tray") {
        let tip = if now.title.is_empty() { "YouTube Mini".to_string() } else { format!("{} {}", if now.playing { "▶" } else { "⏸" }, cut(&now.title, 80)) };
        let _ = tray.set_tooltip(Some(tip));
    }
    let _ = app.emit_to("ui", "player-state", now);
    Ok(())
}

