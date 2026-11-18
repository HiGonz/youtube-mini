use crate::{commands, windows};
use tauri::{
    menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem},
    tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent},
    AppHandle,
};

pub fn build(app: &AppHandle) -> tauri::Result<()> {
    let open = MenuItem::with_id(app, "open", "Abrir", true, None::<&str>)?;
    let pp = MenuItem::with_id(app, "playpause", "Reproducir / Pausar", true, None::<&str>)?;
    let next = MenuItem::with_id(app, "next", "Siguiente", true, None::<&str>)?;
    let prev = MenuItem::with_id(app, "prev", "Anterior", true, None::<&str>)?;
    let mini = MenuItem::with_id(app, "mini", "Activar / desactivar Mini Mode", true, None::<&str>)?;
    let top = CheckMenuItem::with_id(app, "top", "Siempre encima", true, false, None::<&str>)?;
    let settings = MenuItem::with_id(app, "settings", "Ajustes", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Salir", true, None::<&str>)?;
    let sep = || PredefinedMenuItem::separator(app);
    let menu = Menu::with_items(app, &[&open, &sep()?, &pp, &next, &prev, &sep()?, &mini, &top, &sep()?, &settings, &quit])?;

    TrayIconBuilder::with_id("tray")
        .icon(app.default_window_icon().cloned().expect("icono de app"))
        .tooltip("YouTube Mini")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_menu_event(move |app, ev| match ev.id().as_ref() {
            "open" => windows::show_main(app),
            "playpause" => { let _ = commands::run_player_action(app, "playpause"); }
            "next" => { let _ = commands::run_player_action(app, "next"); }
            "prev" => { let _ = commands::run_player_action(app, "prev"); }
            "mini" => windows::toggle_mini(app),
            "top" => { let _ = windows::toggle_always_on_top(app); }
            "settings" => windows::open_settings(app),
            "quit" => windows::quit(app),
            _ => {}
        })
        .on_tray_icon_event(|tray, ev| {
            if let TrayIconEvent::Click { button: MouseButton::Left, button_state: MouseButtonState::Up, .. } = ev {
                windows::toggle_visibility(tray.app_handle());
            }
        })
        .build(app)?;
    Ok(())
}
