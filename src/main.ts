import { createApp } from "vue";
import "./styles/theme.css";
import { api, on } from "./lib/api";
import { applyTheme } from "./lib/theme";

// Misma bundle para 3 vistas locales, elegidas por ?view= (main | settings | quick).
const view = new URLSearchParams(location.search).get("view") ?? "main";
const loaders = {
  main: () => import("./views/TopBar.vue"),
  settings: () => import("./views/Settings.vue"),
  quick: () => import("./views/Quick.vue"),
} as const;

async function boot() {
  const snap = await api.state();
  applyTheme(snap.settings);
  on("settings-changed", async () => applyTheme((await api.state()).settings));
  const comp = await (loaders[view as keyof typeof loaders] ?? loaders.main)();
  createApp(comp.default).mount("#app");
}
boot();
