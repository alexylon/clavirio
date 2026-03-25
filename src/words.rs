const ENGLISH_200: &str = include_str!("../assets/words/english_200.txt");
const ENGLISH_1K: &str = include_str!("../assets/words/english_1k.txt");

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WordList {
    English200,
    English1k,
}

impl WordList {
    pub fn words(self) -> Vec<&'static str> {
        let src = match self {
            Self::English200 => ENGLISH_200,
            Self::English1k => ENGLISH_1K,
        };
        src.split_whitespace().collect()
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::English200 => "english 200",
            Self::English1k => "english 1k",
        }
    }

    pub fn all() -> &'static [WordList] {
        &[Self::English200, Self::English1k]
    }
}

/// Simple xorshift64 PRNG seeded from system time — no extra dependencies.
struct Rng(u64);

impl Rng {
    fn from_time() -> Self {
        let seed = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(42);
        // Avoid zero seed
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

/// Generate `word_count` random words from the list, wrapped into lines.
pub fn generate_text(list: WordList, word_count: usize) -> String {
    let pool = list.words();
    if pool.is_empty() {
        return String::new();
    }

    let mut rng = Rng::from_time();
    let mut lines: Vec<String> = Vec::new();
    let mut current_line = String::new();
    let mut remaining = word_count;

    while remaining > 0 {
        let word = pool[rng.usize(pool.len())];
        if !current_line.is_empty() && current_line.len() + 1 + word.len() > LINE_WIDTH {
            lines.push(std::mem::take(&mut current_line));
        }
        if !current_line.is_empty() {
            current_line.push(' ');
        }
        current_line.push_str(word);
        remaining -= 1;
    }
    if !current_line.is_empty() {
        lines.push(current_line);
    }
    lines.join("\n")
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
    fn word_list_all_contains_both() {
        let all = WordList::all();
        assert_eq!(all.len(), 2);
        assert_eq!(all[0], WordList::English200);
        assert_eq!(all[1], WordList::English1k);
    }
}
