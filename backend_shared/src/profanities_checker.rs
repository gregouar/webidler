use aho_corasick::AhoCorasick;
use itertools::Itertools;
use regex::Regex;
use std::{collections::HashMap, fs};
use unicode_normalization::UnicodeNormalization;

pub struct ProfanitiesChecker {
    // profanity_list: HashSet<String>,
    strong_matcher: AhoCorasick,
    weak_matcher: Regex,
    leet_map: HashMap<char, char>,
}

impl ProfanitiesChecker {
    pub fn load_from_file(
        strong_path: &'static str,
        weak_path: &'static str,
    ) -> anyhow::Result<Self> {
        let strong_content = fs::read_to_string(strong_path)?;
        let weak_content = fs::read_to_string(weak_path)?;

        Ok(Self {
            strong_matcher: AhoCorasick::builder()
                // .ascii_case_insensitive(true)
                .build(
                    strong_content
                        .lines()
                        .map(str::trim)
                        .filter(|line| !line.is_empty() && !line.starts_with('#'))
                        .map(|line| line.to_lowercase()),
                )?,
            weak_matcher: Regex::new(&format!(
                r"\b({})\b",
                weak_content
                    .lines()
                    .map(str::trim)
                    .filter(|line| !line.is_empty() && !line.starts_with('#'))
                    .map(|line| line.to_lowercase())
                    .join("|")
            ))?,
            leet_map: HashMap::from([
                ('0', 'o'),
                ('1', 'i'),
                ('3', 'e'),
                ('4', 'a'),
                ('5', 's'),
                ('7', 't'),
                ('@', 'a'),
                ('$', 's'),
                ('!', 'i'),
                ('+', 't'),
            ]),
        })
    }

    pub fn find_profanity(&self, content: &str) -> Option<String> {
        let weak_normalized = self.weak_normalize(content);
        let strong_normalized = self.strong_normalize(&weak_normalized);

        if let Some(m) = self.strong_matcher.find(&strong_normalized) {
            return Some(strong_normalized[m.start()..m.end()].to_string());
        }

        if let Some(m) = self.weak_matcher.find(&weak_normalized) {
            return Some(weak_normalized[m.start()..m.end()].to_string());
        }

        None
    }

    pub fn weak_normalize(&self, content: &str) -> String {
        collapse_repeated_chars(
            content.nfkc().flat_map(|c| c.to_lowercase()).map(|c| {
                if let Some(repl) = self.leet_map.get(&c) {
                    *repl
                } else {
                    c
                }
            }),
            content.len(),
        )
    }

    pub fn strong_normalize(&self, content: &str) -> String {
        content.chars().filter(|c| c.is_alphanumeric()).collect()
    }
}

fn collapse_repeated_chars(input: impl Iterator<Item = char>, capacity: usize) -> String {
    let mut result = String::with_capacity(capacity);

    let mut prev: Option<char> = None;
    let mut count = 0;

    for c in input {
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
