# Investigación técnica (Fase 0)

Fecha: 2026-11-18. **Alcance honesto:** esta sesión corrió en un contenedor Linux sin Windows ni WebView2. Las
conclusiones se apoyan en (a) búsquedas web hechas en la sesión (fuentes abajo), (b) compilación real contra
`tauri 2.12.6` / `wry 0.57` (también `--target x86_64-pc-windows-gnu`), y (c) conocimiento general de las
plataformas, marcado como **«no verificado»** cuando no pude comprobarlo. No hay benchmarks inventados.

## Fuentes consultadas
- Tauri v2 – webviews múltiples / capabilities remotas: <https://v2.tauri.app/blog/tauri-2-0-0-beta/>, docs.rs `tauri::webview::WebviewBuilder`.
  Hallazgo: crear webviews hijos (`Window::add_child`) requiere la feature `unstable`; las capabilities admiten `remote.urls` y `webviews`.
- Google, política de navegadores seguros (OAuth en webviews embebidos, desde 2021): <https://developers.googleblog.com/2021/06/upcoming-security-changes-to-googles-oauth-2.0-authorization-endpoint.html>.
  Foros (JUCE, Esri) reportan «This browser or app may not be secure» también en WebView2. No encontré un cambio de Google en 2025–26 que lo revierta.
- Microsoft, SMTC / WebView2 overview: <https://learn.microsoft.com/microsoft-edge/webview2/> (sin información concluyente sobre media keys/PiP).
- Verificado por compilación: `cargo info` → Tauri estable 2.12.6 (existe 3.0.0-alpha; **no** se usa), `@tauri-apps/cli` 2.12.5, Vue 3.5, Vite 7 (instalado: 7.3.8).

## Comparativa de tecnologías
Sin mediciones propias; las columnas de RAM/CPU son **cualitativas** hasta ejecutar `docs/PERFORMANCE.md`.

| | Tauri 2 + WebView2 | Electron | CEF (p. ej. CefSharp/Tauri-CEF) | WinUI 3 + WebView2 |
|---|---|---|---|---|
| Runtime de render | WebView2 (Edge) del sistema, compartido | Chromium propio por app | Chromium propio por app | WebView2 del sistema |
| Instalador | pequeño (shell Rust, sin Chromium) | grande (~70–100 MB+ por Chromium propio) *(no medido)* | grande | pequeño-medio (+ Windows App SDK) |
| Procesos | shell + árbol msedgewebview2 | árbol Chromium propio | árbol Chromium propio | shell + árbol msedgewebview2 |
| Compat. YouTube | = Edge (códecs, DRM del sistema) | Chromium sin Widevine/H.264 propietario salvo builds específicas *(no verificado)* | similar a Electron | = Edge |
| Login Google | restringido (ver abajo) | restringido | restringido | restringido |
| Multi-webview en una ventana | `unstable` (usado aquí) | `WebContentsView` | posible | un control por vista |
| Integración Windows (tray, hotkeys, autostart) | plugins oficiales / Rust | nativo en Electron | manual | nativa, la mejor |
| Complejidad / mantenimiento | baja-media | baja | alta | alta (C#/C++ + UI nativa) |
| Riesgo largo plazo | API `unstable` de multiwebview | bajo | medio | bajo |

Observación clave: **el peso dominante lo pone el sitio de YouTube + decodificación de vídeo, no el shell.** Por eso la
variable decisiva es *no cargar un Chromium propio y no duplicar webviews*, no el framework en sí.

## WebView2: qué se puede hacer (y estado de verificación)
| Capacidad | Estado |
|---|---|
| Usar el runtime de Evergreen instalado | Sí: instalador NSIS con `downloadBootstrapper` por si falta (Win10 antiguo). Compilado, no ejecutado. |
| Perfil persistente (cookies, localStorage, sesión) | Sí por defecto: Tauri usa `%LOCALAPPDATA%\<identifier>\EBWebView`. Perfil **independiente** del navegador del usuario (no se reutiliza el de Edge/Opera). |
| Aceleración por hardware | La de Edge por defecto; no se tocan flags de Chromium. |
| Control de navegación / popups | `on_navigation` (usado, lista blanca). `on_new_window`: **no usado** — pendiente de validar flujo de popups del login en Windows. |
| Ejecutar JS | `Webview::eval` + `initialization_script` (usados). |
| Detectar cambios de URL | SPA de YouTube: evento `yt-navigate-finish` + `on_page_load` (usados; el primero **no verificado en vivo**). |
| Fullscreen HTML5 | WebView2 lo soporta a nivel de **ventana del webview hijo**; con webview hijo puede no ocupar toda la ventana nativa → *riesgo conocido, verificar* (ver ROADMAP). |
| Suspensión al ocultar/minimizar | El audio de una ventana oculta normalmente continúa; **no verificado** en WebView2 concreto. |
| Notificaciones | No implementadas (sin valor claro). |

## Google Login
- Google prohíbe OAuth en «embedded user-agents»; WebView2 es un webview embebido, y hay reportes de bloqueo.
  El inicio de sesión de **youtube.com** (no OAuth de terceros) en WebView2 a veces funciona y a veces muestra el bloqueo: **no puedo confirmarlo sin Windows**.
- Lo que **no** se hace: falsear User-Agent, inyectar cookies, extraer sesiones del navegador del sistema. Compromete la cuenta y viola términos.
- Estrategia implementada: abrir el login dentro del webview (lista blanca `*.google.com`). Si Google lo bloquea → **la app sigue siendo útil sin cuenta**
  (YouTube anónimo: búsqueda, reproducción, URLs) y se documenta como limitación (README). No existe una vía oficial para
  «importar» la sesión del navegador del sistema a WebView2.
- Pendiente de decisión del usuario tras probar en Windows: si el login falla, valorar modo «solo anónimo + enlaces».

## Picture-in-Picture
Opciones evaluadas: PiP nativo del navegador (`video.requestPictureInPicture()`), PiP de YouTube, ventana nativa propia, Mini Mode propio.
- Decisión: **Mini Mode propio es el mecanismo principal** (no depende de que YouTube/Chromium permitan PiP): misma ventana/webview redimensionada.
- PiP nativo = *best effort*. Chromium exige activación del usuario **en la página**; un clic en nuestra barra local no cuenta. Implementación: se intenta; si falla con
  `NotAllowedError`, el overlay muestra un aviso en la propia página y el clic ahí sí cuenta. **No verificado en WebView2.**
- Se evita depender de selectores frágiles: se usa el `<video>` y la API pública del `#movie_player`; CSS de Mini Mode degrada a overlay opaco con carátula si cambia el DOM.

## Integración multimedia de Windows
- Teclas multimedia: se registran como atajos globales (`MediaPlayPause`, `MediaTrackNext`, `MediaTrackPrevious`) vía `tauri-plugin-global-shortcut`. Si WebView2/Chromium ya las tiene registradas, el registro falla y se avisa en Ajustes. **No verificado.**
- SMTC (flyout de medios con carátula/barra de progreso): Chromium puede publicar la sesión de medios por su cuenta; **no verificado** en WebView2. Una integración propia (`windows` crate, `SystemMediaTransportControls`) queda en el ROADMAP.
- Tray, autostart, hotkeys, single-instance: plugins oficiales/Tauri nativo — implementados y compilados.
