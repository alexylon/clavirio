mod app;
mod history;
mod input;
mod keyboard;
mod lessons;
mod settings;
mod storage;
mod ui;
mod words;

use std::collections::HashMap;
use std::io::{Result, Stdout, Write, stdout};

use clap::Parser;
use ratatui::crossterm::event::KeyCode;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;

use app::{App, append_line};
use input::run_input_loop;
use keyboard::{GridCoord, KeyDef, build_keyboard_rows, build_keycode_grid_map};
use settings::{Settings, load_settings};
use ui::{compute_regions, draw, keyboard_fits};

// Limits for -w and -t, so a huge number can't use up the memory while the text
// is generated
const MAX_WORDS: u64 = 10_000;
const MAX_SECONDS: u64 = 3_600;

/// A terminal typing tutor with virtual keyboard, built-in lessons, and session tracking
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// File path to load as typing text
    #[arg(short, long)]
    file: Option<String>,

    /// Start practice mode with N random words, up to 10000 (default: 100)
    #[arg(
        short,
        long,
        value_parser = clap::builder::RangedU64ValueParser::<usize>::new().range(1..=MAX_WORDS)
    )]
    words: Option<Option<usize>>,

    /// Start practice mode for N seconds, up to 3600 (e.g. 30, 60)
    #[arg(short, long, value_parser = clap::value_parser!(u64).range(1..=MAX_SECONDS))]
    time: Option<u64>,

    /// Word list: "200", "1k", "rust", "python", "javascript"/"js", "go", "c"/"cpp", "java", "html"/"css"
    #[arg(short, long, default_value = "200", value_parser = parse_word_list)]
    list: words::WordList,

    /// Include punctuation in word practice (this run only)
    #[arg(short, long)]
    punctuation: bool,

    /// Include numbers in word practice (this run only)
    #[arg(short, long)]
    numbers: bool,
}

fn parse_word_list(s: &str) -> std::result::Result<words::WordList, String> {
    match s {
        "200" => Ok(words::WordList::English200),
        "1k" => Ok(words::WordList::English1k),
        "rust" => Ok(words::WordList::Rust),
        "python" => Ok(words::WordList::Python),
        "javascript" | "js" => Ok(words::WordList::JavaScript),
        "go" => Ok(words::WordList::Go),
        "c" | "cpp" | "c++" => Ok(words::WordList::CCpp),
        "java" => Ok(words::WordList::Java),
        "html" | "css" => Ok(words::WordList::HtmlCss),
        _ => Err(format!(
            "Unknown word list '{s}'. Use '200', '1k', 'rust', 'python', 'javascript', 'go', 'c', 'java', or 'html'."
        )),
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    enable_raw_mode()?;
    if let Err(e) = execute!(stdout(), EnterAlternateScreen) {
        let _ = disable_raw_mode();
        return Err(e);
    }

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        restore_terminal();
        default_hook(info);
    }));

    // The input thread can hang reading a closed terminal, and dropping the
    // runtime would wait for it forever. So the runtime is shut down without
    // waiting, even after a panic, such as ratatui failing to write to that
    // terminal.
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        runtime.block_on(run_app(cli))
    }));
    runtime.shutdown_background();
    let result = result.unwrap_or_else(|panic| std::panic::resume_unwind(panic));

    restore_terminal();
    let (message, session) = result?;
    if let Some(message) = message {
        // Unlike eprintln!, this doesn't panic if stderr is a terminal that has closed
        let _ = writeln!(std::io::stderr(), "{message}");
    }
    session
}

/// Leaves the alternate screen and raw mode. Errors are ignored, as the terminal
/// may have closed, and what comes after must still run.
fn restore_terminal() {
    let _ = execute!(stdout(), LeaveAlternateScreen);
    let _ = disable_raw_mode();
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};
        let (Ok(mut term), Ok(mut hup)) = (
            signal(SignalKind::terminate()),
            signal(SignalKind::hangup()),
        ) else {
            // Without the handlers, quitting relies on Esc/Ctrl-C, as on Windows
            return std::future::pending().await;
        };
        tokio::select! {
            _ = term.recv() => {}
            _ = hup.recv() => {}
        }
    }
    #[cfg(not(unix))]
    {
        // On Windows, quitting relies on Esc/Ctrl-C
        std::future::pending::<()>().await;
    }
}

/// The value to save for a toggle that a CLI flag may have turned on for this
/// run only: the flag's effect isn't saved, and changing the toggle in the app
/// ends the override.
fn toggle_to_save(cli_override: &mut bool, value: bool) -> bool {
    *cli_override &= value;
    value && !*cli_override
}

/// Runs the app until it quits, and returns a message to print once the
/// terminal is restored, and the error that ended the session early, if any.
async fn run_app(cli: Cli) -> Result<(Option<String>, Result<()>)> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    // Saving over settings that couldn't be read would replace them with the defaults
    let loaded = load_settings();
    let can_save_settings = loaded.is_ok();
    let (mut settings, settings_message) = match loaded {
        Ok(Some(settings)) => (settings, None),
        Ok(None) => (
            Settings::default(),
            Some("Couldn't parse the settings, so the defaults are used".to_string()),
        ),
        Err(e) => (
            Settings::default(),
            Some(format!(
                "Couldn't read the settings: {e}\nUsing the defaults; changes won't be saved this time"
            )),
        ),
    };
    let mut rows = build_keyboard_rows(settings.keyboard.layout);
    let mut grid_map = build_keycode_grid_map(&rows);
    let mut app = App::new();
    app.layout = settings.keyboard.layout;
    app.show_keyboard = settings.display.show_keyboard;
    app.show_hints = settings.display.show_hints;
    app.show_fingers = settings.display.show_fingers;
    app.theme = settings.display.theme;
    app.include_punctuation = settings.display.include_punctuation;
    app.include_numbers = settings.display.include_numbers;
    app.error_stop = settings.display.error_stop;
    if settings.display.practice_mode {
        app.menu_mode = crate::app::MenuMode::Practice;
    }
    app.selected_lesson = crate::history::resume_lesson(app.layout);

    // CLI flags override the saved settings for this run only
    let mut punctuation_override = cli.punctuation && !app.include_punctuation;
    let mut numbers_override = cli.numbers && !app.include_numbers;
    app.include_punctuation |= cli.punctuation;
    app.include_numbers |= cli.numbers;

    // --time wins over --words, and --words over --file
    let list = cli.list;
    if let Some(secs) = cli.time {
        app.start_timed_practice(secs, list);
    } else if let Some(maybe_count) = cli.words {
        app.start_word_practice(list, maybe_count.unwrap_or(App::DEFAULT_WORD_COUNT));
    } else if let Some(path) = cli.file {
        app.load_file(&path);
    }
    if let Some(message) = settings_message {
        app.show_error(message);
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(run_input_loop(tx));

    // The message showing in the last frame drawn, so one shown since but never
    // drawn can be printed after quitting
    let mut drawn_error = None;
    // A failed draw, such as on a terminal that has closed, ends the session
    // here, so that it is still saved below
    let session: Result<()> = async {
        app.keyboard_fits = draw_frame(&mut terminal, &app, &rows, &grid_map)?;
        drawn_error.clone_from(&app.error);

        // Created once, so a signal that arrives while an event is handled isn't missed
        let shutdown = shutdown_signal();
        tokio::pin!(shutdown);

        loop {
            let event = tokio::select! {
                ev = rx.recv() => ev,
                _ = &mut shutdown => None,
            };

            let Some(event) = event else {
                break;
            };

            if app.handle_event(event) {
                break;
            }

            let include_punctuation =
                toggle_to_save(&mut punctuation_override, app.include_punctuation);
            let include_numbers = toggle_to_save(&mut numbers_override, app.include_numbers);
            let display_changed = app.show_keyboard != settings.display.show_keyboard
                || app.show_hints != settings.display.show_hints
                || app.show_fingers != settings.display.show_fingers
                || app.theme != settings.display.theme
                || include_punctuation != settings.display.include_punctuation
                || include_numbers != settings.display.include_numbers
                || (app.menu_mode == crate::app::MenuMode::Practice)
                    != settings.display.practice_mode
                || app.error_stop != settings.display.error_stop;
            let layout_changed = app.layout != settings.keyboard.layout;

            if layout_changed || display_changed {
                settings.keyboard.layout = app.layout;
                settings.display.show_keyboard = app.show_keyboard;
                settings.display.show_hints = app.show_hints;
                settings.display.show_fingers = app.show_fingers;
                settings.display.theme = app.theme;
                settings.display.include_punctuation = include_punctuation;
                settings.display.include_numbers = include_numbers;
                settings.display.practice_mode = app.menu_mode == crate::app::MenuMode::Practice;
                settings.display.error_stop = app.error_stop;
                if can_save_settings {
                    match settings::save_settings(&settings) {
                        Ok(Some(notice)) => app.show_error(notice),
                        Ok(None) => {}
                        Err(e) => app.show_error(format!("Couldn't save the settings: {e}")),
                    }
                }
                if layout_changed {
                    rows = build_keyboard_rows(settings.keyboard.layout);
                    grid_map = build_keycode_grid_map(&rows);
                }
            }

            app.keyboard_fits = draw_frame(&mut terminal, &app, &rows, &grid_map)?;
            drawn_error.clone_from(&app.error);
        }
        Ok(())
    }
    .await;

    let mut message = app
        .error
        .take()
        .and_then(|shown| unseen_lines(shown, drawn_error));
    if let Some(saved) = app.save_on_exit() {
        append_line(&mut message, saved);
    }
    Ok((message, session))
}

/// The lines of the message `shown` that the last frame drawn didn't show. New
/// lines are only ever added below a message.
fn unseen_lines(shown: String, drawn: Option<String>) -> Option<String> {
    let Some(drawn) = drawn else {
        return Some(shown);
    };
    if shown == drawn {
        return None;
    }
    match shown
        .strip_prefix(&drawn)
        .and_then(|rest| rest.strip_prefix('\n'))
    {
        Some(added) => Some(added.to_string()),
        None => Some(shown),
    }
}

/// Draws the app, and returns whether the keyboard fitted in the frame
fn draw_frame(
    terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    app: &App,
    rows: &[Vec<KeyDef>],
    grid_map: &HashMap<KeyCode, GridCoord>,
) -> Result<bool> {
    let frame = terminal.draw(|frame| {
        let regions = compute_regions(frame.area(), app.show_keyboard);
        draw(frame, app, &regions, rows, grid_map);
    })?;
    Ok(keyboard_fits(frame.area))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_rejects_word_and_second_counts_out_of_range() {
        let parses = |flag: &str, value: u64| {
            Cli::try_parse_from(["clavirio", flag, &value.to_string()]).is_ok()
        };

        assert!(Cli::try_parse_from(["clavirio", "-w"]).is_ok());
        assert!(parses("-w", MAX_WORDS) && parses("-t", MAX_SECONDS));
        assert!(!parses("-w", 0) && !parses("-w", MAX_WORDS + 1));
        assert!(!parses("-t", 0) && !parses("-t", MAX_SECONDS + 1));
    }

    #[test]
    fn only_lines_never_drawn_are_unseen() {
        let unseen = |shown: &str, drawn: Option<&str>| {
            unseen_lines(shown.to_string(), drawn.map(String::from))
        };

        assert_eq!(unseen("a", None).as_deref(), Some("a"));
        assert_eq!(unseen("a", Some("a")), None);
        assert_eq!(unseen("a\nb", Some("a")).as_deref(), Some("b"));
        assert_eq!(unseen("b", Some("a")).as_deref(), Some("b"));
    }

    #[test]
    fn cli_override_saved_only_after_change_in_app() {
        let mut cli_override = true;
        assert!(!toggle_to_save(&mut cli_override, true));
        assert!(!toggle_to_save(&mut cli_override, false));
        assert!(toggle_to_save(&mut cli_override, true));
    }
}
