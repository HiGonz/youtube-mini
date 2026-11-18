# Arquitectura

```
┌────────────────────── Ventana "main" (sin decoración, shadow) ──────────────────────┐
│ webview "ui"  (local, Vue)   48 px: atrás/adelante/inicio/buscar/mini/PiP/ajustes    │
│ webview "yt"  (REMOTO)       https://www.youtube.com  ← única sesión y reproducción │
│                              + bridge.js (initialization_script) + overlay mini      │
└──────────────────────────────────────────────────────────────────────────────────────┘
 Ventanas locales bajo demanda: "quick" (Quick Launch, siempre encima), "settings".
```

## Mapa de módulos (`src-tauri/src`)
| Módulo | Responsabilidad |
|---|---|
| `lib.rs` | Arranque, plugins, eventos de ventana |
| `windows.rs` | Window Manager: layout de los 2 webviews, Mini Mode, show/hide, quick, ajustes, bounds |
| `commands.rs` | IPC: superficie local (UI) y remota (`yt_cmd`, `yt_report`) |
| `bridge.js` | Puente en YouTube: control del `<video>`, estado por eventos, overlay Mini (Shadow DOM) |
| `hotkeys.rs` | Atajos globales y teclas multimedia, reporte de conflictos |
| `tray.rs` | Icono y menú de bandeja |
| `settings.rs`, `state.rs` | Ajustes, recientes, estado en memoria; JSON atómico |
| `url_guard.rs` | Lista blanca de navegación, interpretación de entrada, tests |
| `src/` (Vue) | `TopBar`, `Settings`, `Quick` + `styles/theme.css` (tokens) |

## Flujos clave
- **Mini Mode:** `set_mini(true)` guarda bounds → `mini=true` → min-size 240×135, tamaño preset, posición (guardada o esquina inferior derecha), always-on-top → `layout()` oculta `ui` y da todo el área a `yt` → `bridge.setMini(true)` aplica CSS (reproductor a pantalla completa de ventana) y muestra overlay. **No hay recarga ni segundo webview.** Aspect 16:9 forzado en `Resized`.
- **Control de reproducción** (hotkeys/tray/UI): Rust → `eval("__ytm.cmd(...)")` con acción validada contra lista fija.
- **Estado:** `bridge.js` escucha `play/pause/ended/loadedmetadata/emptied` (captura en `document`) y `yt-navigate-finish`, debounce 250 ms → `yt_report` → Rust valida → evento `player-state` a `ui`, tooltip del tray, y «recientes/reanudar».
- **Quick Launch:** hotkey → ventana `quick` (creada la 1.ª vez, luego oculta/mostrada) → `quick_submit` → `url_guard::resolve_input` → `navigate` en `yt`.
- **Cierre:** `CloseRequested` se intercepta → bandeja (sigue sonando) o salir según ajuste.

## Modelo de seguridad
Tres zonas: UI local (CSP estricta, capability `ui`), contenido YouTube (capability `youtube-remote`, 2 comandos), nativo (solo accesible vía comandos propios). Navegación del webview `yt` restringida por `url_guard::is_allowed_nav`; lo demás abre el navegador del sistema (solo http/https). Sin logs de cookies/tokens (no se registran).

## Cambiar el nombre
`productName`/`title` en `tauri.conf.json`, `title`/textos en `window title` de `windows.rs`, `identifier` (¡cambia el perfil WebView2!) y tooltip del tray. No hay nombre hardcodeado en la lógica.
