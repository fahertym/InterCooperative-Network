// File: crates/icn_demo/src/main.rs

use icn_core::{IcnNode, Config};
use icn_common::{Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus, IcnResult};
use std::io::{self, Write};
use chrono::{Duration, Utc};
use log::{info, warn, error};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> IcnResult<()> {
    env_logger::init();

    let config = Config {
        shard_count: 4,
        consensus_threshold: 0.66,
        consensus_quorum: 0.51,
        network_port: 8080,
    };

    info!("Starting InterCooperative Network demo...");
    let node = IcnNode::new(config).await?;
    node.start().await?;

    info!("Node started successfully. Type 'help' for available commands.");

    loop {
        print!("> ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let input = input.trim();

        match input {
            "help" => print_help(),
            "exit" => break,
            "transaction" => process_transaction(&node).await?,
            "proposal" => create_proposal(&node).await?,
            "balance" => check_balance(&node).await?,
            "identity" => create_identity(&node).await?,
            "allocate" => allocate_resource(&node).await?,
            "network" => get_network_stats(&node).await?,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    info!("Stopping node...");
    node.stop().await?;
    info!("Node stopped. Goodbye!");

    Ok(())
}

fn print_help() {
    println!("Available commands:");
    println!("  help        - Show this help message");
    println!("  transaction - Create a new transaction");
    println!("  proposal    - Create a new proposal");
    println!("  balance     - Check account balance");
    println!("  identity    - Create a new identity");
    println!("  allocate    - Allocate a resource");
    println!("  network     - Get network statistics");
    println!("  exit        - Exit the application");
}

async fn process_transaction(node: &IcnNode) -> IcnResult<()> {
    println!("Creating a new transaction...");
    
    print!("From: ");
    io::stdout().flush()?;
    let mut from = String::new();
    io::stdin().read_line(&mut from)?;
    
    print!("To: ");
    io::stdout().flush()?;
    let mut to = String::new();
    io::stdin().read_line(&mut to)?;
    
    print!("Amount: ");
    io::stdout().flush()?;
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount: f64 = amount_str.trim().parse()?;

    print!("Currency type (BasicNeeds, Education, Environmental, Community): ");
    io::stdout().flush()?;
    let mut currency_type_str = String::new();
    io::stdin().read_line(&mut currency_type_str)?;
    let currency_type = match currency_type_str.trim() {
        "BasicNeeds" => CurrencyType::BasicNeeds,
        "Education" => CurrencyType::Education,
        "Environmental" => CurrencyType::Environmental,
        "Community" => CurrencyType::Community,
        _ => return Err(icn_common::IcnError::InvalidInput("Invalid currency type".to_string())),
    };

    let transaction = Transaction {
        from: from.trim().to_string(),
        to: to.trim().to_string(),
        amount,
        currency_type,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    node.process_transaction(transaction).await?;
    println!("Transaction processed successfully");
    Ok(())
}

async fn create_proposal(node: &IcnNode) -> IcnResult<()> {
    println!("Creating a new proposal...");
    
    print!("Title: ");
    io::stdout().flush()?;
    let mut title = String::new();
    io::stdin().read_line(&mut title)?;
    
    print!("Description: ");
    io::stdout().flush()?;
    let mut description = String::new();
    io::stdin().read_line(&mut description)?;
    
    print!("Proposer: ");
    io::stdout().flush()?;
    let mut proposer = String::new();
    io::stdin().read_line(&mut proposer)?;

    print!("Proposal type (Constitutional, EconomicAdjustment, NetworkUpgrade): ");
    io::stdout().flush()?;
    let mut proposal_type_str = String::new();
    io::stdin().read_line(&mut proposal_type_str)?;
    let proposal_type = match proposal_type_str.trim() {
        "Constitutional" => ProposalType::Constitutional,
        "EconomicAdjustment" => ProposalType::EconomicAdjustment,
        "NetworkUpgrade" => ProposalType::NetworkUpgrade,
        _ => return Err(icn_common::IcnError::InvalidInput("Invalid proposal type".to_string())),
    };

    print!("Proposal category (Economic, Technical, Social): ");
    io::stdout().flush()?;
    let mut category_str = String::new();
    io::stdin().read_line(&mut category_str)?;
    let category = match category_str.trim() {
        "Economic" => ProposalCategory::Economic,
        "Technical" => ProposalCategory::Technical,
        "Social" => ProposalCategory::Social,
        _ => return Err(icn_common::IcnError::InvalidInput("Invalid proposal category".to_string())),
    };

    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        description: description.trim().to_string(),
        proposer: proposer.trim().to_string(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type,
        category,
        required_quorum: 0.66,
        execution_timestamp: None,
    };

    let proposal_id = node.create_proposal(proposal).await?;
    println!("Proposal created successfully. ID: {}", proposal_id);
    Ok(())
}

async fn check_balance(node: &IcnNode) -> IcnResult<()> {
    print!("Enter address: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;

    print!("Enter currency type (BasicNeeds, Education, Environmental, Community): ");
    io::stdout().flush()?;
    let mut currency_type_str = String::new();
    io::stdin().read_line(&mut currency_type_str)?;
    let currency_type = match currency_type_str.trim() {
        "BasicNeeds" => CurrencyType::BasicNeeds,
        "Education" => CurrencyType::Education,
        "Environmental" => CurrencyType::Environmental,
        "Community" => CurrencyType::Community,
        _ => return Err(icn_common::IcnError::InvalidInput("Invalid currency type".to_string())),
    };
    
    let balance = node.get_balance(address.trim(), &currency_type).await?;
    println!("Balance: {} {:?}", balance, currency_type);
    Ok(())
}

async fn create_identity(node: &IcnNode) -> IcnResult<()> {
    println!("Creating a new identity...");
    
    print!("Enter name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;
    
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), name.trim().to_string());
    
    let identity_id = node.create_identity(attributes).await?;
    println!("Identity created successfully. ID: {}", identity_id);
    Ok(())
}

async fn allocate_resource(node: &IcnNode) -> IcnResult<()> {
    println!("Allocating a resource...");

    print!("Enter resource type: ");
    io::stdout().flush()?;
    let mut resource_type = String::new();
    io::stdin().read_line(&mut resource_type)?;

    print!("Enter amount: ");
    io::stdout().flush()?;
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount: u64 = amount_str.trim().parse()?;

    node.allocate_resource(resource_type.trim(), amount).await?;
    println!("Resource allocated successfully");
    Ok(())
}

async fn get_network_stats(node: &IcnNode) -> IcnResult<()> {
    let stats = node.get_network_stats().await?;
    println!("Network Statistics:");
    println!("  Connected Peers: {}", stats.connected_peers);
    println!("  Total Transactions: {}", stats.total_transactions);
    println!("  Active Proposals: {}", stats.active_proposals);
    Ok(())
}