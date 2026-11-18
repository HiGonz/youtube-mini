// Puente inyectado en el webview de YouTube (solo frame principal).
// - Sin polling ni MutationObserver: solo listeners de eventos del <video> y de YouTube.
// - Controla el <video> y la API pública del reproductor; para siguiente/anterior intenta
//   botones conocidos y, si fallan, los atajos de teclado propios de YouTube (Shift+N / Shift+P).
// - Overlay del Mini Mode en Shadow DOM (aislado del CSS de YouTube).
// - Única salida hacia nativo: yt_cmd(action) e yt_report(state), ambos validados en Rust.
(() => {
  if (window.__ytm || window.top !== window) return;
  const invoke = (cmd, args) => {
    try { return window.__TAURI_INTERNALS__.invoke(cmd, args).catch(() => {}); } catch (_) {}
  };
  const isMusic = () => location.hostname === 'music.youtube.com';
  const video = () => document.querySelector('video.html5-main-video') || document.querySelector('video');
  const player = () => document.getElementById('movie_player');
  const clamp = (v, a, b) => Math.min(b, Math.max(a, v));

  function clickFirst(sels) {
    for (const s of sels) {
      const e = document.querySelector(s);
      if (e && !e.hidden && !e.hasAttribute('disabled') && e.getClientRects().length) { e.click(); return true; }
    }
    return false;
  }
  function key(k, code, shift) {
    const t = player() || document.body;
    t.dispatchEvent(new KeyboardEvent('keydown', { key: k, code, shiftKey: !!shift, bubbles: true, cancelable: true }));
  }
  function getVolume() { const p = player(); if (p && p.getVolume) return p.getVolume() / 100; const v = video(); return v ? v.volume : 1; }
  function setVolume(x) {
    x = clamp(x, 0, 1);
    const p = player(), v = video();
    if (p && p.setVolume) { p.setVolume(Math.round(x * 100)); if (x > 0 && p.isMuted && p.isMuted()) p.unMute(); }
    else if (v) { v.volume = x; v.muted = x === 0; }
  }

  async function pip() {
    const v = video();
    if (!v) return;
    try {
      if (document.pictureInPictureElement) await document.exitPictureInPicture();
      else await v.requestPictureInPicture();
    } catch (e) {
      // Chromium exige activación del usuario dentro de ESTA página: pedimos un clic.
      if (e && e.name === 'NotAllowedError') ui.toast('Pulsa aquí para activar Picture-in-Picture', pip);
    }
  }

  const actions = {
    playpause() { const v = video(); if (v) (v.paused ? v.play() : v.pause()); },
    next() { if (!clickFirst(['.ytp-next-button', 'ytmusic-player-bar .next-button', 'tp-yt-paper-icon-button.next-button'])) key('N', 'KeyN', true); },
    prev() {
      const v = video();
      if (v && v.currentTime > 4) { v.currentTime = 0; return; }
      if (!clickFirst(['.ytp-prev-button', 'ytmusic-player-bar .previous-button', 'tp-yt-paper-icon-button.previous-button'])) key('P', 'KeyP', true);
    },
    volume_up() { setVolume(getVolume() + 0.1); },
    volume_down() { setVolume(getVolume() - 0.1); },
    mute() { const p = player(), v = video(); if (p && p.isMuted) (p.isMuted() ? p.unMute() : p.mute()); else if (v) v.muted = !v.muted; },
    seek(d) { const v = video(); if (v) v.currentTime = clamp(v.currentTime + d, 0, v.duration || 1e9); },
    seek_to(f) { const v = video(); if (v && v.duration) v.currentTime = clamp(f, 0, 1) * v.duration; },
    set_volume(f) { setVolume(f); },
    pip,
  };

  // ---- Estado hacia nativo (eventos, sin polling) ----
  let timer = 0;
  function snapshot() {
    const v = video();
    const md = navigator.mediaSession && navigator.mediaSession.metadata;
    const art = md && md.artwork && md.artwork.length ? md.artwork[md.artwork.length - 1].src : '';
    return {
      playing: !!v && !v.paused && !v.ended,
      title: (md && md.title) || document.title.replace(/ - YouTube( Music)?$/, ''),
      artist: (md && md.artist) || '',
      artwork: art || '',
      url: location.href,
      time: v ? v.currentTime || 0 : 0,
      duration: v && isFinite(v.duration) ? v.duration : 0,
      music: isMusic(),
      has_video: !!v,
    };
  }
  function report() {
    clearTimeout(timer);
    timer = setTimeout(() => { const s = snapshot(); ui.sync(s); invoke('yt_report', { state: s }); }, 250);
  }
  for (const ev of ['play', 'pause', 'ended', 'loadedmetadata', 'emptied']) document.addEventListener(ev, report, true);
  document.addEventListener('yt-navigate-finish', report);
  document.addEventListener('timeupdate', () => ui.tick(), true);

  // ---- Overlay del Mini Mode ----
  const ui = {
    mini: false, host: null, root: null, opts: { autoHide: true, reduceMotion: false }, hideT: 0, last: null,
    build() {
      if (this.host || !document.documentElement) return;
      const host = document.createElement('div');
      host.id = 'ytm-root';
      const root = host.attachShadow({ mode: 'closed' });
      root.innerHTML = `
<style>
  :host{all:initial}
  *{box-sizing:border-box;margin:0}
  .wrap{position:fixed;inset:0;z-index:2147483647;display:none;color:#f4f4f5;font:12px/1.3 "Segoe UI Variable Text","Segoe UI",system-ui,sans-serif;user-select:none;-webkit-user-select:none;overflow:hidden}
  :host(.on) .wrap{display:block}
  .bg{position:absolute;inset:0;background:#0b0b0d center/cover no-repeat;display:none}
  .opaque .bg{display:block}
  .bg::after{content:"";position:absolute;inset:0;background:rgba(0,0,0,.55)}
  .ui{position:absolute;inset:0;opacity:0;transition:opacity .16s ease;background:linear-gradient(#000a,#0000 38%,#0000 55%,#000c)}
  .rm .ui{transition:none}
  .active .ui,.pinned .ui{opacity:1}
  .wrap:not(.active):not(.pinned){cursor:none}
  .top{position:absolute;left:0;right:0;top:0;display:flex;align-items:center;gap:6px;padding:6px 8px}
  .title{flex:1;min-width:0;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;font-weight:600;cursor:grab}
  .sub{opacity:.65;font-weight:400}
  button{all:unset;display:grid;place-items:center;width:28px;height:28px;border-radius:8px;cursor:pointer;color:inherit}
  button:hover{background:#ffffff26}button:active{background:#ffffff3d}
  button:focus-visible{outline:2px solid var(--a,#ff3b30)}
  svg{width:16px;height:16px;fill:currentColor}
  .mid{position:absolute;inset:34px 0 44px;display:flex;align-items:center;justify-content:center;gap:14px}
  .mid button{width:36px;height:36px;border-radius:50%}.mid .pp{width:46px;height:46px;background:#ffffff26}
  .mid svg{width:20px;height:20px}.mid .pp svg{width:24px;height:24px}
  .bot{position:absolute;left:0;right:0;bottom:0;padding:0 10px 8px;display:flex;flex-direction:column;gap:4px}
  .bar{height:14px;display:flex;align-items:center;cursor:pointer}
  .rail{flex:1;height:3px;border-radius:2px;background:#ffffff40;overflow:hidden}.bar:hover .rail{height:5px}
  .fill{height:100%;width:0;background:var(--a,#ff3b30)}
  .row{display:flex;align-items:center;gap:8px}.time{font-variant-numeric:tabular-nums;opacity:.8;flex:1}
  input[type=range]{width:64px;accent-color:var(--a,#ff3b30)}
  .toast{position:absolute;left:50%;bottom:54px;transform:translateX(-50%);background:#18181b;border:1px solid #ffffff22;border-radius:10px;padding:8px 12px;cursor:pointer;display:none;z-index:2}
  .toast.show{display:block}
  .small .mid{gap:8px}.small .mid button{width:30px;height:30px}.small .mid .pp{width:38px;height:38px}.small .time,.small input[type=range]{display:none}
</style>
<div class="wrap" id="w">
  <div class="bg" id="bg"></div>
  <div class="ui" id="ui">
    <div class="top"><div class="title" id="title"></div>
      <button id="expand" title="Volver a la ventana normal"><svg viewBox="0 0 16 16"><path d="M2 2h5v1.5H4.6l2.9 2.9-1.1 1.1-2.9-2.9V7H2V2zm12 12H9v-1.5h2.4L8.5 9.6l1.1-1.1 2.9 2.9V9H14v5z"/></svg></button>
      <button id="close" title="Cerrar"><svg viewBox="0 0 16 16"><path d="M3.3 2.2 8 6.9l4.7-4.7 1.1 1.1L9.1 8l4.7 4.7-1.1 1.1L8 9.1l-4.7 4.7-1.1-1.1L6.9 8 2.2 3.3z"/></svg></button></div>
    <div class="mid">
      <button id="prev" title="Anterior"><svg viewBox="0 0 16 16"><path d="M3 3h1.6v10H3zM13 3v10L5.5 8z"/></svg></button>
      <button id="pp" class="pp" title="Reproducir/Pausar"></button>
      <button id="next" title="Siguiente"><svg viewBox="0 0 16 16"><path d="M11.4 3H13v10h-1.6zM3 3l7.5 5L3 13z"/></svg></button>
    </div>
    <div class="bot"><div class="bar" id="bar"><div class="rail"><div class="fill" id="fill"></div></div></div>
      <div class="row"><span class="time" id="time">0:00</span>
        <button id="pip" title="Picture-in-Picture"><svg viewBox="0 0 16 16"><path d="M2 3h12v10H2zM3.5 4.5v7h9v-7zM8 7h4v3.5H8z"/></svg></button>
        <input id="vol" type="range" min="0" max="100" aria-label="Volumen"></div></div>
  </div>
  <div class="toast" id="toast"></div>
</div>`;
      this.root = root; this.host = host;
      const $ = (id) => root.getElementById(id);
      this.$ = $;
      const PLAY = '<svg viewBox="0 0 16 16"><path d="M4 2.5v11L13 8z"/></svg>', PAUSE = '<svg viewBox="0 0 16 16"><path d="M4 3h3v10H4zm5 0h3v10H9z"/></svg>';
      this.icons = { PLAY, PAUSE };
      $('pp').innerHTML = PLAY;
      $('pp').onclick = () => actions.playpause();
      $('prev').onclick = () => actions.prev();
      $('next').onclick = () => actions.next();
      $('pip').onclick = () => pip();
      $('expand').onclick = () => invoke('yt_cmd', { action: 'expand' });
      $('close').onclick = () => invoke('yt_cmd', { action: 'close' });
      $('vol').oninput = (e) => actions.set_volume(e.target.value / 100);
      $('bar').onclick = (e) => { const r = $('bar').getBoundingClientRect(); actions.seek_to((e.clientX - r.left) / r.width); };
      $('title').onmousedown = (e) => { if (e.button === 0) invoke('yt_cmd', { action: 'drag' }); };
      $('title').ondblclick = () => invoke('yt_cmd', { action: 'expand' });
      $('toast').onclick = () => { $('toast').classList.remove('show'); if (this.toastFn) this.toastFn(); };
      $('w').addEventListener('mousemove', () => this.wake());
      $('w').addEventListener('mouseleave', () => this.sleep(300));
      $('w').addEventListener('click', (e) => { if (e.target === $('ui') || e.target === $('bg')) actions.playpause(); });
      window.addEventListener('keydown', (e) => { if (this.mini && e.key === 'Escape') invoke('yt_cmd', { action: 'expand' }); }, true);
      new ResizeObserver(() => this.layout()).observe(document.documentElement);
      document.documentElement.appendChild(host);
    },
    layout() { if (this.root) this.$('w').classList.toggle('small', innerWidth < 380); },
    wake() {
      const w = this.$('w'); w.classList.add('active'); clearTimeout(this.hideT);
      if (this.opts.autoHide) this.hideT = setTimeout(() => this.$('w').classList.remove('active'), 2500);
    },
    sleep(ms) { clearTimeout(this.hideT); this.hideT = setTimeout(() => this.$('w').classList.remove('active'), ms); },
    toast(msg, fn) { if (!this.root) return; const t = this.$('toast'); t.textContent = msg; this.toastFn = fn; t.classList.add('show'); setTimeout(() => t.classList.remove('show'), 6000); },
    fmt(s) { s = Math.max(0, Math.floor(s || 0)); const m = Math.floor(s / 60); return m + ':' + String(s % 60).padStart(2, '0'); },
    sync(s) {
      this.last = s;
      if (!this.root) return;
      const $ = this.$;
      $('pp').innerHTML = s.playing ? this.icons.PAUSE : this.icons.PLAY;
      $('title').innerHTML = '';
      const t = document.createElement('span'); t.textContent = s.title; $('title').appendChild(t);
      if (s.artist) { const a = document.createElement('span'); a.className = 'sub'; a.textContent = '  ·  ' + s.artist; $('title').appendChild(a); }
      $('vol').value = Math.round(getVolume() * 100);
      this.opaqueMode(s);
      this.tick();
    },
    opaqueMode(s) {
      // Sin <video> visible (YouTube Music en modo canción, Shorts...) mostramos carátula opaca.
      const visible = !isMusic() && !!player() && location.pathname === '/watch';
      this.$('w').classList.toggle('opaque', !visible);
      const bg = this.$('bg');
      try { const u = new URL(s.artwork); bg.style.backgroundImage = /(ytimg|ggpht|googleusercontent)\.com$/.test(u.hostname) ? `url("${u.href}")` : 'none'; } catch (_) { bg.style.backgroundImage = 'none'; }
    },
    tick() {
      if (!this.mini || !this.root || !this.$('w').classList.contains('active')) return;
      const v = video(); if (!v) return;
      this.$('fill').style.width = (v.duration ? (v.currentTime / v.duration) * 100 : 0) + '%';
      this.$('time').textContent = this.fmt(v.currentTime) + ' / ' + this.fmt(v.duration);
    },
    setMini(on, opts) {
      this.mini = !!on;
      this.build();
      if (opts) { this.opts = opts; document.documentElement.style.setProperty('--a', opts.accent || '#ff3b30'); }
      document.documentElement.classList.toggle('ytm-mini', this.mini);
      if (!this.root) return;
      this.$('w').classList.toggle('rm', !!this.opts.reduceMotion);
      this.$('w').parentNode.host.classList.toggle('on', this.mini);
      this.$('w').classList.toggle('pinned', this.mini && !this.opts.autoHide);
      if (this.mini) { this.sync(snapshot()); this.wake(); }
    },
  };

  // CSS de página para Mini Mode: el reproductor ocupa toda la ventana; el resto se oculta.
  // Si YouTube cambia estos selectores, el overlay opaco sigue funcionando (degradación).
  const css = document.createElement('style');
  css.textContent = `
    html.ytm-mini, html.ytm-mini body { overflow:hidden !important; background:#000 !important }
    html.ytm-mini body { visibility:hidden !important }
    html.ytm-mini #movie_player, html.ytm-mini #movie_player * { visibility:visible }
    html.ytm-mini #movie_player { position:fixed !important; inset:0 !important; width:100vw !important; height:100vh !important; z-index:2147483000 !important; background:#000 }
    html.ytm-mini .html5-video-container, html.ytm-mini video.html5-main-video { width:100% !important; height:100% !important; left:0 !important; top:0 !important }
    html.ytm-mini video.html5-main-video { object-fit:contain !important }
    html.ytm-mini .ytp-chrome-top, html.ytm-mini .ytp-chrome-bottom, html.ytm-mini .ytp-gradient-top, html.ytm-mini .ytp-gradient-bottom,
    html.ytm-mini .ytp-ce-element, html.ytm-mini .ytp-pause-overlay, html.ytm-mini .ytp-endscreen-content, html.ytm-mini .ytp-paid-content-overlay { display:none !important }
    html.ytm-mini #ytm-root { visibility:visible !important }`;

  window.__ytm = {
    cmd(name, arg) { try { const f = actions[name]; if (f) f(arg); } catch (_) {} },
    setMini: (on, opts) => ui.setMini(on, opts),
    report,
  };
  // A document_start <html> puede no existir aún: se monta CSS y overlay cuando el DOM está disponible.
  const mount = () => { (document.head || document.documentElement).appendChild(css); ui.build(); report(); };
  if (document.documentElement && document.readyState !== 'loading') mount();
  else document.addEventListener('DOMContentLoaded', mount, { once: true });
})();
