use std::collections::HashMap;
use std::time::{Duration, Instant};

const DEFAULT_INTEREST_LIFETIME: Duration = Duration::from_secs(4);

struct PitEntry {
    interfaces: Vec<String>,
    timestamp: Instant,
}

pub struct PendingInterestTable {
    entries: HashMap<String, PitEntry>,
}

impl PendingInterestTable {
    pub fn new() -> Self {
        PendingInterestTable {
            entries: HashMap::new(),
        }
    }

    pub fn add_interest(&mut self, name: String, interface: &str) {
        self.entries
            .entry(name)
            .and_modify(|e| {
                if !e.interfaces.contains(&interface.to_string()) {
                    e.interfaces.push(interface.to_string());
                }
                e.timestamp = Instant::now();
            })
            .or_insert(PitEntry {
                interfaces: vec![interface.to_string()],
                timestamp: Instant::now(),
            });
    }

    pub fn remove_interest(&mut self, name: &str) {
        self.entries.remove(name);
    }

    pub fn has_pending_interest(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn get_incoming_interfaces(&self, name: &str) -> Option<Vec<String>> {
        self.entries.get(name).map(|entry| entry.interfaces.clone())
    }

    pub fn add_incoming_interface(&mut self, name: &str, interface: &str) {
        if let Some(entry) = self.entries.get_mut(name) {
            if !entry.interfaces.contains(&interface.to_string()) {
                entry.interfaces.push(interface.to_string());
            }
        }
    }

    pub fn clear_expired(&mut self) {
        self.entries.retain(|_, entry| entry.timestamp.elapsed() < DEFAULT_INTEREST_LIFETIME);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pending_interest_table() {
        let mut pit = PendingInterestTable::new();
        
        pit.add_interest("test".to_string(), "interface1");
        assert!(pit.has_pending_interest("test"));
        
        pit.add_incoming_interface("test", "interface2");
        let interfaces = pit.get_incoming_interfaces("test").unwrap();
        assert_eq!(interfaces.len(), 2);
        assert!(interfaces.contains(&"interface1".to_string()));
        assert!(interfaces.contains(&"interface2".to_string()));
        
        pit.remove_interest("test");
        assert!(!pit.has_pending_interest("test"));

        pit.add_interest("test_expired".to_string(), "interface1");
        std::thread::sleep(Duration::from_secs(5));
        pit.clear_expired();
        assert!(!pit.has_pending_interest("test_expired"));
    }
}
