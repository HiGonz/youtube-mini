//! Gestión de ventanas. Una sola ventana "main" con DOS webviews:
//!   - "ui": interfaz local (barra superior). Sin acceso a contenido remoto.
//!   - "yt": YouTube real. Es el único que reproduce; el Mini Mode solo cambia tamaño de la
//!           ventana y oculta "ui", por eso la reproducción nunca se reinicia ni se duplica.
use crate::{
    settings::{mini_dims, Bounds},
    state::AppState,
    url_guard::{self, is_allowed_nav},
};
use std::sync::atomic::Ordering;
use tauri::{
    webview::{PageLoadEvent, WebviewBuilder},
    window::WindowBuilder,
    AppHandle, Emitter, LogicalPosition, LogicalSize, Manager, PhysicalPosition, PhysicalSize, Rect, WebviewUrl,
    WebviewWindowBuilder, Window,
};
use tauri_plugin_opener::OpenerExt;

pub const TOP: f64 = 48.0;
const MIN_NORMAL: (f64, f64) = (640.0, 400.0);
const MIN_MINI: (f64, f64) = (240.0, 135.0);

pub fn main_window(app: &AppHandle) -> Option<Window> { app.get_window("main") }

pub fn open_external(app: &AppHandle, url: &str) {
    if url.starts_with("https://") || url.starts_with("http://") {
        let _ = app.opener().open_url(url, None::<&str>);
    }
}

fn mini_opts_js(app: &AppHandle) -> String {
    let st = app.state::<AppState>();
    let s = st.settings.lock().unwrap();
    serde_json::json!({ "autoHide": s.auto_hide_controls, "reduceMotion": s.reduce_motion, "accent": s.accent }).to_string()
}

pub fn eval_yt(app: &AppHandle, js: &str) {
    if let Some(wv) = app.get_webview("yt") { let _ = wv.eval(js); }
}

fn sync_mini_to_page(app: &AppHandle) {
    let on = app.state::<AppState>().mini.load(Ordering::Relaxed);
    eval_yt(app, &format!("window.__ytm&&window.__ytm.setMini({on},{})", mini_opts_js(app)));
}

fn bounds_on_screen(win: &Window, b: &Bounds) -> bool {
    win.available_monitors().map(|ms| {
        ms.iter().any(|m| {
            let p = m.position();
            let s = m.size();
            b.x + 60 > p.x && b.x < p.x + s.width as i32 - 60 && b.y >= p.y - 10 && b.y < p.y + s.height as i32 - 60
        })
    }).unwrap_or(false)
}

pub fn build_main(app: &AppHandle, start_hidden: bool) -> tauri::Result<()> {
    let st = app.state::<AppState>();
    let saved = if st.settings.lock().unwrap().remember_window { st.settings.lock().unwrap().window } else { None };

    let win = WindowBuilder::new(app, "main")
        .title("YouTube Mini")
        .inner_size(1100.0, 720.0)
        .min_inner_size(MIN_NORMAL.0, MIN_NORMAL.1)
        .decorations(false)
        .shadow(true)
        .visible(false)
        .build()?;

    let size = win.inner_size()?.to_logical::<f64>(win.scale_factor()?);
    let bg = tauri::window::Color(11, 11, 13, 255);

    win.add_child(
        WebviewBuilder::new("ui", WebviewUrl::App("index.html?view=main".into())).background_color(bg),
        LogicalPosition::new(0.0, 0.0),
        LogicalSize::new(size.width, TOP),
    )?;

    let nav_app = app.clone();
    let load_app = app.clone();
    let yt = WebviewBuilder::new("yt", WebviewUrl::External(url::Url::parse(url_guard::HOME).unwrap()))
        .background_color(bg)
        .initialization_script(include_str!("bridge.js"))
        .on_navigation(move |u| {
            if is_allowed_nav(u) { true } else { open_external(&nav_app, u.as_str()); false }
        })
        .on_page_load(move |_wv, payload| {
            if matches!(payload.event(), PageLoadEvent::Finished) { sync_mini_to_page(&load_app); }
        });
    win.add_child(yt, LogicalPosition::new(0.0, TOP), LogicalSize::new(size.width, size.height - TOP))?;

    if let Some(b) = saved {
        if bounds_on_screen(&win, &b) {
            let _ = win.set_position(PhysicalPosition::new(b.x, b.y));
            let _ = win.set_size(PhysicalSize::new(b.w, b.h));
            if b.maximized { let _ = win.maximize(); }
        }
    }
    layout(app);
    if !start_hidden { show_main(app); }
    Ok(())
}

/// Reposiciona los dos webviews según el tamaño de la ventana y el modo.
pub fn layout(app: &AppHandle) {
    let Some(win) = main_window(app) else { return };
    let (Ok(sf), Ok(px)) = (win.scale_factor(), win.inner_size()) else { return };
    let size = px.to_logical::<f64>(sf);
    let mini = app.state::<AppState>().mini.load(Ordering::Relaxed);
    let top = if mini { 0.0 } else { TOP };
    if let Some(ui) = app.get_webview("ui") {
        if mini { let _ = ui.hide(); } else { let _ = ui.show(); }
        let _ = ui.set_bounds(Rect { position: LogicalPosition::new(0.0, 0.0).into(), size: LogicalSize::new(size.width, top).into() });
    }
    if let Some(yt) = app.get_webview("yt") {
        let _ = yt.set_bounds(Rect {
            position: LogicalPosition::new(0.0, top).into(),
            size: LogicalSize::new(size.width, (size.height - top).max(1.0)).into(),
        });
    }
}

/// En Mini Mode mantiene 16:9 cuando el usuario redimensiona.
pub fn enforce_mini_aspect(app: &AppHandle) {
    if !app.state::<AppState>().mini.load(Ordering::Relaxed) { return; }
    let Some(win) = main_window(app) else { return };
    let (Ok(sf), Ok(px)) = (win.scale_factor(), win.inner_size()) else { return };
    let s = px.to_logical::<f64>(sf);
    let want = (s.width * 9.0 / 16.0).round();
    if (s.height - want).abs() > 2.0 { let _ = win.set_size(LogicalSize::new(s.width, want)); }
}

fn current_bounds(win: &Window) -> Option<Bounds> {
    let p = win.outer_position().ok()?;
    let s = win.inner_size().ok()?;
    Some(Bounds { x: p.x, y: p.y, w: s.width, h: s.height, maximized: win.is_maximized().unwrap_or(false) })
}

pub fn save_bounds(app: &AppHandle) {
    let Some(win) = main_window(app) else { return };
    let st = app.state::<AppState>();
    let mini = st.mini.load(Ordering::Relaxed);
    if win.is_minimized().unwrap_or(false) { return; }
    if let Some(b) = current_bounds(&win) {
        let mut s = st.settings.lock().unwrap();
        if mini { s.mini_window = Some(Bounds { maximized: false, ..b }); }
        else if s.remember_window { s.window = Some(b); }
    }
    st.save_settings();
}

pub fn set_mini(app: &AppHandle, on: bool) {
    let st = app.state::<AppState>();
    if st.mini.load(Ordering::Relaxed) == on { return; }
    let Some(win) = main_window(app) else { return };
    let _ = win.show();
    let (mini_size, on_top, mini_saved, remember) = {
        let s = st.settings.lock().unwrap();
        (s.mini_size.clone(), s.mini_always_on_top, s.mini_window, s.remember_window)
    };
    if on {
        if win.is_maximized().unwrap_or(false) { let _ = win.unmaximize(); }
        save_bounds(app); // guarda la ventana normal antes de cambiar el flag
        st.mini.store(true, Ordering::Relaxed);
        let _ = win.set_min_size(Some(LogicalSize::new(MIN_MINI.0, MIN_MINI.1)));
        let (w, h) = mini_dims(&mini_size);
        let _ = win.set_size(LogicalSize::new(w as f64, h as f64));
        match mini_saved.filter(|b| remember && bounds_on_screen(&win, b)) {
            Some(b) => {
                let _ = win.set_position(PhysicalPosition::new(b.x, b.y));
                let _ = win.set_size(PhysicalSize::new(b.w, b.h));
            }
            None => {
                if let (Ok(Some(m)), Ok(sf)) = (win.current_monitor(), win.scale_factor()) {
                    let ms = m.size();
                    let mp = m.position();
                    let x = mp.x + ms.width as i32 - (w as f64 * sf) as i32 - (24.0 * sf) as i32;
                    let y = mp.y + ms.height as i32 - (h as f64 * sf) as i32 - (72.0 * sf) as i32;
                    let _ = win.set_position(PhysicalPosition::new(x, y));
                }
            }
        }
        let _ = win.set_always_on_top(on_top);
    } else {
        save_bounds(app); // guarda posición/tamaño del mini
        st.mini.store(false, Ordering::Relaxed);
        let _ = win.set_always_on_top(false);
        let _ = win.set_min_size(Some(LogicalSize::new(MIN_NORMAL.0, MIN_NORMAL.1)));
        let normal = st.settings.lock().unwrap().window;
        match normal.filter(|b| bounds_on_screen(&win, b)) {
            Some(b) => {
                let _ = win.set_size(PhysicalSize::new(b.w, b.h));
                let _ = win.set_position(PhysicalPosition::new(b.x, b.y));
                if b.maximized { let _ = win.maximize(); }
            }
            None => { let _ = win.set_size(LogicalSize::new(1100.0, 720.0)); let _ = win.center(); }
        }
    }
    layout(app);
    sync_mini_to_page(app);
    let _ = app.emit("mini-changed", on);
    let _ = win.set_focus();
}

pub fn toggle_mini(app: &AppHandle) {
    let on = app.state::<AppState>().mini.load(Ordering::Relaxed);
    set_mini(app, !on);
}

pub fn toggle_always_on_top(app: &AppHandle) -> bool {
    let Some(win) = main_window(app) else { return false };
    let now = !win.is_always_on_top().unwrap_or(false);
    let _ = win.set_always_on_top(now);
    let st = app.state::<AppState>();
    if st.mini.load(Ordering::Relaxed) {
        st.settings.lock().unwrap().mini_always_on_top = now;
        st.save_settings();
    }
    let _ = app.emit("settings-changed", ());
    now
}

pub fn show_main(app: &AppHandle) {
    if let Some(w) = main_window(app) {
        let _ = w.show();
        let _ = w.unminimize();
        let _ = w.set_focus();
    }
}

pub fn toggle_visibility(app: &AppHandle) {
    let Some(w) = main_window(app) else { return };
    if w.is_visible().unwrap_or(false) && !w.is_minimized().unwrap_or(false) && w.is_focused().unwrap_or(false) {
        save_bounds(app);
        let _ = w.hide();
    } else {
        show_main(app);
    }
}

/// Botón cerrar / Alt+F4: según ajuste, oculta a la bandeja (sigue sonando) o sale.
pub fn request_close(app: &AppHandle) {
    let behavior = app.state::<AppState>().settings.lock().unwrap().close_behavior.clone();
    if behavior == "quit" { quit(app); } else {
        save_bounds(app);
        if let Some(w) = main_window(app) { let _ = w.hide(); }
    }
}

pub fn quit(app: &AppHandle) {
    save_bounds(app);
    app.state::<AppState>().save_recent();
    app.exit(0);
}

pub fn navigate(app: &AppHandle, url: &url::Url) {
    if !is_allowed_nav(url) { return; }
    if let Some(wv) = app.get_webview("yt") { let _ = wv.navigate(url.clone()); }
}

pub fn open_settings(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("settings") { let _ = w.show(); let _ = w.set_focus(); return; }
    let _ = WebviewWindowBuilder::new(app, "settings", WebviewUrl::App("index.html?view=settings".into()))
        .title("Ajustes · YouTube Mini")
        .inner_size(520.0, 680.0)
        .min_inner_size(420.0, 480.0)
        .center()
        .build();
}

pub fn show_quick(app: &AppHandle) {
    let win = match app.get_webview_window("quick") {
        Some(w) => w,
        None => match WebviewWindowBuilder::new(app, "quick", WebviewUrl::App("index.html?view=quick".into()))
            .title("Quick Launch")
            .inner_size(620.0, 420.0)
            .decorations(false)
            .shadow(true)
            .resizable(false)
            .always_on_top(true)
            .skip_taskbar(true)
            .center()
            .visible(false)
            .build()
        {
            Ok(w) => w,
            Err(_) => return,
        },
    };
    let _ = win.center();
    let _ = win.show();
    let _ = win.set_focus();
    let _ = app.emit_to("quick", "quick-open", ());
}

pub fn hide_quick(app: &AppHandle) {
    if let Some(w) = app.get_webview_window("quick") { let _ = w.hide(); }
}
