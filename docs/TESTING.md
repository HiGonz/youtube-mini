# Pruebas

## Ejecutadas en esta sesión (Linux, sin Windows)
| Prueba | Resultado |
|---|---|
| `cargo test` — `settings`, `url_guard` (6 tests: lista blanca de navegación, resolución de URL/youtu.be/búsqueda, hosts de artwork, saneado de ajustes, hotkeys únicos) | ✅ 6/6 |
| `cargo check` (Linux) y `cargo check --target x86_64-pc-windows-gnu` (valida capabilities/permisos en build y tipos para Windows) | ✅ |
| `npm run build` (`vue-tsc --noEmit` + Vite) | ✅ |
| `bridge.js` en Chromium real (Playwright) con `<video>` y `invoke` simulado: play/pausa, volumen, `yt_report` con título/artista, `setMini`, Esc→`expand`, overlay a 420×236 y 320×180, auto-ocultar | ✅ (encontró y corregí un bug: `documentElement` nulo en `document_start`) |
| Render de `TopBar`, `Settings`, `Quick` con backend simulado (capturas revisadas) | ✅ |

## NO ejecutadas (requieren Windows + WebView2) — checklist manual
Marcar al probar. Cada caso fallido → anotarlo en `PROGRESS.md`.

**YouTube:** [ ] Home carga [ ] buscar [ ] reproducir/pausar [ ] cambiar vídeo [ ] playlist [ ] login Google (o limitación) [ ] sesión persiste tras reiniciar [ ] historial [ ] atrás/adelante [ ] **fullscreen HTML5 (riesgo del webview hijo)** [ ] YouTube Music.
**Mini Mode:** [ ] activar sin reiniciar el vídeo [ ] volver [ ] resize 16:9 [ ] always-on-top [ ] arrastrar por el título [ ] controles/auto-ocultar [ ] Esc [ ] cerrar [ ] Mini en YouTube Music (carátula).
**PiP:** [ ] botón → ¿entra directo o aparece el aviso? [ ] salir.
**Windows:** [ ] Ctrl+Shift+Y/M/Space/←/→/↑/↓/End [ ] teclas multimedia (¿conflicto reportado en Ajustes?) [ ] tray (menú, clic izq.) [ ] minimizar/restaurar [ ] cerrar→tray sigue sonando [ ] salir [ ] iniciar con Windows (oculto) [ ] 2 monitores [ ] DPI 100/150/200 % [ ] cambio de resolución [ ] 2.ª instancia enfoca la 1.ª.
**Robustez:** [ ] sin Internet [ ] red lenta [ ] vídeo no disponible [ ] sesión caducada [ ] reinicio [ ] suspender/reanudar Windows [ ] WebView2 ausente (instalador).
**Rendimiento:** ver `PERFORMANCE.md`.

## Problemas conocidos/anticipados (sin confirmar)
- Fullscreen puede quedar limitado al área del webview hijo.
- Aspect lock en Mini por `Resized` puede parpadear; solución nativa (WM_SIZING) en ROADMAP.
- Selectores de CSS del Mini (`.ytp-*`) pueden cambiar; degrada a overlay opaco.
