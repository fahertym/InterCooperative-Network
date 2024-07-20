// ===============================================
// Content Store, FIB, and PIT Implementation
// ===============================================
// This file defines the structures and methods for the Content Store, Forwarding Information Base (FIB),
// and Pending Interest Table (PIT) used in the ICN Node.
//
// Key concepts:
// - Content Store: A cache for storing data packets temporarily.
// - Forwarding Information Base (FIB): A table that stores routing information for named data.
// - Pending Interest Table (PIT): A table that keeps track of interests that have been forwarded but not yet satisfied.

use std::collections::HashMap;

/// A struct representing a packet in the ICN Node.
#[derive(Clone)]
pub struct Packet {
    pub packet_type: PacketType,
    pub name: String,
    pub content: Vec<u8>,
}

/// Enum representing the type of a packet, either Interest or Data.
#[derive(Clone)]
pub enum PacketType {
    Interest,
    Data,
}

/// A struct representing the Content Store.
pub struct ContentStore {
    store: HashMap<String, Vec<u8>>,
}

impl ContentStore {
    /// Creates a new Content Store.
    pub fn new() -> Self {
        ContentStore {
            store: HashMap::new(),
        }
    }

    /// Inserts data into the Content Store.
    /// # Arguments
    /// * `name` - The name of the data.
    /// * `content` - The content of the data.
    pub fn insert(&mut self, name: String, content: Vec<u8>) {
        self.store.insert(name, content);
    }

    /// Retrieves data from the Content Store.
    /// # Arguments
    /// * `name` - The name of the data.
    /// # Returns
    /// An optional reference to the content if it exists.
    pub fn get(&self, name: &str) -> Option<&Vec<u8>> {
        self.store.get(name)
    }

    /// Checks if the Content Store is empty.
    /// # Returns
    /// True if the Content Store is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.store.is_empty()
    }
}

/// A struct representing the Forwarding Information Base (FIB).
pub struct ForwardingInformationBase {
    table: HashMap<String, Vec<String>>,
}

impl ForwardingInformationBase {
    /// Creates a new FIB.
    pub fn new() -> Self {
        ForwardingInformationBase {
            table: HashMap::new(),
        }
    }

    /// Adds an entry to the FIB.
    /// # Arguments
    /// * `name` - The name of the data.
    /// * `interface` - The interface to forward the data.
    pub fn add_entry(&mut self, name: String, interface: String) {
        self.table.entry(name).or_insert(Vec::new()).push(interface);
    }

    /// Retrieves the interfaces for a given name from the FIB.
    /// # Arguments
    /// * `name` - The name of the data.
    /// # Returns
    /// An optional reference to a vector of interfaces if they exist.
    pub fn get_interfaces(&self, name: &str) -> Option<&Vec<String>> {
        self.table.get(name)
    }

    /// Checks if the FIB is empty.
    /// # Returns
    /// True if the FIB is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}

/// A struct representing the Pending Interest Table (PIT).
pub struct PendingInterestTable {
    table: HashMap<String, Vec<String>>,
}

impl PendingInterestTable {
    /// Creates a new PIT.
    pub fn new() -> Self {
        PendingInterestTable {
            table: HashMap::new(),
        }
    }

    /// Adds an interest to the PIT.
    /// # Arguments
    /// * `name` - The name of the interest.
    /// * `interface` - The interface from which the interest came.
    pub fn add_interest(&mut self, name: String, interface: String) {
        self.table.entry(name).or_insert(Vec::new()).push(interface);
    }

    /// Retrieves the incoming interfaces for a given name from the PIT.
    /// # Arguments
    /// * `name` - The name of the interest.
    /// # Returns
    /// An optional reference to a vector of interfaces if they exist.
    pub fn get_incoming_interfaces(&self, name: &str) -> Option<&Vec<String>> {
        self.table.get(name)
    }

    /// Checks if the PIT is empty.
    /// # Returns
    /// True if the PIT is empty, otherwise false.
    pub fn is_empty(&self) -> bool {
        self.table.is_empty()
    }
}
