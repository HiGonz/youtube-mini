<script setup lang="ts">
const props = defineProps<{ name: string; size?: number }>();
// Iconos propios en trazo (24x24), sin librería externa.
const paths: Record<string, string> = {
  back: "M15 5l-7 7 7 7", forward: "M9 5l7 7-7 7", home: "M4 11l8-7 8 7v9h-5v-6H9v6H4z",
  mini: "M4 5h16v14H4zM12 12h6v5h-6z", pip: "M3 5h18v14H3zM12 11h7v6h-7z", pin: "M9 4h6l-1 6 3 3H7l3-3zM12 13v7",
  settings: "M4 7h10M18 7h2M4 17h2M10 17h10M16 4v6M8 14v6", min: "M5 12h14", max: "M6 6h12v12H6z", close: "M6 6l12 12M18 6L6 18",
  search: "M11 4a7 7 0 100 14 7 7 0 000-14zM20 20l-4-4", music: "M9 18V6l10-2v12M9 18a3 3 0 11-6 0 3 3 0 016 0zM19 16a3 3 0 11-6 0 3 3 0 016 0z",
  play: "M7 4v16l13-8z", link: "M10 14a4 4 0 005.7 0l3-3a4 4 0 00-5.7-5.7l-1 1M14 10a4 4 0 00-5.7 0l-3 3a4 4 0 005.7 5.7l1-1",
  clock: "M12 4a8 8 0 100 16 8 8 0 000-16zM12 8v4l3 2", resume: "M5 4v16M9 12l10-7v14z",
};
</script>
<template>
  <svg :width="size ?? 18" :height="size ?? 18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round" aria-hidden="true">
    <path :d="paths[props.name]" />
  </svg>
</template>
