use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::time::Instant;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use time::{format_description::well_known::Iso8601, OffsetDateTime, UtcOffset};

use crate::input::InputEvent;
use crate::settings::{KeyboardLayout, Theme};
use crate::words::WordList;

fn chrono_now() -> String {
    let now = OffsetDateTime::now_utc()
        .to_offset(UtcOffset::current_local_offset().unwrap_or(UtcOffset::UTC));
    let format = Iso8601::DEFAULT;
    now.format(&format)
        .map(|s| s[..19].to_string())
        .unwrap_or_else(|_| "1970-01-01T00:00:00".into())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Progress {
    Active,
    Finished,
}

#[derive(Debug, Clone)]
pub struct Document {
    lines: Vec<String>,
    line_idx: usize,
    char_idx: usize,
    pub current_line: String,
    current_chars: Vec<char>,
    pub progress: Progress,
}

impl Document {
    pub fn from_text(text: &str) -> Result<Self, String> {
        let lines: Vec<String> = text.lines().map(String::from).collect();
        if lines.is_empty() {
            return Err("Text is empty".into());
        }
        let first_line = lines
            .iter()
            .find(|l| !l.is_empty())
            .cloned()
            .unwrap_or_default();
        let current_chars = first_line.chars().collect();
        Ok(Self {
            lines,
            line_idx: 0,
            char_idx: 0,
            current_line: first_line,
            current_chars,
            progress: Progress::Active,
        })
    }

    pub fn load(raw_path: &str) -> Result<Self, String> {
        let trimmed = raw_path.trim();
        let resolved = if Path::new(trimmed).is_absolute() {
            Path::new(trimmed).to_path_buf()
        } else {
            std::env::current_dir()
                .map_err(|e| format!("Cannot resolve working directory: {e}"))?
                .join(trimmed)
        };

        if !resolved.exists() {
            return Err(format!("File not found: {}", resolved.display()));
        }

        const MAX_FILE_SIZE: u64 = 1_000_000; // 1 MB
        let meta = fs::metadata(&resolved).map_err(|e| format!("Cannot read file: {e}"))?;
        if meta.len() > MAX_FILE_SIZE {
            return Err(format!(
                "File too large ({:.1} MB, max 1 MB)",
                meta.len() as f64 / 1_000_000.0
            ));
        }

        let content =
            fs::read_to_string(&resolved).map_err(|e| format!("Cannot read file: {e}"))?;

        Self::from_text(&content)
    }

    pub fn reset(&mut self) {
        self.line_idx = 0;
        self.char_idx = 0;
        self.current_line = self
            .lines
            .iter()
            .find(|l| !l.is_empty())
            .cloned()
            .unwrap_or_default();
        self.current_chars = self.current_line.chars().collect();
        self.progress = Progress::Active;
    }

    pub fn cursor_position(&self) -> usize {
        self.char_idx
    }

    pub fn expected_char(&self) -> Option<char> {
        self.current_chars.get(self.char_idx).copied()
    }

    pub fn upcoming_lines(&self, count: usize) -> Vec<&str> {
        let mut result = Vec::new();
        let mut idx = self.line_idx + 1;
        while result.len() < count {
            match self.lines.get(idx) {
                Some(line) if !line.is_empty() => result.push(line.as_str()),
                Some(_) => {}
                None => break,
            }
            idx += 1;
        }
        result
    }

    pub fn line_progress(&self) -> (usize, usize) {
        let total = self.lines.iter().filter(|l| !l.is_empty()).count();
        let current = self.lines[..self.line_idx]
            .iter()
            .filter(|l| !l.is_empty())
            .count()
            + 1;
        (current, total)
    }

    pub fn advance(&mut self) {
        self.char_idx += 1;

        if self.char_idx >= self.current_chars.len() {
            self.line_idx += 1;
            loop {
                match self.lines.get(self.line_idx) {
                    Some(line) if !line.is_empty() => {
                        self.char_idx = 0;
                        self.current_line = line.clone();
                        self.current_chars = self.current_line.chars().collect();
                        self.progress = Progress::Active;
                        return;
                    }
                    Some(_) => self.line_idx += 1,
                    None => {
                        self.progress = Progress::Finished;
                        return;
                    }
                }
            }
        }
    }
}

pub struct App {
    pub document: Option<Document>,
    pub file_path_buf: String,
    pub searching: bool,
    pub error: Option<String>,
    pub correct_count: u32,
    pub total_count: u32,
    pub last_correct: bool,
    pub last_error_char: Option<char>,
    pub highlighted_key: Option<KeyCode>,
    pub highlight_until: Option<Instant>,
    pub start_time: Option<Instant>,
    pub end_time: Option<Instant>,
    pub key_stats: HashMap<char, (u32, u32)>,
    pub viewing_history: bool,
    pub history: Vec<crate::history::SessionRecord>,
    pub history_scroll: usize,
    pub selected_lesson: usize,
    pub lesson_id: String,
    pub layout: KeyboardLayout,
    pub show_keyboard: bool,
    pub show_hints: bool,
    pub show_fingers: bool,
    pub theme: Theme,
    pub lesson_title: String,
    pub paused: bool,
    pub pause_menu_index: usize,
    pub paused_at: Option<Instant>,
    pub word_count: usize,
}

impl App {
    pub fn new() -> Self {
        let selected_lesson = crate::history::resume_lesson(KeyboardLayout::default());
        Self {
            document: None,
            file_path_buf: String::new(),
            searching: false,
            error: None,
            correct_count: 0,
            total_count: 0,
            last_correct: false,
            last_error_char: None,
            highlighted_key: None,
            highlight_until: None,
            start_time: None,
            end_time: None,
            key_stats: HashMap::new(),
            viewing_history: false,
            history: Vec::new(),
            history_scroll: 0,
            selected_lesson,
            lesson_id: String::new(),
            layout: KeyboardLayout::default(),
            show_keyboard: true,
            show_hints: true,
            show_fingers: true,
            theme: Theme::default(),
            lesson_title: String::new(),
            paused: false,
            pause_menu_index: 0,
            paused_at: None,
            word_count: 50,
        }
    }

    pub fn wpm(&self) -> f64 {
        let start = match self.start_time {
            Some(t) => t,
            None => return 0.0,
        };
        let end = self
            .end_time
            .or(self.paused_at)
            .unwrap_or_else(Instant::now);
        let secs = end.duration_since(start).as_secs_f64();
        if secs < 1.0 {
            return 0.0;
        }
        (self.correct_count as f64 / 5.0) / (secs / 60.0)
    }

    pub fn elapsed_secs(&self) -> f64 {
        match self.start_time {
            Some(t) => {
                let end = self
                    .end_time
                    .or(self.paused_at)
                    .unwrap_or_else(Instant::now);
                end.duration_since(t).as_secs_f64()
            }
            None => 0.0,
        }
    }

    pub fn error_count(&self) -> u32 {
        self.total_count - self.correct_count
    }

    fn save_history(&self, completed: bool) {
        if self.total_count == 0 || self.lesson_id.is_empty() {
            return;
        }
        let elapsed = self.elapsed_secs();
        let accuracy = if self.total_count > 0 {
            self.correct_count as f64 / self.total_count as f64 * 100.0
        } else {
            0.0
        };
        crate::history::save_session(crate::history::SessionRecord {
            timestamp: chrono_now(),
            wpm: self.wpm(),
            accuracy,
            correct: self.correct_count,
            total: self.total_count,
            duration_secs: elapsed,
            completed,
            id: self.lesson_id.clone(),
        });
    }

    pub fn worst_keys(&self, count: usize) -> Vec<(char, f32)> {
        let mut keys: Vec<(char, f32)> = self
            .key_stats
            .iter()
            .filter(|(_, (hits, misses))| *misses > 0 && (*hits + *misses) >= 2)
            .map(|(&ch, (hits, misses))| {
                let accuracy = *hits as f32 / (*hits + *misses) as f32 * 100.0;
                (ch, accuracy)
            })
            .collect();
        keys.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
        keys.truncate(count);
        keys
    }

    pub fn is_finished(&self) -> bool {
        self.document
            .as_ref()
            .is_some_and(|d| d.progress == Progress::Finished)
    }

    pub fn save_on_exit(&self) {
        if self.document.is_some() && !self.is_finished() {
            self.save_history(false);
        }
    }

    fn reset_session(&mut self) {
        self.correct_count = 0;
        self.total_count = 0;
        self.start_time = None;
        self.end_time = None;
        self.key_stats.clear();
        self.last_error_char = None;
        self.paused_at = None;
    }

    fn restart(&mut self) {
        if let Some(doc) = self.document.as_mut() {
            doc.reset();
        }
        self.reset_session();
    }

    fn resume(&mut self) {
        if let Some(paused_at) = self.paused_at.take() {
            let pause_duration = Instant::now().duration_since(paused_at);
            if let Some(ref mut start) = self.start_time {
                *start += pause_duration;
            }
        }
        self.paused = false;
        self.pause_menu_index = 0;
    }

    fn next_lesson(&mut self) {
        if !self.is_finished() {
            self.save_history(false);
        }
        let lessons = crate::lessons::lessons_for_layout(self.layout);
        self.selected_lesson = (self.selected_lesson + 1).min(lessons.len().saturating_sub(1));
        if let Some(lesson) = lessons.get(self.selected_lesson) {
            match Document::from_text(lesson.text) {
                Ok(doc) => {
                    self.document = Some(doc);
                    self.reset_session();
                    self.lesson_id = lesson.id.to_string();
                    self.lesson_title = lesson.title.to_string();
                }
                Err(e) => self.error = Some(e),
            }
        }
    }

    fn go_to_lesson_index(&mut self) {
        if !self.is_finished() {
            self.save_history(false);
        }
        self.document = None;
        self.error = None;
        self.reset_session();
        self.paused = false;
        self.pause_menu_index = 0;
        self.lesson_title.clear();
    }

    fn handle_pause_key(&mut self, code: KeyCode) -> bool {
        const MENU_LEN: usize = 3;
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.pause_menu_index = self.pause_menu_index.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.pause_menu_index + 1 < MENU_LEN {
                    self.pause_menu_index += 1;
                }
            }
            KeyCode::Home => self.pause_menu_index = 0,
            KeyCode::End => self.pause_menu_index = MENU_LEN - 1,
            KeyCode::Enter => return self.activate_pause_item(),
            KeyCode::Char(' ') => self.resume(),
            KeyCode::Char('r' | 'R') => {
                self.resume();
                self.restart();
            }
            KeyCode::Char('n' | 'N') => {
                self.resume();
                self.next_lesson();
            }
            KeyCode::Char('q' | 'Q') => {
                self.save_on_exit();
                return true;
            }
            _ => {}
        }
        false
    }

    fn activate_pause_item(&mut self) -> bool {
        match self.pause_menu_index {
            0 => {
                self.resume();
                self.restart();
            }
            1 => {
                self.resume();
                self.next_lesson();
            }
            2 => {
                self.save_on_exit();
                return true;
            }
            _ => {}
        }
        false
    }

    pub fn handle_event(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::Tick => {
                if let Some(until) = self.highlight_until {
                    if Instant::now() >= until {
                        self.highlighted_key = None;
                        self.highlight_until = None;
                    }
                }
                false
            }
            InputEvent::Press(key) => self.handle_key(key),
        }
    }

    fn handle_key(&mut self, key: KeyEvent) -> bool {
        if self.searching {
            match key.code {
                KeyCode::Enter => {
                    self.searching = false;
                    let path = self.file_path_buf.clone();
                    self.file_path_buf.clear();
                    match Document::load(&path) {
                        Ok(doc) => {
                            self.document = Some(doc);
                            self.error = None;
                            self.reset_session();
                            self.lesson_id = Path::new(&path)
                                .file_name()
                                .and_then(|n| n.to_str())
                                .unwrap_or(&path)
                                .to_string();
                            self.lesson_title = self.lesson_id.clone();
                        }
                        Err(e) => self.error = Some(e),
                    }
                }
                KeyCode::Esc => {
                    self.searching = false;
                    self.file_path_buf.clear();
                }
                KeyCode::Char('c') if key.modifiers == KeyModifiers::CONTROL => {
                    self.searching = false;
                    self.file_path_buf.clear();
                    self.save_on_exit();
                    return true;
                }
                _ if key.modifiers == KeyModifiers::CONTROL => {}
                KeyCode::Char(c) => self.file_path_buf.push(c),
                KeyCode::Backspace => {
                    self.file_path_buf.pop();
                }
                _ => {}
            }
            return false;
        }

        match (key.code, key.modifiers) {
            (KeyCode::Char('f'), KeyModifiers::CONTROL) => {
                self.searching = true;
                self.file_path_buf.clear();
                self.error = None;
            }
            (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
                if self.document.is_some() {
                    self.restart();
                }
            }
            (KeyCode::Char('c'), KeyModifiers::CONTROL) => {
                self.save_on_exit();
                return true;
            }
            (KeyCode::Esc, KeyModifiers::NONE) => {
                if self.viewing_history {
                    self.viewing_history = false;
                    return false;
                }
                if self.paused {
                    self.go_to_lesson_index();
                    return false;
                }
                if self.document.is_none() && self.error.is_none() {
                    return true;
                }
                if self.error.is_some() {
                    self.error = None;
                    self.reset_session();
                    return false;
                }
                if self.is_finished() {
                    self.document = None;
                    self.reset_session();
                    self.lesson_title.clear();
                    return false;
                }
                // Active typing: open pause menu
                self.paused = true;
                self.paused_at = if self.start_time.is_some() {
                    Some(Instant::now())
                } else {
                    None
                };
                self.pause_menu_index = 0;
            }
            _ if self.paused => {
                return self.handle_pause_key(key.code);
            }
            _ if self.viewing_history => match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    self.history_scroll = self.history_scroll.saturating_sub(1);
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.history_scroll + 1 < self.history.len() {
                        self.history_scroll += 1;
                    }
                }
                _ => {}
            },
            (KeyCode::Char('q'), _) if self.document.is_none() && self.error.is_none() => {
                return true;
            }
            _ if self.document.is_none() && self.error.is_none() => {
                self.handle_menu_key(key.code);
            }
            (KeyCode::Char('r'), _) if self.is_finished() => {
                self.restart();
            }
            (KeyCode::Char(typed), _) if self.last_error_char.is_none() => {
                self.handle_typed_char(typed);
            }
            (KeyCode::Backspace, _) => {
                if self.last_error_char.is_some() {
                    self.last_error_char = None;
                }
            }
            _ => {}
        }
        false
    }

    pub fn menu_item_count(&self) -> usize {
        crate::lessons::lessons_for_layout(self.layout).len() + crate::words::WordList::all().len()
    }

    fn handle_menu_key(&mut self, code: KeyCode) {
        let lessons = crate::lessons::lessons_for_layout(self.layout);
        let total = self.menu_item_count();
        match code {
            KeyCode::Up | KeyCode::Char('k') => {
                self.selected_lesson = self.selected_lesson.saturating_sub(1);
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.selected_lesson + 1 < total {
                    self.selected_lesson += 1;
                }
            }
            KeyCode::Enter => {
                let word_lists = crate::words::WordList::all();
                if self.selected_lesson < lessons.len() {
                    if let Some(lesson) = lessons.get(self.selected_lesson) {
                        match Document::from_text(lesson.text) {
                            Ok(doc) => {
                                self.document = Some(doc);
                                self.error = None;
                                self.reset_session();
                                self.lesson_id = lesson.id.to_string();
                                self.lesson_title = lesson.title.to_string();
                            }
                            Err(e) => self.error = Some(e),
                        }
                    }
                } else {
                    let word_idx = self.selected_lesson - lessons.len();
                    if let Some(&list) = word_lists.get(word_idx) {
                        self.start_word_practice(list);
                    }
                }
            }
            KeyCode::Char('h') => {
                self.history = crate::history::load_history();
                self.history_scroll = self.history.len().saturating_sub(1);
                self.viewing_history = true;
            }
            KeyCode::Char('l') => {
                self.layout = self.layout.cycle();
            }
            KeyCode::Char('1') if self.show_hints => {
                self.show_fingers = !self.show_fingers;
            }
            KeyCode::Char('2') => {
                self.show_hints = !self.show_hints;
                if !self.show_hints {
                    self.show_fingers = false;
                }
            }
            KeyCode::Char('3') => {
                self.show_keyboard = !self.show_keyboard;
            }
            KeyCode::Char('4') => {
                self.theme = self.theme.cycle();
            }
            _ => {}
        }
    }

    fn start_word_practice(&mut self, list: WordList) {
        let text = crate::words::generate_text(list, self.word_count);
        match Document::from_text(&text) {
            Ok(doc) => {
                self.document = Some(doc);
                self.error = None;
                self.reset_session();
                self.lesson_id = format!("words_{}", list.label().replace(' ', "_"));
                self.lesson_title = format!("Random Words ({})", list.label());
            }
            Err(e) => self.error = Some(e),
        }
    }

    fn handle_typed_char(&mut self, typed: char) {
        let expected = match self.document.as_ref().and_then(|d| d.expected_char()) {
            Some(c) => c,
            None => return,
        };

        if self.start_time.is_none() {
            self.start_time = Some(Instant::now());
        }

        self.total_count += 1;
        let entry = self.key_stats.entry(expected).or_insert((0, 0));

        if typed == expected {
            entry.0 += 1;
            self.correct_count += 1;
            self.last_correct = true;
            self.last_error_char = None;
            if let Some(doc) = self.document.as_mut() {
                doc.advance();
                if doc.progress == Progress::Finished {
                    self.end_time = Some(Instant::now());
                    self.save_history(true);
                    self.selected_lesson = (self.selected_lesson + 1)
                        .min(crate::lessons::lesson_count().saturating_sub(1));
                }
            }
        } else {
            entry.1 += 1;
            self.last_correct = false;
            self.last_error_char = Some(typed);
        }

        self.highlight_until = Some(Instant::now() + std::time::Duration::from_millis(400));
        let display_char = if typed.is_whitespace() {
            ' '
        } else {
            typed.to_ascii_uppercase()
        };
        self.highlighted_key = Some(KeyCode::Char(display_char));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // --- Document tests ---

    #[test]
    fn from_text_single_line() {
        let doc = Document::from_text("hello").unwrap();
        assert_eq!(doc.current_line, "hello");
        assert_eq!(doc.cursor_position(), 0);
        assert_eq!(doc.expected_char(), Some('h'));
        assert_eq!(doc.progress, Progress::Active);
    }

    #[test]
    fn from_text_empty_returns_error() {
        assert!(Document::from_text("").is_err());
    }

    #[test]
    fn from_text_skips_leading_blank_lines() {
        let doc = Document::from_text("\n\nabc").unwrap();
        assert_eq!(doc.current_line, "abc");
        assert_eq!(doc.expected_char(), Some('a'));
    }

    #[test]
    fn advance_through_single_line() {
        let mut doc = Document::from_text("ab").unwrap();
        assert_eq!(doc.expected_char(), Some('a'));
        doc.advance();
        assert_eq!(doc.expected_char(), Some('b'));
        assert_eq!(doc.progress, Progress::Active);
        doc.advance();
        assert_eq!(doc.progress, Progress::Finished);
    }

    #[test]
    fn advance_across_lines() {
        let mut doc = Document::from_text("ab\ncd").unwrap();
        doc.advance(); // past 'a'
        doc.advance(); // past 'b' -> moves to line "cd"
        assert_eq!(doc.current_line, "cd");
        assert_eq!(doc.cursor_position(), 0);
        assert_eq!(doc.expected_char(), Some('c'));
    }

    #[test]
    fn advance_skips_blank_lines() {
        let mut doc = Document::from_text("a\n\nb").unwrap();
        doc.advance(); // past 'a' -> skip blank -> land on "b"
        assert_eq!(doc.current_line, "b");
        assert_eq!(doc.expected_char(), Some('b'));
    }

    #[test]
    fn advance_to_finished() {
        let mut doc = Document::from_text("a").unwrap();
        doc.advance();
        assert_eq!(doc.progress, Progress::Finished);
    }

    #[test]
    fn reset_restores_initial_state() {
        let mut doc = Document::from_text("ab\ncd").unwrap();
        doc.advance();
        doc.advance();
        assert_eq!(doc.current_line, "cd");
        doc.reset();
        assert_eq!(doc.current_line, "ab");
        assert_eq!(doc.cursor_position(), 0);
        assert_eq!(doc.progress, Progress::Active);
    }

    #[test]
    fn upcoming_lines_returns_next_nonempty() {
        let doc = Document::from_text("first\n\nsecond\nthird").unwrap();
        let upcoming = doc.upcoming_lines(3);
        assert_eq!(upcoming, vec!["second", "third"]);
    }

    #[test]
    fn line_progress_tracks_position() {
        let mut doc = Document::from_text("a\nb\nc").unwrap();
        assert_eq!(doc.line_progress(), (1, 3));
        doc.advance(); // finish "a", move to "b"
        assert_eq!(doc.line_progress(), (2, 3));
        doc.advance(); // finish "b", move to "c"
        assert_eq!(doc.line_progress(), (3, 3));
    }

    #[test]
    fn utf8_document_works() {
        let mut doc = Document::from_text("café").unwrap();
        assert_eq!(doc.expected_char(), Some('c'));
        doc.advance();
        assert_eq!(doc.expected_char(), Some('a'));
        doc.advance();
        assert_eq!(doc.expected_char(), Some('f'));
        doc.advance();
        assert_eq!(doc.expected_char(), Some('é'));
        doc.advance();
        assert_eq!(doc.progress, Progress::Finished);
    }

    #[test]
    fn load_nonexistent_file_returns_error() {
        let result = Document::load("/nonexistent/path/to/file.txt");
        assert!(result.is_err());
    }

    // --- App / WPM tests ---

    #[test]
    fn wpm_zero_without_start() {
        let app = App::new();
        assert_eq!(app.wpm(), 0.0);
    }

    #[test]
    fn wpm_zero_under_one_second() {
        let mut app = App::new();
        app.start_time = Some(Instant::now());
        app.correct_count = 100;
        assert_eq!(app.wpm(), 0.0);
    }

    #[test]
    fn wpm_calculation_with_fixed_times() {
        let mut app = App::new();
        let start = Instant::now() - std::time::Duration::from_secs(60);
        app.start_time = Some(start);
        app.end_time = Some(start + std::time::Duration::from_secs(60));
        app.correct_count = 50; // 50 chars / 5 = 10 words in 1 minute = 10 WPM
        let wpm = app.wpm();
        assert!((wpm - 10.0).abs() < 0.1, "expected ~10 WPM, got {wpm}");
    }

    #[test]
    fn elapsed_secs_zero_without_start() {
        let app = App::new();
        assert_eq!(app.elapsed_secs(), 0.0);
    }

    // --- worst_keys tests ---

    #[test]
    fn worst_keys_empty_when_no_stats() {
        let app = App::new();
        assert!(app.worst_keys(5).is_empty());
    }

    #[test]
    fn worst_keys_filters_by_minimum_attempts() {
        let mut app = App::new();
        // Only 1 attempt — should be filtered out (minimum is 2)
        app.key_stats.insert('a', (0, 1));
        assert!(app.worst_keys(5).is_empty());
    }

    #[test]
    fn worst_keys_sorted_by_accuracy() {
        let mut app = App::new();
        app.key_stats.insert('a', (8, 2)); // 80% accuracy
        app.key_stats.insert('b', (5, 5)); // 50% accuracy
        app.key_stats.insert('c', (9, 1)); // 90% accuracy (but 1 miss, 10 total >= 2)
        let worst = app.worst_keys(5);
        assert_eq!(worst[0].0, 'b'); // worst first
        assert_eq!(worst[1].0, 'a');
        assert_eq!(worst[2].0, 'c');
    }

    #[test]
    fn worst_keys_truncates() {
        let mut app = App::new();
        for (i, ch) in "abcdefgh".chars().enumerate() {
            app.key_stats.insert(ch, (5, (i as u32) + 1));
        }
        let worst = app.worst_keys(3);
        assert_eq!(worst.len(), 3);
    }

    // --- Key handling tests ---

    fn key_event(code: KeyCode) -> KeyEvent {
        KeyEvent::new(code, KeyModifiers::NONE)
    }

    fn ctrl_key(c: char) -> KeyEvent {
        KeyEvent::new(KeyCode::Char(c), KeyModifiers::CONTROL)
    }

    #[test]
    fn esc_on_main_menu_returns_quit() {
        let mut app = App::new();
        let quit = app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(quit);
    }

    #[test]
    fn ctrl_f_opens_search() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('f')));
        assert!(app.searching);
    }

    #[test]
    fn search_typing_builds_path() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('f')));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a'))));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('b'))));
        assert_eq!(app.file_path_buf, "ab");
    }

    #[test]
    fn search_backspace_removes_char() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('f')));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('x'))));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Backspace)));
        assert_eq!(app.file_path_buf, "");
    }

    #[test]
    fn search_esc_cancels() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('f')));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('x'))));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(!app.searching);
        assert!(app.file_path_buf.is_empty());
    }

    #[test]
    fn typing_correct_char_advances() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hi").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('h'))));
        assert_eq!(app.correct_count, 1);
        assert_eq!(app.total_count, 1);
        assert!(app.last_correct);
        assert_eq!(app.document.as_ref().unwrap().expected_char(), Some('i'));
    }

    #[test]
    fn typing_wrong_char_sets_error() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hi").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('x'))));
        assert_eq!(app.last_error_char, Some('x'));
        assert!(!app.last_correct);
        assert_eq!(app.total_count, 1);
        assert_eq!(app.correct_count, 0);
        assert_eq!(app.document.as_ref().unwrap().expected_char(), Some('h'));
    }

    #[test]
    fn typing_blocked_during_error() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hi").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('x')))); // error
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('h')))); // blocked
        assert_eq!(app.total_count, 1); // no new keystroke counted
        assert_eq!(app.last_error_char, Some('x'));
    }

    #[test]
    fn backspace_clears_error() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hi").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('x'))));
        assert!(app.last_error_char.is_some());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Backspace)));
        assert!(app.last_error_char.is_none());
    }

    #[test]
    fn menu_navigation() {
        let mut app = App::new();
        app.selected_lesson = 0;
        app.handle_event(InputEvent::Press(key_event(KeyCode::Down)));
        assert_eq!(app.selected_lesson, 1);
        app.handle_event(InputEvent::Press(key_event(KeyCode::Down)));
        assert_eq!(app.selected_lesson, 2);
        app.handle_event(InputEvent::Press(key_event(KeyCode::Up)));
        assert_eq!(app.selected_lesson, 1);
    }

    #[test]
    fn menu_up_at_zero_stays_zero() {
        let mut app = App::new();
        app.selected_lesson = 0;
        app.handle_event(InputEvent::Press(key_event(KeyCode::Up)));
        assert_eq!(app.selected_lesson, 0);
    }

    #[test]
    fn enter_selects_lesson() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(key_event(KeyCode::Enter)));
        assert!(app.document.is_some());
        assert!(!app.lesson_id.is_empty());
    }

    #[test]
    fn tick_clears_expired_highlight() {
        let mut app = App::new();
        app.highlighted_key = Some(KeyCode::Char('A'));
        app.highlight_until = Some(Instant::now() - std::time::Duration::from_secs(1));
        app.handle_event(InputEvent::Tick);
        assert!(app.highlighted_key.is_none());
        assert!(app.highlight_until.is_none());
    }

    #[test]
    fn complete_document_sets_finished() {
        let mut app = App::new();
        app.document = Some(Document::from_text("a").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a'))));
        assert!(app.is_finished());
        assert!(app.end_time.is_some());
    }

    // --- chrono_now ---

    #[test]
    fn chrono_now_format() {
        let ts = chrono_now();
        // Should be "YYYY-MM-DDThh:mm:ss" (19 chars)
        assert_eq!(ts.len(), 19);
        assert_eq!(ts.as_bytes()[4], b'-');
        assert_eq!(ts.as_bytes()[7], b'-');
        assert_eq!(ts.as_bytes()[10], b'T');
        assert_eq!(ts.as_bytes()[13], b':');
        assert_eq!(ts.as_bytes()[16], b':');
    }

    // --- Document::load happy path ---

    #[test]
    fn load_real_file() {
        let result = Document::load("sample.txt");
        assert!(result.is_ok());
        let doc = result.unwrap();
        assert_eq!(doc.progress, Progress::Active);
        assert!(!doc.current_line.is_empty());
    }

    // --- restart ---

    #[test]
    fn restart_resets_all_state() {
        let mut app = App::new();
        app.document = Some(Document::from_text("abc").unwrap());
        // Type two correct chars
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a'))));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('b'))));
        assert_eq!(app.correct_count, 2);
        assert!(app.start_time.is_some());

        // Restart via Ctrl-R
        app.handle_event(InputEvent::Press(ctrl_key('r')));
        assert_eq!(app.correct_count, 0);
        assert_eq!(app.total_count, 0);
        assert!(app.start_time.is_none());
        assert!(app.end_time.is_none());
        assert!(app.key_stats.is_empty());
        assert!(app.last_error_char.is_none());
        let doc = app.document.as_ref().unwrap();
        assert_eq!(doc.cursor_position(), 0);
        assert_eq!(doc.expected_char(), Some('a'));
    }

    // --- key stats tracking ---

    #[test]
    fn key_stats_tracks_hits_and_misses() {
        let mut app = App::new();
        app.document = Some(Document::from_text("aa").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a')))); // hit
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('x')))); // miss
        app.handle_event(InputEvent::Press(key_event(KeyCode::Backspace))); // clear error
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a')))); // hit

        let stats = app.key_stats.get(&'a').unwrap();
        assert_eq!(stats.0, 2); // 2 hits
        assert_eq!(stats.1, 1); // 1 miss
    }

    // --- space highlight ---

    #[test]
    fn space_highlight_uses_space_char() {
        let mut app = App::new();
        app.document = Some(Document::from_text("a b").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a'))));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char(' '))));
        assert_eq!(app.highlighted_key, Some(KeyCode::Char(' ')));
    }

    // --- menu bounds ---

    #[test]
    fn menu_down_at_last_stays_at_last() {
        let mut app = App::new();
        let last = app.menu_item_count() - 1;
        app.selected_lesson = last;
        app.handle_event(InputEvent::Press(key_event(KeyCode::Down)));
        assert_eq!(app.selected_lesson, last);
    }

    // --- history view ---

    #[test]
    fn h_key_opens_history() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('h'))));
        assert!(app.viewing_history);
    }

    #[test]
    fn esc_closes_history() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('h'))));
        assert!(app.viewing_history);
        let quit = app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(!quit);
        assert!(!app.viewing_history);
    }

    #[test]
    fn history_scroll_navigation() {
        let mut app = App::new();
        // Manually set up history with entries
        app.history = vec![
            crate::history::SessionRecord {
                timestamp: "t1".into(),
                wpm: 30.0,
                accuracy: 90.0,
                correct: 10,
                total: 11,
                duration_secs: 60.0,
                completed: true,
                id: String::new(),
            },
            crate::history::SessionRecord {
                timestamp: "t2".into(),
                wpm: 40.0,
                accuracy: 95.0,
                correct: 20,
                total: 21,
                duration_secs: 60.0,
                completed: true,
                id: String::new(),
            },
        ];
        app.history_scroll = 1;
        app.viewing_history = true;

        app.handle_event(InputEvent::Press(key_event(KeyCode::Up)));
        assert_eq!(app.history_scroll, 0);
        app.handle_event(InputEvent::Press(key_event(KeyCode::Up)));
        assert_eq!(app.history_scroll, 0); // stays at 0
        app.handle_event(InputEvent::Press(key_event(KeyCode::Down)));
        assert_eq!(app.history_scroll, 1);
        app.handle_event(InputEvent::Press(key_event(KeyCode::Down)));
        assert_eq!(app.history_scroll, 1); // stays at last
    }

    // --- save_on_exit ---

    #[test]
    fn save_on_exit_does_nothing_without_document() {
        let app = App::new();
        app.save_on_exit();
    }

    #[test]
    fn save_on_exit_does_nothing_when_finished() {
        let mut app = App::new();
        app.document = Some(Document::from_text("a").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a'))));
        assert!(app.is_finished());
        app.save_on_exit();
    }

    // --- vim-style keys ---

    #[test]
    fn j_k_navigate_menu() {
        let mut app = App::new();
        app.selected_lesson = 0;
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('j'))));
        assert_eq!(app.selected_lesson, 1);
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('k'))));
        assert_eq!(app.selected_lesson, 0);
    }

    #[test]
    fn r_restarts_on_finished_screen() {
        let mut app = App::new();
        app.document = Some(Document::from_text("a").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('a'))));
        assert!(app.is_finished());

        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('r'))));
        assert!(!app.is_finished());
        assert_eq!(app.correct_count, 0);
        assert_eq!(app.document.as_ref().unwrap().expected_char(), Some('a'));
    }

    // --- search ignores ctrl keys ---

    #[test]
    fn search_ignores_ctrl_chars() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('f'))); // open search
        app.handle_event(InputEvent::Press(ctrl_key('a'))); // ctrl-a ignored
        assert_eq!(app.file_path_buf, "");
    }

    // --- Esc from active typing ---

    #[test]
    fn esc_from_active_typing_opens_pause_menu() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hello").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('h'))));
        assert!(app.document.is_some());
        assert!(app.start_time.is_some());

        let quit = app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(!quit);
        assert!(app.paused);
        assert!(app.document.is_some());
    }

    #[test]
    fn esc_from_pause_menu_goes_to_menu() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hello").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('h'))));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(app.paused);

        app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(!app.paused);
        assert!(app.document.is_none());
    }

    #[test]
    fn pause_menu_esc_returns_to_menu() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hello").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Char('h'))));
        app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(app.paused);

        app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(!app.paused);
        assert!(app.document.is_none());
    }

    #[test]
    fn pause_menu_quit() {
        let mut app = App::new();
        app.document = Some(Document::from_text("hello").unwrap());
        app.handle_event(InputEvent::Press(key_event(KeyCode::Esc)));
        assert!(app.paused);

        let quit = app.handle_event(InputEvent::Press(key_event(KeyCode::Char('q'))));
        assert!(quit);
    }

    // --- Ctrl-R no-op without document ---

    #[test]
    fn ctrl_r_without_document_is_noop() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('r')));
        assert!(app.document.is_none());
        assert_eq!(app.correct_count, 0);
    }

    // --- search submit ---

    #[test]
    fn search_enter_invalid_path_sets_error() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('f')));
        for c in "/no/such/file.txt".chars() {
            app.handle_event(InputEvent::Press(key_event(KeyCode::Char(c))));
        }
        app.handle_event(InputEvent::Press(key_event(KeyCode::Enter)));
        assert!(!app.searching);
        assert!(app.error.is_some());
        assert!(app.document.is_none());
    }

    #[test]
    fn search_enter_valid_file_loads_document() {
        let mut app = App::new();
        app.handle_event(InputEvent::Press(ctrl_key('f')));
        for c in "sample.txt".chars() {
            app.handle_event(InputEvent::Press(key_event(KeyCode::Char(c))));
        }
        app.handle_event(InputEvent::Press(key_event(KeyCode::Enter)));
        assert!(!app.searching);
        assert!(app.error.is_none());
        assert!(app.document.is_some());
        assert_eq!(app.lesson_id, "sample.txt");
    }

    // --- word practice ---

    #[test]
    fn select_word_practice_from_menu() {
        let mut app = App::new();
        let lesson_count = crate::lessons::lesson_count();
        // Navigate to the first word practice entry (right after lessons)
        app.selected_lesson = lesson_count;
        app.handle_event(InputEvent::Press(key_event(KeyCode::Enter)));
        assert!(app.document.is_some());
        assert!(app.lesson_title.starts_with("Random Words"));
        let doc = app.document.as_ref().unwrap();
        let word_count: usize = doc.lines.iter().map(|l| l.split_whitespace().count()).sum();
        assert_eq!(word_count, 50);
    }

    #[test]
    fn menu_includes_word_lists() {
        let app = App::new();
        let lesson_count = crate::lessons::lesson_count();
        let word_list_count = crate::words::WordList::all().len();
        assert_eq!(app.menu_item_count(), lesson_count + word_list_count);
    }
}
