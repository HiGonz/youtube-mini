fn main() {
    // Con app manifest, cada comando propio exige un permiso explícito (`allow-<comando>`).
    // Así el contenido remoto de YouTube solo puede invocar los pocos comandos que se le conceden.
    let manifest = tauri_build::AppManifest::new().commands(&[
        // UI local
        "get_settings", "set_settings", "get_state", "get_recent", "navigate", "toggle_mini",
        "set_mini_size", "toggle_pip", "toggle_always_on_top", "window_ctl", "open_settings",
        "quick_submit", "hide_quick", "player_cmd",
        // Contenido remoto (YouTube), superficie mínima
        "yt_cmd", "yt_report",
    ]);
    tauri_build::try_build(tauri_build::Attributes::new().app_manifest(manifest))
        .expect("tauri build failed");
}
