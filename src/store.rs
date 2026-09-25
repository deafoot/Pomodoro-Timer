use std::fs;
use std::path::PathBuf;

use crate::model::{App, STATE_VERSION};

fn data_dir() -> PathBuf {
    let base = std::env::var("APPDATA").unwrap_or_else(|_| ".".into());
    PathBuf::from(base).join("PomodoroTimer")
}

pub fn state_path() -> PathBuf {
    data_dir().join("state.json")
}

pub fn load() -> App {
    let path = state_path();
    let mut app = fs::read_to_string(&path)
        .ok()
        .and_then(|raw| serde_json::from_str::<App>(&raw).ok())
        .filter(|app: &App| app.version == STATE_VERSION)
        .unwrap_or_default();
    app.normalize();
    app
}

pub fn save(app: &App) {
    let path = state_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }
    if let Ok(json) = serde_json::to_string_pretty(app) {
        let tmp = path.with_extension("json.tmp");
        if fs::write(&tmp, json).is_ok() {
            let _ = fs::rename(&tmp, &path);
        }
    }
}
