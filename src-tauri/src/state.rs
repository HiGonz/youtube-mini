use crate::settings::{self, Settings};
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{atomic::AtomicBool, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct NowPlaying {
    pub playing: bool,
    pub title: String,
    pub artist: String,
    pub artwork: String,
    pub url: String,
    pub time: f64,
    pub duration: f64,
    pub music: bool,
    pub has_video: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct RecentItem {
    pub kind: String, // "video" | "search"
    pub title: String,
    pub url: String,
    pub at: u64,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RecentStore {
    pub items: Vec<RecentItem>,
    pub last: Option<NowPlaying>,
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub recent: Mutex<RecentStore>,
    pub now: Mutex<NowPlaying>,
    pub mini: AtomicBool,
    pub hotkey_map: Mutex<HashMap<u32, String>>,
    pub hotkey_errors: Mutex<Vec<String>>,
    pub config_dir: PathBuf,
}

impl AppState {
    pub fn new(config_dir: PathBuf) -> Self {
        let mut s: Settings = settings::load(&config_dir.join("settings.json"));
        s.sanitize();
        let recent: RecentStore = settings::load(&config_dir.join("recent.json"));
        Self {
            settings: Mutex::new(s),
            recent: Mutex::new(recent),
            now: Mutex::new(NowPlaying::default()),
            mini: AtomicBool::new(false),
            hotkey_map: Mutex::new(HashMap::new()),
            hotkey_errors: Mutex::new(vec![]),
            config_dir,
        }
    }
    pub fn save_settings(&self) {
        let s = self.settings.lock().unwrap().clone();
        settings::save(&self.config_dir.join("settings.json"), &s);
    }
    pub fn save_recent(&self) {
        let r = self.recent.lock().unwrap().clone();
        settings::save(&self.config_dir.join("recent.json"), &r);
    }
    pub fn push_recent(&self, kind: &str, title: &str, url: &str) {
        let at = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let mut r = self.recent.lock().unwrap();
        r.items.retain(|i| i.url != url);
        r.items.insert(0, RecentItem { kind: kind.into(), title: title.chars().take(160).collect(), url: url.into(), at });
        r.items.truncate(20);
    }
}
