<div align="center"><img src="/assets/images/icon_gunmetal_scanlines_512.png" style="width: 85px;" alt="clavirio"></div>

<h1 align="center"><code>clavirio</code></h1>

![](https://github.com/alexylon/clavirio/actions/workflows/rust.yml/badge.svg) &nbsp; [![Crates.io](https://img.shields.io/crates/v/clavirio.svg?color=blue)](https://crates.io/crates/clavirio)

[www.clavir.io](https://www.clavir.io)

### Learn touch typing in your terminal

Clavirio is a typing tutor with step-by-step lessons for QWERTY, Dvorak, and Colemak.
It shows your speed and accuracy as you type, highlights keys and finger positions on a
virtual keyboard, and saves your session history. It is built with Rust and
[Ratatui](https://ratatui.rs).

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

[Research on typing skill](https://www.crumplab.com/publications/Crump/files/3900/Liu%20et%20al.%20-%202010%20-%20Do%20you%20know%20where%20your%20fingers%20have%20been%20Explicit.pdf)
found that skilled typists could type quickly while having difficulty recalling where
individual keys were. The authors suggest that typing relies on knowledge used during
typing that is less accessible to conscious recall. They also discuss keyboard rows
and columns as a possible frame of reference for remembering key positions.

Clavirio teaches the keyboard **row by row**. Each row follows the same seven-lesson sequence:

*index pair → middle pair → ring pair → pinky pair → reach pair → full row → row with Shift*

Each finger-pair lesson adds two keys, with practice text built from those keys and
keys introduced earlier. The same sequence applies to **QWERTY, Dvorak, and
Colemak**, so you can follow it on any of the three layouts. The sequence is
Clavirio's approach to practicing key movements; the study did not test these lessons.

## Features

- **Lessons:** 27 drills covering the home, top, and bottom rows, followed by numbers and symbols. Lessons 1–21 adapt to your layout; lessons 22–27 are shared.
- **Practice:** Random words from the English 200 or 1k lists; code terms for Rust, Python, JavaScript, Go, C/C++, Java, and HTML/CSS; 30- or 60-second sessions; weak keys; common bigrams; quotes; and Zen mode for free typing.
- **Punctuation and numbers:** Optionally add them to random word or code practice. Use `5` and `6` in the practice menu to toggle them.
- **Live feedback:** WPM, accuracy, errors, time (elapsed, or remaining in timed sessions), and line progress appear in a single status row. The results screen shows a graph of WPM over time.
- **Keyboard and layouts:** Switch between QWERTY, Dvorak, and Colemak. The virtual keyboard highlights the next key and shows finger hints: **P**inky, **R**ing, **M**iddle, **I**ndex, and **T**humb.
- **Error handling:** By default, typing continues after a mistake, and Backspace lets you correct it within the current word. Turn on error stop to require a correction before continuing.
- **Weak key practice:** Accuracy for each key is saved across sessions. After a drill, press `w` to practice that session's weakest keys, or your saved weak keys if the session has none. Choose Weak Keys from the practice menu to use your cumulative stats.
- **History and progress:** Sessions are saved to `~/.clavirio/history.json`, with per-lesson tracking and averages. Clavirio also remembers your lesson and whether you last used the lessons or practice menu.
- **Custom text:** Load a file with `Ctrl-F` or `--file`. Curly quotes, dashes, and ellipses are converted to characters you can type (`'`, `"`, `-`, `--`, `...`). Separately stored accents are joined to their letters, tabs become four spaces, and invisible characters such as soft hyphens are removed.
- **Themes:** Choose a dark or light theme.

## Lessons

Lessons 1–21 follow your chosen layout: seven lessons each for the home, top, and
bottom rows. The first five lessons in each row add two keys at a time; the last
two practice the full row and Shift. Lessons 22–27 cover numbers and symbols and
are the same for every layout.

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

Download a standalone binary from [GitHub Releases](https://github.com/alexylon/clavirio/releases/latest). Builds are available for macOS (Intel and Apple Silicon), Linux (x86_64), and Windows (x86_64).

```bash
# macOS / Linux
tar xzf clavirio-*.tar.gz
sudo mv clavirio /usr/local/bin/

# Windows: extract the .zip, then add the folder containing clavirio.exe to PATH
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

Run `clavirio` to open the menu, or start a practice session directly:

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
| `-w, --words [N]` | Practice with 1–10,000 random words (default: 100) |
| `-t, --time N` | Practice for 1–3,600 seconds |
| `-l, --list LIST` | Choose a word or code list (default: `200`) |
| `-p, --punctuation` | Add punctuation to word or code practice for this run |
| `-n, --numbers` | Add numbers to word or code practice for this run |
| `-f, --file FILE` | Load a custom text file |
| `-h, --help` | Print help |
| `-V, --version` | Print version |

Lists: `200`, `1k`, `rust`, `python`, `javascript` (`js`), `go`, `c` (`cpp`, `c++`), `java`, and `html` (`css`). Names in parentheses are aliases.

## Controls

A maximized terminal gives the text, keyboard, and stats more room. The virtual keyboard hides automatically when the terminal is shorter than 36 rows, or when a narrow window needs extra rows for the menu.

### Main Menu

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Navigate |
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
| `Esc` | Pause menu (in zen mode, stop) |
| `Ctrl-R` | Restart |
| `Ctrl-F` | Load file |

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
| `↑`/`↓` or `k`/`j` | Navigate |
| `Space` / `Enter` | Choose the selected item (Resume is selected first) |
| `r` | Restart |
| `n` | Next lesson (lessons mode) |
| `Esc` | Back to menu |
| `q` | Quit |

### History

| Key | Action |
|-----|--------|
| `↑`/`↓` or `k`/`j` | Scroll |
| `Esc` | Back to menu |

## Settings

Preferences are saved automatically in `~/.clavirio/settings.toml`. These are the default values:

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
error_stop = false          # true = stop until corrected; false = continue past mistakes
```

## License

Clavirio is licensed under the Apache License 2.0.

## Attribution

If you use or redistribute Clavirio, please preserve its copyright, license, and
NOTICE information.

If your distribution includes a credits page, acknowledgements, about page,
documentation, or similar project metadata, please mention the original author:

- Author: Alexander Alexandrov
- Project: Clavirio
- Homepage: https://www.clavir.io
- Repository: https://github.com/alexylon/clavirio

[![forthebadge](https://forthebadge.com/images/badges/made-with-rust.svg)](https://forthebadge.com)
