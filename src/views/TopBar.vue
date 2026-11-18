<script setup lang="ts">
import { onMounted, ref } from "vue";
import Icon from "../components/Icon.vue";
import { api, on, type NowPlaying } from "../lib/api";

const query = ref("");
const mini = ref(false);
const pinned = ref(false);
const now = ref<NowPlaying | null>(null);
const input = ref<HTMLInputElement>();

onMounted(async () => {
  const s = await api.state();
  mini.value = s.mini; pinned.value = s.always_on_top; now.value = s.now;
  on<boolean>("mini-changed", async (v) => { mini.value = v; pinned.value = (await api.state()).always_on_top; });
  on<NowPlaying>("player-state", (n) => (now.value = n));
  on("settings-changed", async () => (pinned.value = (await api.state()).always_on_top));
  addEventListener("keydown", (e) => {
    if ((e.ctrlKey && e.key.toLowerCase() === "l") || (e.key === "/" && document.activeElement !== input.value)) { e.preventDefault(); input.value?.focus(); input.value?.select(); }
  });
});

const search = () => { if (query.value.trim()) api.nav("go", query.value, !!now.value?.music); };
const togglePin = async () => (pinned.value = await api.alwaysOnTop());
const drag = (e: MouseEvent) => { if (e.button === 0 && e.target === e.currentTarget) api.win("drag"); };
</script>

<template>
  <header class="bar" @mousedown="drag" @dblclick.self="api.win('maximize')">
    <div class="brand" title="YouTube Mini" @mousedown.stop="api.win('drag')"><span class="logo"><span /></span></div>
    <nav class="group">
      <button class="icon-btn" title="Atrás" @click="api.nav('back')"><Icon name="back" /></button>
      <button class="icon-btn" title="Adelante" @click="api.nav('forward')"><Icon name="forward" /></button>
      <button class="icon-btn" title="Inicio" @click="api.nav('home')"><Icon name="home" /></button>
      <button class="icon-btn" title="YouTube Music" @click="api.nav('music')"><Icon name="music" /></button>
    </nav>
    <form class="search" role="search" @submit.prevent="search">
      <Icon name="search" :size="16" />
      <input ref="input" v-model="query" type="text" spellcheck="false" autocomplete="off"
        :placeholder="now?.music ? 'Buscar en YouTube Music o pegar URL' : 'Buscar en YouTube o pegar URL'" aria-label="Buscar" />
    </form>
    <div class="spacer" @mousedown="drag" />
    <div class="group">
      <button class="icon-btn" :aria-pressed="mini" title="Mini Mode (Ctrl+Shift+M)" @click="api.toggleMini()"><Icon name="mini" /></button>
      <button class="icon-btn" title="Picture-in-Picture" @click="api.pip()"><Icon name="pip" /></button>
      <button class="icon-btn" :aria-pressed="pinned" title="Siempre encima" @click="togglePin"><Icon name="pin" /></button>
      <button class="icon-btn" title="Ajustes" @click="api.openSettings()"><Icon name="settings" /></button>
    </div>
    <div class="group win">
      <button class="icon-btn" title="Minimizar" @click="api.win('minimize')"><Icon name="min" :size="16" /></button>
      <button class="icon-btn" title="Maximizar" @click="api.win('maximize')"><Icon name="max" :size="14" /></button>
      <button class="icon-btn danger" title="Cerrar" @click="api.win('close')"><Icon name="close" :size="16" /></button>
    </div>
  </header>
</template>

<style scoped>
.bar { display: flex; align-items: center; gap: 10px; height: 48px; padding: 0 8px 0 12px; background: var(--bg); border-bottom: 1px solid var(--border); }
.brand { display: grid; place-items: center; width: 28px; height: 28px; }
.logo { width: 24px; height: 17px; border-radius: 5px; background: var(--accent); display: grid; place-items: center; }
.logo span { border-left: 7px solid #fff; border-top: 4px solid transparent; border-bottom: 4px solid transparent; margin-left: 2px; }
.group { display: flex; gap: 2px; }
.search { display: flex; align-items: center; gap: 8px; flex: 1 1 420px; max-width: 560px; min-width: 160px; height: 32px; padding: 0 12px; border-radius: 999px; background: var(--surface); border: 1px solid var(--border); color: var(--muted); transition: border-color var(--dur) var(--ease); }
.search:focus-within { border-color: var(--accent); color: var(--text); }
.search input { flex: 1; min-width: 0; background: none; border: 0; outline: 0; }
.search input::placeholder { color: var(--muted); }
.spacer { flex: 1 1 0; align-self: stretch; min-width: 8px; }
.win { margin-left: 6px; }
</style>
