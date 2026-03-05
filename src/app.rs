use std::fs;
use std::path::Path;

use crossterm::event::KeyCode;

use crate::input::{Action, InputEvent, Mode};

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
    pub progress: Progress,
}

impl Document {
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

        let content =
            fs::read_to_string(&resolved).map_err(|e| format!("Cannot read file: {e}"))?;

        let lines: Vec<String> = content.lines().map(String::from).collect();

        if lines.is_empty() {
            return Err("File is empty".into());
        }

        let first_line = lines
            .iter()
            .find(|l| !l.is_empty())
            .cloned()
            .unwrap_or_default();

        Ok(Self {
            lines,
            line_idx: 0,
            char_idx: 0,
            current_line: first_line,
            progress: Progress::Active,
        })
    }

    pub fn cursor_position(&self) -> usize {
        self.char_idx
    }

    pub fn expected_char(&self) -> Option<char> {
        self.current_line.chars().nth(self.char_idx)
    }

    pub fn advance(&mut self) {
        self.char_idx += 1;

        if self.char_idx >= self.current_line.len() {
            self.line_idx += 1;
            loop {
                match self.lines.get(self.line_idx) {
                    Some(line) if !line.is_empty() => {
                        self.char_idx = 0;
                        self.current_line = line.clone();
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
    pub highlighted_key: Option<KeyCode>,
    pub show_highlight: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            document: None,
            file_path_buf: String::new(),
            searching: false,
            error: None,
            correct_count: 0,
            total_count: 0,
            last_correct: false,
            highlighted_key: None,
            show_highlight: false,
        }
    }

    pub fn handle_event(&mut self, event: InputEvent) -> bool {
        match event {
            InputEvent::Tick => {
                self.show_highlight = false;
                self.highlighted_key = None;
                false
            }
            InputEvent::Press(action) => self.handle_action(action),
        }
    }

    fn handle_action(&mut self, action: Action) -> bool {
        match action.mode {
            Mode::Quit => return true,

            Mode::OpenSearch => {
                self.searching = true;
                self.file_path_buf.clear();
                self.error = None;
            }

            Mode::Search => {
                if action.key.modifiers == crossterm::event::KeyModifiers::CONTROL {
                    return false;
                }
                match action.key.code {
                    KeyCode::Char(c) => self.file_path_buf.push(c),
                    KeyCode::Backspace => {
                        self.file_path_buf.pop();
                    }
                    _ => {}
                }
            }

            Mode::SubmitSearch => {
                self.searching = false;
                let path = self.file_path_buf.clone();
                self.file_path_buf.clear();
                match Document::load(&path) {
                    Ok(doc) => {
                        self.document = Some(doc);
                        self.error = None;
                        self.correct_count = 0;
                        self.total_count = 0;
                    }
                    Err(e) => self.error = Some(e),
                }
            }

            Mode::CancelSearch => {
                self.searching = false;
                self.file_path_buf.clear();
            }

            Mode::Typing => {
                if let KeyCode::Char(typed) = action.key.code {
                    self.handle_typed_char(typed);
                }
            }
        }
        false
    }

    fn handle_typed_char(&mut self, typed: char) {
        let expected = match self.document.as_ref().and_then(|d| d.expected_char()) {
            Some(c) => c,
            None => return,
        };

        self.total_count += 1;

        let is_match = typed == expected
            || (expected.is_lowercase() && typed == expected)
            || (expected.is_uppercase() && typed == expected);

        if is_match {
            self.correct_count += 1;
            self.last_correct = true;
            if let Some(doc) = self.document.as_mut() {
                doc.advance();
            }
        } else {
            self.last_correct = false;
        }

        self.show_highlight = true;
        let display_char = if typed.is_whitespace() {
            ' '
        } else {
            typed.to_ascii_uppercase()
        };
        self.highlighted_key = Some(KeyCode::Char(display_char));
    }
}
