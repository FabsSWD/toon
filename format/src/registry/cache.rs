use std::collections::{HashMap, VecDeque};
use std::hash::Hash;

pub(crate) struct LruCache<K, V> {
    map: HashMap<K, V>,
    order: VecDeque<K>,
    max_entries: Option<usize>,
}

impl<K, V> LruCache<K, V>
where
    K: Eq + Hash + Clone,
{
    pub(crate) fn new_unbounded() -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            max_entries: None,
        }
    }

    pub(crate) fn with_max_entries(max_entries: usize) -> Self {
        Self {
            map: HashMap::new(),
            order: VecDeque::new(),
            max_entries: Some(max_entries),
        }
    }

    pub(crate) fn insert(&mut self, key: K, value: V) {
        self.map.insert(key.clone(), value);
        self.touch(&key);
        self.evict_if_needed();
    }

    pub(crate) fn get_cloned(&mut self, key: &K) -> Option<V>
    where
        V: Clone,
    {
        let value = self.map.get(key).cloned();
        if value.is_some() {
            self.touch(key);
        }
        value
    }

    fn touch(&mut self, key: &K) {
        if let Some(pos) = self.order.iter().position(|x| x == key) {
            self.order.remove(pos);
        }
        self.order.push_back(key.clone());
    }

    fn evict_if_needed(&mut self) {
        let Some(max) = self.max_entries else {
            return;
        };

        while self.map.len() > max {
            if let Some(evict_key) = self.order.pop_front() {
                self.map.remove(&evict_key);
            } else {
                break;
            }
        }
    }
}
