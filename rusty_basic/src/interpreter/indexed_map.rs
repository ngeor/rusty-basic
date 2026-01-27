use std::collections::HashMap;
use std::hash::Hash;

/// A key-value map that can also access values by their insertion order.
#[derive(Debug)]
pub struct IndexedMap<K, V>
where
    K: Clone + Eq + Hash,
{
    entries: Vec<(K, V)>,
    keys_to_indices: HashMap<K, usize>,
}

impl<K, V> IndexedMap<K, V>
where
    K: Clone + Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            entries: vec![],
            keys_to_indices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.keys_to_indices.get(&key) {
            Some(existing_index) => {
                self.entries[*existing_index] = (key, value);
            }
            _ => {
                let index = self.entries.len();
                self.entries.push((key.clone(), value));
                self.keys_to_indices.insert(key, index);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys_to_indices
            .get(key)
            .and_then(|index| self.get_by_index(*index))
    }

    pub fn get_by_index(&self, index: usize) -> Option<&V> {
        self.entries.get(index).map(|(_, v)| v)
    }

    pub fn get_by_index_mut(&mut self, index: usize) -> Option<&mut V> {
        self.entries.get_mut(index).map(|(_, v)| v)
    }

    pub fn get_or_create<F>(&mut self, key: K, creator: F) -> &mut V
    where
        F: FnOnce(&K) -> V,
    {
        let opt_value = match self.keys_to_indices.get(&key) {
            Some(existing_index) => self.get_by_index_mut(*existing_index),
            _ => {
                let v = creator(&key);
                self.insert(key, v);
                self.entries.last_mut().map(|(_, v)| v)
            }
        };
        // guaranteed to be present
        opt_value.unwrap()
    }

    pub fn entries(&self) -> impl Iterator<Item = &(K, V)> {
        self.entries.iter()
    }

    pub fn values(&self) -> impl Iterator<Item = &V> {
        self.entries.iter().map(|(_, v)| v)
    }

    pub fn values_mut(&mut self) -> impl Iterator<Item = &mut V> {
        self.entries.iter_mut().map(|(_, v)| v)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }
}

impl<K: Clone + Eq + Hash, V> Default for IndexedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}
