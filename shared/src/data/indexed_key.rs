use std::{
    cmp::Ordering,
    fmt,
    hash::{Hash, Hasher},
};

use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(transparent)]
pub struct IndexedKey<T> {
    value: T,
    #[serde(skip_serializing, skip_deserializing)]
    key: Option<usize>,
}

impl<T> IndexedKey<T> {
    pub fn new(value: impl Into<T>) -> Self {
        Self {
            value: value.into(),
            key: None,
        }
    }

    pub fn with_key(mut self, key: usize) -> Self {
        self.key = Some(key);
        self
    }

    pub fn key(&self) -> Option<usize> {
        self.key
    }

    pub fn value(&self) -> &T {
        &self.value
    }
}

impl IndexedKey<String> {
    pub fn as_str(&self) -> &str {
        &self.value
    }
}

impl From<String> for IndexedKey<String> {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for IndexedKey<String> {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<IndexedKey<String>> for String {
    fn from(value: IndexedKey<String>) -> Self {
        value.value
    }
}

impl<T: fmt::Display> fmt::Display for IndexedKey<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.value.fmt(f)
    }
}

impl<T: PartialEq> PartialEq for IndexedKey<T> {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl<T: Eq> Eq for IndexedKey<T> {}

impl<T: Ord> PartialOrd for IndexedKey<T> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for IndexedKey<T> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialEq<str> for IndexedKey<String> {
    fn eq(&self, other: &str) -> bool {
        self.value == other
    }
}

impl PartialEq<&str> for IndexedKey<String> {
    fn eq(&self, other: &&str) -> bool {
        self.value == *other
    }
}

impl<T: Hash> Hash for IndexedKey<T> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}
