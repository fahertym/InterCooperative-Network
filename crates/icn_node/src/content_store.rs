use std::collections::HashMap;
use std::time::{Duration, Instant};

const MAX_CACHE_SIZE: usize = 1000;
const DEFAULT_TTL: Duration = Duration::from_secs(3600);

struct CacheEntry {
    content: Vec<u8>,
    timestamp: Instant,
    ttl: Duration,
}

pub struct ContentStore {
    cache: HashMap<String, CacheEntry>,
}

impl ContentStore {
    pub fn new() -> Self {
        ContentStore {
            cache: HashMap::new(),
        }
    }

    pub fn add(&mut self, name: String, content: Vec<u8>) {
        self.cache.insert(name, CacheEntry {
            content,
            timestamp: Instant::now(),
            ttl: DEFAULT_TTL,
        });

        if self.cache.len() > MAX_CACHE_SIZE {
            self.evict_oldest();
        }
    }

    pub fn get(&self, name: &str) -> Option<Vec<u8>> {
        self.cache.get(name).and_then(|entry| {
            if entry.timestamp.elapsed() < entry.ttl {
                Some(entry.content.clone())
            } else {
                None
            }
        })
    }

    pub fn get_and_pop(&mut self, name: &str) -> Option<Vec<u8>> {
        if let Some(entry) = self.cache.remove(name) {
            if entry.timestamp.elapsed() < entry.ttl {
                Some(entry.content)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn evict_oldest(&mut self) {
        if let Some(oldest_key) = self.cache.keys().next().cloned() {
            self.cache.remove(&oldest_key);
        }
    }

    pub fn remove_expired(&mut self) {
        let now = Instant::now();
        self.cache.retain(|_, entry| now.duration_since(entry.timestamp) < entry.ttl);
    }

    pub fn set_ttl(&mut self, name: &str, ttl: Duration) {
        if let Some(entry) = self.cache.get_mut(name) {
            entry.ttl = ttl;
        }
    }

    pub fn is_empty(&self) -> bool {
        self.cache.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_store() {
        let mut cs = ContentStore::new();
        let content = vec![1, 2, 3, 4];
        cs.add("test".to_string(), content.clone());

        assert_eq!(cs.get("test"), Some(content.clone()));
        assert_eq!(cs.get("nonexistent"), None);

        cs.set_ttl("test", Duration::from_secs(1));
        std::thread::sleep(Duration::from_secs(2));
        assert_eq!(cs.get("test"), None);

        cs.add("test2".to_string(), vec![5, 6, 7, 8]);
        assert!(!cs.is_empty());

        cs.remove_expired();
        assert_eq!(cs.get("test2"), Some(vec![5, 6, 7, 8]));
    }
}
