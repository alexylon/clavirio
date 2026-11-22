use std::collections::{BTreeMap, HashMap};
use std::io;
use std::path::{Path, PathBuf};

const MAX_HISTORY_RECORDS: usize = 10_000;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use crate::settings::KeyboardLayout;
use crate::storage::{Loaded, file_name};

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
    /// Empty in records saved before titles were kept
    #[serde(default)]
    pub title: String,
    /// `None` in records saved before layouts were kept
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<KeyboardLayout>,
}

impl SessionRecord {
    /// Completed with a speed, as one under a second has none, and passed if it is
    /// a lesson, so a faster attempt that failed isn't held up as the one to beat
    fn can_be_best(&self) -> bool {
        self.completed
            && self.wpm > 0.0
            && (!crate::lessons::is_lesson_id(&self.id) || crate::lessons::is_passed(self.accuracy))
    }
}

/// What saving a session found
pub struct SavedSession {
    /// The best wpm of the earlier sessions to compare this one with
    pub earlier_best_wpm: Option<f64>,
    /// Whether this session can be a best, which a failed lesson attempt can't
    pub can_be_best: bool,
    /// Where the old file was kept if it couldn't be parsed
    pub notice: Option<String>,
}

fn history_path() -> PathBuf {
    crate::storage::data_dir().join("history.json")
}

fn load_json_for_rewrite<T: DeserializeOwned + Default>(
    path: &Path,
) -> io::Result<(T, Option<PathBuf>)> {
    crate::storage::load_for_rewrite(path, |text| serde_json::from_str(text).ok())
}

/// Loads a JSON file, or the default if there is none. An error is a message
/// saying that the file can't be read or parsed.
fn load_json<T: DeserializeOwned + Default>(path: &Path) -> Result<T, String> {
    match crate::storage::load(path, |text| serde_json::from_str(text).ok()) {
        Ok(Loaded::Parsed(value)) => Ok(value),
        Ok(Loaded::Missing) => Ok(T::default()),
        Ok(Loaded::Unparseable) => Err(format!("Couldn't parse {}", file_name(path))),
        Err(e) => Err(format!("Couldn't read {}: {e}", file_name(path))),
    }
}

pub fn save_session(record: SessionRecord) -> io::Result<SavedSession> {
    let path = history_path();
    let (mut records, backup) = load_json_for_rewrite::<Vec<SessionRecord>>(&path)?;

    let earlier_best_wpm = earlier_best_wpm(&records, &record);
    let can_be_best = record.can_be_best();
    records.push(record);
    if records.len() > MAX_HISTORY_RECORDS {
        records.drain(..records.len() - MAX_HISTORY_RECORDS);
    }

    let json = serde_json::to_string_pretty(&records)?;
    let notice = crate::storage::finish_rewrite(&path, &json, backup)?;
    Ok(SavedSession {
        earlier_best_wpm,
        can_be_best,
        notice,
    })
}

/// The best wpm in `records` of the same lesson or practice as `record`, on the
/// same layout; `None` if there is none, or `record` wasn't finished. A record
/// whose layout isn't known is left out, as it could be of any layout
fn earlier_best_wpm(records: &[SessionRecord], record: &SessionRecord) -> Option<f64> {
    record.layout?;
    if !record.completed {
        return None;
    }
    records
        .iter()
        .filter(|r| r.id == record.id && r.layout == record.layout && r.can_be_best())
        .map(|r| r.wpm)
        .reduce(f64::max)
}

pub fn resume_lesson(layout: KeyboardLayout) -> Result<usize, String> {
    let lessons = crate::lessons::lessons_for_layout(layout);
    load_history().map(|history| resume_lesson_from(&history, &lessons))
}

fn resume_lesson_from(history: &[SessionRecord], lessons: &[&crate::lessons::Lesson]) -> usize {
    for entry in history.iter().rev() {
        if entry.id.is_empty() {
            continue;
        }
        let idx = lessons.iter().position(|l| l.id == entry.id);
        match idx {
            Some(i) if entry.completed && crate::lessons::is_passed(entry.accuracy) => {
                return (i + 1).min(lessons.len().saturating_sub(1));
            }
            Some(i) => return i,
            None => continue,
        }
    }
    0
}

fn keystats_path() -> PathBuf {
    crate::storage::data_dir().join("keystats.json")
}

/// Adds `session_stats` to the saved key stats, and returns a message saying
/// where the old file was kept if it couldn't be parsed.
pub fn save_key_stats(session_stats: &HashMap<char, (u32, u32)>) -> io::Result<Option<String>> {
    if session_stats.is_empty() {
        return Ok(None);
    }
    let path = keystats_path();
    let (saved, backup) = load_json_for_rewrite(&path)?;
    let mut cumulative = key_stats_by_char(saved);
    for (&ch, &(hits, misses)) in session_stats {
        let entry = cumulative.entry(ch).or_insert((0, 0));
        entry.0 = entry.0.saturating_add(hits);
        entry.1 = entry.1.saturating_add(misses);
    }

    let map: BTreeMap<String, (u32, u32)> = cumulative
        .into_iter()
        .map(|(k, v)| (k.to_string(), v))
        .collect();
    let json = serde_json::to_string_pretty(&map)?;
    crate::storage::finish_rewrite(&path, &json, backup)
}

pub fn load_key_stats() -> Result<HashMap<char, (u32, u32)>, String> {
    load_json(&keystats_path()).map(key_stats_by_char)
}

fn key_stats_by_char(stored: HashMap<String, (u32, u32)>) -> HashMap<char, (u32, u32)> {
    stored
        .into_iter()
        .filter_map(|(k, v)| k.chars().next().map(|c| (c, v)))
        .collect()
}

pub fn load_history() -> Result<Vec<SessionRecord>, String> {
    load_json(&history_path())
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
            title: "Home Row".into(),
            layout: Some(KeyboardLayout::default()),
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
        assert_eq!(record.layout, None);
    }

    // --- resume_lesson_from ---

    fn record(id: &str, completed: bool) -> SessionRecord {
        SessionRecord {
            timestamp: String::new(),
            wpm: 0.0,
            accuracy: 100.0,
            correct: 0,
            total: 1,
            duration_secs: 0.0,
            completed,
            id: id.into(),
            title: String::new(),
            layout: Some(KeyboardLayout::default()),
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
    fn resume_stays_on_completed_lesson_below_pass_mark() {
        let lessons = qwerty_lessons();
        let mut failed = record(lessons[0].id, true);
        failed.accuracy = 94.4;
        let mut passed = record(lessons[1].id, true);
        passed.accuracy = 94.6;
        assert_eq!(resume_lesson_from(&[failed], &lessons), 0);
        assert_eq!(resume_lesson_from(&[passed], &lessons), 2);
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
    fn earlier_best_is_of_the_same_session_and_layout_that_could_be_a_best() {
        let id = qwerty_lessons()[0].id;
        let lesson = |wpm, accuracy, completed, layout| SessionRecord {
            wpm,
            accuracy,
            layout,
            ..record(id, completed)
        };
        let practice = |wpm, accuracy| SessionRecord {
            wpm,
            accuracy,
            ..record("words_english_200", true)
        };
        let records = [
            lesson(30.0, 100.0, true, Some(KeyboardLayout::Qwerty)),
            lesson(50.0, 100.0, true, Some(KeyboardLayout::Dvorak)),
            lesson(55.0, 100.0, true, None),
            lesson(60.0, 100.0, false, Some(KeyboardLayout::Qwerty)),
            lesson(70.0, 90.0, true, Some(KeyboardLayout::Qwerty)),
            practice(80.0, 50.0),
            practice(0.0, 100.0),
        ];

        let passed = lesson(40.0, 100.0, true, Some(KeyboardLayout::Qwerty));
        let failed = lesson(40.0, 90.0, true, Some(KeyboardLayout::Qwerty));
        assert_eq!(earlier_best_wpm(&records, &passed), Some(30.0));
        assert_eq!(earlier_best_wpm(&records, &failed), Some(30.0));
        assert_eq!(
            earlier_best_wpm(&[practice(0.0, 100.0)], &practice(10.0, 50.0)),
            None
        );
        assert_eq!(
            earlier_best_wpm(&records, &practice(10.0, 50.0)),
            Some(80.0)
        );
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
