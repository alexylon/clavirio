pub struct Lesson {
    pub id: &'static str,
    pub label: &'static str,
    pub text: &'static str,
}

pub const LESSONS: &[Lesson] = &[
    Lesson {
        id: "fjdk",
        label: "f j d k",
        text: include_str!("../assets/lessons/01_fjdk.txt"),
    },
    Lesson {
        id: "home_row",
        label: "a s d f j k l ;",
        text: include_str!("../assets/lessons/02_home_row.txt"),
    },
    Lesson {
        id: "ghfj",
        label: "g h (home row)",
        text: include_str!("../assets/lessons/03_ghfj.txt"),
    },
    Lesson {
        id: "eiru",
        label: "e i r u",
        text: include_str!("../assets/lessons/04_eiru.txt"),
    },
    Lesson {
        id: "top_row",
        label: "q w e r t y u i o p",
        text: include_str!("../assets/lessons/05_top_row.txt"),
    },
    Lesson {
        id: "bottom_row",
        label: "z x c v b n m , .",
        text: include_str!("../assets/lessons/06_bottom_row.txt"),
    },
    Lesson {
        id: "all_keys",
        label: "All Letters",
        text: include_str!("../assets/lessons/07_all_keys.txt"),
    },
    Lesson {
        id: "capitals",
        label: "Capitals & Shift",
        text: include_str!("../assets/lessons/08_capitals.txt"),
    },
    Lesson {
        id: "numbers",
        label: "0-9 Numbers",
        text: include_str!("../assets/lessons/09_numbers.txt"),
    },
    Lesson {
        id: "punctuation",
        label: "Punctuation & Symbols",
        text: include_str!("../assets/lessons/10_punctuation.txt"),
    },
    Lesson {
        id: "common_words",
        label: "Common Words",
        text: include_str!("../assets/lessons/11_common_words.txt"),
    },
    Lesson {
        id: "paragraphs",
        label: "Full Paragraphs",
        text: include_str!("../assets/lessons/12_paragraphs.txt"),
    },
];
