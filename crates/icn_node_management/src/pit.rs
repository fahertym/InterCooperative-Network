use icn_common::{IcnError, IcnResult};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use log::{info, error};

/// Represents a Pending Interest Table (PIT) in the ICN project.
pub struct PIT {
    table: Arc<RwLock<HashMap<String, Vec<String>>>>,
}

impl PIT {
    /// Creates a new instance of PIT.
    pub fn new() -> Self {
        PIT {
            table: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Adds an interest to the PIT.
    ///
    /// # Arguments
    ///
    /// * `interest` - The interest to be added.
    /// * `requester` - The requester of the interest.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn add_interest(&self, interest: String, requester: String) -> IcnResult<()> {
        let mut table = self.table.write().unwrap();
        table.entry(interest.clone()).or_default().push(requester.clone());
        info!("PIT entry added: {} requested by {}", interest, requester);
        Ok(())
    }

    /// Retrieves the requesters for a given interest.
    ///
    /// # Arguments
    ///
    /// * `interest` - The interest to retrieve the requesters for.
    ///
    /// # Returns
    ///
    /// * `IcnResult<Vec<String>>` - The list of requesters.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn get_requesters(&self, interest: &str) -> IcnResult<Vec<String>> {
        let table = self.table.read().unwrap();
        table.get(interest).cloned().ok_or_else(|| IcnError::Network("Interest not found".into()))
    }

    /// Removes an interest from the PIT.
    ///
    /// # Arguments
    ///
    /// * `interest` - The interest to be removed.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn remove_interest(&self, interest: &str) -> IcnResult<()> {
        let mut table = self.table.write().unwrap();
        table.remove(interest);
        info!("PIT entry removed: {}", interest);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add_and_get_requesters() {
        let pit = PIT::new();
        let interest = "interest_1".to_string();
        let requester = "requester_1".to_string();
        assert!(pit.add_interest(interest.clone(), requester.clone()).is_ok());
        let requesters = pit.get_requesters(&interest).unwrap();
        assert_eq!(requesters, vec![requester]);
    }

    #[test]
    fn test_get_non_existent_requesters() {
        let pit = PIT::new();
        assert!(pit.get_requesters("non_existent_interest").is_err());
    }

    #[test]
    fn test_remove_interest() {
        let pit = PIT::new();
        let interest = "interest_1".to_string();
        let requester = "requester_1".to_string();
        pit.add_interest(interest.clone(), requester).unwrap();
        assert!(pit.remove_interest(&interest).is_ok());
        assert!(pit.get_requesters(&interest).is_err());
    }
}
