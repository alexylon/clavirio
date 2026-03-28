use crate::settings::KeyboardLayout;

pub struct Lesson {
    pub id: &'static str,
    pub title: &'static str,
    pub keys: &'static str,
    pub text: &'static str,
}

// Each row follows the same finger pattern:
// 1. index, 2. middle, 3. ring, 4. pinky, 5. reach, 6. full row, 7. row + Shift

const LESSON_META: [(&str, &str); 27] = [
    ("lesson_01", "Home: Index"),
    ("lesson_02", "Home: Middle"),
    ("lesson_03", "Home: Ring"),
    ("lesson_04", "Home: Pinky"),
    ("lesson_05", "Home: Reach"),
    ("lesson_06", "Home Row"),
    ("lesson_07", "Home + Shift"),
    ("lesson_08", "Top: Index"),
    ("lesson_09", "Top: Middle"),
    ("lesson_10", "Top: Ring"),
    ("lesson_11", "Top: Pinky"),
    ("lesson_12", "Top: Reach"),
    ("lesson_13", "Top Row"),
    ("lesson_14", "Top + Shift"),
    ("lesson_15", "Bottom: Index"),
    ("lesson_16", "Bottom: Middle"),
    ("lesson_17", "Bottom: Ring"),
    ("lesson_18", "Bottom: Pinky"),
    ("lesson_19", "Bottom: Reach"),
    ("lesson_20", "Bottom Row"),
    ("lesson_21", "Bottom + Shift"),
    ("lesson_22", "Numbers"),
    ("lesson_23", "Symbols I"),
    ("lesson_24", "Symbols II"),
    ("lesson_25", "Symbols III"),
    ("lesson_26", "Symbols IV"),
    ("lesson_27", "Numbers & Symbols"),
];

macro_rules! layout_lessons {
    ($( [$idx:literal] $keys:literal => $path:literal ),* $(,)?) => {
        &[ $( Lesson {
            id: LESSON_META[$idx].0,
            title: LESSON_META[$idx].1,
            keys: $keys,
            text: include_str!($path),
        } ),* ]
    };
}

const QWERTY_LESSONS: &[Lesson] = layout_lessons![
    [ 0] "f j"                   => "../assets/lessons/qwerty/01_home_index.txt",
    [ 1] "d k"                   => "../assets/lessons/qwerty/02_home_middle.txt",
    [ 2] "s l"                   => "../assets/lessons/qwerty/03_home_ring.txt",
    [ 3] "a ;"                   => "../assets/lessons/qwerty/04_home_pinky.txt",
    [ 4] "g h"                   => "../assets/lessons/qwerty/05_home_reach.txt",
    [ 5] "a s d f g h j k l ;"   => "../assets/lessons/qwerty/06_home_row.txt",
    [ 6] "A S D F G H J K L"     => "../assets/lessons/qwerty/07_home_shift.txt",
    [ 7] "r u"                   => "../assets/lessons/qwerty/08_top_index.txt",
    [ 8] "e i"                   => "../assets/lessons/qwerty/09_top_middle.txt",
    [ 9] "w o"                   => "../assets/lessons/qwerty/10_top_ring.txt",
    [10] "q p"                   => "../assets/lessons/qwerty/11_top_pinky.txt",
    [11] "t y"                   => "../assets/lessons/qwerty/12_top_reach.txt",
    [12] "q w e r t y u i o p"   => "../assets/lessons/qwerty/13_top_row.txt",
    [13] "Q W E R T Y U I O P"   => "../assets/lessons/qwerty/14_top_shift.txt",
    [14] "v m"                   => "../assets/lessons/qwerty/15_bottom_index.txt",
    [15] "c ,"                   => "../assets/lessons/qwerty/16_bottom_middle.txt",
    [16] "x ."                   => "../assets/lessons/qwerty/17_bottom_ring.txt",
    [17] "z /"                   => "../assets/lessons/qwerty/18_bottom_pinky.txt",
    [18] "b n"                   => "../assets/lessons/qwerty/19_bottom_reach.txt",
    [19] "z x c v b n m , . /"   => "../assets/lessons/qwerty/20_bottom_row.txt",
    [20] "Z X C V B N M"         => "../assets/lessons/qwerty/21_bottom_shift.txt",
];

const DVORAK_LESSONS: &[Lesson] = layout_lessons![
    [ 0] "u h"                   => "../assets/lessons/dvorak/01_home_index.txt",
    [ 1] "e t"                   => "../assets/lessons/dvorak/02_home_middle.txt",
    [ 2] "o n"                   => "../assets/lessons/dvorak/03_home_ring.txt",
    [ 3] "a s"                   => "../assets/lessons/dvorak/04_home_pinky.txt",
    [ 4] "i d"                   => "../assets/lessons/dvorak/05_home_reach.txt",
    [ 5] "a o e u i d h t n s"   => "../assets/lessons/dvorak/06_home_row.txt",
    [ 6] "A O E U I D H T N S"   => "../assets/lessons/dvorak/07_home_shift.txt",
    [ 7] "p g"                   => "../assets/lessons/dvorak/08_top_index.txt",
    [ 8] ". c"                   => "../assets/lessons/dvorak/09_top_middle.txt",
    [ 9] ", r"                   => "../assets/lessons/dvorak/10_top_ring.txt",
    [10] "' l"                   => "../assets/lessons/dvorak/11_top_pinky.txt",
    [11] "y f"                   => "../assets/lessons/dvorak/12_top_reach.txt",
    [12] "' , . p y f g c r l"   => "../assets/lessons/dvorak/13_top_row.txt",
    [13] "P Y F G C R L"         => "../assets/lessons/dvorak/14_top_shift.txt",
    [14] "k m"                   => "../assets/lessons/dvorak/15_bottom_index.txt",
    [15] "j w"                   => "../assets/lessons/dvorak/16_bottom_middle.txt",
    [16] "q v"                   => "../assets/lessons/dvorak/17_bottom_ring.txt",
    [17] "; z"                   => "../assets/lessons/dvorak/18_bottom_pinky.txt",
    [18] "x b"                   => "../assets/lessons/dvorak/19_bottom_reach.txt",
    [19] "; q j k x b m w v z"   => "../assets/lessons/dvorak/20_bottom_row.txt",
    [20] "Q J K X B M W V Z"     => "../assets/lessons/dvorak/21_bottom_shift.txt",
];

const COLEMAK_LESSONS: &[Lesson] = layout_lessons![
    [ 0] "t n"                   => "../assets/lessons/colemak/01_home_index.txt",
    [ 1] "s e"                   => "../assets/lessons/colemak/02_home_middle.txt",
    [ 2] "r i"                   => "../assets/lessons/colemak/03_home_ring.txt",
    [ 3] "a o"                   => "../assets/lessons/colemak/04_home_pinky.txt",
    [ 4] "d h"                   => "../assets/lessons/colemak/05_home_reach.txt",
    [ 5] "a r s t d h n e i o"   => "../assets/lessons/colemak/06_home_row.txt",
    [ 6] "A R S T D H N E I O"   => "../assets/lessons/colemak/07_home_shift.txt",
    [ 7] "p l"                   => "../assets/lessons/colemak/08_top_index.txt",
    [ 8] "f u"                   => "../assets/lessons/colemak/09_top_middle.txt",
    [ 9] "w y"                   => "../assets/lessons/colemak/10_top_ring.txt",
    [10] "q ;"                   => "../assets/lessons/colemak/11_top_pinky.txt",
    [11] "g j"                   => "../assets/lessons/colemak/12_top_reach.txt",
    [12] "q w f p g j l u y ;"   => "../assets/lessons/colemak/13_top_row.txt",
    [13] "Q W F P G J L U Y"     => "../assets/lessons/colemak/14_top_shift.txt",
    [14] "v m"                   => "../assets/lessons/colemak/15_bottom_index.txt",
    [15] "c ,"                   => "../assets/lessons/colemak/16_bottom_middle.txt",
    [16] "x ."                   => "../assets/lessons/colemak/17_bottom_ring.txt",
    [17] "z /"                   => "../assets/lessons/colemak/18_bottom_pinky.txt",
    [18] "b k"                   => "../assets/lessons/colemak/19_bottom_reach.txt",
    [19] "z x c v b k m , . /"   => "../assets/lessons/colemak/20_bottom_row.txt",
    [20] "Z X C V B K M"         => "../assets/lessons/colemak/21_bottom_shift.txt",
];

const SHARED_LESSONS: &[Lesson] = layout_lessons![
    [21] "1 2 3 4 5 6 7 8 9 0"   => "../assets/lessons/shared/22_numbers.txt",
    [22] "/ ? : ' \""             => "../assets/lessons/shared/23_symbols_1.txt",
    [23] "[ ] { } < > | \\"       => "../assets/lessons/shared/24_symbols_2.txt",
    [24] "` ~ ! - _ + ="          => "../assets/lessons/shared/25_symbols_3.txt",
    [25] "@ # $ % ^ & * ( )"     => "../assets/lessons/shared/26_symbols_4.txt",
    [26] ""                       => "../assets/lessons/shared/27_review.txt",
];

pub fn lessons_for_layout(layout: KeyboardLayout) -> Vec<&'static Lesson> {
    let specific = match layout {
        KeyboardLayout::Qwerty => QWERTY_LESSONS,
        KeyboardLayout::Dvorak => DVORAK_LESSONS,
        KeyboardLayout::Colemak => COLEMAK_LESSONS,
    };
    specific.iter().chain(SHARED_LESSONS.iter()).collect()
}

pub fn lesson_count() -> usize {
    LESSON_META.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LAYOUT_LESSON_COUNT: usize = 21;

    #[test]
    fn all_layouts_have_same_lesson_count() {
        assert_eq!(QWERTY_LESSONS.len(), LAYOUT_LESSON_COUNT);
        assert_eq!(DVORAK_LESSONS.len(), LAYOUT_LESSON_COUNT);
        assert_eq!(COLEMAK_LESSONS.len(), LAYOUT_LESSON_COUNT);
    }

    #[test]
    fn lesson_ids_match_across_layouts() {
        for i in 0..LAYOUT_LESSON_COUNT {
            assert_eq!(QWERTY_LESSONS[i].id, DVORAK_LESSONS[i].id);
            assert_eq!(QWERTY_LESSONS[i].id, COLEMAK_LESSONS[i].id);
        }
    }

    #[test]
    fn total_lesson_count() {
        assert_eq!(lesson_count(), 27);
    }

    #[test]
    fn lessons_for_layout_includes_shared() {
        let lessons = lessons_for_layout(KeyboardLayout::Qwerty);
        assert_eq!(lessons.len(), 27);
        assert_eq!(lessons[0].id, "lesson_01");
        assert_eq!(lessons[26].id, "lesson_27");
    }
}
