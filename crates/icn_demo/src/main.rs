use icn_core::{IcnNode, Config};
use icn_common::{Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, IcnResult};
use icn_identity::DecentralizedIdentity;
use icn_market::{Resource, Market};
use icn_smart_contracts::{SmartContract, SmartContractExecutor};
use chrono::{Duration, Utc};
use log::{info, error};
use std::collections::HashMap;
use rand::Rng;

#[tokio::main]
async fn main() -> IcnResult<()> {
    env_logger::init();

    let config = Config {
        shard_count: 4,
        consensus_threshold: 0.66,
        consensus_quorum: 0.51,
        network_port: 8080,
        zkp_bit_size: 64,
    };

    info!("Starting InterCooperative Network demo...");
    let node = IcnNode::new(config)?;
    node.start().await?;

    // Demo operations
    let (alice, bob) = create_identities(&node).await?;
    mint_initial_currency(&node, &alice, &bob).await?;
    process_transactions(&node, &alice, &bob).await?;
    create_and_vote_on_proposal(&node, &alice, &bob).await?;
    allocate_and_trade_resources(&node, &alice, &bob).await?;
    execute_smart_contract(&node, &alice, &bob).await?;

    // Display final network stats
    let stats = node.get_network_stats().await?;
    info!("Final network stats: {:?}", stats);

    info!("Demo completed successfully!");
    node.stop().await?;

    Ok(())
}

async fn create_identities(node: &IcnNode) -> IcnResult<(DecentralizedIdentity, DecentralizedIdentity)> {
    info!("Creating identities...");
    let alice = node.create_identity(vec![("name".to_string(), "Alice".to_string())].into_iter().collect())?;
    let bob = node.create_identity(vec![("name".to_string(), "Bob".to_string())].into_iter().collect())?;
    info!("Created identities: Alice ({}), Bob ({})", alice.id, bob.id);
    Ok((alice, bob))
}

async fn mint_initial_currency(node: &IcnNode, alice: &DecentralizedIdentity, bob: &DecentralizedIdentity) -> IcnResult<()> {
    info!("Minting initial currency...");
    node.mint_currency(&alice.id, 1000.0, CurrencyType::BasicNeeds)?;
    node.mint_currency(&bob.id, 500.0, CurrencyType::BasicNeeds)?;
    info!("Initial currency minted");
    Ok(())
}

async fn process_transactions(node: &IcnNode, alice: &DecentralizedIdentity, bob: &DecentralizedIdentity) -> IcnResult<()> {
    info!("Processing transactions...");
    let transaction = Transaction {
        from: alice.id.clone(),
        to: bob.id.clone(),
        amount: 100.0,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None, // In a real scenario, this should be signed
    };
    node.process_transaction(transaction).await?;
    info!("Transaction processed");

    // Display updated balances
    let alice_balance = node.get_balance(&alice.id, &CurrencyType::BasicNeeds)?;
    let bob_balance = node.get_balance(&bob.id, &CurrencyType::BasicNeeds)?;
    info!("Updated balances - Alice: {}, Bob: {}", alice_balance, bob_balance);

    Ok(())
}

async fn create_and_vote_on_proposal(node: &IcnNode, alice: &DecentralizedIdentity, bob: &DecentralizedIdentity) -> IcnResult<()> {
    info!("Creating and voting on a proposal...");
    let proposal = Proposal {
        id: "proposal1".to_string(),
        title: "Increase node count".to_string(),
        description: "Proposal to increase the number of nodes in the network".to_string(),
        proposer: alice.id.clone(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: icn_common::ProposalStatus::Active,
        proposal_type: ProposalType::Constitutional,
        category: ProposalCategory::Technical,
        required_quorum: 0.5,
        execution_timestamp: None,
    };

    let proposal_id = node.create_proposal(proposal)?;
    info!("Proposal created with ID: {}", proposal_id);

    // Simulate voting
    node.vote_on_proposal(&proposal_id, &alice.id, true)?;
    node.vote_on_proposal(&proposal_id, &bob.id, false)?;
    info!("Votes cast on the proposal");

    Ok(())
}

async fn allocate_and_trade_resources(node: &IcnNode, alice: &DecentralizedIdentity, bob: &DecentralizedIdentity) -> IcnResult<()> {
    info!("Allocating and trading resources...");
    let market = Market::new();
    
    // Alice allocates computing power
    let computing_power = Resource {
        name: "Computing Power".to_string(),
        quantity: 100.0,
        owner: alice.id.clone(),
    };
    market.add_resource(computing_power.clone())?;
    
    // Bob allocates storage space
    let storage_space = Resource {
        name: "Storage Space".to_string(),
        quantity: 500.0,
        owner: bob.id.clone(),
    };
    market.add_resource(storage_space.clone())?;
    
    // Perform a trade
    market.trade_resource(&alice.id, &bob.id, &computing_power.name, 50.0)?;
    market.trade_resource(&bob.id, &alice.id, &storage_space.name, 200.0)?;
    
    info!("Resources allocated and traded");
    Ok(())
}

async fn execute_smart_contract(node: &IcnNode, alice: &DecentralizedIdentity, bob: &DecentralizedIdentity) -> IcnResult<()> {
    info!("Executing smart contract...");
    let contract_code = r#"
        function transfer(from, to, amount) {
            if (balanceOf(from) >= amount) {
                balanceOf[from] -= amount;
                balanceOf[to] += amount;
                return true;
            }
            return false;
        }
    "#;
    
    let mut executor = SmartContractExecutor::new();
    executor.deploy_contract("transfer_contract".to_string(), contract_code.to_string())?;
    
    let result = executor.execute_contract(
        "transfer_contract",
        "transfer",
        vec![
            alice.id.clone().into(),
            bob.id.clone().into(),
            50.0.into(),
        ],
    )?;
    
    info!("Smart contract execution result: {:?}", result);
    Ok(())
}