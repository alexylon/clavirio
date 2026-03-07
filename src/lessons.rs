pub struct Lesson {
    pub key: char,
    pub label: &'static str,
    pub text: &'static str,
}

pub const LESSONS: &[Lesson] = &[
    Lesson {
        key: '1',
        label: "f j d k",
        text: include_str!("../assets/lessons/01_fjdk.txt"),
    },
    Lesson {
        key: '2',
        label: "a s d f j k l ;",
        text: include_str!("../assets/lessons/02_home_row.txt"),
    },
    Lesson {
        key: '3',
        label: "g h (home row)",
        text: include_str!("../assets/lessons/03_ghfj.txt"),
    },
    Lesson {
        key: '4',
        label: "e i r u",
        text: include_str!("../assets/lessons/04_eiru.txt"),
    },
    Lesson {
        key: '5',
        label: "q w e r t y u i o p",
        text: include_str!("../assets/lessons/05_top_row.txt"),
    },
    Lesson {
        key: '6',
        label: "z x c v b n m , .",
        text: include_str!("../assets/lessons/06_bottom_row.txt"),
    },
    Lesson {
        key: '7',
        label: "All Letters",
        text: include_str!("../assets/lessons/07_all_keys.txt"),
    },
    Lesson {
        key: '8',
        label: "Capitals & Shift",
        text: include_str!("../assets/lessons/08_capitals.txt"),
    },
    Lesson {
        key: '9',
        label: "0-9 Numbers",
        text: include_str!("../assets/lessons/09_numbers.txt"),
    },
    Lesson {
        key: 'a',
        label: "Punctuation & Symbols",
        text: include_str!("../assets/lessons/10_punctuation.txt"),
    },
    Lesson {
        key: 'b',
        label: "Common Words",
        text: include_str!("../assets/lessons/11_common_words.txt"),
    },
    Lesson {
        key: 'c',
        label: "Full Paragraphs",
        text: include_str!("../assets/lessons/12_paragraphs.txt"),
    },
];
