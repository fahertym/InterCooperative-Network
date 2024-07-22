use icn_common::{IcnError, IcnResult};
use serde::{Serialize, Deserialize};

/// Represents a network packet in the ICN project.
#[derive(Serialize, Deserialize, Debug)]
pub struct Packet {
    pub source: String,
    pub destination: String,
    pub payload: Vec<u8>,
}

impl Packet {
    /// Creates a new packet.
    ///
    /// # Arguments
    ///
    /// * `source` - The source address.
    /// * `destination` - The destination address.
    /// * `payload` - The packet payload.
    ///
    /// # Returns
    ///
    /// * `Packet` - A new instance of Packet.
    pub fn new(source: String, destination: String, payload: Vec<u8>) -> Self {
        Packet {
            source,
            destination,
            payload,
        }
    }

    /// Validates the packet.
    ///
    /// # Errors
    ///
    /// Returns `IcnResult` if the packet is invalid.
    pub fn validate(&self) -> IcnResult<()> {
        if self.source.is_empty() || self.destination.is_empty() {
            return Err(IcnError::Network("Invalid packet: source or destination is empty".into()));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_packet_creation_and_validation() {
        let packet = Packet::new("Source".to_string(), "Destination".to_string(), vec![1, 2, 3]);
        assert!(packet.validate().is_ok());

        let invalid_packet = Packet::new("".to_string(), "Destination".to_string(), vec![1, 2, 3]);
        assert!(invalid_packet.validate().is_err());
    }
}
