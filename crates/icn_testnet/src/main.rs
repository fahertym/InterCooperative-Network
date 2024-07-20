// File: icn_testnet/src/main.rs

use icn_types::{IcnResult, IcnError, Block, Transaction, CurrencyType};
use icn_blockchain::Blockchain;
use icn_consensus::PoCConsensus;
use icn_currency::CurrencySystem;
use icn_governance::DemocraticSystem;
use icn_identity::IdentityManager;
use icn_network::Network;
use icn_sharding::ShardingManager;
use icn_storage::StorageManager;
use icn_vm::CoopVM;
use icn_api::ApiLayer;

use std::sync::Arc;
use tokio::sync::RwLock;
use log::{info, error};

#[tokio::main]
async fn main() -> IcnResult<()> {
    // Initialize logging
    env_logger::init();

    info!("Starting ICN TestNet");

    // Initialize components
    let blockchain = Arc::new(RwLock::new(Blockchain::new()));
    let consensus = Arc::new(RwLock::new(PoCConsensus::new(0.66, 0.51)));
    let currency_system = Arc::new(RwLock::new(CurrencySystem::new()));
    let governance = Arc::new(RwLock::new(DemocraticSystem::new()));
    let identity_manager = Arc::new(RwLock::new(IdentityManager::new()));
    let network = Arc::new(RwLock::new(Network::new()));
    let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(4)));
    let storage_manager = Arc::new(RwLock::new(StorageManager::new()));
    let vm = Arc::new(RwLock::new(CoopVM::new(vec![])));

    // Initialize API layer
    let api = ApiLayer::new(
        blockchain.clone(),
        consensus.clone(),
        currency_system.clone(),
        governance.clone(),
    );

    // Simulate network setup
    setup_network(network.clone()).await?;

    // Simulate initial transactions
    process_initial_transactions(blockchain.clone(), currency_system.clone()).await?;

    // Simulate governance actions
    simulate_governance(governance.clone()).await?;

    // Simulate smart contract execution
    simulate_smart_contract(vm.clone()).await?;

    // Simulate cross-shard transaction
    simulate_cross_shard_transaction(sharding_manager.clone()).await?;

    // Print final state
    print_final_state(
        blockchain.clone(),
        consensus.clone(),
        currency_system.clone(),
        governance.clone(),
        sharding_manager.clone(),
    ).await?;

    info!("ICN TestNet simulation completed successfully");
    Ok(())
}

async fn setup_network(network: Arc<RwLock<Network>>) -> IcnResult<()> {
    let mut net = network.write().await;
    // Add nodes to the network
    net.add_node("Node1".to_string(), "127.0.0.1:8001".parse().unwrap())?;
    net.add_node("Node2".to_string(), "127.0.0.1:8002".parse().unwrap())?;
    net.add_node("Node3".to_string(), "127.0.0.1:8003".parse().unwrap())?;
    info!("Network setup completed");
    Ok(())
}

async fn process_initial_transactions(
    blockchain: Arc<RwLock<Blockchain>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
) -> IcnResult<()> {
    let mut chain = blockchain.write().await;
    let mut currency = currency_system.write().await;

    // Create some initial transactions
    let tx1 = Transaction::new("Alice".to_string(), "Bob".to_string(), 100.0, CurrencyType::BasicNeeds, 0);
    let tx2 = Transaction::new("Bob".to_string(), "Charlie".to_string(), 50.0, CurrencyType::Education, 1);

    chain.add_transaction(tx1.clone())?;
    chain.add_transaction(tx2.clone())?;

    // Update balances
    currency.transfer("Alice", "Bob", &CurrencyType::BasicNeeds, 100.0)?;
    currency.transfer("Bob", "Charlie", &CurrencyType::Education, 50.0)?;

    info!("Initial transactions processed");
    Ok(())
}

async fn simulate_governance(governance: Arc<RwLock<DemocraticSystem>>) -> IcnResult<()> {
    let mut gov = governance.write().await;

    // Create a proposal
    let proposal_id = gov.create_proposal(
        "Community Garden".to_string(),
        "Create a community garden in the local park".to_string(),
        "Alice".to_string(),
        chrono::Duration::days(7),
        icn_types::ProposalType::Community,
        icn_types::ProposalCategory::Environmental,
        0.51,
        None,
    )?;

    // Simulate voting
    gov.vote("Bob".to_string(), proposal_id.clone(), true, 1.0)?;
    gov.vote("Charlie".to_string(), proposal_id.clone(), false, 1.0)?;
    gov.vote("Dave".to_string(), proposal_id.clone(), true, 1.0)?;

    // Tally votes
    gov.tally_votes(&proposal_id)?;

    info!("Governance simulation completed");
    Ok(())
}

async fn simulate_smart_contract(vm: Arc<RwLock<CoopVM>>) -> IcnResult<()> {
    let mut coop_vm = vm.write().await;

    // Simple smart contract to add two numbers
    let contract = vec![
        icn_vm::Opcode::Push(icn_vm::Value::Int(5)),
        icn_vm::Opcode::Push(icn_vm::Value::Int(3)),
        icn_vm::Opcode::Add,
    ];

    coop_vm.load_program(contract);
    coop_vm.run()?;

    info!("Smart contract simulation completed");
    Ok(())
}

async fn simulate_cross_shard_transaction(sharding_manager: Arc<RwLock<ShardingManager>>) -> IcnResult<()> {
    let mut sharding = sharding_manager.write().await;

    // Simulate a cross-shard transaction
    let tx = Transaction::new("Alice".to_string(), "Bob".to_string(), 75.0, CurrencyType::BasicNeeds, 2);
    sharding.process_cross_shard_transaction(0, 1, &tx)?;

    info!("Cross-shard transaction simulation completed");
    Ok(())
}

async fn print_final_state(
    blockchain: Arc<RwLock<Blockchain>>,
    consensus: Arc<RwLock<PoCConsensus>>,
    currency_system: Arc<RwLock<CurrencySystem>>,
    governance: Arc<RwLock<DemocraticSystem>>,
    sharding_manager: Arc<RwLock<ShardingManager>>,
) -> IcnResult<()> {
    let chain = blockchain.read().await;
    let cons = consensus.read().await;
    let curr = currency_system.read().await;
    let gov = governance.read().await;
    let shard = sharding_manager.read().await;

    info!("Final State of ICN TestNet:");
    info!("Blockchain:");
    info!("  - Number of blocks: {}", chain.chain.len());
    if let Some(last_block) = chain.chain.last() {
        info!("  - Last block hash: {}", last_block.hash);
    }

    info!("Consensus:");
    info!("  - Number of validators: {}", cons.get_validators().len());
    info!("  - Consensus threshold: {}", cons.threshold);

    info!("Currency System:");
    for (currency_type, currency) in &curr.currencies {
        info!("  - {}: Total supply: {}", currency_type, currency.total_supply);
    }

    info!("Governance:");
    info!("  - Number of active proposals: {}", gov.list_active_proposals().len());

    info!("Sharding:");
    info!("  - Number of shards: {}", shard.get_shard_count());

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_testnet_simulation() -> IcnResult<()> {
        // Initialize components for testing
        let blockchain = Arc::new(RwLock::new(Blockchain::new()));
        let consensus = Arc::new(RwLock::new(PoCConsensus::new(0.66, 0.51)));
        let currency_system = Arc::new(RwLock::new(CurrencySystem::new()));
        let governance = Arc::new(RwLock::new(DemocraticSystem::new()));
        let network = Arc::new(RwLock::new(Network::new()));
        let sharding_manager = Arc::new(RwLock::new(ShardingManager::new(4)));
        let vm = Arc::new(RwLock::new(CoopVM::new(vec![])));

        // Run test simulations
        setup_network(network.clone()).await?;
        process_initial_transactions(blockchain.clone(), currency_system.clone()).await?;
        simulate_governance(governance.clone()).await?;
        simulate_smart_contract(vm.clone()).await?;
        simulate_cross_shard_transaction(sharding_manager.clone()).await?;

        // Verify final state
        let chain = blockchain.read().await;
        assert!(chain.chain.len() > 1, "Blockchain should have more than one block");

        let curr = currency_system.read().await;
        assert!(!curr.currencies.is_empty(), "Currency system should have currencies");

        let gov = governance.read().await;
        assert!(!gov.list_active_proposals().is_empty(), "There should be active proposals");

        let shard = sharding_manager.read().await;
        assert_eq!(shard.get_shard_count(), 4, "There should be 4 shards");

        Ok(())
    }
}