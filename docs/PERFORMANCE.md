# Rendimiento

**Estado: NO MEDIDO.** Esta sesión no tuvo acceso a Windows, así que **no hay cifras**. Aquí están el diseño orientado a rendimiento, los objetivos y la metodología reproducible.

## Decisiones que ya reducen consumo (verificables en código)
- Sin Chromium propio: WebView2 del sistema. Un solo webview de YouTube; Mini Mode no crea otro.
- Sin polling, sin `setInterval`, sin `MutationObserver`; solo eventos (`bridge.js`). Overlay: `ResizeObserver` solo sobre `<html>`; `timeupdate` solo pinta si el overlay está visible.
- Overlay en reposo: `display:none`; sin animaciones permanentes. Opción «Reducir animaciones».
- Frontend local: 124 KB sin comprimir en total (`dist/`, build real), 3 vistas con carga diferida.
- Quick Launch y Ajustes: webviews creados bajo demanda (Ajustes se destruye al cerrar).
- Release: `opt-level=s`, LTO, `panic=abort`, `strip`. Sin telemetría, updater, SQLite ni procesos auxiliares.

## Objetivos (hipótesis a validar, no promesas)
- Shell `youtube-mini.exe`: decenas de MB de working set, CPU ≈ 0 % en reposo.
- Total (shell + msedgewebview2) reproduciendo audio de fondo: comparable o menor que una pestaña de YouTube en un navegador **sin extensiones**; el objetivo real es menor que Opera GX con tus extensiones.
- Instalador NSIS: pocos MB (sin Chromium).

## Metodología
1. `npm run app:build` (o artefacto de GitHub Actions) → instalar el `.exe` NSIS.
2. Misma red, mismo vídeo (p. ej. un directo de lofi y un vídeo 1080p), brillo/energía fijos, resto de apps cerradas, 3 repeticiones por escenario.
3. Con la app abierta: `.\scripts\measure.ps1 -Label <escenario> -Minutes <n>` (separa **shell** y **webview**).
4. Escenarios: arranque (cold/warm), reposo en Home, reproduciendo (normal), Mini Mode, oculta en bandeja, 30 min, 2 h.
5. Tiempo de arranque: `Measure-Command { Start-Process youtube-mini -Wait:$false }` no basta; usar `Process.StartTime` → primera ventana visible (`MainWindowHandle`) y, para «primera reproducción», cronómetro manual o Playwright/CDP (`--remote-debugging-port` vía `WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS`).
6. GPU: Administrador de tareas → Detalles → columna GPU, o `typeperf "\GPU Engine(*)\Utilization Percentage"`.
7. Comparación base: la misma reproducción en Opera GX (con tus extensiones) y en Edge limpio.
8. Reportar mediana de 3 ejecuciones y adjuntar los CSV.

## Resultados
| Escenario | Shell RAM | WebView RAM | CPU | GPU | Fecha |
|---|---|---|---|---|---|
| (pendiente de medir en Windows) | | | | | |
