mod commands;
mod hotkeys;
mod settings;
mod state;
mod tray;
mod url_guard;
mod windows;

use state::AppState;
use tauri::{Manager, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Segunda instancia: enfoca la existente en vez de abrir otro proceso.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| windows::show_main(app)))
        .plugin(tauri_plugin_global_shortcut::Builder::new().with_handler(hotkeys::handler).build())
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--minimized"])))
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let dir = app.path().app_config_dir()?;
            app.manage(AppState::new(dir));
            let handle = app.handle().clone();
            let hidden = std::env::args().any(|a| a == "--minimized");
            windows::build_main(&handle, hidden)?;
            tray::build(&handle)?;
            hotkeys::register_all(&handle);
            Ok(())
        })
        .on_window_event(|win, event| {
            if win.label() != "main" { return; }
            let app = win.app_handle();
            match event {
                WindowEvent::Resized(_) => { windows::layout(app); windows::enforce_mini_aspect(app); }
                WindowEvent::CloseRequested { api, .. } => { api.prevent_close(); windows::request_close(app); }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_settings, commands::set_settings, commands::get_state, commands::get_recent,
            commands::navigate, commands::toggle_mini, commands::set_mini_size, commands::toggle_pip,
            commands::toggle_always_on_top, commands::window_ctl, commands::open_settings,
            commands::quick_submit, commands::hide_quick, commands::player_cmd,
            commands::yt_cmd, commands::yt_report,
        ])
        .run(tauri::generate_context!())
        .expect("error al iniciar YouTube Mini");
}
