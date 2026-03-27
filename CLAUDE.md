# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project

Clavirio is a terminal typing tutor built with Rust, Ratatui, and Crossterm. It features a virtual keyboard display, 15 built-in lessons across 3 keyboard layouts (QWERTY/Dvorak/Colemak), random words and timed practice modes, real-time WPM/accuracy stats with a sparkline graph, CLI flags, and session history tracking.

## Build & Test Commands

```bash
cargo build                    # Debug build
cargo build --release          # Release build (LTO enabled)
cargo run                      # Run in debug mode
cargo run -- -f path/to/file   # Run with custom text file
cargo run -- -w                # Random words (50, english 200)
cargo run -- -t 60             # Timed 60s mode
cargo run -- -t 30 -l 1k      # Timed 30s, english 1k word list
cargo test                     # Run all tests
cargo test --lib app           # Run tests in app module only
cargo test test_name           # Run a single test by name
```

## Architecture

**Event loop** (`main.rs`): Single-threaded Tokio runtime. `run_input_loop()` spawns a blocking task reading crossterm events, sends them via mpsc channel. Main loop uses `tokio::select!` over events and shutdown signals, calling `app.handle_event()` then `draw()`.

**App state** (`app.rs`, 1250+ lines — largest module): Contains `App` struct (all mutable state) and `Document` struct (text content, line/char position tracking). `handle_event()` is the central dispatcher — routes input based on current screen (menu, typing, finished, history, search, paused). Also holds per-key error stats (`key_stats: HashMap<char, (correct, total)>`).

**Rendering** (`ui.rs`): Stateless rendering via `draw()`. Uses `compute_regions()` to split the terminal into header, body, and keyboard areas. Themes are defined as `ThemeColors` structs with named color constants. Keyboard rendering calculates per-key rects with variable widths.

**Keyboard layouts** (`keyboard.rs`): Static `KeyDef` arrays for 5 rows per layout. Each key has primary/secondary (shift) characters, a `KeyWidth`, and finger assignments via `GridCoord → Finger` mapping. `build_keycode_grid_map()` creates a `HashMap<KeyCode, GridCoord>` for O(1) lookup.

**Lessons** (`lessons.rs`): 9 layout-specific + 6 shared lessons per layout, embedded via `include_str!()`. Progressive curriculum: home row → full alphabet → capitals → numbers → punctuation → common words → paragraphs → code.

**Settings** (`settings.rs`): TOML config at `~/.clavirio/settings.toml`. Uses `#[serde(default)]` for backward compatibility. Atomic writes (temp file + rename).

**Words** (`words.rs`): English word lists (200 and 1k) embedded via `include_str!()`. Xorshift64 PRNG for random word selection, text generation with line wrapping. `WordList::all()` provides the menu entries.

**History** (`history.rs`): JSON array of `SessionRecord` at `~/.clavirio/history.json`. Tracks WPM, accuracy, duration, completion status, lesson ID.

## Key Patterns

- **No unsafe code** anywhere in the project
- Settings and history use **atomic file writes** (write to `.tmp`, then rename)
- Document skips blank lines transparently when advancing
- Keyboard events during typing are **blocked on error** until backspace is pressed
- The `App` struct syncs display toggles back to `Settings` in the main loop (not inside `handle_event`)
- All lesson text is compiled into the binary — no runtime file loading for built-in content
- Platform-specific modifier labels: macOS uses ⌘⌥⌃, others use Win/Alt/Ctrl

## Code Guidelines

- Do not add self-explanatory comments; only add comments where the logic is non-obvious
- Avoid magic strings and numbers — use named constants
- Keep code DRY — extract shared logic into helpers
- Handle unwraps — prefer returning errors or using safe alternatives
- Tests should be self-contained and only cover important behavior
- After each new feature, update README.md and CHANGELOG.md (under `[Unreleased]`)
- After each session, double-check all changes against these guidelines before finishing
