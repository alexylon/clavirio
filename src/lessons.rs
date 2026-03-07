pub struct Lesson {
    pub label: &'static str,
    pub text: &'static str,
}

pub const LESSONS: &[Lesson] = &[
    Lesson {
        label: "f j d k",
        text: include_str!("../assets/lessons/01_fjdk.txt"),
    },
    Lesson {
        label: "a s d f j k l ;",
        text: include_str!("../assets/lessons/02_home_row.txt"),
    },
    Lesson {
        label: "g h (home row)",
        text: include_str!("../assets/lessons/03_ghfj.txt"),
    },
    Lesson {
        label: "e i r u",
        text: include_str!("../assets/lessons/04_eiru.txt"),
    },
    Lesson {
        label: "q w e r t y u i o p",
        text: include_str!("../assets/lessons/05_top_row.txt"),
    },
    Lesson {
        label: "z x c v b n m , .",
        text: include_str!("../assets/lessons/06_bottom_row.txt"),
    },
    Lesson {
        label: "All Letters",
        text: include_str!("../assets/lessons/07_all_keys.txt"),
    },
    Lesson {
        label: "Capitals & Shift",
        text: include_str!("../assets/lessons/08_capitals.txt"),
    },
    Lesson {
        label: "0-9 Numbers",
        text: include_str!("../assets/lessons/09_numbers.txt"),
    },
    Lesson {
        label: "Punctuation & Symbols",
        text: include_str!("../assets/lessons/10_punctuation.txt"),
    },
    Lesson {
        label: "Common Words",
        text: include_str!("../assets/lessons/11_common_words.txt"),
    },
    Lesson {
        label: "Full Paragraphs",
        text: include_str!("../assets/lessons/12_paragraphs.txt"),
    },
];
