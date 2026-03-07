# FerroType

<div align="center"><img src="/assets/screenshot.png" style="width: 800px;" alt="Sofos Code"></div>

A terminal typing tutor built with Rust and [ratatui](https://ratatui.rs). Practice with built-in lessons or any text file while a virtual keyboard tracks your keystrokes in real time.

## Features

- **Built-in lessons** — pangrams, home row drills, common words, numbers & symbols
- **Live stats** — WPM, elapsed time, accuracy, and line progress update as you type
- **Virtual keyboard** — highlights the expected next key; shows Shift for capitals and symbols
- **Error feedback** — wrong keystrokes are shown inline and block progress until corrected with Backspace
- **Completion summary** — final WPM, accuracy, and weakest keys
- **Session history** — results saved to `~/.ferrotype/history.json`, viewable from the main menu
- **Custom text** — load any text file via `Ctrl-F` or as a CLI argument

## Build & Run

```
cargo build --release
cargo run
```

Optionally pass a file directly:

```
cargo run -- sample.txt
```

## Controls

| Key | Action |
|---|---|
| `1`–`4` | Select a built-in lesson (from main menu) |
| `h` | View session history (from main menu) |
| `Ctrl-F` | Open file path input |
| `Ctrl-R` | Restart current text |
| `Esc` | Back to main menu / quit |
| `Backspace` | Correct a mistake |
| `r` | Restart (on completion screen) |
