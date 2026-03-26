use std::any::Any;

use egui::{ahash::HashMap, util::cache::CacheTrait};

type Generation = u32;

pub struct FrameCacheDyn<Value, const TTL: Generation> {
    generation: Generation,
    cache: HashMap<u64, (Generation, Value)>,
}

impl<Value, const TTL: Generation> FrameCacheDyn<Value, TTL> {
    pub fn new() -> Self {
        Self {
            generation: 0,
            cache: Default::default(),
        }
    }

    /// Must be called once per frame to clear the cache.
    pub fn evict_cache(&mut self) {
        let current_generation = self.generation;
        self.cache.retain(|_key, (cached_generation, _cached_val)| {
            current_generation.wrapping_sub(*cached_generation) <= TTL // only keep those that were used recently
        });
        self.generation = self.generation.wrapping_add(1);
    }
}

impl<Value, const TTL: Generation> Default for FrameCacheDyn<Value, TTL> {
    fn default() -> Self {
        Self::new()
    }
}

impl<Value, const TTL: Generation> FrameCacheDyn<Value, TTL> {
    /// Get from cache (if the same key was used last frame)
    /// or recompute and store in the cache.
    pub fn get_or_else_insert<Key, Computer>(&mut self, key: Key, computer: Computer) -> Value
    where
        Key: Copy + std::hash::Hash,
        Value: Clone,
        Computer: FnOnce() -> Value,
    {
        let hash = egui::util::hash(key);

        match self.cache.entry(hash) {
            std::collections::hash_map::Entry::Occupied(entry) => {
                let cached = entry.into_mut();
                cached.0 = self.generation;
                cached.1.clone()
            }
            std::collections::hash_map::Entry::Vacant(entry) => {
                let value = computer();
                entry.insert((self.generation, value.clone()));
                value
            }
        }
    }
}

impl<Value: 'static + Send + Sync, const TTL: Generation> CacheTrait for FrameCacheDyn<Value, TTL> {
    fn update(&mut self) {
        self.evict_cache()
    }

    fn len(&self) -> usize {
        self.cache.len()
    }
}
