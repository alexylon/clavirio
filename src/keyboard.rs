use crossterm::event::{KeyCode, ModifierKeyCode};
use std::collections::HashMap;

use crate::settings::KeyboardLayout;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum KeyWidth {
    Normal,
    Wide,
    Spacebar,
}

pub type GridCoord = (usize, usize);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct KeyDef {
    pub primary: KeyCode,
    pub secondary: Option<KeyCode>,
    pub width: KeyWidth,
    pub label: &'static str,
}

macro_rules! key {
    ($code:expr, $label:literal) => {
        KeyDef {
            primary: $code,
            secondary: None,
            width: KeyWidth::Normal,
            label: $label,
        }
    };
    ($code:expr, $shift:expr, $label:literal) => {
        KeyDef {
            primary: $code,
            secondary: Some($shift),
            width: KeyWidth::Normal,
            label: $label,
        }
    };
}

macro_rules! wide {
    ($code:expr, $label:literal) => {
        KeyDef {
            primary: $code,
            secondary: None,
            width: KeyWidth::Wide,
            label: $label,
        }
    };
}

// --- QWERTY layout ---

/// The keys in the number row, the widest
pub const NUMBER_ROW_KEYS: usize = 14;

const QWERTY_NUMBER_ROW: [KeyDef; NUMBER_ROW_KEYS] = [
    key!(KeyCode::Char('`'), KeyCode::Char('~'), "`"),
    key!(KeyCode::Char('1'), KeyCode::Char('!'), "1"),
    key!(KeyCode::Char('2'), KeyCode::Char('@'), "2"),
    key!(KeyCode::Char('3'), KeyCode::Char('#'), "3"),
    key!(KeyCode::Char('4'), KeyCode::Char('$'), "4"),
    key!(KeyCode::Char('5'), KeyCode::Char('%'), "5"),
    key!(KeyCode::Char('6'), KeyCode::Char('^'), "6"),
    key!(KeyCode::Char('7'), KeyCode::Char('&'), "7"),
    key!(KeyCode::Char('8'), KeyCode::Char('*'), "8"),
    key!(KeyCode::Char('9'), KeyCode::Char('('), "9"),
    key!(KeyCode::Char('0'), KeyCode::Char(')'), "0"),
    key!(KeyCode::Char('-'), KeyCode::Char('_'), "-"),
    key!(KeyCode::Char('='), KeyCode::Char('+'), "="),
    key!(KeyCode::Char('\\'), KeyCode::Char('|'), "\\"),
];

const QWERTY_TOP_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('Q'), "Q"),
    key!(KeyCode::Char('W'), "W"),
    key!(KeyCode::Char('E'), "E"),
    key!(KeyCode::Char('R'), "R"),
    key!(KeyCode::Char('T'), "T"),
    key!(KeyCode::Char('Y'), "Y"),
    key!(KeyCode::Char('U'), "U"),
    key!(KeyCode::Char('I'), "I"),
    key!(KeyCode::Char('O'), "O"),
    key!(KeyCode::Char('P'), "P"),
    key!(KeyCode::Char('['), KeyCode::Char('{'), "["),
    key!(KeyCode::Char(']'), KeyCode::Char('}'), "]"),
];

const QWERTY_HOME_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('A'), "A"),
    key!(KeyCode::Char('S'), "S"),
    key!(KeyCode::Char('D'), "D"),
    key!(KeyCode::Char('F'), "F"),
    key!(KeyCode::Char('G'), "G"),
    key!(KeyCode::Char('H'), "H"),
    key!(KeyCode::Char('J'), "J"),
    key!(KeyCode::Char('K'), "K"),
    key!(KeyCode::Char('L'), "L"),
    key!(KeyCode::Char(';'), KeyCode::Char(':'), ";"),
    key!(KeyCode::Char('\''), KeyCode::Char('"'), "'"),
    wide!(KeyCode::Enter, "⏎"),
];

const QWERTY_BOTTOM_ROW: [KeyDef; 12] = [
    wide!(KeyCode::Modifier(ModifierKeyCode::LeftShift), "⇧"),
    key!(KeyCode::Char('Z'), "Z"),
    key!(KeyCode::Char('X'), "X"),
    key!(KeyCode::Char('C'), "C"),
    key!(KeyCode::Char('V'), "V"),
    key!(KeyCode::Char('B'), "B"),
    key!(KeyCode::Char('N'), "N"),
    key!(KeyCode::Char('M'), "M"),
    key!(KeyCode::Char(','), KeyCode::Char('<'), ","),
    key!(KeyCode::Char('.'), KeyCode::Char('>'), "."),
    key!(KeyCode::Char('/'), KeyCode::Char('?'), "/"),
    wide!(KeyCode::Modifier(ModifierKeyCode::RightShift), "⇧"),
];

#[cfg(target_os = "macos")]
const MODIFIER_ROW: [KeyDef; 7] = [
    key!(KeyCode::Modifier(ModifierKeyCode::LeftControl), "⌃"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftAlt), "⌥"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftSuper), "⌘"),
    KeyDef {
        primary: KeyCode::Char(' '),
        secondary: None,
        width: KeyWidth::Spacebar,
        label: "",
    },
    key!(KeyCode::Modifier(ModifierKeyCode::RightSuper), "⌘"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightAlt), "⌥"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightControl), "⌃"),
];

#[cfg(not(target_os = "macos"))]
const MODIFIER_ROW: [KeyDef; 7] = [
    key!(KeyCode::Modifier(ModifierKeyCode::LeftControl), "⌃"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftSuper), "Win"),
    key!(KeyCode::Modifier(ModifierKeyCode::LeftAlt), "Alt"),
    KeyDef {
        primary: KeyCode::Char(' '),
        secondary: None,
        width: KeyWidth::Spacebar,
        label: "",
    },
    key!(KeyCode::Modifier(ModifierKeyCode::RightAlt), "Alt"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightSuper), "Win"),
    key!(KeyCode::Modifier(ModifierKeyCode::RightControl), "⌃"),
];

// --- Dvorak layout ---

const DVORAK_NUMBER_ROW: [KeyDef; NUMBER_ROW_KEYS] = [
    key!(KeyCode::Char('`'), KeyCode::Char('~'), "`"),
    key!(KeyCode::Char('1'), KeyCode::Char('!'), "1"),
    key!(KeyCode::Char('2'), KeyCode::Char('@'), "2"),
    key!(KeyCode::Char('3'), KeyCode::Char('#'), "3"),
    key!(KeyCode::Char('4'), KeyCode::Char('$'), "4"),
    key!(KeyCode::Char('5'), KeyCode::Char('%'), "5"),
    key!(KeyCode::Char('6'), KeyCode::Char('^'), "6"),
    key!(KeyCode::Char('7'), KeyCode::Char('&'), "7"),
    key!(KeyCode::Char('8'), KeyCode::Char('*'), "8"),
    key!(KeyCode::Char('9'), KeyCode::Char('('), "9"),
    key!(KeyCode::Char('0'), KeyCode::Char(')'), "0"),
    key!(KeyCode::Char('['), KeyCode::Char('{'), "["),
    key!(KeyCode::Char(']'), KeyCode::Char('}'), "]"),
    key!(KeyCode::Char('\\'), KeyCode::Char('|'), "\\"),
];

const DVORAK_TOP_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('\''), KeyCode::Char('"'), "'"),
    key!(KeyCode::Char(','), KeyCode::Char('<'), ","),
    key!(KeyCode::Char('.'), KeyCode::Char('>'), "."),
    key!(KeyCode::Char('P'), "P"),
    key!(KeyCode::Char('Y'), "Y"),
    key!(KeyCode::Char('F'), "F"),
    key!(KeyCode::Char('G'), "G"),
    key!(KeyCode::Char('C'), "C"),
    key!(KeyCode::Char('R'), "R"),
    key!(KeyCode::Char('L'), "L"),
    key!(KeyCode::Char('/'), KeyCode::Char('?'), "/"),
    key!(KeyCode::Char('='), KeyCode::Char('+'), "="),
];

const DVORAK_HOME_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('A'), "A"),
    key!(KeyCode::Char('O'), "O"),
    key!(KeyCode::Char('E'), "E"),
    key!(KeyCode::Char('U'), "U"),
    key!(KeyCode::Char('I'), "I"),
    key!(KeyCode::Char('D'), "D"),
    key!(KeyCode::Char('H'), "H"),
    key!(KeyCode::Char('T'), "T"),
    key!(KeyCode::Char('N'), "N"),
    key!(KeyCode::Char('S'), "S"),
    key!(KeyCode::Char('-'), KeyCode::Char('_'), "-"),
    wide!(KeyCode::Enter, "⏎"),
];

const DVORAK_BOTTOM_ROW: [KeyDef; 12] = [
    wide!(KeyCode::Modifier(ModifierKeyCode::LeftShift), "⇧"),
    key!(KeyCode::Char(';'), KeyCode::Char(':'), ";"),
    key!(KeyCode::Char('Q'), "Q"),
    key!(KeyCode::Char('J'), "J"),
    key!(KeyCode::Char('K'), "K"),
    key!(KeyCode::Char('X'), "X"),
    key!(KeyCode::Char('B'), "B"),
    key!(KeyCode::Char('M'), "M"),
    key!(KeyCode::Char('W'), "W"),
    key!(KeyCode::Char('V'), "V"),
    key!(KeyCode::Char('Z'), "Z"),
    wide!(KeyCode::Modifier(ModifierKeyCode::RightShift), "⇧"),
];

// --- Colemak layout ---

const COLEMAK_TOP_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('Q'), "Q"),
    key!(KeyCode::Char('W'), "W"),
    key!(KeyCode::Char('F'), "F"),
    key!(KeyCode::Char('P'), "P"),
    key!(KeyCode::Char('G'), "G"),
    key!(KeyCode::Char('J'), "J"),
    key!(KeyCode::Char('L'), "L"),
    key!(KeyCode::Char('U'), "U"),
    key!(KeyCode::Char('Y'), "Y"),
    key!(KeyCode::Char(';'), KeyCode::Char(':'), ";"),
    key!(KeyCode::Char('['), KeyCode::Char('{'), "["),
    key!(KeyCode::Char(']'), KeyCode::Char('}'), "]"),
];

const COLEMAK_HOME_ROW: [KeyDef; 12] = [
    key!(KeyCode::Char('A'), "A"),
    key!(KeyCode::Char('R'), "R"),
    key!(KeyCode::Char('S'), "S"),
    key!(KeyCode::Char('T'), "T"),
    key!(KeyCode::Char('D'), "D"),
    key!(KeyCode::Char('H'), "H"),
    key!(KeyCode::Char('N'), "N"),
    key!(KeyCode::Char('E'), "E"),
    key!(KeyCode::Char('I'), "I"),
    key!(KeyCode::Char('O'), "O"),
    key!(KeyCode::Char('\''), KeyCode::Char('"'), "'"),
    wide!(KeyCode::Enter, "⏎"),
];

const COLEMAK_BOTTOM_ROW: [KeyDef; 12] = [
    wide!(KeyCode::Modifier(ModifierKeyCode::LeftShift), "⇧"),
    key!(KeyCode::Char('Z'), "Z"),
    key!(KeyCode::Char('X'), "X"),
    key!(KeyCode::Char('C'), "C"),
    key!(KeyCode::Char('V'), "V"),
    key!(KeyCode::Char('B'), "B"),
    key!(KeyCode::Char('K'), "K"),
    key!(KeyCode::Char('M'), "M"),
    key!(KeyCode::Char(','), KeyCode::Char('<'), ","),
    key!(KeyCode::Char('.'), KeyCode::Char('>'), "."),
    key!(KeyCode::Char('/'), KeyCode::Char('?'), "/"),
    wide!(KeyCode::Modifier(ModifierKeyCode::RightShift), "⇧"),
];

pub fn build_keyboard_rows(layout: KeyboardLayout) -> Vec<Vec<KeyDef>> {
    match layout {
        KeyboardLayout::Qwerty => vec![
            QWERTY_NUMBER_ROW.to_vec(),
            QWERTY_TOP_ROW.to_vec(),
            QWERTY_HOME_ROW.to_vec(),
            QWERTY_BOTTOM_ROW.to_vec(),
            MODIFIER_ROW.to_vec(),
        ],
        KeyboardLayout::Dvorak => vec![
            DVORAK_NUMBER_ROW.to_vec(),
            DVORAK_TOP_ROW.to_vec(),
            DVORAK_HOME_ROW.to_vec(),
            DVORAK_BOTTOM_ROW.to_vec(),
            MODIFIER_ROW.to_vec(),
        ],
        KeyboardLayout::Colemak => vec![
            QWERTY_NUMBER_ROW.to_vec(),
            COLEMAK_TOP_ROW.to_vec(),
            COLEMAK_HOME_ROW.to_vec(),
            COLEMAK_BOTTOM_ROW.to_vec(),
            MODIFIER_ROW.to_vec(),
        ],
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Finger {
    Pinky,
    Ring,
    Middle,
    Index,
    Thumb,
}

impl Finger {
    pub fn label(self) -> &'static str {
        match self {
            Finger::Pinky => "P",
            Finger::Ring => "R",
            Finger::Middle => "M",
            Finger::Index => "I",
            Finger::Thumb => "T",
        }
    }
}

/// A key's column counted as in the top and home rows: the number row starts
/// with ` and the bottom row with the left Shift, both typed by the pinky
/// before the 1 and Z keys, so their other keys sit one column right.
fn aligned_col((row, col): GridCoord) -> usize {
    match row {
        0 | 3 => col.saturating_sub(1),
        _ => col,
    }
}

fn finger_for_col(col: usize) -> Finger {
    match col {
        0 => Finger::Pinky,
        1 => Finger::Ring,
        2 => Finger::Middle,
        3..=6 => Finger::Index,
        7 => Finger::Middle,
        8 => Finger::Ring,
        _ => Finger::Pinky,
    }
}

pub fn finger_for_coord(coord: GridCoord) -> Option<Finger> {
    match coord.0 {
        0..=3 => Some(finger_for_col(aligned_col(coord))),
        4 => Some(Finger::Thumb),
        _ => None,
    }
}

/// Keys per row typed by the left hand, counted as in the top row
const LEFT_HAND_KEYS: usize = 5;

fn is_left_hand(coord: GridCoord) -> bool {
    coord.0 <= 3 && aligned_col(coord) < LEFT_HAND_KEYS
}

/// The Shift to hold with a key: the one on the other hand.
pub fn shift_for(coord: GridCoord) -> KeyCode {
    KeyCode::Modifier(if is_left_hand(coord) {
        ModifierKeyCode::RightShift
    } else {
        ModifierKeyCode::LeftShift
    })
}

pub fn build_keycode_grid_map(rows: &[Vec<KeyDef>]) -> HashMap<KeyCode, GridCoord> {
    let mut map = HashMap::new();

    for (row_idx, row) in rows.iter().enumerate() {
        for (col_idx, key_def) in row.iter().enumerate() {
            map.insert(key_def.primary, (row_idx, col_idx));
            if let Some(secondary) = key_def.secondary {
                map.insert(secondary, (row_idx, col_idx));
            }
        }
    }

    map
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grid_map_contains_primary_keys() {
        let rows = build_keyboard_rows(KeyboardLayout::Qwerty);
        let map = build_keycode_grid_map(&rows);
        // 'A' is in home row (row 2), first key (col 0)
        assert_eq!(map.get(&KeyCode::Char('A')), Some(&(2, 0)));
        // 'Q' is in top row (row 1), first key (col 0)
        assert_eq!(map.get(&KeyCode::Char('Q')), Some(&(1, 0)));
    }

    #[test]
    fn grid_map_contains_secondary_keys() {
        let rows = build_keyboard_rows(KeyboardLayout::Qwerty);
        let map = build_keycode_grid_map(&rows);
        // '!' is secondary of '1', which is number row (row 0), col 1 after '`'
        assert_eq!(map.get(&KeyCode::Char('!')), Some(&(0, 1)));
        // '@' is secondary of '2', row 0, col 2
        assert_eq!(map.get(&KeyCode::Char('@')), Some(&(0, 2)));
    }

    #[test]
    fn grid_map_secondary_shares_coord_with_primary() {
        let rows = build_keyboard_rows(KeyboardLayout::Qwerty);
        let map = build_keycode_grid_map(&rows);
        // ';' and ':' should map to the same grid position
        let semi = map.get(&KeyCode::Char(';'));
        let colon = map.get(&KeyCode::Char(':'));
        assert!(semi.is_some());
        assert_eq!(semi, colon);
    }

    #[test]
    fn shift_is_on_the_other_hand() {
        let left = KeyCode::Modifier(ModifierKeyCode::LeftShift);
        let right = KeyCode::Modifier(ModifierKeyCode::RightShift);
        let map = build_keycode_grid_map(&build_keyboard_rows(KeyboardLayout::Qwerty));
        for (ch, shift) in [
            ('~', right),
            ('%', right),
            ('^', left),
            ('T', right),
            ('Y', left),
            ('G', right),
            ('H', left),
            ('B', right),
            ('N', left),
            ('"', left),
        ] {
            assert_eq!(shift_for(map[&KeyCode::Char(ch)]), shift, "{ch}");
        }
    }

    #[test]
    fn number_row_fingers_follow_the_backtick() {
        let map = build_keycode_grid_map(&build_keyboard_rows(KeyboardLayout::Qwerty));
        for (ch, finger) in [
            ('`', Finger::Pinky),
            ('1', Finger::Pinky),
            ('2', Finger::Ring),
            ('3', Finger::Middle),
            ('4', Finger::Index),
            ('7', Finger::Index),
            ('8', Finger::Middle),
            ('0', Finger::Pinky),
        ] {
            assert_eq!(
                finger_for_coord(map[&KeyCode::Char(ch)]),
                Some(finger),
                "{ch}"
            );
        }
    }

    #[test]
    fn grid_map_spacebar_mapped() {
        let rows = build_keyboard_rows(KeyboardLayout::Qwerty);
        let map = build_keycode_grid_map(&rows);
        assert!(map.contains_key(&KeyCode::Char(' ')));
    }
}
