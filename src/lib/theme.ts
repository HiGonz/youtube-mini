import type { Settings } from "./api";

export function applyTheme(s: Pick<Settings, "theme" | "accent" | "reduce_motion">) {
  const root = document.documentElement;
  const theme = s.theme === "system" ? (matchMedia("(prefers-color-scheme: light)").matches ? "light" : "dark") : s.theme;
  root.dataset.theme = theme;
  root.dataset.motion = s.reduce_motion ? "reduced" : "";
  root.style.setProperty("--accent", s.accent);
}
