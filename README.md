<div align="center"><img src="/assets/images/icon_gunmetal_scanlines_512.png" style="width: 85px;" alt="clavirio"></div>

<h1 align="center"><code>clavirio</code></h1>

![](https://github.com/alexylon/clavirio/actions/workflows/rust.yml/badge.svg) &nbsp; [![Crates.io](https://img.shields.io/crates/v/clavirio.svg?color=blue)](https://crates.io/crates/clavirio)

[www.clavir.io](https://www.clavir.io)

### Learn touch typing without leaving the terminal.

Progressive lessons for QWERTY, Dvorak & Colemak.
Real-time stats, a virtual keyboard with finger hints, and session history — in a fast,
lightweight binary built with Rust and [ratatui](https://ratatui.rs).

*clavirio* — from Latin *clavis* (key).

<div align="center"><img src="/assets/images/screenshot_dark.webp" style="width: 800px;" alt="clavirio"></div>

<div align="center"><img src="/assets/images/screenshot_dark_2.webp" style="width: 800px;" alt="clavirio"></div>

<div align="center"><img src="/assets/images/screenshot_light.webp" style="width: 800px;" alt="clavirio"></div>

## Table of Contents

- [Features](#features)
- [Lessons](#lessons)
- [Install](#install)
- [CLI](#cli)
- [Controls](#controls)
- [Settings](#settings)

## Features

- **Lessons mode** — 15 progressive drills from home row to full paragraphs and code; lessons 1-9 are layout-specific, 10-15 are shared
- **Practice mode** — random words (english 200/1k), code keywords (Rust, Python, JavaScript, Go, C/C++, Java, HTML/CSS), timed sessions (30s/60s), weak keys, common bigrams, quotes, and zen free-typing
- **Punctuation & numbers** — optional injection into word and code drills, toggle with `5`/`6` in practice mode
- **Live stats** — WPM, accuracy, errors, time, and line progress in a one-row status bar
- **WPM sparkline** — speed over time on the results screen
- **3 keyboard layouts** — QWERTY, Dvorak, and Colemak
- **Virtual keyboard** — highlights the next key with finger hints (**P**inky, **R**ing, **M**iddle, **I**ndex, **T**humb)
- **Error feedback** — wrong keystrokes shown inline, blocked until corrected with Backspace
- **Weak key tracking** — per-key accuracy saved across sessions; press `w` after a drill to practice that session's weakest keys, or select Weak Keys from the menu for cumulative stats
- **Session history** — saved to `~/.clavirio/history.json` with per-lesson tracking and averages
- **Progress tracking** — remembers your lesson and menu mode across sessions
- **Custom text** — load any file via `Ctrl-F` or `--file`
- **Dark & light themes**

## Lessons

Lessons 1-9 are **layout-specific** (each layout has its own drills matched to that layout's finger positions). Lessons 10-15 are **shared** across all layouts.

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

## Install

### Prebuilt binary

Download from [GitHub Releases](https://github.com/alexylon/clavirio/releases/latest) (macOS, Linux, Windows):

```bash
# macOS / Linux
tar xzf clavirio-*.tar.gz
sudo mv clavirio /usr/local/bin/

# Windows — extract the .zip, then add the folder to your PATH
```

> **macOS:** On first run, macOS may block the binary. Go to System Settings → Privacy & Security and click *Allow Anyway*.

### With Rust

```bash
cargo install clavirio
```

### From source

```bash
git clone https://github.com/alexylon/clavirio.git
cd clavirio
cargo build --release
```

## CLI

```
clavirio                        # interactive menu
clavirio -w                     # 100 random words (english 200)
clavirio -w 50 -l 1k            # 50 random words (english 1k)
clavirio -w -l rust             # 100 code tokens (rust)
clavirio -t 60                  # 60-second timed session
clavirio -t 30 -l python        # 30-second timed (python)
clavirio -w -p -n               # words with punctuation and numbers
clavirio -f path/to/file.txt    # custom file
```

| Flag | Description |
|------|-------------|
| `-w, --words [N]` | Practice with N random words (default: 100) |
| `-t, --time N` | Timed practice for N seconds |
| `-l, --list LIST` | Word list (default: `200`) |
| `-p, --punctuation` | Inject punctuation into word practice |
| `-n, --numbers` | Inject numbers into word practice |
| `-f, --file FILE` | Load a custom text file |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

Available word lists: `200`, `1k`, `rust`, `python`, `javascript` (`js`), `go`, `c` (`cpp`), `java`, `html` (`css`)

## Controls

Best with a maximized terminal window so text, keyboard, and stats fit comfortably.

### Main Menu

| Key | Action |
|-----|--------|
| `↑`/`↓` `k`/`j` | Navigate |
| `Enter` | Start |
| `m` | Toggle lessons / practice |
| `l` | Cycle layout |
| `1`-`4` | Toggle fingers / hints / keyboard / theme |
| `5`/`6` | Toggle punctuation / numbers (practice only) |
| `h` | Session history |
| `Ctrl-F` | Load file |
| `q` / `Esc` | Quit |

### Typing

| Key | Action |
|-----|--------|
| `Backspace` | Correct mistake |
| `Esc` | Pause menu |
| `Ctrl-R` | Restart |

### Finished Screen

| Key | Action |
|-----|--------|
| `r` | Restart |
| `w` | Practice weak keys |
| `n` | Next lesson (lessons mode) |
| `Esc` | Back to menu |

### Pause Menu

| Key | Action |
|-----|--------|
| `Space` | Resume |
| `r` | Restart |
| `n` | Next lesson (lessons mode) |
| `q` | Quit |
| `Esc` | Back to menu |

## Settings

Preferences are stored in `~/.clavirio/settings.toml` and saved automatically.

```toml
[keyboard]
layout = "qwerty"           # qwerty, dvorak, colemak

[display]
show_keyboard = true
show_hints = true
show_fingers = true
theme = "dark"              # dark, light
include_punctuation = false
include_numbers = false
practice_mode = false
```

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)