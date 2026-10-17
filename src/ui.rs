use std::collections::HashMap;
use std::ops::Range;
use std::rc::Rc;

use crossterm::event::KeyCode;
use ratatui::Frame;
use ratatui::layout::{Constraint, Direction, Flex, Layout, Rect};
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Padding, Paragraph};
use unicode_width::{UnicodeWidthChar, UnicodeWidthStr};

use crate::app::{App, Progress};
use crate::keyboard::*;
use crate::settings::Theme;

const KEYBOARD_ROWS: usize = 5;
/// Rows per key: 2 for its border and 2 for its shifted and unshifted labels
const KEY_ROW_HEIGHT: u16 = 4;
const KEYBOARD_FRAME_ROWS: u16 = 1;
const KEYBOARD_HEIGHT: u16 = KEYBOARD_ROWS as u16 * KEY_ROW_HEIGHT + 2 * KEYBOARD_FRAME_ROWS;
const HEADER_ROWS: u16 = 2;
/// The menu's rows around its items: the panel frame, a blank row, the controls
/// and the settings
const MENU_CHROME_ROWS: u16 = PANEL_FRAME_ROWS + 3;
const MIN_MENU_ITEMS: u16 = 5;
/// The typing panel's rows: its frame, the current line and the next 2
const TYPING_PANEL_ROWS: u16 = PANEL_FRAME_ROWS + 3;
/// Enough for 5 menu items, the typing view, the results and the pause menu,
/// but only 2 history sessions. The keyboard is hidden on terminals too short
/// for this and the keyboard.
const MIN_BODY_ROWS: u16 = MENU_CHROME_ROWS + MIN_MENU_ITEMS;
const MAX_WIDTH: u16 = 120;
const SPACE_ERROR_SYMBOL: &str = "\u{2423}";
const ELLIPSIS: &str = "\u{2026}";
const PANEL_PADDING: Padding = Padding::symmetric(2, 1);
/// Rows a panel takes around its text: its 2 border rows and its padding
const PANEL_FRAME_ROWS: u16 = 2 + PANEL_PADDING.top + PANEL_PADDING.bottom;

#[derive(Clone, Copy)]
pub struct ThemeColors {
    pub dim_border: Color,
    pub title: Color,
    pub accent: Color,
    pub dim_text: Color,
    pub correct: Color,
    pub incorrect: Color,
    pub error_text: Color,
    pub text: Color,
    pub key_label: Color,
    pub cursor_fg: Color,
    pub cursor_bg: Color,
    pub finger_label: Color,
    pub disabled: Color,
    pub bg: Color,
}

impl ThemeColors {
    pub fn from_theme(theme: Theme) -> Self {
        match theme {
            Theme::Dark => Self {
                dim_border: Color::DarkGray,
                title: Color::Rgb(0x2a, 0xd0, 0x5c),
                accent: Color::Cyan,
                dim_text: Color::DarkGray,
                correct: Color::Rgb(100, 180, 255),
                incorrect: Color::Rgb(255, 170, 60),
                error_text: Color::Rgb(0xca, 0x47, 0x54),
                text: Color::White,
                key_label: Color::Gray,
                cursor_fg: Color::Black,
                cursor_bg: Color::White,
                finger_label: Color::Yellow,
                disabled: Color::Rgb(60, 60, 60),
                bg: Color::Reset,
            },
            Theme::Light => Self {
                dim_border: Color::Rgb(180, 180, 180),
                title: Color::Rgb(0x00, 0x99, 0x00),
                accent: Color::Rgb(0, 120, 180),
                dim_text: Color::Rgb(140, 140, 140),
                correct: Color::Rgb(30, 120, 200),
                incorrect: Color::Rgb(210, 100, 20),
                error_text: Color::Rgb(0xca, 0x47, 0x54),
                text: Color::Black,
                key_label: Color::Rgb(80, 80, 80),
                cursor_fg: Color::White,
                cursor_bg: Color::Rgb(40, 40, 40),
                finger_label: Color::Rgb(180, 130, 0),
                disabled: Color::Rgb(200, 200, 200),
                bg: Color::Rgb(235, 233, 230),
            },
        }
    }

    /// The colours of a toggle's key and label, greyed out when it has no effect
    fn toggle_colors(&self, enabled: bool) -> (Color, Color) {
        if enabled {
            (self.accent, self.dim_text)
        } else {
            (self.disabled, self.disabled)
        }
    }
}

pub struct Regions {
    header: Rect,
    body: Rect,
    text_area: Rect,
    search_area: Rect,
    keyboard_area: Rect,
}

fn mix_label_suffix(punctuation: bool, numbers: bool) -> String {
    let mut s = String::new();
    if punctuation {
        s.push_str(" +punct");
    }
    if numbers {
        s.push_str(" +num");
    }
    s
}

fn clamp_width(area: Rect) -> Rect {
    if area.width <= MAX_WIDTH {
        return area;
    }
    let pad = (area.width - MAX_WIDTH) / 2;
    Rect::new(area.x + pad, area.y, MAX_WIDTH, area.height)
}

/// The middle `height` rows of `area`, or all of it if it is no taller.
fn panel_area(area: Rect, height: u16) -> Rect {
    if height >= area.height {
        return area;
    }
    let [panel] = Layout::vertical([Constraint::Length(height)])
        .flex(Flex::Center)
        .areas(area);
    panel
}

/// Splits `text` into lines at most `width` columns wide, breaking at spaces, or
/// inside a word too wide for a line of its own.
fn wrap_text(text: &str, width: usize) -> Vec<String> {
    let width = width.max(1);
    let mut lines = Vec::new();
    for paragraph in text.lines() {
        let mut line = String::new();
        let mut line_width = 0;
        for word in paragraph.split_whitespace() {
            if !line.is_empty() && line_width + 1 + word.width() > width {
                lines.push(std::mem::take(&mut line));
                line_width = 0;
            }
            if !line.is_empty() {
                line.push(' ');
                line_width += 1;
            }
            for c in word.chars() {
                let c_width = c.width().unwrap_or(0);
                if !line.is_empty() && line_width + c_width > width {
                    lines.push(std::mem::take(&mut line));
                    line_width = 0;
                }
                line.push(c);
                line_width += c_width;
            }
        }
        lines.push(line);
    }
    lines
}

pub fn keyboard_fits(height: u16) -> bool {
    height >= HEADER_ROWS + MIN_BODY_ROWS + KEYBOARD_HEIGHT
}

pub fn compute_regions(area: Rect, show_keyboard: bool) -> Regions {
    let clamped = clamp_width(area);

    let kbd_height = if show_keyboard && keyboard_fits(clamped.height) {
        KEYBOARD_HEIGHT
    } else {
        0
    };

    let [header, body, keyboard_area] = Layout::vertical([
        Constraint::Length(HEADER_ROWS),
        Constraint::Fill(1),
        Constraint::Length(kbd_height),
    ])
    .areas(clamped);

    let [text_area] = Layout::horizontal([Constraint::Percentage(80)])
        .flex(Flex::Center)
        .areas(body);

    let [search_area] = Layout::horizontal([Constraint::Percentage(60)])
        .flex(Flex::Center)
        .areas(body);

    Regions {
        header,
        body,
        text_area,
        search_area,
        keyboard_area,
    }
}

fn build_keyboard_rects(area: Rect, rows: &[Vec<KeyDef>]) -> Vec<Rc<[Rect]>> {
    let row_rects = Layout::new(
        Direction::Vertical,
        vec![Constraint::Ratio(1, KEYBOARD_ROWS as u32); KEYBOARD_ROWS],
    )
    .split(area);

    // The widest odd key width that fits the 13 keys of the widest row. An odd
    // width leaves an odd space inside the border, so labels sit exactly centred.
    let raw = area.width / 13;
    let unit_width = if raw.is_multiple_of(2) {
        raw.saturating_sub(1).max(1)
    } else {
        raw
    };
    // Wide keys, such as Shift and Tab, are 1.5 times as wide and the spacebar 6
    // times, also rounded down to odd widths
    let raw_wide = unit_width * 3 / 2;
    let wide_width = if raw_wide % 2 == 0 {
        raw_wide - 1
    } else {
        raw_wide
    };
    let raw_space = unit_width * 6;
    let space_width = if raw_space % 2 == 0 {
        raw_space - 1
    } else {
        raw_space
    };

    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let constraints: Vec<Constraint> = row
                .iter()
                .map(|k| match k.width {
                    KeyWidth::Normal => Constraint::Length(unit_width),
                    KeyWidth::Wide => Constraint::Length(wide_width),
                    KeyWidth::Spacebar => Constraint::Length(space_width),
                })
                .collect();
            Layout::new(Direction::Horizontal, constraints)
                .flex(Flex::Center)
                .split(row_rects[i])
        })
        .collect()
}

fn bounding_rect(kbd_rects: &[Rc<[Rect]>]) -> Option<Rect> {
    let mut min_x = u16::MAX;
    let mut min_y = u16::MAX;
    let mut max_x = 0u16;
    let mut max_y = 0u16;
    for row in kbd_rects {
        for cell in row.iter() {
            if cell.width == 0 || cell.height == 0 {
                continue;
            }
            min_x = min_x.min(cell.x);
            min_y = min_y.min(cell.y);
            max_x = max_x.max(cell.x + cell.width);
            max_y = max_y.max(cell.y + cell.height);
        }
    }
    if min_x >= max_x || min_y >= max_y {
        return None;
    }
    Some(Rect::new(min_x, min_y, max_x - min_x, max_y - min_y))
}

pub fn draw(
    frame: &mut Frame,
    app: &App,
    regions: &Regions,
    rows: &[Vec<KeyDef>],
    grid_map: &HashMap<KeyCode, GridCoord>,
) {
    let tc = ThemeColors::from_theme(app.theme);

    let bg_block = Block::default().style(Style::default().bg(tc.bg));
    frame.render_widget(bg_block, frame.area());

    let on_menu = app.document.is_none() && app.error.is_none() && !app.searching && !app.zen_mode;

    let hint_coords: Vec<GridCoord> = if app.show_hints {
        if on_menu {
            // On the menu, Enter is highlighted to show what the display toggles do
            grid_map.get(&KeyCode::Enter).copied().into_iter().collect()
        } else {
            app.document
                .as_ref()
                .and_then(|d| d.expected_char())
                .map(|ch| {
                    let mut coords = Vec::new();
                    let key = KeyCode::Char(ch.to_ascii_uppercase());
                    if let Some(&coord) = grid_map.get(&key) {
                        coords.push(coord);
                    }
                    let needs_shift = ch.is_uppercase()
                        || matches!(
                            ch,
                            '!' | '@'
                                | '#'
                                | '$'
                                | '%'
                                | '^'
                                | '&'
                                | '*'
                                | '('
                                | ')'
                                | '_'
                                | '+'
                                | '{'
                                | '}'
                                | '|'
                                | ':'
                                | '"'
                                | '<'
                                | '>'
                                | '?'
                                | '~'
                        );
                    if needs_shift {
                        if let Some(&coord) = grid_map.get(&KeyCode::Modifier(
                            crossterm::event::ModifierKeyCode::LeftShift,
                        )) {
                            coords.push(coord);
                        }
                    }
                    coords
                })
                .unwrap_or_default()
        }
    } else {
        Vec::new()
    };

    let hint_finger = if app.show_fingers {
        hint_coords
            .first()
            .and_then(|&coord| finger_for_coord(coord))
    } else {
        None
    };
    draw_header(frame, app, regions.header, hint_finger, &tc);
    if app.paused {
        draw_pause_menu(frame, app, regions.body, &tc);
    } else if app.viewing_history {
        draw_history(frame, app, regions.text_area, &tc);
    } else {
        draw_text_panel(frame, app, regions.text_area, &tc);
        draw_search_overlay(frame, app, regions.search_area, &tc);
    }

    if regions.keyboard_area.height > 0 {
        let keys_area = Rect::new(
            regions.keyboard_area.x,
            regions.keyboard_area.y + KEYBOARD_FRAME_ROWS,
            regions.keyboard_area.width,
            regions
                .keyboard_area
                .height
                .saturating_sub(2 * KEYBOARD_FRAME_ROWS),
        );
        let kbd_rects = build_keyboard_rects(keys_area, rows);
        let highlight_coord: Option<GridCoord> = if app.show_hints {
            app.highlighted_key
                .and_then(|code| grid_map.get(&code))
                .copied()
        } else {
            None
        };
        let highlight_color = if app.last_correct {
            tc.correct
        } else {
            tc.incorrect
        };
        draw_keyboard(
            frame,
            rows,
            &kbd_rects,
            &KeyboardState {
                hint_coords: &hint_coords,
                highlight_coord,
                highlight_color,
                show_fingers: app.show_fingers,
            },
            &tc,
        );
    }
}

fn draw_header(
    frame: &mut Frame,
    app: &App,
    area: Rect,
    hint_finger: Option<Finger>,
    tc: &ThemeColors,
) {
    frame.render_widget(
        Block::new()
            .borders(Borders::BOTTOM)
            .border_style(Style::new().fg(tc.dim_border)),
        area,
    );

    let on_menu = app.document.is_none() && app.error.is_none() && !app.zen_mode;
    if on_menu {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "clavirio",
                Style::new().fg(tc.title).bold(),
            )))
            .centered(),
            area,
        );
        return;
    }

    let [left, center, right] = Layout::horizontal([
        Constraint::Percentage(30),
        Constraint::Min(0),
        Constraint::Percentage(30),
    ])
    .areas(area);

    // LEFT: lesson_title · cur/total
    let mut left_spans: Vec<Span> = vec![Span::raw(" ")];
    if !app.lesson_title.is_empty() {
        left_spans.push(Span::styled(&app.lesson_title, Style::new().fg(tc.text)));
        if app.document.is_some() {
            left_spans.push(Span::styled(" \u{b7} ", Style::new().fg(tc.dim_text)));
        }
    }
    if let Some(doc) = &app.document {
        let (cur, total) = doc.line_progress();
        left_spans.push(Span::styled(
            format!("{cur}/{total}"),
            Style::new().fg(tc.text),
        ));
    }
    frame.render_widget(Paragraph::new(Line::from(left_spans)), left);

    // MIDDLE: finger hint (dedicated reserved slot)
    if let Some(finger) = hint_finger {
        let name = match finger {
            Finger::Pinky => "pinky",
            Finger::Ring => "ring",
            Finger::Middle => "middle",
            Finger::Index => "index",
            Finger::Thumb => "thumb",
        };
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    format!("{} ", finger.label()),
                    Style::new().fg(tc.finger_label).bold(),
                ),
                Span::styled(name, Style::new().fg(tc.dim_text)),
            ]))
            .centered(),
            center,
        );
    }

    // RIGHT: wpm · acc · err · mm:ss
    let wpm = app.wpm();
    let started = app.start_time.is_some();
    let acc_pct = if app.total_count > 0 {
        app.correct_count as f64 / app.total_count as f64 * 100.0
    } else {
        0.0
    };
    let err = app.error_count();
    let elapsed = app.elapsed_secs();
    let mins = (elapsed as u64) / 60;
    let secs = (elapsed as u64) % 60;

    let (wpm_str, wpm_fg) = if started {
        (
            format!("{wpm:.0}"),
            if wpm > 0.0 { tc.accent } else { tc.dim_text },
        )
    } else {
        ("\u{2014}".into(), tc.dim_text)
    };
    let (acc_str, acc_fg) = if started {
        (format!("{acc_pct:.0}%"), tc.text)
    } else {
        ("\u{2014}".into(), tc.dim_text)
    };
    let err_fg = if err > 0 { tc.incorrect } else { tc.dim_text };
    let dim = Style::new().fg(tc.dim_text);

    let remaining = app.remaining_secs();
    let time_str = if let Some(r) = remaining {
        let r_mins = (r as u64) / 60;
        let r_secs = (r as u64) % 60;
        format!("{r_mins:02}:{r_secs:02} ")
    } else {
        format!("{mins:02}:{secs:02} ")
    };
    let time_fg = if remaining.is_some_and(|r| r <= 5.0) && started {
        tc.incorrect
    } else {
        tc.dim_text
    };

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(wpm_str, Style::new().fg(wpm_fg)),
            Span::styled(" wpm", dim),
            Span::styled(" \u{b7} ", dim),
            Span::styled(acc_str, Style::new().fg(acc_fg)),
            Span::styled(" acc", dim),
            Span::styled(" \u{b7} ", dim),
            Span::styled(format!("{err}"), Style::new().fg(err_fg)),
            Span::styled(" err", dim),
            Span::styled(" \u{b7} ", dim),
            Span::styled(time_str, Style::new().fg(time_fg)),
        ]))
        .right_aligned(),
        right,
    );
}

fn draw_pause_menu(frame: &mut Frame, app: &App, area: Rect, tc: &ThemeColors) {
    if !app.paused {
        return;
    }

    let menu_width = 36_u16;
    let items = app.pause_menu_items();
    let menu_height = items.len() as u16 + 4; // items + border + padding

    let [v_area] = Layout::vertical([Constraint::Length(menu_height)])
        .flex(Flex::Center)
        .areas(area);
    let [menu_area] = Layout::horizontal([Constraint::Length(menu_width)])
        .flex(Flex::Center)
        .areas(v_area);

    // Clear the entire text area so no underlying borders bleed through
    frame.render_widget(Block::default().style(Style::default().bg(tc.bg)), area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(tc.dim_border))
        .padding(Padding::vertical(1))
        .title(Span::styled(" Paused ", Style::new().fg(tc.accent).bold()));

    let inner = block.inner(menu_area);
    frame.render_widget(block, menu_area);

    for (i, (label, shortcut)) in items.iter().enumerate() {
        if i as u16 >= inner.height {
            break;
        }
        let row = Rect::new(inner.x, inner.y + i as u16, inner.width, 1);
        let selected = i == app.pause_menu_index;
        let label_fg = if selected { tc.text } else { tc.dim_text };
        let shortcut_fg = if selected { tc.accent } else { tc.dim_text };
        let marker = if selected { "\u{25b8} " } else { "  " };

        let display_shortcut = match *shortcut {
            " " => "Space",
            "" => "Esc",
            s => s,
        };
        let pad_len =
            (inner.width as usize).saturating_sub(2 + label.len() + display_shortcut.len() + 1);

        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(marker, Style::new().fg(tc.accent)),
                Span::styled(*label, Style::new().fg(label_fg)),
                Span::raw(" ".repeat(pad_len)),
                Span::styled(display_shortcut, Style::new().fg(shortcut_fg)),
                Span::raw(" "),
            ])),
            row,
        );
    }
}

/// Draws `message` in place of the text panel, wrapped to its width, in a panel
/// as tall as the message needs.
fn draw_error(frame: &mut Frame, message: &str, area: Rect, block: Block, tc: &ThemeColors) {
    let lines = wrap_text(message, block.inner(area).width.into());
    let height = lines.len() as u16 + PANEL_FRAME_ROWS;
    // Short of rows, the padding goes before any line of the message does
    let block = if height > area.height {
        block.padding(Padding {
            top: 0,
            bottom: 0,
            ..PANEL_PADDING
        })
    } else {
        block
    };
    frame.render_widget(
        Paragraph::new(lines.join("\n"))
            .style(Style::new().fg(tc.incorrect))
            .block(block)
            .centered(),
        panel_area(area, height),
    );
}

fn draw_text_panel(frame: &mut Frame, app: &App, area: Rect, tc: &ThemeColors) {
    if app.searching {
        return;
    }

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(tc.dim_border))
        .padding(PANEL_PADDING);

    if let Some(err) = &app.error {
        draw_error(frame, err, area, block, tc);
        return;
    }

    let ideal_height = if app.document.is_none() && !app.zen_mode {
        (app.current_menu_count() as u16) + MENU_CHROME_ROWS
    } else {
        TYPING_PANEL_ROWS
    };
    let ideal_height = if app.is_finished() {
        let has_spark = app.sparkline().is_some();
        let has_worst = !app.worst_keys(5).is_empty();
        let content = 1 + if has_spark { 3 } else { 0 } + if has_worst { 1 } else { 0 };
        ideal_height.max(content + PANEL_FRAME_ROWS)
    } else {
        ideal_height
    };
    let inner = panel_area(area, ideal_height);

    if app.zen_mode {
        let mut lines: Vec<Line> = Vec::new();
        let visible = inner.height.saturating_sub(4) as usize;
        let start = app.zen_lines.len().saturating_sub(visible);
        for (i, line) in app.zen_lines[start..].iter().enumerate() {
            let is_current = start + i == app.zen_lines.len() - 1;
            if is_current {
                lines.push(Line::from(vec![
                    Span::styled(line.clone(), Style::new().fg(tc.correct)),
                    Span::styled(" ", Style::new().fg(tc.cursor_fg).bg(tc.cursor_bg)),
                ]));
            } else {
                lines.push(Line::from(Span::styled(
                    line.clone(),
                    Style::new().fg(tc.correct),
                )));
            }
        }
        while lines.len() < visible {
            lines.push(Line::from(""));
        }
        lines.push(Line::from(""));
        let mut controls = vec![
            Span::styled(
                format!("{:.0} wpm", app.wpm()),
                Style::new().fg(tc.accent).bold(),
            ),
            Span::styled("  ", Style::new().fg(tc.dim_text)),
        ];
        if app.start_time.is_some() {
            let elapsed = app.elapsed_secs();
            let mins = (elapsed / 60.0) as u32;
            let secs = (elapsed % 60.0) as u32;
            controls.push(Span::styled(
                format!("{mins}:{secs:02}"),
                Style::new().fg(tc.dim_text),
            ));
            controls.push(Span::styled("  ", Style::new().fg(tc.dim_text)));
        }
        controls.push(Span::styled("Esc", Style::new().fg(tc.accent).bold()));
        controls.push(Span::styled(" stop", Style::new().fg(tc.dim_text)));
        lines.push(Line::from(controls));

        frame.render_widget(Paragraph::new(lines).block(block), inner);
        return;
    }

    match &app.document {
        None => {
            use crate::app::MenuMode;
            let total_items = app.current_menu_count();
            let cursor = match app.menu_mode {
                MenuMode::Lessons => app.selected_lesson,
                MenuMode::Practice => app.selected_practice,
            };
            let visible_slots = inner.height.saturating_sub(MENU_CHROME_ROWS) as usize;

            let scroll = if visible_slots >= total_items || cursor < visible_slots / 2 {
                0
            } else if cursor + visible_slots / 2 >= total_items {
                total_items.saturating_sub(visible_slots)
            } else {
                cursor.saturating_sub(visible_slots / 2)
            };

            let mut lines: Vec<Line> = Vec::new();
            match app.menu_mode {
                MenuMode::Lessons => {
                    let lessons = crate::lessons::lessons_for_layout(app.layout);
                    const NUM_PREFIX_LEN: usize = 4; // "01. "
                    let label_len = |l: &&crate::lessons::Lesson| {
                        let keys_width = if l.keys.is_empty() {
                            0
                        } else {
                            2 + l.keys.len() + 1
                        };
                        NUM_PREFIX_LEN + l.title.len() + keys_width
                    };
                    let max_label: usize = [
                        crate::settings::KeyboardLayout::Qwerty,
                        crate::settings::KeyboardLayout::Dvorak,
                        crate::settings::KeyboardLayout::Colemak,
                    ]
                    .iter()
                    .flat_map(|lay| crate::lessons::lessons_for_layout(*lay))
                    .map(|l| label_len(&l))
                    .max()
                    .unwrap_or(20);

                    for (i, lesson) in lessons
                        .iter()
                        .enumerate()
                        .skip(scroll)
                        .take(visible_slots.min(total_items - scroll))
                    {
                        let selected = i == cursor;
                        let marker = if selected { "▸" } else { " " };
                        let title_fg = if selected { tc.text } else { tc.dim_text };
                        let marker_fg = if selected { tc.accent } else { tc.dim_text };
                        let num = i + 1;
                        let mut spans = vec![
                            Span::styled(format!(" {marker} "), Style::new().fg(marker_fg).bold()),
                            Span::styled(format!("{num:02}. "), Style::new().fg(tc.dim_text)),
                            Span::styled(lesson.title.to_string(), Style::new().fg(title_fg)),
                        ];
                        if !lesson.keys.is_empty() {
                            spans.push(Span::styled(
                                format!(" ({})", lesson.keys),
                                Style::new().fg(tc.dim_text),
                            ));
                        }
                        let current_len = label_len(&lesson);
                        if current_len < max_label {
                            spans.push(Span::raw(" ".repeat(max_label - current_len)));
                        }
                        lines.push(Line::from(spans));
                    }
                }
                MenuMode::Practice => {
                    let word_lists = crate::words::WordList::all();
                    let timed_options = crate::app::App::TIMED_OPTIONS;
                    let special_items = crate::app::App::SPECIAL_PRACTICE;
                    let mix_suffix = mix_label_suffix(app.include_punctuation, app.include_numbers);
                    let suffix_len = mix_suffix.len();
                    let max_label: usize =
                        special_items
                            .iter()
                            .map(|(title, _)| title.len())
                            .chain(word_lists.iter().map(|wl| {
                                wl.category().len() + 2 + wl.label().len() + suffix_len + 1
                            }))
                            .chain(timed_options.iter().map(|(s, wl)| {
                                wl.category().len()
                                    + 2
                                    + format!("{s}s · {}", wl.label()).chars().count()
                                    + suffix_len
                                    + 1
                            }))
                            .max()
                            .unwrap_or(20);

                    for i in scroll..(scroll + visible_slots).min(total_items) {
                        let selected = i == cursor;
                        let marker = if selected { "▸" } else { " " };
                        let title_fg = if selected { tc.text } else { tc.dim_text };
                        let marker_fg = if selected { tc.accent } else { tc.dim_text };

                        if i < special_items.len() {
                            let title = special_items[i].0;
                            let current_len = title.len();
                            let mut spans = vec![
                                Span::styled(
                                    format!(" {marker} "),
                                    Style::new().fg(marker_fg).bold(),
                                ),
                                Span::styled(title, Style::new().fg(title_fg)),
                            ];
                            if current_len < max_label {
                                spans.push(Span::raw(" ".repeat(max_label - current_len)));
                            }
                            lines.push(Line::from(spans));
                            continue;
                        }

                        let idx = i - special_items.len();
                        let (title, mut keys) = if idx < word_lists.len() {
                            let wl = word_lists[idx];
                            (wl.category(), wl.label().to_string())
                        } else {
                            let (secs, wl) = timed_options[idx - word_lists.len()];
                            (wl.category(), format!("{secs}s · {}", wl.label()))
                        };
                        keys.push_str(&mix_suffix);
                        let current_len = title.len() + 2 + keys.chars().count() + 1;
                        let mut spans = vec![
                            Span::styled(format!(" {marker} "), Style::new().fg(marker_fg).bold()),
                            Span::styled(title, Style::new().fg(title_fg)),
                            Span::styled(format!(" ({keys})"), Style::new().fg(tc.dim_text)),
                        ];
                        if current_len < max_label {
                            spans.push(Span::raw(" ".repeat(max_label - current_len)));
                        }
                        lines.push(Line::from(spans));
                    }
                }
            }
            let on_off = |on: bool| if on { "on" } else { "off" };
            lines.push(Line::from(""));
            let mode_label = match app.menu_mode {
                MenuMode::Lessons => "lessons",
                MenuMode::Practice => "practice",
            };
            lines.push(Line::from(vec![
                Span::styled("Enter", Style::new().fg(tc.accent)),
                Span::styled(" start  ", Style::new().fg(tc.dim_text)),
                Span::styled("m", Style::new().fg(tc.accent)),
                Span::styled(format!(" {mode_label}  "), Style::new().fg(tc.dim_text)),
                Span::styled("h", Style::new().fg(tc.accent)),
                Span::styled(" history  ", Style::new().fg(tc.dim_text)),
                Span::styled("t", Style::new().fg(tc.accent)),
                Span::styled(format!(" {}  ", app.theme), Style::new().fg(tc.dim_text)),
                Span::styled("l", Style::new().fg(tc.accent)),
                Span::styled(format!(" {}  ", app.layout), Style::new().fg(tc.dim_text)),
                Span::styled("^F", Style::new().fg(tc.accent)),
                Span::styled(" file  ", Style::new().fg(tc.dim_text)),
                Span::styled("q", Style::new().fg(tc.accent)),
                Span::styled("/", Style::new().fg(tc.dim_text)),
                Span::styled("Esc", Style::new().fg(tc.accent)),
                Span::styled(" quit", Style::new().fg(tc.dim_text)),
            ]));
            let (fingers_key_fg, fingers_fg) = tc.toggle_colors(app.show_hints);
            let (keyboard_key_fg, keyboard_fg) =
                tc.toggle_colors(keyboard_fits(frame.area().height));
            let mut toggle_spans = vec![
                Span::styled("1", Style::new().fg(fingers_key_fg)),
                Span::styled(
                    format!(" fingers {}  ", on_off(app.show_fingers)),
                    Style::new().fg(fingers_fg),
                ),
                Span::styled("2", Style::new().fg(tc.accent)),
                Span::styled(
                    format!(" hints {}  ", on_off(app.show_hints)),
                    Style::new().fg(tc.dim_text),
                ),
                Span::styled("3", Style::new().fg(keyboard_key_fg)),
                Span::styled(
                    format!(" keyboard {}  ", on_off(app.show_keyboard)),
                    Style::new().fg(keyboard_fg),
                ),
                Span::styled("4", Style::new().fg(tc.accent)),
                Span::styled(
                    format!(" error stop {}  ", on_off(app.error_stop)),
                    Style::new().fg(tc.dim_text),
                ),
            ];
            if app.menu_mode == MenuMode::Practice {
                toggle_spans.extend([
                    Span::styled("5", Style::new().fg(tc.accent)),
                    Span::styled(
                        format!(" punct {}  ", on_off(app.include_punctuation)),
                        Style::new().fg(tc.dim_text),
                    ),
                    Span::styled("6", Style::new().fg(tc.accent)),
                    Span::styled(
                        format!(" num {}", on_off(app.include_numbers)),
                        Style::new().fg(tc.dim_text),
                    ),
                ]);
            }
            lines.push(Line::from(toggle_spans));
            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
        Some(doc) if doc.progress == Progress::Finished => {
            let pct = if app.total_count > 0 {
                (app.correct_count as f32 / app.total_count as f32) * 100.0
            } else {
                0.0
            };
            let mut actions = vec![
                Span::styled("Done! ", Style::new().fg(tc.correct).bold()),
                Span::styled(
                    format!("{:.0} wpm", app.wpm()),
                    Style::new().fg(tc.accent).bold(),
                ),
                Span::styled(format!("  {:.0}% accuracy", pct), Style::new().fg(tc.text)),
                Span::styled(
                    format!("  ({}/{})", app.correct_count, app.total_count),
                    Style::new().fg(tc.dim_text),
                ),
                Span::styled("  r", Style::new().fg(tc.accent).bold()),
                Span::styled(" restart  ", Style::new().fg(tc.dim_text)),
                Span::styled("w", Style::new().fg(tc.accent).bold()),
                Span::styled(" weak keys  ", Style::new().fg(tc.dim_text)),
            ];
            if app.has_next_lesson() {
                actions.push(Span::styled("n", Style::new().fg(tc.accent).bold()));
                actions.push(Span::styled(" next  ", Style::new().fg(tc.dim_text)));
            }
            actions.extend_from_slice(&[
                Span::styled("Esc", Style::new().fg(tc.accent).bold()),
                Span::styled(" menu", Style::new().fg(tc.dim_text)),
            ]);
            let mut lines = vec![Line::from(actions)];
            if let Some((spark, min, max)) = app.sparkline() {
                lines.push(Line::from(""));
                lines.push(Line::from(Span::styled(spark, Style::new().fg(tc.accent))));
                lines.push(Line::from(Span::styled(
                    format!("{:.0} min - {:.0} max wpm", min, max),
                    Style::new().fg(tc.dim_text),
                )));
            }
            let worst = app.worst_keys(5);
            if !worst.is_empty() {
                let mut spans = vec![Span::styled("Weakest: ", Style::new().fg(tc.dim_text))];
                for (i, (ch, correct, total)) in worst.iter().enumerate() {
                    if i > 0 {
                        spans.push(Span::styled("  ", Style::new().fg(tc.dim_text)));
                    }
                    let label = if *ch == ' ' {
                        "space".to_string()
                    } else {
                        ch.to_string()
                    };
                    spans.push(Span::styled(label, Style::new().fg(tc.incorrect).bold()));
                    spans.push(Span::styled(
                        format!(" {correct}/{total}"),
                        Style::new().fg(tc.dim_text),
                    ));
                }
                lines.push(Line::from(spans));
            }
            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
        Some(doc) => {
            let text_width = block.inner(inner).width as usize;
            let line_chars: Vec<char> = doc.current_line.chars().collect();
            let pos = doc.cursor_position().min(line_chars.len());
            let mut cells: Vec<Span> = line_chars[..pos]
                .iter()
                .enumerate()
                .map(|(i, &ch)| {
                    if app.error_chars.iter().any(|&(p, _)| p == i) {
                        let display = if ch == ' ' {
                            SPACE_ERROR_SYMBOL.to_string()
                        } else {
                            ch.to_string()
                        };
                        Span::styled(display, Style::new().fg(tc.error_text))
                    } else {
                        Span::styled(ch.to_string(), Style::new().fg(tc.correct))
                    }
                })
                .collect();
            let cursor_style = Style::new().fg(tc.cursor_fg).bg(tc.cursor_bg);
            let (cursor, rest_start) = if let Some(err_ch) = app.last_error_char {
                if pos < line_chars.len() {
                    cells.push(Span::styled(
                        err_ch.to_string(),
                        Style::new().fg(tc.cursor_fg).bg(tc.incorrect),
                    ));
                }
                if let Some(&cursor_ch) = line_chars.get(pos + 1) {
                    cells.push(Span::styled(cursor_ch.to_string(), cursor_style));
                }
                (pos + 1, pos + 2)
            } else {
                if let Some(&next_ch) = line_chars.get(pos) {
                    cells.push(Span::styled(next_ch.to_string(), cursor_style));
                }
                (pos, pos + 1)
            };
            cells.extend(
                line_chars
                    .iter()
                    .skip(rest_start)
                    .map(|ch| Span::styled(ch.to_string(), Style::new().fg(tc.text))),
            );

            let mut lines = vec![fit_line(cells, cursor, text_width, tc)];
            for upcoming in doc.upcoming_lines(2) {
                let cells = upcoming
                    .chars()
                    .map(|ch| Span::styled(ch.to_string(), Style::new().fg(tc.dim_text)))
                    .collect();
                lines.push(fit_line(cells, 0, text_width, tc));
            }

            frame.render_widget(Paragraph::new(lines).block(block).centered(), inner);
        }
    }
}

/// The part of a line, given as one span per character, that fits in `width`
/// columns. A line too wide scrolls to keep `anchor` near the middle, with an
/// ellipsis on each side that is cut off. Characters with the same style are
/// joined back into one span, so accents render with the letter before them.
fn fit_line<'a>(
    mut cells: Vec<Span<'a>>,
    anchor: usize,
    width: usize,
    tc: &ThemeColors,
) -> Line<'a> {
    let widths: Vec<usize> = cells.iter().map(Span::width).collect();
    let range = visible_range(&widths, anchor, width);
    let ellipsis = || Span::styled(ELLIPSIS, Style::new().fg(tc.dim_text));
    if !range.is_empty() {
        if range.start > 0 && range.start != anchor {
            cells[range.start] = ellipsis();
        }
        if range.end < cells.len() && range.end - 1 != anchor {
            cells[range.end - 1] = ellipsis();
        }
    }
    let mut spans: Vec<Span> = Vec::new();
    for cell in cells.drain(range) {
        match spans.last_mut() {
            Some(last) if last.style == cell.style => last.content.to_mut().push_str(&cell.content),
            _ => spans.push(cell),
        }
    }
    Line::from(spans)
}

/// The cells that fit in `max` columns, keeping `anchor` near the middle without
/// leaving space empty after the end of the line.
fn visible_range(widths: &[usize], anchor: usize, max: usize) -> Range<usize> {
    if widths.iter().sum::<usize>() <= max {
        return 0..widths.len();
    }
    let mut start = anchor.min(widths.len());
    let mut used = 0;
    while start > 0 && used + widths[start - 1] <= max / 2 {
        start -= 1;
        used += widths[start];
    }
    let mut end = start;
    let mut used = 0;
    while end < widths.len() && used + widths[end] <= max {
        used += widths[end];
        end += 1;
    }
    while start > 0 && used + widths[start - 1] <= max {
        start -= 1;
        used += widths[start];
    }
    start..end
}

fn draw_search_overlay(frame: &mut Frame, app: &App, area: Rect, tc: &ThemeColors) {
    if !app.searching {
        return;
    }

    let [inner] = Layout::vertical([Constraint::Length(3)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(tc.accent))
        .title(Span::styled(
            " File path ",
            Style::new().fg(tc.accent).bold(),
        ))
        .padding(Padding::horizontal(1));

    let cursor_line = Line::from(vec![
        Span::raw(&app.file_path_buf),
        Span::styled("▌", Style::new().fg(tc.accent)),
    ]);

    frame.render_widget(Paragraph::new(cursor_line).block(block), inner);
}

/// Turn "2026-03-06T22:01:05" into "Mar 06  22:01"
pub(crate) fn friendly_timestamp(ts: &str) -> String {
    const MONTHS: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    // Expected format: YYYY-MM-DDThh:mm:ss
    match (ts.get(5..7), ts.get(8..10), ts.get(11..16)) {
        (Some(mo), Some(day), Some(time)) if ts.len() >= 19 => {
            let mo: usize = mo.parse().unwrap_or(1);
            let month = MONTHS.get(mo.wrapping_sub(1)).unwrap_or(&"???");
            format!("{month} {day}  {time}")
        }
        _ => ts.to_string(),
    }
}

fn draw_history(frame: &mut Frame, app: &App, area: Rect, tc: &ThemeColors) {
    let records = &app.history;

    let panel_h = area.height;
    let [inner] = Layout::vertical([Constraint::Length(panel_h)])
        .flex(Flex::Center)
        .areas(area);

    let block = Block::new()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::new().fg(tc.dim_border))
        .title(Span::styled(" History ", Style::new().fg(tc.accent).bold()))
        .padding(Padding::symmetric(2, 1));

    let mut lines = Vec::new();

    if records.is_empty() {
        lines.push(Line::from(Span::styled(
            "No sessions yet",
            Style::new().fg(tc.dim_text),
        )));
    } else {
        // Rows around the sessions: the panel frame, the column titles, the
        // averages and the controls, with a row to spare
        let chrome = 10_u16;
        let visible_slots = inner.height.saturating_sub(chrome) as usize;

        let scroll_pos = app.history_scroll;
        let scroll = if visible_slots >= records.len() || scroll_pos < visible_slots / 2 {
            0
        } else if scroll_pos + visible_slots / 2 >= records.len() {
            records.len().saturating_sub(visible_slots)
        } else {
            scroll_pos.saturating_sub(visible_slots / 2)
        };

        lines.push(Line::from(Span::styled(
            format!(
                "  {:<14} {:>5}  {:>5}  {:>6}  {}",
                "date", "wpm", "acc", "time", "lesson"
            ),
            Style::new().fg(tc.text).bold(),
        )));

        let lessons = crate::lessons::lessons_for_layout(app.layout);
        for (i, r) in records.iter().enumerate().skip(scroll).take(visible_slots) {
            let display_ts = friendly_timestamp(&r.timestamp);
            let mins = (r.duration_secs as u64) / 60;
            let secs = (r.duration_secs as u64) % 60;
            let status = if r.completed { "" } else { "*" };
            let lesson_display = if r.id.is_empty() {
                "—".to_string()
            } else if let Some(lesson) = lessons.iter().find(|l| l.id == r.id) {
                lesson.title.to_string()
            } else {
                r.id.replace('_', " ")
            };
            let selected = i == scroll_pos;
            let fg = if selected { tc.text } else { tc.dim_text };
            let marker = if selected { "▸ " } else { "  " };
            lines.push(Line::from(Span::styled(
                format!(
                    "{}{:<14} {:>5.0}  {:>4.0}%  {:>6}  {}",
                    marker,
                    display_ts,
                    r.wpm,
                    r.accuracy,
                    format!("{mins}:{secs:02}{status}"),
                    lesson_display
                ),
                Style::new().fg(fg),
            )));
        }

        let completed: Vec<_> = records.iter().filter(|r| r.completed).collect();
        if !completed.is_empty() {
            let avg_wpm: f64 =
                completed.iter().map(|r| r.wpm).sum::<f64>() / completed.len() as f64;
            let avg_acc: f64 =
                completed.iter().map(|r| r.accuracy).sum::<f64>() / completed.len() as f64;
            lines.push(Line::from(""));
            lines.push(Line::from(vec![
                Span::styled("Avg: ", Style::new().fg(tc.dim_text)),
                Span::styled(
                    format!("{avg_wpm:.0} wpm"),
                    Style::new().fg(tc.accent).bold(),
                ),
                Span::styled("  ", Style::new().fg(tc.dim_text)),
                Span::styled(
                    format!("{avg_acc:.0}% acc"),
                    Style::new().fg(tc.correct).bold(),
                ),
                Span::styled(
                    format!("  ({} sessions)", completed.len()),
                    Style::new().fg(tc.dim_text),
                ),
            ]));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(vec![
        Span::styled("↑↓", Style::new().fg(tc.accent)),
        Span::styled(" scroll  ", Style::new().fg(tc.dim_text)),
        Span::styled("Esc", Style::new().fg(tc.accent)),
        Span::styled(" back  ", Style::new().fg(tc.dim_text)),
        Span::styled("*", Style::new().fg(tc.accent)),
        Span::styled(" = incomplete", Style::new().fg(tc.dim_text)),
    ]));

    frame.render_widget(Paragraph::new(lines).block(block), inner);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn friendly_timestamp_full_iso() {
        assert_eq!(friendly_timestamp("2026-03-06T22:01:05"), "Mar 06  22:01");
    }

    #[test]
    fn friendly_timestamp_january() {
        assert_eq!(friendly_timestamp("2025-01-15T09:30:00"), "Jan 15  09:30");
    }

    #[test]
    fn friendly_timestamp_december() {
        assert_eq!(friendly_timestamp("2025-12-31T23:59:59"), "Dec 31  23:59");
    }

    #[test]
    fn friendly_timestamp_short_string_passthrough() {
        assert_eq!(friendly_timestamp("short"), "short");
    }

    #[test]
    fn friendly_timestamp_non_ascii_shown_as_is() {
        assert_eq!(
            friendly_timestamp("2026-1é-16T15:50:00"),
            "2026-1é-16T15:50:00"
        );
    }

    #[test]
    fn friendly_timestamp_invalid_month_shows_fallback() {
        // month "00" → wrapping_sub(1) overflows → fallback "???"
        assert_eq!(friendly_timestamp("2025-00-01T12:00:00"), "??? 01  12:00");
    }

    #[test]
    fn keyboard_hidden_when_terminal_too_short() {
        let needed = HEADER_ROWS + MIN_BODY_ROWS + KEYBOARD_HEIGHT;
        let short = compute_regions(Rect::new(0, 0, 80, needed - 1), true);
        let tall = compute_regions(Rect::new(0, 0, 80, needed), true);
        let tiny = compute_regions(Rect::new(0, 0, 80, HEADER_ROWS + 1), true);

        assert_eq!(short.keyboard_area.height, 0);
        assert!(!keyboard_fits(needed - 1));
        assert_eq!(tall.keyboard_area.height, KEYBOARD_HEIGHT);
        assert_eq!(tall.body.height, MIN_BODY_ROWS);
        assert_eq!(tiny.header.height, HEADER_ROWS);
    }

    #[test]
    fn long_line_scrolls_to_keep_cursor_in_view() {
        let narrow = [1; 100];

        assert_eq!(visible_range(&[1; 10], 9, 10), 0..10);
        assert_eq!(visible_range(&narrow, 5, 20), 0..20);
        assert_eq!(visible_range(&narrow, 50, 20), 40..60);
        assert_eq!(visible_range(&narrow, 95, 20), 80..100);
        assert_eq!(visible_range(&[2; 100], 50, 20), 45..55);
    }

    #[test]
    fn fit_line_keeps_the_cursor_and_joins_characters() {
        let tc = ThemeColors::from_theme(Theme::Dark);
        let cells = |text: &str| text.chars().map(|ch| Span::raw(ch.to_string())).collect();

        for width in 0..4 {
            let line = fit_line(cells("abcdefghij"), 5, width, &tc);
            assert!(
                width == 0 || line.to_string().contains('f'),
                "width {width}"
            );
        }
        assert_eq!(fit_line(cells("e\u{301}"), 0, 10, &tc).spans.len(), 1);
    }

    #[test]
    fn wrap_text_fits_width_without_losing_text() {
        let message = "Couldn't save: ~/文書/練習練習練習.toml\nUsing the defaults";
        let lines = wrap_text(message, 12);
        let squash = |text: &str| text.split_whitespace().collect::<String>();

        assert!(lines.iter().all(|line| line.width() <= 12));
        assert_eq!(squash(&lines.concat()), squash(message));
        assert!(lines.iter().any(|line| line.starts_with("Using")));
    }
}

struct KeyboardState<'a> {
    hint_coords: &'a [GridCoord],
    highlight_coord: Option<GridCoord>,
    highlight_color: Color,
    show_fingers: bool,
}

fn draw_keyboard(
    frame: &mut Frame,
    rows: &[Vec<KeyDef>],
    kbd_rects: &[Rc<[Rect]>],
    state: &KeyboardState,
    tc: &ThemeColors,
) {
    if let Some(outer) = bounding_rect(kbd_rects) {
        let pad_x: u16 = 2;
        let pad_y: u16 = 1;
        let padded = Rect::new(
            outer.x.saturating_sub(pad_x),
            outer.y.saturating_sub(pad_y),
            (outer.width + pad_x * 2).min(
                frame
                    .area()
                    .width
                    .saturating_sub(outer.x.saturating_sub(pad_x)),
            ),
            (outer.height + pad_y * 2).min(
                frame
                    .area()
                    .height
                    .saturating_sub(outer.y.saturating_sub(pad_y)),
            ),
        );
        frame.render_widget(
            Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(tc.dim_border)),
            padded,
        );
    }

    for (row_idx, row) in rows.iter().enumerate() {
        let Some(row_rects) = kbd_rects.get(row_idx) else {
            continue;
        };

        for (col_idx, key_def) in row.iter().enumerate() {
            let Some(&cell) = row_rects.get(col_idx) else {
                continue;
            };

            let is_hint = state.hint_coords.contains(&(row_idx, col_idx));
            let is_highlight = state.highlight_coord == Some((row_idx, col_idx));

            let border_color = if is_highlight {
                state.highlight_color
            } else if is_hint {
                tc.correct
            } else {
                tc.dim_border
            };
            let block = Block::new()
                .borders(Borders::ALL)
                .border_type(BorderType::Rounded)
                .border_style(Style::new().fg(border_color));

            let inner = block.inner(cell);
            frame.render_widget(block, cell);

            if is_hint && state.show_fingers {
                if let Some(finger) = finger_for_coord((row_idx, col_idx)) {
                    frame.render_widget(
                        Paragraph::new(Span::styled(
                            finger.label(),
                            Style::new().fg(tc.finger_label),
                        ))
                        .centered(),
                        cell,
                    );
                }
            }

            let label = key_def.label;
            let has_secondary = key_def.secondary.and_then(|s| match s {
                KeyCode::Char(c) => Some(c),
                _ => None,
            });

            let buf = frame.buffer_mut();
            let label_w = label.chars().count() as u16;
            let cx = inner.x + inner.width.saturating_sub(label_w) / 2;

            let label_fg = if is_highlight {
                tc.cursor_fg
            } else if is_hint {
                tc.correct
            } else {
                tc.key_label
            };
            let sec_fg = if is_highlight {
                tc.cursor_fg
            } else {
                tc.dim_text
            };

            if let Some(sec_char) = has_secondary {
                // A key with a shifted character shows the unshifted one at the bottom
                let cy = inner.y + inner.height.saturating_sub(1);
                if cx < inner.x + inner.width && cy < inner.y + inner.height {
                    buf.set_string(cx, cy, label, Style::new().fg(label_fg));
                }
                let s = sec_char.to_string();
                let sw = s.chars().count() as u16;
                let sx = inner.x + inner.width.saturating_sub(sw) / 2;
                if sx < inner.x + inner.width && inner.y < cy {
                    buf.set_string(sx, inner.y, &s, Style::new().fg(sec_fg));
                }
            } else {
                let cy = inner.y + inner.height / 2;
                if cx < inner.x + inner.width && cy < inner.y + inner.height {
                    buf.set_string(cx, cy, label, Style::new().fg(label_fg));
                }
            }
        }
    }
}
