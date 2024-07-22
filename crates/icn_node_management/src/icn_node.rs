use crate::{content_store::ContentStore, fib::FIB, pit::PIT};
use icn_common::{IcnError, IcnResult};
use log::{info, error};
use std::sync::{Arc, RwLock};

/// Represents an ICN node.
pub struct IcnNode {
    content_store: Arc<ContentStore>,
    fib: Arc<FIB>,
    pit: Arc<PIT>,
}

impl IcnNode {
    /// Creates a new instance of IcnNode.
    ///
    /// # Arguments
    ///
    /// * `content_store` - Arc to the content store.
    /// * `fib` - Arc to the FIB.
    /// * `pit` - Arc to the PIT.
    ///
    /// # Returns
    ///
    /// * `IcnNode` - A new instance of IcnNode.
    pub fn new(content_store: Arc<ContentStore>, fib: Arc<FIB>, pit: Arc<PIT>) -> Self {
        IcnNode {
            content_store,
            fib,
            pit,
        }
    }

    /// Processes an interest packet.
    ///
    /// # Arguments
    ///
    /// * `interest` - The interest packet to be processed.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn process_interest(&self, interest: &str) -> IcnResult<()> {
        // Check content store for the data
        if let Ok(data) = self.content_store.retrieve(interest) {
            // Data found, send data packet
            info!("Data found for interest: {}", interest);
            self.send_data_packet(interest, data)?;
        } else {
            // Data not found, forward interest packet
            info!("Data not found for interest: {}, forwarding", interest);
            let next_hop = self.fib.get_next_hop(interest)?;
            self.forward_interest_packet(interest, &next_hop)?;
        }
        Ok(())
    }

    /// Sends a data packet.
    ///
    /// # Arguments
    ///
    /// * `interest` - The interest associated with the data.
    /// * `data` - The data to be sent.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    fn send_data_packet(&self, interest: &str, data: Vec<u8>) -> IcnResult<()> {
        info!("Sending data packet for interest: {}", interest);
        // Simulate sending data packet
        Ok(())
    }

    /// Forwards an interest packet.
    ///
    /// # Arguments
    ///
    /// * `interest` - The interest packet to be forwarded.
    /// * `next_hop` - The next hop to forward the interest packet to.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    fn forward_interest_packet(&self, interest: &str, next_hop: &str) -> IcnResult<()> {
        info!("Forwarding interest packet: {} to next hop: {}", interest, next_hop);
        // Simulate forwarding interest packet
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::content_store::MockContentStore;
    use crate::fib::MockFIB;
    use crate::pit::MockPIT;

    #[test]
    fn test_process_interest() {
        let content_store = Arc::new(MockContentStore::new());
        let fib = Arc::new(MockFIB::new());
        let pit = Arc::new(MockPIT::new());

        let node = IcnNode::new(content_store, fib, pit);

        assert!(node.process_interest("interest_1").is_ok());
    }
}
