// Filename: src/tests/icn_node_tests.rs

// Import necessary modules and types
use crate::icn_node::{IcnNode, Packet, PacketType};
use std::net::SocketAddr;

#[test]
fn test_fib_functionality() {
    let mut node = IcnNode::new();
    let addr1: SocketAddr = "127.0.0.1:8000".parse().unwrap();
    let addr2: SocketAddr = "127.0.0.1:8001".parse().unwrap();

    node.fib.add_entry("/example/path".to_string(), addr1);
    node.fib.add_entry("/example/path".to_string(), addr2);

    let next_hops = node.fib.get_next_hops("/example/path");
    assert!(next_hops.is_some());
    assert_eq!(next_hops.unwrap().len(), 2);
    assert!(next_hops.unwrap().contains(&addr1));
    assert!(next_hops.unwrap().contains(&addr2));

    let longest_match = node.fib.longest_prefix_match("/example/path/subpath");
    assert!(longest_match.is_some());
    assert_eq!(longest_match.unwrap().name, "/example/path");
}

#[test]
fn test_content_store() {
    let mut node = IcnNode::new();
    let packet = Packet {
        packet_type: PacketType::Data,
        name: "/test/data".to_string(),
        content: vec![1, 2, 3, 4],
    };

    node.content_store.add(packet.name.clone(), packet.clone());
    let retrieved = node.content_store.get(&packet.name);
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().content, vec![1, 2, 3, 4]);
}

#[test]
fn test_packet_processing() {
    let mut node = IcnNode::new();
    let addr: SocketAddr = "127.0.0.1:8000".parse().unwrap();
    node.add_interface("eth0".to_string(), addr);
    node.fib.add_entry("/test".to_string(), addr);

    let interest_packet = Packet {
        packet_type: PacketType::Interest,
        name: "/test/data".to_string(),
        content: vec![],
    };

    node.process_packet(interest_packet.clone(), "eth0");
    assert!(node.pit.has_pending_interest(&interest_packet.name));

    let data_packet = Packet {
        packet_type: PacketType::Data,
        name: "/test/data".to_string(),
        content: vec![1, 2, 3, 4],
    };

    node.process_packet(data_packet.clone(), "eth0");
    assert!(!node.pit.has_pending_interest(&data_packet.name));
    assert!(node.content_store.get(&data_packet.name).is_some());
}
