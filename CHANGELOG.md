# Changelog

## [Unreleased]

### Changed
- Changed license to Apache License 2.0

### Fixed
- Settings and history files failing to save on Windows (`fs::rename` does not overwrite on Windows)
- Timestamp formatting crash on unexpectedly short strings (safe slicing with bounds check)
- Potential crash in pause menu End key on empty menu (use `saturating_sub`)
- Unbounded history growth (capped at 10,000 records, oldest trimmed)
- Cumulative key stats overflow after billions of keypresses (use `saturating_add`)
- Empty word pool panic in weak-key text generation (guard with fallback)
- Cursor position bounds check in text rendering (clamp to line length)

## [0.4.0] - 2026-03-28

### Added
- Error stop toggle (`4`) — when off (default), typing continues past mistakes; wrong characters shown in red, backspace corrects within the current word; when on, cursor blocks until the error is corrected
- Next lesson shortcut (`n`) on the finished screen and pause menu — only shown in lessons mode when a next lesson exists
- Pause menu reorganized: Resume (Space), Restart (r), Next lesson (n), Menu (Esc), Quit (q) — with shortcuts shown inline
- Punctuation and numbers toggle for word practice — press `5`/`6` in practice mode to inject punctuation marks and numbers into random word and timed drills; affected menu labels update to show `+punct`/`+num`
- Theme shortcut changed from `4` to `t`
- CLI flags `--punctuation` (`-p`) and `--numbers` (`-n`) for word practice
- Programming language word lists: Rust, Python, JavaScript, Go, C/C++, Java, HTML/CSS — available in the practice menu and via `-l` CLI flag
- Menu mode (lessons/practice) persisted across sessions in `settings.toml`

### Changed
- Complete lesson redesign: 27 lessons (was 15) using a universal finger-pair system — 2 new keys per lesson, same physical positions across all 3 layouts (index → middle → ring → pinky → reach → full row → row + Shift for each of 3 rows), plus 6 shared lessons for numbers and symbols
- Removed Common Words, Paragraphs, and Code lessons (redundant with practice mode)
- Each lesson strictly uses only its declared characters, verified by automated audit
- Weak keys drill now always includes all character types (punct/digits) regardless of toggle state
- Invalid `--list` values now show an error and exit instead of silently falling back to english 200

## [0.3.6] - 2026-03-26

### Added
- Weak Keys practice — generates word drills targeting your least accurate characters; uses cumulative stats from all past sessions (`~/.clavirio/keystats.json`), accuracy improves naturally as you get better
- `w` shortcut on the finished screen — quick drill using weak keys from the current session only
- Common Bigrams practice — words weighted by the 40 most frequent English letter pairs (th, he, in, er, etc.)
- Quotes practice — type real quotes with natural prose and punctuation; no duplicates or truncated quotes per session
- Zen mode — free typing with no prompt, just WPM tracking; press Esc to stop
- Persistent per-key accuracy tracking across sessions

### Changed
- History table header is now bold for better readability
- Refactored word generation to share line-wrap logic across all generators

## [0.3.5] - 2026-03-25

### Added
- Separate Lessons and Practice modes toggled with `m` key
- Practice mode includes random words and timed entries

### Fixed
- Line progress showing beyond total (e.g. 4/3) on document completion

### Changed
- Restored history-based lesson resume (immune to lesson reordering) instead of settings-based index
- Removed `selected_lesson` from settings.toml
- Weakest keys now show correct/total (e.g. `k 3/6`) instead of accuracy percentage
- Default word count changed from 50 to 100
- Extracted `start_lesson`, `load_file`, `start_word_practice`, `start_timed_practice` methods to eliminate duplicated session setup logic

## [0.3.4] - 2026-03-25

### Added
- Random words mode — practice with the 200 or 1000 most common English words
- Timed mode — 30s and 60s countdown sessions with random words (english 200 and 1k)
- WPM sparkline — results screen shows speed-over-time graph with min/max labels
- CLI flags — `--words`, `--time`, `--list`, `--file` for launching modes directly
- Menu cursor persistence — selected lesson remembered across sessions in settings
- File size limit — custom files capped at 1 MB

### Fixed
- Space typed at the start of a new line is now ignored (habitual space after end of previous line)
- History view columns aligned with header
- Pause menu hides "Next lesson" option in random/timed modes

### Changed
- QWERTY lessons 02 (Middle Keys) and 03 (Ring & Pinky) extended to 15 lines for consistency

## [0.3.3] - 2026-03-16

### Added
- `Q` key to quit from main menu

## [0.3.2] - 2026-03-16

### Added
- Pause menu with resume, restart, next lesson, and quit options
- Rearranged stats bar layout

## [0.3.1] - 2026-03-10

### Changed
- Updated README and screenshots

## [0.3.0] - 2026-03-09

### Added
- Dark and light themes with `4` key toggle
- Keyboard border styling per theme
- Display settings: toggle fingers (`1`), hints (`2`), keyboard (`3`)
- Settings persistence in `~/.clavirio/settings.toml`
- Dvorak and Colemak keyboard layouts with `l` key cycling
- Layout-specific lessons (1–9) for Dvorak and Colemak with shared lessons (10–15)
- Layout-aware session resume with shared lesson IDs across layouts

## [0.2.3] - 2026-03-08

### Changed
- Renamed project from Clavirio to clavirio (lowercase)

## [0.2.2] - 2026-03-08

### Added
- `Ctrl-C` to save and quit immediately
- Cargo.toml metadata (description, license, keywords, categories)

### Changed
- Renamed project from ferrocrypt to clavirio

## [0.2.1] - 2026-03-08

### Added
- Unit tests and GitHub Actions CI workflow
- Finger hints on virtual keyboard and in header

## [0.2.0] - 2026-03-07

### Added
- Live WPM tracking and elapsed timer
- Virtual keyboard with next-key highlighting
- Upcoming lines preview (multiline text panel)
- Inline error display with backspace correction
- Built-in lessons with CLI file argument
- Line progress indicator in header
- Per-key error stats with weakest keys on completion
- Restart with `r` on completion, `Ctrl-R` to restart, `Esc` to return to menu
- Session history saved to `~/.clavirio/history.json` with history view
- Shift key support on virtual keyboard
- PC keyboard layout support
- Scrollable lesson menu with arrow-key navigation

### Fixed
- UTF-8 panics with panic hook
- Hardened history writes

### Changed
- Replaced datetime FFI with time crate
- Slimmed tokio features, cached char lookup

## [0.1.0] - 2026-03-06

- Initial release
