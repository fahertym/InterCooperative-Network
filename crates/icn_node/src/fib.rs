use std::collections::HashMap;
use std::net::SocketAddr;

/// Represents an entry in the Forwarding Information Base (FIB).
#[derive(Debug, Clone)]
pub struct FibEntry {
    pub name: String,           // The name of the content or prefix.
    pub next_hops: Vec<SocketAddr>, // The list of next hop addresses.
}

impl FibEntry {
    /// Creates a new FIB entry with a given name and initial next hop.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name or prefix.
    /// * `next_hop` - A `SocketAddr` representing the initial next hop address.
    ///
    /// # Returns
    ///
    /// * A new `FibEntry` instance.
    pub fn new(name: String, next_hop: SocketAddr) -> Self {
        FibEntry {
            name,
            next_hops: vec![next_hop],
        }
    }

    /// Adds a new next hop to the FIB entry if it does not already exist.
    ///
    /// # Arguments
    ///
    /// * `next_hop` - A `SocketAddr` representing the new next hop address.
    pub fn add_next_hop(&mut self, next_hop: SocketAddr) {
        if !self.next_hops.contains(&next_hop) {
            self.next_hops.push(next_hop);
        }
    }

    /// Removes an existing next hop from the FIB entry.
    ///
    /// # Arguments
    ///
    /// * `next_hop` - A reference to the `SocketAddr` to be removed.
    pub fn remove_next_hop(&mut self, next_hop: &SocketAddr) {
        self.next_hops.retain(|&x| x != *next_hop);
    }
}

/// Represents the Forwarding Information Base (FIB) which stores FIB entries.
pub struct ForwardingInformationBase {
    entries: HashMap<String, FibEntry>, // The collection of FIB entries, indexed by name.
}

impl ForwardingInformationBase {
    /// Creates a new, empty Forwarding Information Base.
    ///
    /// # Returns
    ///
    /// * A new `ForwardingInformationBase` instance.
    pub fn new() -> Self {
        ForwardingInformationBase {
            entries: HashMap::new(),
        }
    }

    /// Adds a new entry to the FIB or updates an existing entry with a new next hop.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name or prefix.
    /// * `next_hop` - A `SocketAddr` representing the next hop address.
    pub fn add_entry(&mut self, name: String, next_hop: SocketAddr) {
        self.entries
            .entry(name.clone())
            .and_modify(|e| e.add_next_hop(next_hop))
            .or_insert_with(|| FibEntry::new(name, next_hop));
    }

    /// Removes an entry from the FIB.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name or prefix to be removed.
    pub fn remove_entry(&mut self, name: &str) {
        self.entries.remove(name);
    }

    /// Retrieves the list of next hop addresses for a given name.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name or prefix.
    ///
    /// # Returns
    ///
    /// * An `Option` containing a reference to a vector of `SocketAddr` if the name exists, otherwise `None`.
    pub fn get_next_hops(&self, name: &str) -> Option<&Vec<SocketAddr>> {
        self.entries.get(name).map(|entry| &entry.next_hops)
    }

    /// Performs a longest prefix match to find the best entry for a given name.
    ///
    /// # Arguments
    ///
    /// * `name` - A string representing the name or prefix.
    ///
    /// # Returns
    ///
    /// * An `Option` containing a reference to the best matching `FibEntry` if found, otherwise `None`.
    pub fn longest_prefix_match(&self, name: &str) -> Option<&FibEntry> {
        self.entries
            .iter()
            .filter(|(prefix, _)| name.starts_with(*prefix))
            .max_by_key(|(prefix, _)| prefix.len())
            .map(|(_, entry)| entry)
    }

    /// Checks if the FIB is empty.
    ///
    /// # Returns
    ///
    /// * `true` if the FIB is empty, otherwise `false`.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fib() {
        // Create a new Forwarding Information Base
        let mut fib = ForwardingInformationBase::new();

        // Define some socket addresses for testing
        let addr1: SocketAddr = "127.0.0.1:8000".parse().unwrap();
        let addr2: SocketAddr = "127.0.0.1:8001".parse().unwrap();

        // Add entries to the FIB
        fib.add_entry("/test".to_string(), addr1);
        fib.add_entry("/test/nested".to_string(), addr2);

        // Check if the next hops are correctly added
        assert_eq!(fib.get_next_hops("/test").unwrap().len(), 1);
        assert_eq!(fib.get_next_hops("/test/nested").unwrap().len(), 1);

        // Perform a longest prefix match
        let longest_match = fib.longest_prefix_match("/test/nested/deep");
        assert!(longest_match.is_some());
        assert_eq!(longest_match.unwrap().name, "/test/nested");

        // Test remove_entry
        fib.remove_entry("/test");
        assert!(fib.get_next_hops("/test").is_none());

        // Test FibEntry methods
        let mut entry = FibEntry::new("/example".to_string(), addr1);
        entry.add_next_hop(addr2);
        assert_eq!(entry.next_hops.len(), 2);
        entry.remove_next_hop(&addr1);
        assert_eq!(entry.next_hops.len(), 1);
        assert_eq!(entry.next_hops[0], addr2);
    }
}
