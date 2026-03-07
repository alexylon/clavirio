# FerroType

<div align="center"><img src="/assets/images/screenshot.png" style="width: 800px;" alt="FerroType"></div>

A terminal typing tutor built with Rust and [ratatui](https://ratatui.rs). Practice with built-in lessons or any text file while a virtual keyboard tracks your keystrokes in real time.

## Features

- **12 built-in lessons** — progressive drills from home row basics to full paragraphs, ordered by difficulty
- **Live stats** — WPM, elapsed time, keystrokes, and line progress update as you type
- **Virtual keyboard** — highlights the expected next key, including Shift for capitals and symbols; adapts to macOS and PC layouts
- **Error feedback** — wrong keystrokes are shown inline and block progress until corrected with Backspace
- **Completion summary** — final WPM, accuracy percentage, and your weakest keys
- **Session history** — results saved to `~/.ferrotype/history.json` with per-session averages; incomplete sessions are marked
- **Custom text** — load any text file via `Ctrl-F` or as a CLI argument

## Lessons

| # | Lesson | Focus |
|---|--------|-------|
| 1 | f j d k | Index fingers |
| 2 | a s d f j k l ; | Home row |
| 3 | g h (home row) | Home row complete |
| 4 | e i r u | Top row reach |
| 5 | q w e r t y u i o p | Full top row |
| 6 | z x c v b n m , . | Bottom row |
| 7 | All Letters | Pangrams |
| 8 | Capitals & Shift | Mixed case |
| 9 | 0-9 Numbers | Numbers in context |
| 10 | Punctuation & Symbols | Special characters |
| 11 | Common Words | High-frequency words |
| 12 | Full Paragraphs | Real-world text |

## Build & Run

```
cargo build --release
cargo run
```

Optionally pass a file directly:

```
cargo run -- sample.txt
```

## Terminal Size

FerroType is a terminal UI application — your terminal window should be large enough to display all elements (text panel, keyboard, stats).
On a laptop screen this usually means a maximized terminal window.

## Controls

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate lesson menu |
| `Enter` | Start selected lesson |
| `h` | View session history (main menu) |
| `Ctrl-F` | Open file path input |
| `Ctrl-R` | Restart current text |
| `Esc` | Back to main menu / quit |
| `Backspace` | Correct a mistake |
| `r` | Restart (completion screen) |
