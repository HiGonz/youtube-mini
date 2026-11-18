# PROGRESS — YouTube Mini  (actualizado 2026-11-18)

## Fase actual
Fases 0–6 **implementadas en código y verificadas solo por compilación/tests/Chromium**. Fase 7 (testing en Windows) y Fase 8 (instalador) **pendientes: requieren Windows**.

## Completado
- F0 Research + ADR (`docs/`): fuentes y límites de verificación explícitos.
- F1 Foundation: ventana `main` con webviews `ui` + `yt`, perfil persistente, navegación a YouTube, barra superior.
- F2 Integración: tray, hotkeys globales + teclas multimedia, mostrar/ocultar, single-instance, autostart, bounds persistentes.
- F3 Mini Mode: 3 tamaños, always-on-top, overlay (Shadow DOM), 16:9, sin reinicio de vídeo.
- F4 PiP: best-effort con fallback in-page.
- F5 Quick Launch: búsqueda/URL/recientes/reanudar/Music/Mini.
- F6 Polish: tokens de tema, Dark/OLED/Sistema, Ajustes, iconos propios, estados vacíos.
- Tooling: `scripts/measure.ps1`, `.github/workflows/build-windows.yml`.

## Verificado (real)
`cargo test` 6/6 · `cargo check` Linux y `x86_64-pc-windows-gnu` · `npm run build` (vue-tsc + vite) · `bridge.js` y vistas Vue en Chromium (capturas revisadas).

## Pendiente
1. En Windows: `npm install && npm run app:build`, instalar, recorrer `docs/TESTING.md`.
2. Medir con `scripts/measure.ps1` y rellenar `docs/PERFORMANCE.md` (hoy **no hay cifras**).
3. Resolver lo que falle de la lista «riesgos» abajo.

## Decisiones
Tauri 2 + WebView2 (ADR-001) · IPC mínimo para contenido remoto (ADR-002) · Quick Launch = Ctrl+Shift+Y, toggle app = Ctrl+Alt+Y (ADR-003) · sin telemetría/updater/SQLite (ADR-004) · Mini Mode propio como PiP principal.

## Problemas encontrados
- Bug corregido: `bridge.js` rompía si `document.documentElement` era null en `document_start` (hallado por prueba en Chromium).
- Sin Windows → no se pudo generar instalador (`.exe`) ni medir recursos; no se declara cumplido ningún criterio de aceptación que dependa de ello.

## Riesgos sin verificar (Windows)
Login Google en WebView2 · fullscreen en webview hijo · conflicto de teclas multimedia/SMTC con WebView2 · gesto de usuario para PiP · `yt-navigate-finish`/selectores `.ytp-*` en vivo · audio con ventana oculta · aspect lock con parpadeo.

## Criterios de aceptación (§21)
Sin verificar en Windows: abre, instalador, YouTube carga, reproduce, login, sesión, minimizar, audio, Mini/always-on-top/volver, tray, hotkeys, medición. **Cumplidos por construcción/revisión de código:** no telemetría, sin procesos locales, un solo webview de YouTube, documentación, build de frontend, compilación de Rust.

## Siguiente paso
Ejecutar el workflow de GitHub Actions (o build local en Windows), instalar y marcar la checklist de `docs/TESTING.md`; reportar fallos para iterar.
