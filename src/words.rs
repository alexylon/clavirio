const ENGLISH_200: &str = include_str!("../assets/words/english_200.txt");
const ENGLISH_1K: &str = include_str!("../assets/words/english_1k.txt");
const RUST: &str = include_str!("../assets/words/rust.txt");
const PYTHON: &str = include_str!("../assets/words/python.txt");
const JAVASCRIPT: &str = include_str!("../assets/words/javascript.txt");
const GO: &str = include_str!("../assets/words/go.txt");
const C_CPP: &str = include_str!("../assets/words/c_cpp.txt");
const JAVA: &str = include_str!("../assets/words/java.txt");
const HTML_CSS: &str = include_str!("../assets/words/html_css.txt");
const QUOTES: &str = include_str!("../assets/quotes/english.txt");

use std::sync::Arc;

use unicode_normalization::char::is_combining_mark;
use unicode_width::UnicodeWidthStr;

const WORDS_CATEGORY: &str = "Random Words";
const CODE_CATEGORY: &str = "Code";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordList {
    English200,
    English1k,
    Rust,
    Python,
    JavaScript,
    Go,
    CCpp,
    Java,
    HtmlCss,
}

impl WordList {
    pub fn words(self) -> Vec<&'static str> {
        let src = match self {
            Self::English200 => ENGLISH_200,
            Self::English1k => ENGLISH_1K,
            Self::Rust => RUST,
            Self::Python => PYTHON,
            Self::JavaScript => JAVASCRIPT,
            Self::Go => GO,
            Self::CCpp => C_CPP,
            Self::Java => JAVA,
            Self::HtmlCss => HTML_CSS,
        };
        src.split_whitespace().collect()
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::English200 => "english 200",
            Self::English1k => "english 1k",
            Self::Rust => "rust",
            Self::Python => "python",
            Self::JavaScript => "javascript",
            Self::Go => "go",
            Self::CCpp => "c/c++",
            Self::Java => "java",
            Self::HtmlCss => "html/css",
        }
    }

    /// The names `-l` takes for the list, the first being the one shown in messages
    pub fn names(self) -> &'static [&'static str] {
        match self {
            Self::English200 => &["200"],
            Self::English1k => &["1k"],
            Self::Rust => &["rust"],
            Self::Python => &["python"],
            Self::JavaScript => &["javascript", "js"],
            Self::Go => &["go"],
            Self::CCpp => &["c", "cpp", "c++"],
            Self::Java => &["java"],
            Self::HtmlCss => &["html", "css"],
        }
    }

    pub fn from_name(name: &str) -> Option<Self> {
        Self::all()
            .iter()
            .copied()
            .find(|list| list.names().contains(&name))
    }

    pub fn is_code(self) -> bool {
        !matches!(self, Self::English200 | Self::English1k)
    }

    pub fn category(self) -> &'static str {
        if self.is_code() {
            CODE_CATEGORY
        } else {
            WORDS_CATEGORY
        }
    }

    pub fn all() -> &'static [WordList] {
        &[
            Self::English200,
            Self::English1k,
            Self::Rust,
            Self::Python,
            Self::JavaScript,
            Self::Go,
            Self::CCpp,
            Self::Java,
            Self::HtmlCss,
        ]
    }
}

/// Words loaded from a file for random words and timed practice
#[derive(Debug, Clone)]
pub struct CustomList {
    /// The file name, shown as the list's label
    name: String,
    /// The file's full path, which tells files of the same name apart in the history
    pub path: String,
    /// Shared, as `n` starts the practice again from the same words
    words: Arc<[String]>,
}

/// The widest word a custom list keeps, as a file in a language written without
/// spaces would give a whole line as one word
pub const MAX_WORD_WIDTH: usize = LINE_WIDTH;

impl CustomList {
    /// Words that can't be seen, such as a lone joiner, or are wider than
    /// `MAX_WORD_WIDTH` are left out; `None` if no word is left
    pub fn new(name: String, path: String, words: Vec<String>) -> Option<Self> {
        let words: Vec<String> = words
            .into_iter()
            .filter(|word| (1..=MAX_WORD_WIDTH).contains(&word.width()))
            .collect();
        (!words.is_empty()).then(|| Self {
            name,
            path,
            words: words.into(),
        })
    }
}

/// Where random words and timed practice take their words from
#[derive(Debug, Clone)]
pub enum WordSource {
    BuiltIn(WordList),
    Custom(CustomList),
}

impl From<WordList> for WordSource {
    fn from(list: WordList) -> Self {
        Self::BuiltIn(list)
    }
}

impl WordSource {
    fn words(&self) -> Vec<&str> {
        match self {
            Self::BuiltIn(list) => list.words(),
            Self::Custom(list) => list.words.iter().map(String::as_str).collect(),
        }
    }

    pub fn label(&self) -> &str {
        match self {
            Self::BuiltIn(list) => list.label(),
            Self::Custom(list) => &list.name,
        }
    }

    /// A custom list counts as words, not code
    fn is_code(&self) -> bool {
        matches!(self, Self::BuiltIn(list) if list.is_code())
    }

    pub fn category(&self) -> &'static str {
        match self {
            Self::BuiltIn(list) => list.category(),
            Self::Custom(_) => WORDS_CATEGORY,
        }
    }
}

/// A simple random number generator (xorshift64) seeded from the clock, so no
/// dependency is needed.
struct Rng(u64);

impl Rng {
    fn from_time() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        // A zero seed would only ever give zeros
        Self(seed | 1)
    }

    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }

    fn usize(&mut self, bound: usize) -> usize {
        (self.next() % bound as u64) as usize
    }
}

const LINE_WIDTH: usize = 60;

fn wrap_words<I: IntoIterator<Item = S>, S: AsRef<str>>(words: I) -> String {
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut current_width = 0;
    for word in words {
        let word = word.as_ref();
        let width = word.width();
        if !current_line.is_empty() && current_width + 1 + width > LINE_WIDTH {
            lines.push(std::mem::take(&mut current_line));
            current_width = 0;
        }
        if !current_line.is_empty() {
            current_line.push(' ');
            current_width += 1;
        }
        current_line.push_str(word);
        current_width += width;
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines.join("\n")
}

fn random_pick_and_wrap(pool: &[&str], word_count: usize) -> String {
    if pool.is_empty() {
        return String::new();
    }
    let mut rng = Rng::from_time();
    let words: Vec<&str> = (0..word_count)
        .map(|_| pool[rng.usize(pool.len())])
        .collect();
    wrap_words(words)
}

/// Build a weighted pool: repeat each word by its score.
fn weighted_pool<'a>(scored: &[(&'a str, usize)]) -> Vec<&'a str> {
    scored
        .iter()
        .flat_map(|&(w, score)| std::iter::repeat_n(w, score))
        .collect()
}

fn weighted_text(scored: &[(&str, usize)], word_count: usize) -> String {
    random_pick_and_wrap(&weighted_pool(scored), word_count)
}

pub fn generate_text(list: WordList, word_count: usize) -> String {
    let pool = list.words();
    if pool.is_empty() {
        return String::new();
    }
    random_pick_and_wrap(&pool, word_count)
}

enum Mark {
    After(&'static str),
    /// A quote or bracket from `PAIRED_PUNCTUATION`
    Around(char),
}

const PUNCTUATION_MARKS: &[Mark] = &[
    Mark::After("."),
    Mark::After(","),
    Mark::After(";"),
    Mark::After(":"),
    Mark::After("!"),
    Mark::After("?"),
    Mark::After("..."),
    Mark::Around('\''),
    Mark::Around('"'),
    Mark::Around('('),
];
const HYPHEN: char = '-';
/// Marks that join two words, like `well-known` and `snake_case`
const WORD_JOINERS: [char; 2] = [HYPHEN, '_'];
const NUMBER_TOKENS: &[&str] = &[
    "0", "1", "2", "3", "4", "5", "10", "42", "99", "100", "123", "256", "1000", "2025",
];
const PCT_BASE: usize = 100;
const NUMBER_CHANCE_PCT: usize = 15;
const PUNCTUATION_CHANCE_PCT: usize = 20;

pub fn generate_text_mixed(
    list: &WordSource,
    word_count: usize,
    punctuation: bool,
    numbers: bool,
) -> String {
    let base_pool = list.words();
    if base_pool.is_empty() {
        return String::new();
    }
    let mut rng = Rng::from_time();

    let words: Vec<String> = (0..word_count)
        .map(|_| {
            let word = base_pool[rng.usize(base_pool.len())];

            if numbers && rng.usize(PCT_BASE) < NUMBER_CHANCE_PCT {
                return NUMBER_TOKENS[rng.usize(NUMBER_TOKENS.len())].to_string();
            }

            if punctuation && is_plain(word) && rng.usize(PCT_BASE) < PUNCTUATION_CHANCE_PCT {
                // A hyphen joins two words, as often as each mark is added, but
                // would give nonsense like `fn-impl` in code
                let joined = (!list.is_code() && rng.usize(PUNCTUATION_MARKS.len() + 1) == 0)
                    .then(|| base_pool[rng.usize(base_pool.len())])
                    .filter(|other| is_plain(other))
                    .map(|other| format!("{word}{HYPHEN}{other}"))
                    .filter(|joined| joined.width() <= LINE_WIDTH);
                let punctuated = joined.unwrap_or_else(|| {
                    match PUNCTUATION_MARKS[rng.usize(PUNCTUATION_MARKS.len())] {
                        Mark::After(mark) => format!("{word}{mark}"),
                        Mark::Around(punct) => with_punctuation(word, punct),
                    }
                });
                // A word from a custom list can be nearly as wide as a line
                if punctuated.width() <= LINE_WIDTH {
                    return punctuated;
                }
            }

            word.to_string()
        })
        .collect();

    wrap_words(words)
}

/// A word or identifier in any script, not a code symbol such as `+=` or a word
/// that has punctuation already, which is left as it is. Marks such as accents
/// written apart count, as some scripts have no other way to write them.
fn is_plain(word: &str) -> bool {
    word.chars()
        .all(|c| c.is_alphanumeric() || c == '_' || is_combining_mark(c))
}

const BIGRAMS: &[&str] = &[
    "th", "he", "in", "er", "an", "re", "on", "at", "en", "nd", "ti", "es", "or", "te", "of", "ed",
    "is", "it", "al", "ar", "st", "to", "nt", "ng", "se", "ha", "as", "ou", "io", "le", "ve", "co",
    "me", "de", "hi", "ri", "ro", "ic", "ne", "ea",
];

pub fn generate_bigram_text(word_count: usize) -> String {
    let pool: Vec<&str> = ENGLISH_1K.split_whitespace().collect();
    let scored: Vec<(&str, usize)> = pool
        .iter()
        .filter_map(|&w| {
            let score = BIGRAMS.iter().filter(|&&bg| w.contains(bg)).count();
            if score > 0 { Some((w, score)) } else { None }
        })
        .collect();
    if scored.is_empty() {
        return generate_text(WordList::English1k, word_count);
    }
    weighted_text(&scored, word_count)
}

/// Words that hold a weak key, repeated by how many weak letters each has. The
/// word list is lowercase, so `capital` drills a capital by capitalising the
/// words that start with its letter.
struct WordDrill<'a> {
    words: Vec<&'a str>,
    capital: bool,
}

impl WordDrill<'_> {
    fn pick(&self, rng: &mut Rng) -> String {
        let mut word = self.words[rng.usize(self.words.len())].to_string();
        if self.capital {
            if let Some(first) = word.get_mut(..1) {
                first.make_ascii_uppercase();
            }
        }
        word
    }
}

/// How the weak-key drill practises one key: with words that hold it, on or
/// between words (punctuation), or on its own (digits, and keys no word has).
enum KeyDrill<'a> {
    Words(WordDrill<'a>),
    Punctuation(char),
    Alone(char),
}

/// Punctuation drilled around a word, like `(word)`, instead of after it.
const PAIRED_PUNCTUATION: &[(char, char)] = &[
    ('(', ')'),
    ('[', ']'),
    ('{', '}'),
    ('<', '>'),
    ('"', '"'),
    ('\'', '\''),
];

fn with_punctuation(word: &str, punct: char) -> String {
    match PAIRED_PUNCTUATION
        .iter()
        .find(|&&(open, close)| punct == open || punct == close)
    {
        Some((open, close)) => format!("{open}{word}{close}"),
        None => format!("{word}{punct}"),
    }
}

pub fn generate_weak_key_text(weak_chars: &[char], word_count: usize) -> String {
    let pool: Vec<&str> = ENGLISH_1K.split_whitespace().collect();
    if pool.is_empty() {
        return String::new();
    }
    let letters: Vec<char> = weak_chars
        .iter()
        .copied()
        .filter(|c| c.is_alphabetic() && !c.is_uppercase())
        .collect();
    let weak_letters_in = |w: &str| w.chars().filter(|c| letters.contains(c)).count();

    let drills: Vec<KeyDrill> = weak_chars
        .iter()
        .map(|&c| {
            if !c.is_alphanumeric() {
                return KeyDrill::Punctuation(c);
            }
            if !c.is_alphabetic() {
                return KeyDrill::Alone(c);
            }
            let capital = c.is_ascii_uppercase();
            let letter = c.to_ascii_lowercase();
            let scored: Vec<(&str, usize)> = pool
                .iter()
                .filter(|w| {
                    if capital {
                        w.starts_with(letter)
                    } else {
                        w.contains(letter)
                    }
                })
                .map(|&w| (w, weak_letters_in(w).max(1)))
                .collect();
            if scored.is_empty() {
                KeyDrill::Alone(c)
            } else {
                KeyDrill::Words(WordDrill {
                    words: weighted_pool(&scored),
                    capital,
                })
            }
        })
        .collect();
    if drills.is_empty() {
        return random_pick_and_wrap(&pool, word_count);
    }

    // Punctuation goes on or between, and bare characters are spaced out with,
    // words that drill a weak letter where there are any
    let word_drills: Vec<&WordDrill> = drills
        .iter()
        .filter_map(|drill| match drill {
            KeyDrill::Words(word_drill) => Some(word_drill),
            _ => None,
        })
        .collect();
    let any_word = |rng: &mut Rng| match word_drills.len() {
        0 => pool[rng.usize(pool.len())].to_string(),
        n => word_drills[rng.usize(n)].pick(rng),
    };

    // Each word drills one weak key, so every key gets an equal share, but two
    // bare characters never come in a row
    let mut rng = Rng::from_time();
    let mut words: Vec<String> = Vec::with_capacity(word_count);
    let mut last_alone = false;
    for _ in 0..word_count {
        let drill = &drills[rng.usize(drills.len())];
        let alone = matches!(drill, KeyDrill::Alone(_)) && !last_alone;
        words.push(match drill {
            KeyDrill::Words(word_drill) => word_drill.pick(&mut rng),
            KeyDrill::Punctuation(joiner) if WORD_JOINERS.contains(joiner) => {
                let first = any_word(&mut rng);
                format!("{first}{joiner}{}", any_word(&mut rng).to_lowercase())
            }
            KeyDrill::Punctuation(punct) => with_punctuation(&any_word(&mut rng), *punct),
            KeyDrill::Alone(c) if alone => c.to_string(),
            KeyDrill::Alone(_) => any_word(&mut rng),
        });
        last_alone = alone;
    }

    wrap_words(words)
}

/// Pick random complete quotes without repeats until reaching the target word count.
pub fn generate_quote_text(word_count: usize) -> String {
    let quotes: Vec<&str> = QUOTES
        .split("\n---\n")
        .map(|q| q.trim())
        .filter(|q| !q.is_empty())
        .collect();
    if quotes.is_empty() {
        return generate_text(WordList::English1k, word_count);
    }

    let mut rng = Rng::from_time();
    let mut total_words = 0;
    let mut wrapped: Vec<String> = Vec::new();

    // Shuffle indices to avoid repeats; reshuffle if we exhaust all quotes
    let mut indices: Vec<usize> = (0..quotes.len()).collect();
    let mut pos = indices.len();

    while total_words < word_count {
        if pos >= indices.len() {
            for i in (1..indices.len()).rev() {
                indices.swap(i, rng.usize(i + 1));
            }
            pos = 0;
        }
        let quote = quotes[indices[pos]];
        pos += 1;
        total_words += quote.split_whitespace().count();
        wrapped.push(wrap_words(quote.split_whitespace()));
    }
    wrapped.join("\n\n")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_text_produces_correct_word_count() {
        let text = generate_text(WordList::English200, 25);
        let count: usize = text.lines().map(|l| l.split_whitespace().count()).sum();
        assert_eq!(count, 25);
    }

    #[test]
    fn generate_text_wraps_lines() {
        let text = generate_text(WordList::English200, 50);
        assert!(text.lines().count() > 1, "should produce multiple lines");
        for line in text.lines() {
            assert!(
                line.len() <= LINE_WIDTH + 20,
                "line too long: {}",
                line.len()
            );
        }
    }

    #[test]
    fn word_lists_are_nonempty() {
        assert!(!WordList::English200.words().is_empty());
        assert!(!WordList::English1k.words().is_empty());
    }

    #[test]
    fn word_list_all_contains_all_variants() {
        let all = WordList::all();
        assert_eq!(all.len(), 9);
        assert_eq!(all[0], WordList::English200);
        assert_eq!(all[1], WordList::English1k);
    }

    #[test]
    fn code_word_lists_are_nonempty() {
        for &list in &[
            WordList::Rust,
            WordList::Python,
            WordList::JavaScript,
            WordList::Go,
            WordList::CCpp,
            WordList::Java,
            WordList::HtmlCss,
        ] {
            assert!(
                !list.words().is_empty(),
                "{} should not be empty",
                list.label()
            );
        }
    }

    #[test]
    fn quote_text_produces_at_least_target_word_count() {
        let text = generate_quote_text(30);
        let count: usize = text.lines().map(|l| l.split_whitespace().count()).sum();
        assert!(count >= 30, "got {count} words, expected at least 30");
    }

    #[test]
    fn quote_text_contains_punctuation() {
        let text = generate_quote_text(100);
        let has_punct = text
            .chars()
            .any(|c| c == '.' || c == ',' || c == '\'' || c == ':');
        assert!(has_punct, "quotes should contain punctuation");
    }

    #[test]
    fn quotes_file_has_entries() {
        let quotes: Vec<&str> = QUOTES
            .split("\n---\n")
            .map(|q| q.trim())
            .filter(|q| !q.is_empty())
            .collect();
        assert!(quotes.len() >= 10);
    }

    #[test]
    fn quote_text_uses_complete_quotes() {
        let quotes: Vec<&str> = QUOTES
            .split("\n---\n")
            .map(|q| q.trim())
            .filter(|q| !q.is_empty())
            .collect();
        let text = generate_quote_text(30);
        let flat = text.lines().collect::<Vec<_>>().join(" ");
        // At least one full quote should appear in the output
        let has_full_quote = quotes.iter().any(|q| flat.contains(q));
        assert!(
            has_full_quote,
            "output should contain at least one complete quote"
        );
    }

    #[test]
    fn quote_text_lines_do_not_start_with_space() {
        let text = generate_quote_text(100);
        assert!(text.lines().all(|line| !line.starts_with(' ')));
    }

    #[test]
    fn weak_key_text_produces_correct_word_count() {
        let text = generate_weak_key_text(&['e', 'a'], 25);
        let count: usize = text.lines().map(|l| l.split_whitespace().count()).sum();
        assert_eq!(count, 25);
    }

    #[test]
    fn weak_key_text_contains_target_chars() {
        let text = generate_weak_key_text(&['z', 'x'], 50);
        let has_target = text.chars().any(|c| c == 'z' || c == 'x');
        assert!(has_target, "text should contain weak key chars");
    }

    #[test]
    fn weak_key_text_drills_capitals() {
        let text = generate_weak_key_text(&['A'], 100);
        assert!(text.split_whitespace().all(|w| w.starts_with('A')));
    }

    #[test]
    fn weak_key_text_gives_rare_keys_an_equal_share() {
        let text = generate_weak_key_text(&['q', 'e', 't'], 300);
        let with_q = text.split_whitespace().filter(|w| w.contains('q')).count();
        assert!(with_q >= 50, "only {with_q} of 300 words have a q");
    }

    #[test]
    fn weak_key_text_never_puts_two_bare_chars_together() {
        let text = generate_weak_key_text(&['5'], 200);
        let tokens: Vec<&str> = text.split_whitespace().collect();
        assert!(tokens.contains(&"5"));
        assert!(tokens.windows(2).all(|pair| pair != ["5", "5"]));
    }

    #[test]
    fn weak_key_text_puts_punctuation_on_weak_letter_words() {
        let text = generate_weak_key_text(&['q', ';'], 200);
        let with_semicolon: Vec<&str> = text
            .split_whitespace()
            .filter(|w| w.ends_with(';'))
            .collect();
        assert!(!with_semicolon.is_empty());
        assert!(with_semicolon.iter().all(|w| w.contains('q')));
    }

    #[test]
    fn weak_key_text_puts_brackets_around_words() {
        let text = generate_weak_key_text(&['('], 50);
        assert!(
            text.split_whitespace()
                .all(|w| w.starts_with('(') && w.ends_with(')'))
        );
    }

    #[test]
    fn weak_key_text_joins_words_with_a_hyphen_or_underscore() {
        for joiner in WORD_JOINERS {
            let text = generate_weak_key_text(&[joiner], 50);
            assert!(text.split_whitespace().all(|w| {
                w.split(joiner).count() == 2 && w.split(joiner).all(|part| !part.is_empty())
            }));
        }
    }

    #[test]
    fn weak_key_text_keeps_lowercase_when_its_capital_is_weak_too() {
        let text = generate_weak_key_text(&['a', 'A'], 300);
        assert!(text.split_whitespace().any(|w| w.starts_with('a')));
        assert!(text.split_whitespace().any(|w| w.starts_with('A')));
    }

    #[test]
    fn weak_key_text_drills_unmatched_capitals_alone() {
        let z_starts_no_word = !ENGLISH_1K.split_whitespace().any(|w| w.starts_with('z'));
        assert!(z_starts_no_word, "needs a letter no word starts with");
        let text = generate_weak_key_text(&['Z', 'É'], 200);
        let words: Vec<&str> = text.split_whitespace().collect();
        assert!(words.contains(&"Z"));
        assert!(words.contains(&"É"));
    }

    #[test]
    fn bigram_text_produces_correct_word_count() {
        let text = generate_bigram_text(30);
        let count: usize = text.lines().map(|l| l.split_whitespace().count()).sum();
        assert_eq!(count, 30);
    }

    #[test]
    fn bigram_text_contains_common_bigrams() {
        let text = generate_bigram_text(50);
        let has_bigram = BIGRAMS.iter().any(|bg| text.contains(bg));
        assert!(has_bigram, "text should contain at least one common bigram");
    }

    #[test]
    fn weak_key_text_fallback_on_empty_chars() {
        let text = generate_weak_key_text(&[], 20);
        let count: usize = text.lines().map(|l| l.split_whitespace().count()).sum();
        assert_eq!(count, 20);
    }

    #[test]
    fn weak_key_text_includes_punct_when_enabled() {
        // Large sample to guarantee at least one '-' appears
        let text = generate_weak_key_text(&['-'], 200);
        assert!(
            text.contains('-'),
            "weak key drill should contain weak punct char '-'"
        );
    }

    #[test]
    fn weak_key_text_includes_digits_when_enabled() {
        let text = generate_weak_key_text(&['5'], 200);
        assert!(
            text.contains('5'),
            "weak key drill should contain weak digit char '5'"
        );
    }

    #[test]
    fn mixed_text_produces_correct_word_count() {
        let text = generate_text_mixed(&WordList::English200.into(), 50, true, true);
        let count: usize = text.lines().map(|l| l.split_whitespace().count()).sum();
        assert_eq!(count, 50);
    }

    #[test]
    fn mixed_text_punctuates_only_plain_words() {
        for list in [WordList::English200, WordList::Rust] {
            let pool = list.words();
            let text = generate_text_mixed(&list.into(), 500, true, false);
            assert!(text.split_whitespace().any(|token| !pool.contains(&token)));
            for token in text.split_whitespace() {
                if pool.contains(&token) {
                    continue;
                }
                let word = token.trim_matches(|c| "().,;:!?'\"".contains(c));
                let joined = !list.is_code() && word.split(HYPHEN).all(|w| pool.contains(&w));
                assert!(pool.contains(&word) || joined, "{token}");
                for quote in ['"', '\''] {
                    assert_eq!(token.starts_with(quote), token.ends_with(quote), "{token}");
                }
            }
        }
    }

    #[test]
    fn custom_list_text_wraps_by_width() {
        let words = ["слово", "дом"];
        let too_wide = "x".repeat(MAX_WORD_WIDTH + 1);
        let given = [words[0], words[1], "", "\u{200d}", &too_wide].map(String::from);
        let list = WordSource::Custom(
            CustomList::new("w.txt".into(), "/w.txt".into(), given.to_vec()).unwrap(),
        );

        let text = generate_text_mixed(&list, 200, false, false);
        assert!(text.split_whitespace().all(|w| words.contains(&w)));
        let lines: Vec<&str> = text.lines().collect();
        let longest_word = words.iter().map(|w| w.width()).max().unwrap();
        for line in &lines[..lines.len() - 1] {
            assert!(line.width() <= LINE_WIDTH);
            assert!(line.width() + 1 + longest_word > LINE_WIDTH, "{line}");
        }
        assert!(CustomList::new("w.txt".into(), "/w.txt".into(), vec![too_wide]).is_none());

        let widest = "x".repeat(MAX_WORD_WIDTH);
        let list = WordSource::Custom(
            CustomList::new("w.txt".into(), "/w.txt".into(), vec![widest]).unwrap(),
        );
        let text = generate_text_mixed(&list, 50, true, false);
        assert!(text.lines().all(|line| line.width() <= LINE_WIDTH));
    }

    #[test]
    fn custom_list_punctuates_words_in_any_script_but_not_punctuated_ones() {
        const HINDI: &str = "\u{915}\u{94d}\u{92f}\u{93e}";
        const QUOTED: &str = "\u{bf}eins";
        let words = [HINDI, QUOTED].map(String::from).to_vec();
        let list =
            WordSource::Custom(CustomList::new("w.txt".into(), "/w.txt".into(), words).unwrap());

        let text = generate_text_mixed(&list, 500, true, false);
        assert!(
            text.split_whitespace()
                .any(|w| w != HINDI && w.contains(HINDI))
        );
        let with_quoted: Vec<&str> = text
            .split_whitespace()
            .filter(|w| w.contains(QUOTED))
            .collect();
        assert!(!with_quoted.is_empty());
        assert!(with_quoted.iter().all(|&w| w == QUOTED));
    }

    #[test]
    fn mixed_text_injects_punctuation_and_numbers() {
        let text = generate_text_mixed(&WordList::English200.into(), 200, true, true);
        let has_punct = text
            .chars()
            .any(|c| matches!(c, '.' | ',' | ';' | ':' | '!' | '?' | '"' | '-'));
        let has_digit = text.chars().any(|c| c.is_ascii_digit());
        assert!(has_punct, "should contain punctuation marks");
        assert!(has_digit, "should contain digits");
    }
}
