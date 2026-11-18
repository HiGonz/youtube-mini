# ADR-001 — Stack: Tauri 2 + WebView2 con webview de YouTube y UI local separados

**Estado:** aceptada · 2026-11-18

## Problema
Un reproductor de YouTube independiente, muy ligero, con Mini Mode, hotkeys globales, tray y media keys en Windows,
usando YouTube real (sin reimplementarlo, sin extraer vídeo).

## Opciones evaluadas
Tauri 2 + WebView2, Electron, CEF, WinUI 3 + WebView2 (ver `RESEARCH.md`).

## Decisión
**Tauri 2 (Rust) + WebView2 del sistema, UI en Vue 3 + TypeScript + Vite con CSS propio.**
Una sola ventana con **dos webviews**: `ui` (local) y `yt` (YouTube real). Quick Launch y Ajustes son ventanas locales creadas bajo demanda.

## Motivos
- No empaqueta ni lanza un Chromium propio: reutiliza el WebView2 de Windows (instalador y RAM base menores *— a medir*).
- Rust da acceso directo a Win32/tray/hotkeys sin proceso auxiliar ni backend local.
- Mini Mode = redimensionar la misma ventana y ocultar `ui`: **no hay segundo webview de YouTube**, la reproducción no se reinicia y la sesión es única.
- La separación `ui` / `yt` permite capabilities distintas: el contenido de YouTube solo puede invocar 2 comandos validados.
- WinUI 3 daría mejor integración nativa pero con mucha más complejidad y el mismo coste de WebView2; Electron/CEF aportan un Chromium extra que justamente queremos evitar.

## Riesgos y mitigación
| Riesgo | Mitigación |
|---|---|
| Multiwebview de Tauri es `unstable` | Fijado a Tauri 2.12.x en `Cargo.lock`; la lógica está aislada en `windows.rs` (un solo archivo a adaptar). |
| Google bloquea login en webviews embebidos | No se falsea nada. La app funciona anónima; limitación documentada. |
| YouTube cambia su DOM | Control vía `<video>` + API de `#movie_player` + atajos propios de YouTube; Mini Mode degrada a overlay opaco. Sin MutationObserver ni polling. |
| Media keys/SMTC: comportamiento de WebView2 no verificado | Registro como hotkeys con reporte de conflicto; SMTC propio en ROADMAP. |
| PiP nativo exige gesto en la página | Mini Mode propio como principal; PiP nativo con aviso en página. |
| Fullscreen en webview hijo | Verificar en Windows; plan B: ventana de fullscreen = maximizar y ocultar `ui`. |
| Sin Windows en CI de la sesión | Workflow `build-windows.yml` + checklist manual en `TESTING.md`. |

## Alternativas descartadas
- **Electron / CEF:** Chromium propio por app → más disco y RAM, y duplica lo que Windows ya trae.
- **WinUI 3:** coste de desarrollo desproporcionado para una app pequeña.
- **Navegación propia (reimplementar listas/inicio):** frágil y contra el principio de «YouTube real».
- **Ventana separada para el mini player:** duplicaría webview/sesión/reproducción.
- **Falsear User-Agent / importar cookies:** inseguro; rechazado explícitamente.
- **Tauri 3 alpha:** no estable.

# ADR-002 — Superficie IPC mínima para contenido remoto
- App manifest en `build.rs`: todo comando propio exige permiso explícito.
- `capabilities/ui.json`: solo webviews locales (`ui`, `quick`, `settings`), `local: true`.
- `capabilities/youtube.json`: `remote.urls` = `www.youtube.com`, `music.youtube.com`; únicamente `yt_cmd` (`expand|close|drag`) y `yt_report` (estado de reproducción).
- Rust revalida el origen (`label == "yt"` y host de YouTube), recorta longitudes, y acepta artwork solo desde hosts de imágenes de Google/YouTube.
- Sin plugins ni APIs `core:*` expuestos a la web (la web no puede abrir archivos, ejecutar procesos ni leer ajustes).
- Aun así: un XSS en youtube.com podría llamar a esos 2 comandos (cerrar/expandir/arrastrar ventana, falsear metadatos). Impacto bajo, aceptado.

# ADR-003 — Atajos
Quick Launch usa `Ctrl+Shift+Y` (pedido en §9); «Mostrar/Ocultar» pasa a `Ctrl+Alt+Y` para no colisionar. `Escape` **no** es atajo global (robaría la tecla a todo Windows): solo actúa dentro del Mini Mode.

# ADR-004 — Sin telemetría ni extras
Sin analytics, crash reporter ni auto-updater (ningún servicio local ni conexión salvo YouTube/Google). Sin SQLite: JSON pequeño (`settings.json`, `recent.json`) con escritura atómica.
