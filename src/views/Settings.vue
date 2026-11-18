<script setup lang="ts">
import { onMounted, reactive, ref } from "vue";
import { api, type Settings } from "../lib/api";
import { applyTheme } from "../lib/theme";
import { toAccelerator } from "../lib/hotkey";

const s = ref<Settings | null>(null);
const errors = ref<string[]>([]);
const capturing = ref<string | null>(null);
const saved = reactive({ at: 0 });

const HOTKEYS: [string, string][] = [
  ["quick_launch", "Quick Launch (búsqueda rápida)"], ["toggle_app", "Mostrar / ocultar aplicación"], ["toggle_mini", "Activar Mini Mode"],
  ["play_pause", "Reproducir / Pausar"], ["next", "Siguiente"], ["prev", "Anterior"],
  ["volume_up", "Volumen +"], ["volume_down", "Volumen −"], ["mute", "Silenciar"],
];
const MEDIA: [string, string, string][] = [
  ["media_play_pause", "Tecla multimedia Play/Pausa", "MediaPlayPause"],
  ["media_next", "Tecla multimedia Siguiente", "MediaTrackNext"],
  ["media_prev", "Tecla multimedia Anterior", "MediaTrackPrevious"],
];

onMounted(async () => { const snap = await api.state(); s.value = snap.settings; errors.value = snap.hotkey_errors; });

async function save() {
  if (!s.value) return;
  const snap = await api.setSettings(JSON.parse(JSON.stringify(s.value)));
  s.value = snap.settings; errors.value = snap.hotkey_errors;
  applyTheme(snap.settings); saved.at = Date.now();
}
function capture(e: KeyboardEvent, action: string) {
  e.preventDefault();
  if (e.key === "Escape") { capturing.value = null; return; }
  if (e.key === "Backspace" && !e.ctrlKey && !e.altKey) { s.value!.hotkeys[action] = ""; capturing.value = null; save(); return; }
  const acc = toAccelerator(e);
  if (acc) { s.value!.hotkeys[action] = acc; capturing.value = null; save(); }
}
const toggleMedia = (a: string, def: string, on: boolean) => { s.value!.hotkeys[a] = on ? def : ""; save(); };
</script>

<template>
  <main v-if="s" class="page">
    <h1>Ajustes</h1>

    <section>
      <h2>General</h2>
      <label class="row"><span>Iniciar con Windows <small>Arranca oculto en la bandeja</small></span><input type="checkbox" v-model="s.start_with_windows" @change="save" /></label>
      <label class="row"><span>Al cerrar la ventana</span>
        <select v-model="s.close_behavior" @change="save"><option value="tray">Seguir en la bandeja (continúa sonando)</option><option value="quit">Salir completamente</option></select></label>
      <label class="row"><span>Recordar tamaño y posición</span><input type="checkbox" v-model="s.remember_window" @change="save" /></label>
    </section>

    <section>
      <h2>Reproductor</h2>
      <label class="row"><span>Tamaño del Mini Mode</span>
        <select v-model="s.mini_size" @change="save"><option value="mini">Mini · 320×180</option><option value="compact">Compacto · 420×236</option><option value="standard">Estándar · 560×315</option></select></label>
      <label class="row"><span>Mini Mode siempre encima</span><input type="checkbox" v-model="s.mini_always_on_top" @change="save" /></label>
      <label class="row"><span>Ocultar controles automáticamente</span><input type="checkbox" v-model="s.auto_hide_controls" @change="save" /></label>
    </section>

    <section>
      <h2>Apariencia</h2>
      <label class="row"><span>Tema</span>
        <select v-model="s.theme" @change="save"><option value="dark">Oscuro</option><option value="oled">OLED negro</option><option value="system">Sistema</option></select></label>
      <label class="row"><span>Color de acento</span><input type="color" v-model="s.accent" @change="save" /></label>
    </section>

    <section>
      <h2>Rendimiento</h2>
      <label class="row"><span>Reducir animaciones <small>También aplica al overlay del Mini Mode</small></span><input type="checkbox" v-model="s.reduce_motion" @change="save" /></label>
    </section>

    <section>
      <h2>Atajos globales</h2>
      <div v-for="[a, label] in HOTKEYS" :key="a" class="row">
        <span>{{ label }}</span>
        <button class="btn key" :class="{ rec: capturing === a }" @click="capturing = a" @keydown="capturing === a && capture($event, a)" @blur="capturing = null">
          {{ capturing === a ? "Pulsa la combinación…" : s.hotkeys[a] || "Sin asignar" }}</button>
      </div>
      <label v-for="[a, label, def] in MEDIA" :key="a" class="row"><span>{{ label }} <small>Funciona aunque la app no tenga el foco</small></span>
        <input type="checkbox" :checked="!!s.hotkeys[a]" @change="toggleMedia(a, def, ($event.target as HTMLInputElement).checked)" /></label>
      <p class="hint">Clic en un atajo y pulsa la nueva combinación (Esc cancela, Retroceso lo desactiva). Dentro del Mini Mode, Esc vuelve a la ventana normal.</p>
      <ul v-if="errors.length" class="errors"><li v-for="e in errors" :key="e">{{ e }}</li></ul>
    </section>
  </main>
</template>

<style scoped>
.page { height: 100%; overflow: auto; padding: 20px 24px 32px; user-select: none; }
h1 { font-size: 20px; font-weight: 650; margin-bottom: 16px; }
h2 { font-size: 11px; letter-spacing: .08em; text-transform: uppercase; color: var(--muted); margin: 22px 0 6px; font-weight: 600; }
section { border-bottom: 1px solid var(--border); padding-bottom: 6px; }
.row { display: flex; align-items: center; justify-content: space-between; gap: 16px; min-height: 40px; }
.row > span { display: flex; flex-direction: column; }
small { color: var(--muted); font-size: 11.5px; }
select, .key { background: var(--surface-2); color: var(--text); border: 1px solid var(--border); border-radius: var(--r-sm); padding: 5px 8px; max-width: 250px; }
.key { min-width: 150px; font-variant-numeric: tabular-nums; }
.key.rec { border-color: var(--accent); }
input[type="checkbox"] { width: 18px; height: 18px; accent-color: var(--accent); }
input[type="color"] { width: 36px; height: 26px; border: 1px solid var(--border); border-radius: 6px; background: none; padding: 0; }
.hint { color: var(--muted); font-size: 12px; margin: 8px 0; }
.errors { margin: 8px 0 0 16px; color: #ffb454; font-size: 12px; }
</style>
