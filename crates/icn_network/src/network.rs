use crate::{discovery::NodeDiscovery, protocol::NetworkProtocol, routing::Router, security::NetworkSecurity};
use icn_common::{IcnError, IcnResult};
use std::sync::{Arc, RwLock};

/// Represents a network in the ICN project.
pub struct Network {
    discovery: Arc<RwLock<NodeDiscovery>>,
    protocol: Arc<RwLock<NetworkProtocol>>,
    router: Arc<RwLock<Router>>,
    security: Arc<RwLock<NetworkSecurity>>,
}

impl Network {
    /// Creates a new network instance.
    ///
    /// # Arguments
    ///
    /// * `discovery` - Arc to the node discovery module.
    /// * `protocol` - Arc to the network protocol module.
    /// * `router` - Arc to the routing module.
    /// * `security` - Arc to the network security module.
    ///
    /// # Returns
    ///
    /// * `Network` - A new instance of Network.
    pub fn new(
        discovery: Arc<RwLock<NodeDiscovery>>,
        protocol: Arc<RwLock<NetworkProtocol>>,
        router: Arc<RwLock<Router>>,
        security: Arc<RwLock<NetworkSecurity>>,
    ) -> Self {
        Network {
            discovery,
            protocol,
            router,
            security,
        }
    }

    /// Starts the network.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn start(&self) -> IcnResult<()> {
        let discovery = self.discovery.read().unwrap();
        discovery.start()?;

        let protocol = self.protocol.read().unwrap();
        protocol.start()?;

        let router = self.router.read().unwrap();
        router.start()?;

        let security = self.security.read().unwrap();
        security.start()?;

        Ok(())
    }

    /// Stops the network.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the operation fails.
    pub fn stop(&self) -> IcnResult<()> {
        let discovery = self.discovery.read().unwrap();
        discovery.stop()?;

        let protocol = self.protocol.read().unwrap();
        protocol.stop()?;

        let router = self.router.read().unwrap();
        router.stop()?;

        let security = self.security.read().unwrap();
        security.stop()?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::MockNodeDiscovery;
    use crate::protocol::MockNetworkProtocol;
    use crate::routing::MockRouter;
    use crate::security::MockNetworkSecurity;
    use std::sync::Arc;

    #[test]
    fn test_network_start_and_stop() {
        let discovery = Arc::new(RwLock::new(MockNodeDiscovery::new()));
        let protocol = Arc::new(RwLock::new(MockNetworkProtocol::new()));
        let router = Arc::new(RwLock::new(MockRouter::new()));
        let security = Arc::new(RwLock::new(MockNetworkSecurity::new()));

        let network = Network::new(discovery, protocol, router, security);

        assert!(network.start().is_ok());
        assert!(network.stop().is_ok());
    }
}
