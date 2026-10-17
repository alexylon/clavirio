# Changelog

## [Unreleased]

### Fixed
- Next lesson (`n`) after finishing a lesson skipping one lesson, and not offered after the second-to-last lesson (it now follows the lesson just typed, not the menu cursor)
- Typing or backspace on the finished screen saving duplicate history records (in timed mode, one for every key pressed after time ran out)
- Tabs and other characters that can't be typed (a byte order mark, stray carriage returns, control characters) blocking custom files (tabs now become 4 spaces; the rest are removed)
- Trailing whitespace in custom files having to be typed (lines are now trimmed, and whitespace-only lines count as blank)
- Custom files with only blank lines opening a session that could never finish (now rejected as empty)
- Custom files starting with a blank line making you type the first line twice
- A history or key-stats file that couldn't be read or parsed being overwritten by the next session, losing all earlier records (one that can't be parsed is now kept next to it as a timestamped `.bak` file, and one that can't be read is left alone)
- `Ctrl-F` while paused or in the history view starting a file search you couldn't see (it now does nothing there)
- A weak space in the weak-key drill adding extra spaces after words
- Weak capitals, such as `A` from the Shift lessons, never appearing in the weak-key drill (it now capitalises the words starting with that letter, or drills the capital on its own when no word does)
- The weak-key drill mostly practising your common weak letters and hardly the rare ones (each weak key now gets about an equal share, though digits and other characters drilled on their own fill at most every other word)
- Weak brackets and quotes in the weak-key drill being added after a word, like `word(` (they now go around it, like `(word)`)
- The history view crashing on a timestamp with non-ASCII characters
- `w` after a session without mistakes ignoring your saved weak keys
- `Ctrl-R` while paused leaving the pause menu open after restarting
- Quotes after the first starting with a space you had to type
- Timed practice running out of text, and ending early, for typists above about 100 wpm (the text now lasts to about 200 wpm)
- A `settings.toml` that couldn't be read or parsed being overwritten at the next settings change, losing all saved settings (one that can't be parsed is now kept next to it as a timestamped `.bak` file, and one that can't be read is left alone; if it can't be read at startup, the app says so and doesn't save settings until it is restarted)
- Saving settings, history or key stats deleting the old file before the new one was in place, so a crash at that moment could lose it (saves now replace the file in one step, after flushing the new one to disk)
- `-p` and `-n` being saved to `settings.toml`, so punctuation or numbers stayed on in later runs without the flag (they now apply to that run only)
- Timed sessions running past the time limit while you kept typing (they now end at the limit, and for a second afterwards only Ctrl shortcuts work, so leftover typing can't restart or leave the results screen)
- Esc on an error message, such as a failed `Ctrl-F` load or `w` without weak keys, wiping the stats of the session behind it (it now only closes the message)
- Keys reaching the session hidden behind an error message, so typing counted out of sight and `r` or `Ctrl-R` restarted behind it (only Esc and the Ctrl shortcuts work while it shows, and restarting closes it)
- The session clock running while the file search or an error message covered the text, lowering WPM and letting a timed session run out behind them (the clock now waits until they close)
- A dip in the WPM graph right after resuming from the pause menu (the paused time is now left out)
- Failed saves of settings, sessions or key stats going unnoticed, for example on a full disk or a read-only `~/.clavirio` (the app now shows a message, or prints one after quitting)
- Long messages, such as a failed save on an 80-column terminal, being cut off at the edge (they now wrap, and the panel grows to fit them)
- A settings, history or key-stats file that couldn't be parsed being moved aside without a word (the app now says where it was kept, and says at startup when the settings couldn't be parsed)
- The app staying open in the background and using a full CPU core after its terminal window was closed, until it was force-killed
- The menu showing as an empty box, and the typing view showing a single line, on short terminals, such as the common 80×24 (the keyboard is now hidden below 36 rows to leave room for the text)
- Lines wider than the text panel, such as long lines in a custom file or practice text on an 80-column terminal, being cut off so you typed past the edge without seeing the cursor (the current line now scrolls to keep the cursor in view, and an ellipsis shows where a line is cut off)
- The header's stats and line count being cut off on terminals narrower than about 120 columns (the stats now come first, a long title is shortened, and the line count is left out rather than cut)
- The results screen's shortcuts being cut off below about 100 columns, so `Esc menu` and others weren't shown (they now have their own line under the score)
- The keyboard's keys covering or touching its frame at some widths, such as 65, 91 and 117 columns (keys are now sized to fit inside the frame, and the app is now up to 121 columns wide instead of 120, so the largest keys still fit)
- Zen mode's wpm, time and `Esc stop` line never being shown, as the typed lines took all the room
- A huge `-w` or `-t` using up the memory, or crashing and leaving the terminal broken, and `0` being accepted (`-w` now takes 1 to 10000 words and `-t` 1 to 3600 seconds)

## [0.4.1] - 2026-03-28

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
