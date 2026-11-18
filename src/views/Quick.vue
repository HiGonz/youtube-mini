<script setup lang="ts">
import { computed, nextTick, onMounted, ref } from "vue";
import Icon from "../components/Icon.vue";
import { api, on, type NowPlaying, type RecentItem } from "../lib/api";

interface Row { key: string; icon: string; title: string; sub?: string; run: (mini: boolean) => void }
const q = ref("");
const sel = ref(0);
const recent = ref<RecentItem[]>([]);
const last = ref<NowPlaying | null>(null);
const input = ref<HTMLInputElement>();

async function refresh() {
  const r = await api.recent(); recent.value = r.items; last.value = r.last;
  q.value = ""; sel.value = 0;
  await nextTick(); input.value?.focus();
}
onMounted(() => {
  refresh(); on("quick-open", refresh);
  addEventListener("blur", () => api.hideQuick());
});

const looksLikeUrl = (t: string) => /^(https?:\/\/)?([\w-]+\.)*(youtube\.com|youtu\.be)\b/i.test(t.trim());

const rows = computed<Row[]>(() => {
  const t = q.value.trim();
  const out: Row[] = [];
  if (t) {
    if (looksLikeUrl(t)) out.push({ key: "url", icon: "link", title: "Abrir enlace", sub: t, run: (m) => api.quick("go", t, false, m) });
    else {
      out.push({ key: "yt", icon: "search", title: `Buscar «${t}» en YouTube`, run: (m) => api.quick("go", t, false, m) });
      out.push({ key: "ytm", icon: "music", title: `Buscar «${t}» en YouTube Music`, run: (m) => api.quick("go", t, true, m) });
    }
    const m = t.toLowerCase();
    recent.value.filter((r) => r.title.toLowerCase().includes(m)).slice(0, 5).forEach((r) => out.push(item(r)));
  } else {
    if (last.value?.title) out.push({ key: "resume", icon: "resume", title: "Reanudar", sub: last.value.title, run: (m) => api.quick("resume", undefined, undefined, m) });
    recent.value.slice(0, 7).forEach((r) => out.push(item(r)));
  }
  return out;
});
const item = (r: RecentItem): Row => ({
  key: r.url, icon: r.kind === "video" ? "play" : "clock", title: r.title, sub: r.kind === "video" ? "Reproducido" : "Búsqueda reciente",
  run: (m) => api.quick("go", r.url, false, m),
});

function key(e: KeyboardEvent) {
  const n = rows.value.length;
  if (e.key === "Escape") api.hideQuick();
  else if (e.key === "ArrowDown") { e.preventDefault(); sel.value = (sel.value + 1) % Math.max(n, 1); }
  else if (e.key === "ArrowUp") { e.preventDefault(); sel.value = (sel.value - 1 + n) % Math.max(n, 1); }
  else if (e.key === "Enter") { e.preventDefault(); rows.value[sel.value]?.run(e.ctrlKey); }
}
</script>

<template>
  <div class="q">
    <div class="field"><Icon name="search" :size="20" />
      <input ref="input" v-model="q" @input="sel = 0" @keydown="key" spellcheck="false" autocomplete="off" placeholder="Buscar en YouTube o pegar un enlace…" aria-label="Quick Launch" /></div>
    <ul role="listbox">
      <li v-for="(r, i) in rows" :key="r.key" role="option" :aria-selected="i === sel" :class="{ on: i === sel }" @mousemove="sel = i" @click="r.run(false)">
        <Icon :name="r.icon" :size="18" /><span class="t">{{ r.title }}<small v-if="r.sub">{{ r.sub }}</small></span>
      </li>
      <li v-if="!rows.length" class="empty">Aún no hay reproducciones recientes.</li>
    </ul>
    <footer><span><kbd>↵</kbd> abrir</span><span><kbd>Ctrl</kbd>+<kbd>↵</kbd> en Mini Mode</span><span><kbd>Esc</kbd> cerrar</span></footer>
  </div>
</template>

<style scoped>
.q { height: 100%; display: flex; flex-direction: column; background: var(--surface); border: 1px solid var(--border-strong); border-radius: 0; }
.field { display: flex; align-items: center; gap: 12px; padding: 0 18px; height: 60px; border-bottom: 1px solid var(--border); color: var(--muted); }
.field input { flex: 1; background: none; border: 0; outline: 0; font-size: 17px; color: var(--text); }
ul { list-style: none; padding: 6px; flex: 1; overflow: auto; }
li { display: flex; align-items: center; gap: 12px; padding: 0 12px; height: 46px; border-radius: var(--r-md); color: var(--muted); cursor: default; }
li.on { background: var(--surface-2); color: var(--text); }
li.on :deep(svg) { color: var(--accent); }
.t { display: flex; flex-direction: column; min-width: 0; color: var(--text); }
.t, small { overflow: hidden; text-overflow: ellipsis; white-space: nowrap; }
small { color: var(--muted); font-size: 11.5px; }
.empty { justify-content: center; }
footer { display: flex; gap: 16px; padding: 8px 16px; border-top: 1px solid var(--border); color: var(--muted); font-size: 11.5px; }
kbd { font: inherit; padding: 0 5px; border: 1px solid var(--border-strong); border-radius: 4px; }
</style>
