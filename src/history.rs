use std::collections::{BTreeMap, HashMap};
use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

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
    #[serde(default, alias = "lesson")]
    pub id: String,
}

fn history_path() -> PathBuf {
    crate::settings::data_dir().join("history.json")
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
        let tmp = path.with_extension("json.tmp");
        if fs::write(&tmp, &json).is_ok() {
            let _ = fs::rename(&tmp, &path);
        }
    }
}

pub fn resume_lesson(layout: crate::settings::KeyboardLayout) -> usize {
    let lessons = crate::lessons::lessons_for_layout(layout);
    resume_lesson_from(&load_history(), &lessons)
}

fn resume_lesson_from(history: &[SessionRecord], lessons: &[&crate::lessons::Lesson]) -> usize {
    for entry in history.iter().rev() {
        if entry.id.is_empty() {
            continue;
        }
        let idx = lessons.iter().position(|l| l.id == entry.id);
        match idx {
            Some(i) if entry.completed => {
                return (i + 1).min(lessons.len().saturating_sub(1));
            }
            Some(i) => return i,
            None => continue,
        }
    }
    0
}

fn keystats_path() -> PathBuf {
    crate::settings::data_dir().join("keystats.json")
}

pub fn save_key_stats(session_stats: &HashMap<char, (u32, u32)>) {
    if session_stats.is_empty() {
        return;
    }
    let path = keystats_path();
    if let Some(parent) = path.parent() {
        let _ = fs::create_dir_all(parent);
    }

    let mut cumulative = load_key_stats();
    for (&ch, &(hits, misses)) in session_stats {
        let entry = cumulative.entry(ch).or_insert((0, 0));
        entry.0 += hits;
        entry.1 += misses;
    }

    let map: BTreeMap<String, (u32, u32)> = cumulative
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    if let Ok(json) = serde_json::to_string_pretty(&map) {
        let tmp = path.with_extension("json.tmp");
        if fs::write(&tmp, &json).is_ok() {
            let _ = fs::rename(&tmp, &path);
        }
    }
}

pub fn load_key_stats() -> HashMap<char, (u32, u32)> {
    let path = keystats_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str::<HashMap<String, (u32, u32)>>(&s).ok())
        .map(|m| {
            m.into_iter()
                .filter_map(|(k, v)| k.chars().next().map(|c| (c, v)))
                .collect()
        })
        .unwrap_or_default()
}

pub fn load_history() -> Vec<SessionRecord> {
    let path = history_path();
    fs::read_to_string(&path)
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn session_record_roundtrip() {
        let record = SessionRecord {
            timestamp: "2026-03-08T12:00:00".into(),
            wpm: 45.0,
            accuracy: 97.5,
            correct: 195,
            total: 200,
            duration_secs: 120.0,
            completed: true,
            id: "home_row".into(),
        };
        let json = serde_json::to_string(&record).unwrap();
        let deserialized: SessionRecord = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.timestamp, "2026-03-08T12:00:00");
        assert_eq!(deserialized.wpm, 45.0);
        assert_eq!(deserialized.accuracy, 97.5);
        assert_eq!(deserialized.correct, 195);
        assert_eq!(deserialized.total, 200);
        assert!(deserialized.completed);
        assert_eq!(deserialized.id, "home_row");
    }

    #[test]
    fn deserialize_without_optional_fields() {
        // Simulates loading history from before lesson/completed fields existed
        let json = r#"{
            "timestamp": "2026-01-01T00:00:00",
            "wpm": 30.0,
            "accuracy": 90.0,
            "correct": 100,
            "total": 111,
            "duration_secs": 60.0
        }"#;
        let record: SessionRecord = serde_json::from_str(json).unwrap();
        assert!(!record.completed); // default
        assert!(record.id.is_empty()); // default
    }

    // --- resume_lesson_from ---

    fn record(id: &str, completed: bool) -> SessionRecord {
        SessionRecord {
            timestamp: String::new(),
            wpm: 0.0,
            accuracy: 0.0,
            correct: 0,
            total: 1,
            duration_secs: 0.0,
            completed,
            id: id.into(),
        }
    }

    fn qwerty_lessons() -> Vec<&'static crate::lessons::Lesson> {
        crate::lessons::lessons_for_layout(crate::settings::KeyboardLayout::Qwerty)
    }

    #[test]
    fn resume_empty_history_returns_zero() {
        assert_eq!(resume_lesson_from(&[], &qwerty_lessons()), 0);
    }

    #[test]
    fn resume_incomplete_lesson_returns_same() {
        let lessons = qwerty_lessons();
        let history = vec![record(lessons[0].id, false)];
        assert_eq!(resume_lesson_from(&history, &lessons), 0);
    }

    #[test]
    fn resume_completed_lesson_returns_next() {
        let lessons = qwerty_lessons();
        let history = vec![record(lessons[0].id, true)];
        assert_eq!(resume_lesson_from(&history, &lessons), 1);
    }

    #[test]
    fn resume_completed_last_lesson_stays_at_last() {
        let lessons = qwerty_lessons();
        let last = lessons.last().unwrap();
        let history = vec![record(last.id, true)];
        assert_eq!(resume_lesson_from(&history, &lessons), lessons.len() - 1);
    }

    #[test]
    fn resume_unknown_lesson_returns_zero() {
        let history = vec![record("unknown", false)];
        assert_eq!(resume_lesson_from(&history, &qwerty_lessons()), 0);
    }

    #[test]
    fn resume_uses_last_matching_entry() {
        let lessons = qwerty_lessons();
        let history = vec![record(lessons[0].id, true), record(lessons[3].id, false)];
        assert_eq!(resume_lesson_from(&history, &lessons), 3);
    }

    #[test]
    fn resume_skips_non_lesson_sessions() {
        let lessons = qwerty_lessons();
        let history = vec![
            record(lessons[3].id, true),
            record("words_english_200", true),
            record("timed_60s_english_200", true),
        ];
        assert_eq!(resume_lesson_from(&history, &lessons), 4);
    }

    #[test]
    fn resume_cross_layout_shares_ids() {
        let qwerty = qwerty_lessons();
        let dvorak = crate::lessons::lessons_for_layout(crate::settings::KeyboardLayout::Dvorak);
        let history = vec![record(qwerty[3].id, true)];
        assert_eq!(resume_lesson_from(&history, &dvorak), 4);
    }

    #[test]
    fn keystats_roundtrip_json() {
        let json = r#"{"a":[10,5],"b":[20,3]}"#;
        let map: std::collections::HashMap<String, (u32, u32)> =
            serde_json::from_str(json).unwrap();
        assert_eq!(map["a"], (10, 5));
        assert_eq!(map["b"], (20, 3));
    }

    #[test]
    fn deserialize_array_of_records() {
        let json = r#"[
            {"timestamp":"t1","wpm":40.0,"accuracy":95.0,"correct":50,"total":53,"duration_secs":30.0},
            {"timestamp":"t2","wpm":50.0,"accuracy":98.0,"correct":100,"total":102,"duration_secs":60.0,"completed":true,"lesson":"f j d k"}
        ]"#;
        let records: Vec<SessionRecord> = serde_json::from_str(json).unwrap();
        assert_eq!(records.len(), 2);
        assert!(!records[0].completed);
        assert!(records[0].id.is_empty());
        assert!(records[1].completed);
        assert_eq!(records[1].id, "f j d k");
    }
}
