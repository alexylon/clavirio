mod app;
mod history;
mod input;
mod keyboard;
mod lessons;
mod settings;
mod storage;
mod ui;
mod words;

use std::io::{Result, stdout};

use clap::Parser;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode,
};
use ratatui::prelude::*;

use app::App;
use input::run_input_loop;
use keyboard::{build_keyboard_rows, build_keycode_grid_map};
use settings::load_settings;
use ui::{compute_regions, draw};

/// A terminal typing tutor with virtual keyboard, built-in lessons, and session tracking
#[derive(Parser)]
#[command(version, about)]
struct Cli {
    /// File path to load as typing text
    #[arg(short, long)]
    file: Option<String>,

    /// Start practice mode with N random words (default: 100)
    #[arg(short, long)]
    words: Option<Option<usize>>,

    /// Start practice mode for N seconds (e.g. 30, 60)
    #[arg(short, long)]
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

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    enable_raw_mode()?;
    execute!(stdout(), EnterAlternateScreen)?;

    let default_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = execute!(std::io::stderr(), LeaveAlternateScreen);
        let _ = disable_raw_mode();
        default_hook(info);
    }));

    let result = run_app(cli).await;

    execute!(stdout(), LeaveAlternateScreen)?;
    disable_raw_mode()?;

    if let Some(message) = result? {
        eprintln!("{message}");
    }
    Ok(())
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
        // On Windows, graceful shutdown relies on Esc/Ctrl-C
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
/// terminal is restored.
async fn run_app(cli: Cli) -> Result<Option<String>> {
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    terminal.clear()?;

    let mut settings = load_settings();
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

    // Handle CLI flags: --time > --words > --file
    let list = cli.list;
    if let Some(secs) = cli.time {
        app.start_timed_practice(secs, list);
    } else if let Some(maybe_count) = cli.words {
        app.word_count = maybe_count.unwrap_or(App::DEFAULT_WORD_COUNT);
        app.start_word_practice(list);
    } else if let Some(path) = cli.file {
        app.load_file(&path);
    }

    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    tokio::spawn(run_input_loop(tx));

    terminal.draw(|frame| {
        let regions = compute_regions(frame.area(), app.show_keyboard);
        draw(frame, &app, &regions, &rows, &grid_map);
    })?;

    loop {
        let event = tokio::select! {
            ev = rx.recv() => ev,
            _ = shutdown_signal() => None,
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
            || (app.menu_mode == crate::app::MenuMode::Practice) != settings.display.practice_mode
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
            if let Err(e) = settings::save_settings(&settings) {
                app.show_error(format!("Couldn't save the settings: {e}"));
            }
            if layout_changed {
                rows = build_keyboard_rows(settings.keyboard.layout);
                grid_map = build_keycode_grid_map(&rows);
            }
        }

        terminal.draw(|frame| {
            let regions = compute_regions(frame.area(), app.show_keyboard);
            draw(frame, &app, &regions, &rows, &grid_map);
        })?;
    }

    Ok(app.save_on_exit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cli_override_saved_only_after_change_in_app() {
        let mut cli_override = true;
        assert!(!toggle_to_save(&mut cli_override, true));
        assert!(!toggle_to_save(&mut cli_override, false));
        assert!(toggle_to_save(&mut cli_override, true));
    }
}
