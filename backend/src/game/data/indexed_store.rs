use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use shared::data::indexed_key::IndexedKey;
use std::hash::Hash;

use crate::game::utils::json::LoadJsonFromFile;

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(transparent)]
#[serde(bound(
    serialize = "K: Serialize, T: Serialize",
    deserialize = "K: serde::de::DeserializeOwned + Eq + Hash, T: serde::de::DeserializeOwned"
))]
pub struct IndexedStore<K, T>(IndexMap<IndexedKey<K>, T>);

impl<K, T> IndexedStore<K, T>
where
    K: Clone + Eq + Hash,
{
    pub fn iter(&self) -> indexmap::map::Iter<'_, IndexedKey<K>, T> {
        self.0.iter()
    }

    pub fn get(&self, id: &IndexedKey<K>) -> Option<&T> {
        if let Some(key) = id.key()
            && let Some((stored_id, value)) = self.0.get_index(key)
            && stored_id == id
        {
            return Some(value);
        }

        self.0.get(id)
    }

    pub fn id_with_key(&self, id: IndexedKey<K>) -> IndexedKey<K> {
        if let Some(key) = id.key()
            && let Some((stored_id, _)) = self.0.get_index(key)
            && *stored_id == id
        {
            return id;
        }

        if let Some(key) = self.0.get_index_of(&id) {
            id.with_key(key)
        } else {
            id
        }
    }

    pub fn into_indexed_keys(self) -> Self {
        self.0
            .into_iter()
            .enumerate()
            .map(|(key, (id, value))| (id.with_key(key), value))
            .collect()
    }
}

impl<K, T> IntoIterator for IndexedStore<K, T> {
    type Item = (IndexedKey<K>, T);
    type IntoIter = indexmap::map::IntoIter<IndexedKey<K>, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<K, T> FromIterator<(IndexedKey<K>, T)> for IndexedStore<K, T>
where
    K: Eq + Hash,
{
    fn from_iter<I: IntoIterator<Item = (IndexedKey<K>, T)>>(iter: I) -> Self {
        Self(IndexMap::from_iter(iter))
    }
}

impl<K, T> LoadJsonFromFile for IndexedStore<K, T>
where
    K: serde::de::DeserializeOwned + Eq + Hash + Send + Sync + 'static,
    T: LoadJsonFromFile,
{
}
