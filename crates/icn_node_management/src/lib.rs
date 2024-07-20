mod content_store;
mod fib;
mod pit;

pub use content_store::ContentStore;
pub use fib::ForwardingInformationBase;
pub use pit::PendingInterestTable;

use icn_core::error::{Error, Result};
use icn_network::Node as NetworkNode;

pub struct IcnNodeManager {
    network_node: NetworkNode,
    content_store: ContentStore,
    fib: ForwardingInformationBase,
    pit: PendingInterestTable,
}

impl IcnNodeManager {
    pub fn new(network_node: NetworkNode) -> Self {
        IcnNodeManager {
            network_node,
            content_store: ContentStore::new(),
            fib: ForwardingInformationBase::new(),
            pit: PendingInterestTable::new(),
        }
    }

    pub fn add_content(&mut self, name: String, content: Vec<u8>) -> Result<()> {
        self.content_store.add(name, content);
        Ok(())
    }

    pub fn get_content(&self, name: &str) -> Option<Vec<u8>> {
        self.content_store.get(name)
    }

    pub fn add_fib_entry(&mut self, prefix: String, next_hop: String) -> Result<()> {
        self.fib.add_entry(prefix, next_hop);
        Ok(())
    }

    pub fn add_pit_entry(&mut self, name: String, incoming_face: String) -> Result<()> {
        self.pit.add_interest(name, &incoming_face);
        Ok(())
    }

    pub fn remove_expired_content(&mut self) {
        self.content_store.remove_expired();
    }

    pub fn clear_expired_pit_entries(&mut self) {
        self.pit.clear_expired();
    }

    pub fn process_interest_packet(&self, packet: Packet) -> Result<()> {
        // Logic to process interest packets
        Ok(())
    }

    pub fn process_data_packet(&self, packet: Packet) -> Result<()> {
        // Logic to process data packets
        Ok(())
    }
}
