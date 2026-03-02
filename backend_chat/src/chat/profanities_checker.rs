use aho_corasick::AhoCorasick;
use std::{collections::HashMap, fs};
use unicode_normalization::UnicodeNormalization;

pub struct ProfanitiesChecker {
    // profanity_list: HashSet<String>,
    matcher: AhoCorasick,
    leet_map: HashMap<char, char>,
}

impl ProfanitiesChecker {
    pub fn load_from_file(path: &'static str) -> anyhow::Result<Self> {
        let content = fs::read_to_string(path)?;

        Ok(Self {
            matcher: AhoCorasick::builder()
                // .ascii_case_insensitive(true)
                .build(
                    content
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty() && !line.starts_with('#'))
                        .map(|line| line.to_lowercase()),
                )?,
            leet_map: HashMap::from([
                ('0', 'o'),
                ('1', 'l'),
                ('3', 'e'),
                ('4', 'a'),
                ('5', 's'),
                ('7', 't'),
                ('@', 'a'),
                ('$', 's'),
                ('!', 'i'),
            ]),
        })
    }

    pub fn contains_profanities(&self, content: &str) -> bool {
        self.matcher.is_match(&self.normalize(content))
    }

    pub fn normalize(&self, content: &str) -> String {
        // 1. Lowercase + NFKC normalize + strip diacritics
        let mut normalized: String = content
            .nfkc()
            .flat_map(|c| c.to_lowercase())
            .filter(|c| c.is_alphanumeric())
            .map(|c| {
                if let Some(repl) = self.leet_map.get(&c) {
                    *repl
                } else {
                    c
                }
            })
            .collect();

        // 5. Remove separators and punctuation (keep letters + digits only)
        normalized.retain(|c| c.is_alphanumeric());

        // 6. Collapse repeated characters (e.g. "fuuck" → "fuck")
        normalized = collapse_repeated_chars(&normalized);

        normalized
        // content.to_ascii_lowercase()
    }
}

fn collapse_repeated_chars(input: &str) -> String {
    let mut result = String::with_capacity(input.len());

    let mut prev: Option<char> = None;
    let mut count = 0;

    for c in input.chars() {
        match prev {
            Some(p) if p == c => {
                count += 1;
                if count <= 2 {
                    result.push(c);
                }
            }
            _ => {
                prev = Some(c);
                count = 1;
                result.push(c);
            }
        }
    }

    result
}
