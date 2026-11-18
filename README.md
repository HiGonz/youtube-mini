# YouTube Mini

Reproductor de escritorio para Windows dedicado a YouTube: **YouTube real** en una ventana ligera, con Mini Mode flotante,
atajos globales, bandeja del sistema y búsqueda rápida. Sin navegador, pestañas, extensiones, telemetría ni servicios locales.

> **Estado: código completo y compilado, NO probado en Windows.** Se desarrolló en un contenedor Linux. Compila (Linux y target Windows),
> los tests unitarios pasan y el puente JS y la UI se probaron en Chromium, pero la app empaquetada nunca se ejecutó sobre WebView2.
> Mira `PROGRESS.md` y `docs/TESTING.md` antes de fiarte de cualquier función.

## Características
- YouTube / YouTube Music reales, perfil WebView2 **propio y persistente** (no toca tu navegador).
- Barra superior: atrás, adelante, inicio, Music, buscador (texto o URL), Mini, PiP, siempre-encima, ajustes, controles de ventana.
- **Mini Mode** 320×180 / 420×236 / 560×315: misma ventana y mismo vídeo (sin reiniciar), 16:9, arrastrar por el título, controles que se ocultan, Esc para volver.
- **PiP**: nativo del navegador en modo *best effort* (puede pedir un clic extra en la página); el Mini Mode es el mecanismo fiable.
- **Quick Launch** `Ctrl+Shift+Y`: buscar, pegar URL, recientes, reanudar, YouTube Music; `Ctrl+Enter` abre en Mini.
- Atajos globales configurables y teclas multimedia; bandeja con menú; cerrar → bandeja (sigue sonando) o salir; inicio con Windows (oculto).
- Temas Dark / OLED / Sistema + acento. Sin analytics.

| Atajo | Acción |
|---|---|
| Ctrl+Shift+Y | Quick Launch |
| Ctrl+Alt+Y | Mostrar/ocultar app |
| Ctrl+Shift+M | Mini Mode |
| Ctrl+Shift+Space / ← / → | Play-Pausa / Anterior / Siguiente |
| Ctrl+Shift+↑ / ↓ / End | Volumen + / − / Silenciar |
| Esc (en Mini Mode) | Volver a ventana normal |

## Stack
Tauri 2.12 (Rust) · WebView2 · Vue 3 + TypeScript + Vite · CSS propio. Dependencias Rust: 4 plugins oficiales (global-shortcut, single-instance, autostart, opener) + serde + url.

## Desarrollo (en Windows 10/11)
Requisitos: Node 22+, Rust estable (MSVC), WebView2 Runtime (viene en Win11 y Win10 actualizado).
```bash
npm install
npm run app:dev      # desarrollo
npm run app:build    # instalador NSIS en src-tauri/target/release/bundle/nsis/
```
Sin Windows a mano: ejecuta el workflow **build-windows** (GitHub Actions) y descarga el instalador como artefacto.
Tests: `cargo test --manifest-path src-tauri/Cargo.toml`.

## Limitaciones conocidas
- **Login de Google:** Google restringe el inicio de sesión en webviews embebidos. Puede funcionar o mostrar «navegador no seguro». No se falsea el User-Agent ni se importan cookies. Si falla, la app sigue funcionando sin cuenta.
- Fullscreen HTML5 y teclas multimedia/SMTC: no verificados en WebView2 (ver `docs/TESTING.md`).
- Se omitieron a propósito ajustes sin propósito claro (notificaciones, «Mini Mode predeterminado», «preferencias de fullscreen»): la reproducción en segundo plano es el comportamiento por defecto al cerrar a la bandeja.
- Multiwebview de Tauri es `unstable`: fijar versión de Tauri en `Cargo.lock`.
- Esta app no descarga ni extrae vídeo; usa la web de YouTube tal cual. Respeta los términos de YouTube/Google.

## Documentación
`docs/RESEARCH.md` · `docs/ADR.md` · `docs/ARCHITECTURE.md` · `docs/PERFORMANCE.md` · `docs/TESTING.md` · `docs/ROADMAP.md` · `PROGRESS.md`
