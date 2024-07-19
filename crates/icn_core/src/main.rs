use log::{info, warn, error};
use chrono::Utc;
use std::collections::HashMap;
use std::error::Error;
use std::sync::Arc;

use icn_node::blockchain::Transaction;
use icn_node::consensus::PoCConsensus;
use icn_node::currency::CurrencyType;
use icn_node::governance::{DemocraticSystem, ProposalType, ProposalCategory};
use icn_node::identity::DecentralizedIdentity;
use icn_node::network::Network;
use icn_node::network::node::{Node, NodeType};
use icn_node::vm::CSCLCompiler;
use icn_node::IcnNode;
use icn_node::error::Error as IcnNodeError;

fn main() -> Result<(), Box<dyn Error>> {
    env_logger::init();
    info!("Starting ICN Node");

    let node = Arc::new(IcnNode::new());
    let mut network = Network::new();
    let mut consensus = PoCConsensus::new(0.5, 0.66);
    let mut democratic_system = DemocraticSystem::new();

    setup_network_and_consensus(&mut network, &mut consensus)?;
    process_initial_transactions(Arc::clone(&node))?;
    create_and_vote_on_proposal(&mut democratic_system)?;
    compile_and_run_cscl(Arc::clone(&node))?;
    simulate_cross_shard_transaction(Arc::clone(&node))?;
    print_final_state(&node, &consensus, &democratic_system);

    info!("ICN Node simulation completed.");
    Ok(())
}

fn setup_network_and_consensus(network: &mut Network, consensus: &mut PoCConsensus) -> Result<(), Box<dyn Error>> {
    let node1 = Node::new("Node1", NodeType::PersonalDevice, "127.0.0.1:8000");
    let node2 = Node::new("Node2", NodeType::PersonalDevice, "127.0.0.1:8001");
    network.add_node(node1);
    network.add_node(node2);

    consensus.add_member("Alice".to_string(), false)?;
    consensus.add_member("Bob".to_string(), false)?;
    consensus.add_member("Charlie".to_string(), false)?;
    consensus.add_member("CorpX".to_string(), true)?;

    Ok(())
}

fn process_initial_transactions(node: Arc<IcnNode>) -> Result<(), Box<dyn Error>> {
    let (alice_did, _) = DecentralizedIdentity::new(HashMap::new());
    let (bob_did, _) = DecentralizedIdentity::new(HashMap::new());

    let tx = Transaction::new(
        alice_did.id.clone(),
        bob_did.id.clone(),
        100.0,
        CurrencyType::BasicNeeds,
        1000,
    );

    node.with_blockchain(|blockchain| {
        blockchain.add_transaction(tx.clone())?;
        blockchain.create_block("Alice".to_string())?;
        if let Some(latest_block) = blockchain.get_latest_block() {
            info!("New block created: {:?}", latest_block);
        } else {
            warn!("No blocks in the blockchain to broadcast");
        }
        Ok(())
    })?;

    Ok(())
}

fn create_and_vote_on_proposal(democratic_system: &mut DemocraticSystem) -> Result<(), Box<dyn Error>> {
    let proposal_id = democratic_system.create_proposal(
        "Community Garden".to_string(),
        "Create a community garden in the local park".to_string(),
        "Alice".to_string(),
        chrono::Duration::weeks(1),
        ProposalType::Constitutional,
        ProposalCategory::Economic,
        0.51,
        Some(Utc::now() + chrono::Duration::days(30)),
    )?;

    democratic_system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0)?;
    democratic_system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0)?;
    democratic_system.vote("David".to_string(), proposal_id.clone(), true, 1.0)?;
    democratic_system.tally_votes(&proposal_id)?;

    let proposal = democratic_system.get_proposal(&proposal_id)
        .ok_or("Proposal not found after voting")?;
    info!("Proposal status after voting: {:?}", proposal.status);

    Ok(())
}

fn compile_and_run_cscl(node: Arc<IcnNode>) -> Result<(), Box<dyn Error>> {
    let cscl_code = r#"
    x = 100 + 50;
    y = 200 - 25;
    z = x * y / 10;
    emit("Result", z);
    "#;

    let mut compiler = CSCLCompiler::new(cscl_code);
    let opcodes = compiler.compile()?;
    
    node.with_coop_vm(|coop_vm| {
        coop_vm.load_program(opcodes);
        coop_vm.run().map_err(IcnNodeError::VmError)
    })?;

    Ok(())
}

fn simulate_cross_shard_transaction(node: Arc<IcnNode>) -> Result<(), Box<dyn Error>> {
    node.process_cross_shard_transaction(|sharding_manager| {
        sharding_manager.add_address_to_shard("Alice".to_string(), 0);
        sharding_manager.add_address_to_shard("Bob".to_string(), 1);
        sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0)
    })?;

    let transaction = Transaction::new(
        "Alice".to_string(),
        "Bob".to_string(),
        500.0,
        CurrencyType::BasicNeeds,
        1000,
    );

    node.process_cross_shard_transaction(|sharding_manager| {
        sharding_manager.transfer_between_shards(0, 1, &transaction)
    })?;

    let alice_balance = node.with_sharding_manager(|sharding_manager| {
        sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds)
    })?;

    let bob_balance = node.with_sharding_manager(|sharding_manager| {
        sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds)
    })?;

    info!("Alice's balance after cross-shard transaction: {:?}", alice_balance);
    info!("Bob's balance after cross-shard transaction: {:?}", bob_balance);

    Ok(())
}

fn print_final_state(node: &Arc<IcnNode>, consensus: &PoCConsensus, democratic_system: &DemocraticSystem) {
    info!("Blockchain state:");
    if let Err(e) = node.with_blockchain(|blockchain| {
        info!("Number of blocks: {}", blockchain.chain.len());
        if let Some(latest_block) = blockchain.get_latest_block() {
            info!("Latest block hash: {}", latest_block.hash);
        } else {
            warn!("No blocks in the blockchain");
        }
        Ok(())
    }) {
        error!("Failed to read blockchain state: {}", e);
    }

    info!("Consensus state:");
    info!("Number of members: {}", consensus.members.len());
    info!("Current vote threshold: {}", consensus.threshold);

    info!("Democratic system state:");
    info!("Number of active proposals: {}", democratic_system.list_active_proposals().len());

    info!("Sharding state:");
    if let Err(e) = node.with_sharding_manager(|sharding_manager| {
        info!("Number of shards: {}", sharding_manager.get_shard_count());
        Ok(())
    }) {
        error!("Failed to read sharding state: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_node::governance::democracy::ProposalStatus;
    use ed25519_dalek::Keypair;
    use rand::rngs::OsRng;

    #[test]
    fn test_icn_node_creation() {
        let node = Arc::new(IcnNode::new());
        assert!(node.with_blockchain(|blockchain| Ok(blockchain.chain.len() == 1)).unwrap());
        info!("ICN Node creation test passed");
    }

    #[test]
    fn test_cross_shard_transaction() -> Result<(), Box<dyn Error>> {
        let node = Arc::new(IcnNode::new());

        node.process_cross_shard_transaction(|sharding_manager| {
            sharding_manager.add_address_to_shard("Alice".to_string(), 0);
            sharding_manager.add_address_to_shard("Bob".to_string(), 1);
            sharding_manager.initialize_balance("Alice".to_string(), CurrencyType::BasicNeeds, 1000.0)
        })?;

        let mut csprng = OsRng{};
        let keypair: Keypair = Keypair::generate(&mut csprng);

        let mut transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            500.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        transaction.sign(&keypair)?;

        node.process_cross_shard_transaction(|sharding_manager| {
            sharding_manager.transfer_between_shards(0, 1, &transaction)
        })?;

        let alice_balance = node.with_sharding_manager(|sharding_manager| {
            sharding_manager.get_balance("Alice".to_string(), CurrencyType::BasicNeeds)
        })?;

        let bob_balance = node.with_sharding_manager(|sharding_manager| {
            sharding_manager.get_balance("Bob".to_string(), CurrencyType::BasicNeeds)
        })?;

        assert_eq!(alice_balance, 500.0);
        assert_eq!(bob_balance, 500.0);

        info!("Cross-shard transaction test passed");
        Ok(())
    }

    #[test]
    fn test_smart_contract_execution() -> Result<(), Box<dyn Error>> {
        let node = Arc::new(IcnNode::new());
        compile_and_run_cscl(Arc::clone(&node))?;
        info!("Smart contract execution test passed");
        Ok(())
    }

    #[test]
    fn test_democratic_system() -> Result<(), Box<dyn Error>> {
        let mut democratic_system = DemocraticSystem::new();
        
        let proposal_id = democratic_system.create_proposal(
            "Community Garden".to_string(),
            "Create a community garden in the local park".to_string(),
            "Alice".to_string(),
            chrono::Duration::seconds(5), // Shorter duration for testing
            ProposalType::Constitutional,
            ProposalCategory::Economic,
            0.51,
            Some(Utc::now() + chrono::Duration::days(30)),
        )?;

        democratic_system.vote("Bob".to_string(), proposal_id.clone(), true, 1.0)?;
        democratic_system.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0)?;
        democratic_system.vote("David".to_string(), proposal_id.clone(), true, 1.0)?;

        // Wait for the voting period to end
        std::thread::sleep(std::time::Duration::from_secs(6));

        democratic_system.tally_votes(&proposal_id)?;

        let proposal = democratic_system.get_proposal(&proposal_id)
            .ok_or("Proposal not found after voting")?;
        assert_eq!(proposal.status, ProposalStatus::Passed);
        
        info!("Democratic system test passed");
        Ok(())
    }
}
