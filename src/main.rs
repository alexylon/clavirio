mod app;
mod history;
mod input;
mod keyboard;
mod lessons;
mod settings;
mod ui;
mod words;

use std::io::{stdout, Result};

use clap::Parser;
use ratatui::crossterm::execute;
use ratatui::crossterm::terminal::{
    disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen,
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

    /// Word list to use: "200" or "1k"
    #[arg(short, long, default_value = "200")]
    list: String,
}

fn parse_word_list(s: &str) -> std::result::Result<words::WordList, String> {
    match s {
        "200" => Ok(words::WordList::English200),
        "1k" => Ok(words::WordList::English1k),
        _ => Err(format!("Unknown word list '{s}'. Use '200' or '1k'.")),
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

    result
}

async fn shutdown_signal() {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};
        let mut term = signal(SignalKind::terminate()).expect("SIGTERM handler");
        let mut hup = signal(SignalKind::hangup()).expect("SIGHUP handler");
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

async fn run_app(cli: Cli) -> Result<()> {
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
    app.selected_lesson = crate::history::resume_lesson(app.layout);

    // Handle CLI flags: --time > --words > --file
    let list = parse_word_list(&cli.list).unwrap_or(words::WordList::English200);
    if let Some(secs) = cli.time {
        app.start_timed_practice(secs, list);
    } else if let Some(maybe_count) = cli.words {
        app.word_count = maybe_count.unwrap_or(100);
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
            app.save_on_exit();
            break;
        };

        if app.handle_event(event) {
            break;
        }

        let display_changed = app.show_keyboard != settings.display.show_keyboard
            || app.show_hints != settings.display.show_hints
            || app.show_fingers != settings.display.show_fingers
            || app.theme != settings.display.theme;
        let layout_changed = app.layout != settings.keyboard.layout;

        if layout_changed || display_changed {
            settings.keyboard.layout = app.layout;
            settings.display.show_keyboard = app.show_keyboard;
            settings.display.show_hints = app.show_hints;
            settings.display.show_fingers = app.show_fingers;
            settings.display.theme = app.theme;
            settings::save_settings(&settings);
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

    Ok(())
}
