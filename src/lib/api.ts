import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";

export interface Settings {
  close_behavior: "tray" | "quit";
  start_with_windows: boolean;
  remember_window: boolean;
  hotkeys: Record<string, string>;
  mini_size: "mini" | "compact" | "standard";
  mini_always_on_top: boolean;
  auto_hide_controls: boolean;
  theme: "dark" | "oled" | "system";
  accent: string;
  reduce_motion: boolean;
}
export interface NowPlaying {
  playing: boolean; title: string; artist: string; artwork: string; url: string;
  time: number; duration: number; music: boolean; has_video: boolean;
}
export interface Snapshot { settings: Settings; now: NowPlaying; mini: boolean; always_on_top: boolean; hotkey_errors: string[] }
export interface RecentItem { kind: "video" | "search"; title: string; url: string; at: number }

export const api = {
  state: () => invoke<Snapshot>("get_state"),
  setSettings: (settings: Settings) => invoke<Snapshot>("set_settings", { settings }),
  recent: () => invoke<{ items: RecentItem[]; last: NowPlaying | null }>("get_recent"),
  nav: (action: "back" | "forward" | "home" | "music" | "reload" | "resume" | "go", input?: string, music?: boolean) =>
    invoke<void>("navigate", { action, input, music }),
  toggleMini: () => invoke<void>("toggle_mini"),
  setMiniSize: (size: Settings["mini_size"]) => invoke<void>("set_mini_size", { size }),
  pip: () => invoke<void>("toggle_pip"),
  alwaysOnTop: () => invoke<boolean>("toggle_always_on_top"),
  win: (action: "minimize" | "maximize" | "close" | "drag") => invoke<void>("window_ctl", { action }),
  openSettings: () => invoke<void>("open_settings"),
  hideQuick: () => invoke<void>("hide_quick"),
  quick: (action: "go" | "resume", input?: string, music?: boolean, mini?: boolean) =>
    invoke<void>("quick_submit", { action, input, music, mini }),
  player: (action: "playpause" | "next" | "prev" | "volume_up" | "volume_down" | "mute") => invoke<void>("player_cmd", { action }),
};

export const on = <T>(event: string, cb: (payload: T) => void): Promise<UnlistenFn> => listen<T>(event, (e) => cb(e.payload));
