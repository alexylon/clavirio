<div align="center"><img src="/assets/images/icon_gunmetal_scanlines_512.png" style="width: 85px;" alt="clavirio"></div>

<h1 align="center"><code>clavirio</code></h1>

![](https://github.com/alexylon/clavirio/actions/workflows/rust.yml/badge.svg) &nbsp; [![Crates.io](https://img.shields.io/crates/v/clavirio.svg?color=blue)](https://crates.io/crates/clavirio)

[www.clavir.io](https://www.clavir.io)

### Learn touch typing without leaving the terminal.

Progressive lessons for QWERTY, Dvorak & Colemak. 
Real-time stats, a virtual keyboard with finger hints, and session history — in a fast, 
lightweight binary built with Rust and [ratatui](https://ratatui.rs).

*clavirio* — from Latin *clavis* (key).

<div align="center"><img src="/assets/images/screenshot_dark.png" style="width: 800px;" alt="clavirio"></div>

<div align="center"><img src="/assets/images/screenshot_light.png" style="width: 800px;" alt="clavirio"></div>

## Features

- **15 built-in lessons** — progressive drills from home row to full paragraphs and code; lessons 1–9 are layout-specific, 10–15 are shared
- **Live stats** — WPM, accuracy, errors, time, and line progress in a one-row status bar
- **3 keyboard layouts** — QWERTY, Dvorak, and Colemak
- **Virtual keyboard** — highlights the next key with finger hints (**P**inky, **R**ing, **M**iddle, **I**ndex, **T**humb)
- **Error feedback** — wrong keystrokes shown inline, blocked until corrected with Backspace
- **Session history** — saved to `~/.clavirio/history.json` with per-lesson tracking and averages
- **Progress tracking** — remembers your last lesson; advances automatically on completion
- **Custom text** — load any file via `Ctrl-F` or as a CLI argument
- **Dark & light themes**

## Lessons

Lessons 1–9 are **layout-specific** (each layout has its own drills matched to that layout's finger positions). Lessons 10–15 are **shared** across all layouts.

| # | Lesson | QWERTY | Dvorak | Colemak |
|---|--------|--------|--------|---------|
| 1 | Index Keys | f j | u h | t n |
| 2 | Middle Keys | d k (+ f j) | e t (+ u h) | s e (+ t n) |
| 3 | Ring & Pinky | s l ; (+ f j d k) | o n s (+ a) | r i o (+ a) |
| 4 | Home Row | a s d f j k l ; | a o e u i d h t n s | a r s t d h n e i o |
| 5 | Home Reach | g h | i d | d h |
| 6 | Top Intro | e i r u | p c r l | f p l u |
| 7 | Top Row | q w e r t y u i o p | ' , . p y f g c r l | q w f p g j l u y ; |
| 8 | Bottom Row | z x c v b n m , . | ; q j k x b m w v z | z x c v b k m , . |
| 9 | All Letters | Pangrams | Pangrams | Pangrams |

| # | Lesson | Focus |
|---|--------|-------|
| 10 | Capitals & Shift | Mixed case |
| 11 | 0-9 Numbers | Numbers in context |
| 12 | Punctuation & Symbols | Special characters |
| 13 | Common Words | High-frequency words |
| 14 | Full Paragraphs | Real-world text |
| 15 | Code (Rust) | Programming syntax |

## Build & Run

```
cargo build --release
cargo run
```

Optionally pass a file directly:

```
cargo run -- sample.txt
```

## Terminal size

Best with a maximized terminal window so text, keyboard, and stats fit comfortably.

## Controls

### Menu

| Key | Action |
|-----|--------|
| `↑`/`↓` `k`/`j` | Navigate lessons / history |
| `Enter` | Start lesson |
| `l` | Cycle layout |
| `1`–`4` | Toggle fingers / hints / keyboard / theme |
| `h` | Session history |
| `Ctrl-F` | Load file |
| `q` / `Esc` | Quit |

### Typing

| Key | Action |
|-----|--------|
| `Backspace` | Correct mistake |
| `Esc` | Pause menu |
| `Ctrl-R` | Restart lesson |

### Pause menu

| Key | Action |
|-----|--------|
| `Space` | Resume |
| `r` | Restart lesson |
| `n` | Next lesson |
| `q` | Quit |
| `Esc` | Back to menu |

## Settings

Preferences are stored in `~/.clavirio/settings.toml` and saved automatically.

```toml
[keyboard]
layout = "qwerty"      # qwerty, dvorak, colemak

[display]
show_keyboard = true
show_hints = true
show_fingers = true
theme = "dark"          # dark, light
```

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
