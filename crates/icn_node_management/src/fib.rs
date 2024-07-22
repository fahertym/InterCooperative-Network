use icn_common::{IcnError, IcnResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, error};

/// Represents a Forwarding Information Base (FIB) in the ICN project.
pub struct FIB {
    table: Arc<RwLock<HashMap<String, String>>>,
}

impl FIB {
    /// Creates a new instance of FIB.
    pub fn new() -> Self {
        FIB {
            table: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds an entry to the FIB.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix of the entry.
    /// * `next_hop` - The next hop for the entry.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn add_entry(&self, prefix: String, next_hop: String) -> IcnResult<()> {
        let mut table = self.table.write().unwrap();
        table.insert(prefix.clone(), next_hop);
        info!("FIB entry added: {} -> {}", prefix, next_hop);
        Ok(())
    }

    /// Retrieves the next hop for a given prefix.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix to retrieve the next hop for.
    ///
    /// # Returns
    ///
    /// * `IcnResult<String>` - The next hop.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn get_next_hop(&self, prefix: &str) -> IcnResult<String> {
        let table = self.table.read().unwrap();
        table.get(prefix).cloned().ok_or_else(|| IcnError::Network("Next hop not found".into()))
    }

    /// Removes an entry from the FIB.
    ///
    /// # Arguments
    ///
    /// * `prefix` - The prefix of the entry to be removed.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn remove_entry(&self, prefix: &str) -> IcnResult<()> {
        let mut table = self.table.write().unwrap();
        table.remove(prefix);
        info!("FIB entry removed: {}", prefix);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_next_hop() {
        let fib = FIB::new();
        let prefix = "prefix_1".to_string();
        let next_hop = "next_hop_1".to_string();
        assert!(fib.add_entry(prefix.clone(), next_hop.clone()).is_ok());
        let retrieved_next_hop = fib.get_next_hop(&prefix).unwrap();
        assert_eq!(retrieved_next_hop, next_hop);
    }

    #[test]
    fn test_get_non_existent_next_hop() {
        let fib = FIB::new();
        assert!(fib.get_next_hop("non_existent_prefix").is_err());
    }

    #[test]
    fn test_remove_entry() {
        let fib = FIB::new();
        let prefix = "prefix_1".to_string();
        let next_hop = "next_hop_1".to_string();
        fib.add_entry(prefix.clone(), next_hop).unwrap();
        assert!(fib.remove_entry(&prefix).is_ok());
        assert!(fib.get_next_hop(&prefix).is_err());
    }
}
