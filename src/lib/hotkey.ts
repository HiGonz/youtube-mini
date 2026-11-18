// Convierte un KeyboardEvent en un acelerador compatible con tauri-plugin-global-shortcut.
const NAMED: Record<string, string> = {
  Space: "Space", ArrowUp: "Up", ArrowDown: "Down", ArrowLeft: "Left", ArrowRight: "Right", Backspace: "Backspace",
  Enter: "Enter", Tab: "Tab", Home: "Home", End: "End", PageUp: "PageUp", PageDown: "PageDown", Insert: "Insert", Delete: "Delete",
};

export function toAccelerator(e: KeyboardEvent): string | null {
  let key: string | undefined = NAMED[e.code];
  if (!key && /^Key[A-Z]$/.test(e.code)) key = e.code.slice(3);
  if (!key && /^Digit\d$/.test(e.code)) key = e.code.slice(5);
  if (!key && /^F([1-9]|1\d|2[0-4])$/.test(e.code)) key = e.code;
  if (!key) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(Boolean) as string[];
  if (!mods.length && !/^F\d+$/.test(key)) return null; // evita atajos globales sin modificador
  return [...mods, key].join("+");
}
