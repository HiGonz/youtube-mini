# Roadmap

## Hecho (código escrito y compilado; verificación en Windows pendiente)
Shell Tauri 2 multiwebview · YouTube real con perfil persistente · barra superior · Mini Mode (3 tamaños, always-on-top, overlay con controles, 16:9) ·
PiP best-effort · Quick Launch (búsqueda/URL/recientes/reanudar/YouTube Music/Ctrl+Enter→Mini) · hotkeys configurables + teclas multimedia ·
tray · single-instance · inicio con Windows · Ajustes (tema Dark/OLED/Sistema, acento, reducir animaciones) · guardado de ventana · lista blanca de navegación · capabilities mínimas · CI de instalador.

## Pendiente
1. **Validar en Windows** todo `TESTING.md`; medir `PERFORMANCE.md`.
2. Fullscreen robusto si falla en webview hijo (maximizar + ocultar `ui`).
3. Popups del login de Google (`on_new_window`) tras observar el flujo real.
4. SMTC propio (carátula, progreso en el flyout de medios) con el crate `windows`.
5. Aspect-ratio nativo en Mini (WM_SIZING) y animación de transición de ventana.
6. Next/Prev de YouTube Music más allá de los selectores actuales (validar).
7. Firma de código del instalador; icono definitivo diseñado.
8. Opcional: ajuste de tema de la propia página de YouTube; pausa automática al suspender.
