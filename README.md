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

- [Methodology](#methodology)
- [Features](#features)
- [Lessons](#lessons)
- [Install](#install)
- [CLI](#cli)
- [Controls](#controls)
- [Settings](#settings)
- [License](#license)
- [Attribution](#attribution)

## Methodology

[Research on typing skill](https://www.crumplab.com/publications/Crump/files/3900/Liu%20et%20al.%20-%202010%20-%20Do%20you%20know%20where%20your%20fingers%20have%20been%20Explicit.pdf) suggests that skilled typing relies more on **implicit procedural control** than on **explicit knowledge** of key locations. The paper also suggests that the keyboard is represented in terms of its **row-and-column structure**, not as a memorized list of individual letters.

Clavirio follows that idea by teaching the keyboard **row by row** with the same 7-lesson progression for each row:

*index pair → middle pair → ring pair → pinky pair → reach pair → full row → row + Shift*

This is meant to reduce reliance on **explicit recall** of key locations and build more **automatic control** within the keyboard’s **row-and-column structure**.

Each lesson introduces only 2 new keys and uses only characters from earlier lessons. The same progression is used for **QWERTY, Dvorak, and Colemak**, so the lesson structure stays consistent across layouts.

## Features

- **Lessons mode** — 27 progressive drills: home row → top row → bottom row (2 keys per lesson, by finger pair), numbers, and symbols; lessons 1-21 are layout-specific, 22-27 are shared
- **Practice mode** — random words (english 200/1k), code keywords (Rust, Python, JavaScript, Go, C/C++, Java, HTML/CSS), timed sessions (30s/60s), weak keys, common bigrams, quotes, and zen free-typing
- **Punctuation & numbers** — optional injection into word and code drills, toggle with `5`/`6` in practice mode
- **Live stats** — WPM, accuracy, errors, time, and line progress in a one-row status bar
- **WPM sparkline** — speed over time on the results screen
- **3 keyboard layouts** — QWERTY, Dvorak, and Colemak
- **Virtual keyboard** — highlights the next key with finger hints (**P**inky, **R**ing, **M**iddle, **I**ndex, **T**humb)
- **Error feedback** — two modes: error stop off (default) lets you type past mistakes with backspace correction within the current word; error stop on blocks until corrected
- **Weak key tracking** — per-key accuracy saved across sessions; press `w` after a drill to practice that session's weakest keys, or select Weak Keys from the menu for cumulative stats
- **Session history** — saved to `~/.clavirio/history.json` with per-lesson tracking and averages
- **Progress tracking** — remembers your lesson and menu mode across sessions
- **Custom text** — load any file via `Ctrl-F` or `--file`
- **Dark & light themes**

## Lessons

Lessons 1-21 are **layout-specific** — 7 lessons per row (home, top, bottom), 2 new keys each. Lessons 22-27 are **shared** (numbers and symbols).

| # | Lesson | QWERTY | Dvorak | Colemak |
|---|--------|--------|--------|---------|
| 1 | Home: Index | f j | u h | t n |
| 2 | Home: Middle | d k | e t | s e |
| 3 | Home: Ring | s l | o n | r i |
| 4 | Home: Pinky | a ; | a s | a o |
| 5 | Home: Reach | g h | i d | d h |
| 6 | Home Row | full row | full row | full row |
| 7 | Home + Shift | + caps | + caps | + caps |
| 8 | Top: Index | r u | p g | p l |
| 9 | Top: Middle | e i | . c | f u |
| 10 | Top: Ring | w o | , r | w y |
| 11 | Top: Pinky | q p | ' l | q ; |
| 12 | Top: Reach | t y | y f | g j |
| 13 | Top Row | full row | full row | full row |
| 14 | Top + Shift | + caps | + caps | + caps |
| 15 | Bottom: Index | v m | k m | v m |
| 16 | Bottom: Middle | c , | j w | c , |
| 17 | Bottom: Ring | x . | q v | x . |
| 18 | Bottom: Pinky | z / | ; z | z / |
| 19 | Bottom: Reach | b n | x b | b k |
| 20 | Bottom Row | full row | full row | full row |
| 21 | Bottom + Shift | + caps | + caps | + caps |

| # | Lesson | Focus |
|---|--------|-------|
| 22 | Numbers | 1 2 3 4 5 6 7 8 9 0 |
| 23 | Symbols I | / ? : ' " |
| 24 | Symbols II | [ ] { } < > \| \ |
| 25 | Symbols III | \` ~ ! - _ + = |
| 26 | Symbols IV | @ # $ % ^ & * ( ) |
| 27 | Numbers & Symbols | Review |

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
| `h` | Session history |
| `t` | Cycle theme |
| `l` | Cycle layout |
| `Ctrl-F` | Load file |
| `q` / `Esc` | Quit |
| `1`-`3` | Toggle fingers / hints / keyboard |
| `4` | Toggle error stop |
| `5`/`6` | Toggle punctuation / numbers (practice only) |

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
| `Space` / `Enter` | Resume |
| `r` | Restart |
| `n` | Next lesson (lessons mode) |
| `Esc` | Back to menu |
| `q` | Quit |

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
error_stop = false              # true = block on error, false = advance past errors
```

## License

This project is licensed under the Apache License 2.0.

## Attribution

If you use or redistribute this project, please preserve the copyright,
license, and NOTICE information.

If your distribution includes a credits page, acknowledgements, about page,
documentation, or similar project metadata, please mention the original author:

- Author: Alexander Alexandrov
- Project: Clavirio
- Homepage: https://www.clavir.io
- Repository: https://github.com/alexylon/clavirio

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)