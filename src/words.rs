const ENGLISH_200: &str = include_str!("../assets/words/english_200.txt");
const ENGLISH_1K: &str = include_str!("../assets/words/english_1k.txt");
const QUOTES: &str = include_str!("../assets/quotes/english.txt");

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

fn random_pick_and_wrap(pool: &[&str], word_count: usize) -> String {
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

/// Build a weighted pool: repeat each word by its score, then pick randomly.
fn weighted_text(scored: &[(&str, usize)], word_count: usize) -> String {
    let weighted: Vec<&str> = scored
        .iter()
        .flat_map(|&(w, score)| std::iter::repeat(w).take(score))
        .collect();
    random_pick_and_wrap(&weighted, word_count)
}

pub fn generate_text(list: WordList, word_count: usize) -> String {
    let pool = list.words();
    if pool.is_empty() {
        return String::new();
    }
    random_pick_and_wrap(&pool, word_count)
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
            if score > 0 {
                Some((w, score))
            } else {
                None
            }
        })
        .collect();
    if scored.is_empty() {
        return generate_text(WordList::English1k, word_count);
    }
    weighted_text(&scored, word_count)
}

pub fn generate_weak_key_text(weak_chars: &[char], word_count: usize) -> String {
    if weak_chars.is_empty() {
        return generate_text(WordList::English1k, word_count);
    }
    let pool: Vec<&str> = ENGLISH_1K.split_whitespace().collect();
    let scored: Vec<(&str, usize)> = pool
        .iter()
        .filter_map(|&w| {
            let score = w.chars().filter(|c| weak_chars.contains(c)).count();
            if score > 0 {
                Some((w, score))
            } else {
                None
            }
        })
        .collect();
    if scored.is_empty() {
        return generate_text(WordList::English1k, word_count);
    }
    weighted_text(&scored, word_count)
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
    let mut lines: Vec<String> = Vec::new();

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
        for word in quote.split_whitespace() {
            let need_new_line = match lines.last() {
                None => true,
                Some(line) => line.len() + 1 + word.len() > LINE_WIDTH,
            };
            if need_new_line {
                lines.push(word.to_string());
            } else {
                lines.last_mut().unwrap().push(' ');
                lines.last_mut().unwrap().push_str(word);
            }
            total_words += 1;
        }
        lines.push(String::new());
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
}
