use std::collections::HashMap;
use std::hash::Hash;
use std::slice::{Iter, IterMut};

/// A key-value map that can also access values by their insertion order.
#[derive(Debug)]
pub struct IndexedMap<K, V>
where
    K: Eq + Hash,
{
    values: Vec<V>,
    keys_to_indices: HashMap<K, usize>,
}

impl<K, V> IndexedMap<K, V>
where
    K: Eq + Hash,
{
    pub fn new() -> Self {
        Self {
            values: vec![],
            keys_to_indices: HashMap::new(),
        }
    }

    pub fn insert(&mut self, key: K, value: V) {
        match self.keys_to_indices.get(&key) {
            Some(existing_index) => {
                self.values[*existing_index] = value;
            }
            _ => {
                let index = self.values.len();
                self.values.push(value);
                self.keys_to_indices.insert(key, index);
            }
        }
    }

    pub fn get(&self, key: &K) -> Option<&V> {
        self.keys_to_indices
            .get(key)
            .and_then(|index| self.values.get(*index))
    }

    pub fn get_by_index(&self, index: usize) -> Option<&V> {
        self.values.get(index)
    }

    pub fn get_by_index_mut(&mut self, index: usize) -> Option<&mut V> {
        self.values.get_mut(index)
    }

    pub fn get_or_create<F>(&mut self, key: K, creator: F) -> &mut V
    where
        F: FnOnce(&K) -> V,
    {
        match self.keys_to_indices.get(&key) {
            Some(existing_index) => self.values.get_mut(*existing_index).unwrap(),
            _ => {
                let v = creator(&key);
                self.insert(key, v);
                self.values.last_mut().unwrap()
            }
        }
    }

    pub fn values(&self) -> Iter<'_, V> {
        self.values.iter()
    }

    pub fn values_mut(&mut self) -> IterMut<'_, V> {
        self.values.iter_mut()
    }

    pub fn keys(&self) -> impl Iterator<Item = &K> + '_ {
        KeysIterator::new(self)
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }
}

impl<K: Eq + Hash, V> Default for IndexedMap<K, V> {
    fn default() -> Self {
        Self::new()
    }
}

struct KeysIterator<'a, K, V>
where
    K: Eq + Hash,
{
    owner: &'a IndexedMap<K, V>,
    index: usize,
}

impl<'a, K, V> KeysIterator<'a, K, V>
where
    K: Eq + Hash,
{
    pub fn new(owner: &'a IndexedMap<K, V>) -> Self {
        Self { owner, index: 0 }
    }
}

impl<'a, K, V> Iterator for KeysIterator<'a, K, V>
where
    K: Eq + Hash,
{
    type Item = &'a K;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.owner.len() {
            let result = self
                .owner
                .keys_to_indices
                .iter()
                .find(|(_, index)| **index == self.index)
                .map(|(key, _)| key);
            self.index += 1;
            result
        } else {
            None
        }
    }
}
