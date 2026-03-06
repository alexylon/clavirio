use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionRecord {
    pub timestamp: String,
    pub wpm: f64,
    pub accuracy: f64,
    pub correct: u32,
    pub total: u32,
    pub duration_secs: f64,
    #[serde(default)]
    pub completed: bool,
}

fn history_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".into());
    PathBuf::from(home).join(".ferrotype").join("history.json")
}

pub fn save_session(record: SessionRecord) {
    let path = history_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut records: Vec<SessionRecord> = fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default();

    records.push(record);

    if let Ok(json) = serde_json::to_string_pretty(&records) {
        let _ = fs::write(&path, json);
    }
}

pub fn load_history() -> Vec<SessionRecord> {
    let path = history_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}
