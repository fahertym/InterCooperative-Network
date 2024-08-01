// File: crates/icn_testnet/src/main.rs

use icn_core::{IcnNode, Config};
use icn_common::{Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus};
use std::io::{self, Write};
use chrono::{Duration, Utc};
use log::{info, warn, error};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config {
        shard_count: 4,
        consensus_threshold: 0.66,
        consensus_quorum: 0.51,
        network_port: 8080,
    };

    info!("Starting InterCooperative Network testnet...");
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

async fn process_transaction(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a new transaction...");
    
    let from = get_input("From: ")?;
    let to = get_input("To: ")?;
    let amount: f64 = get_input("Amount: ")?.parse()?;
    let currency_type = get_currency_type()?;

    let transaction = Transaction {
        from,
        to,
        amount,
        currency_type,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    node.process_transaction(transaction).await?;
    println!("Transaction processed successfully");
    Ok(())
}

async fn create_proposal(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a new proposal...");
    
    let title = get_input("Title: ")?;
    let description = get_input("Description: ")?;
    let proposer = get_input("Proposer: ")?;
    let proposal_type = get_proposal_type()?;
    let category = get_proposal_category()?;

    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title,
        description,
        proposer,
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

async fn check_balance(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    let address = get_input("Enter address: ")?;
    let currency_type = get_currency_type()?;
    
    let balance = node.get_balance(&address, &currency_type).await?;
    println!("Balance: {} {:?}", balance, currency_type);
    Ok(())
}

async fn create_identity(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    println!("Creating a new identity...");
    
    let name = get_input("Enter name: ")?;
    
    let mut attributes = std::collections::HashMap::new();
    attributes.insert("name".to_string(), name);
    
    let identity_id = node.create_identity(attributes).await?;
    println!("Identity created successfully. ID: {}", identity_id);
    Ok(())
}

async fn allocate_resource(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    println!("Allocating a resource...");

    let resource_type = get_input("Enter resource type: ")?;
    let amount: u64 = get_input("Enter amount: ")?.parse()?;

    node.allocate_resource(&resource_type, amount).await?;
    println!("Resource allocated successfully");
    Ok(())
}

async fn get_network_stats(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    let stats = node.get_network_stats().await?;
    println!("Network Statistics:");
    println!("  Connected Peers: {}", stats.node_count);
    println!("  Total Transactions: {}", stats.total_transactions);
    println!("  Active Proposals: {}", stats.active_proposals);
    Ok(())
}

fn get_input(prompt: &str) -> io::Result<String> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_string())
}

fn get_currency_type() -> Result<CurrencyType, Box<dyn std::error::Error>> {
    println!("Select currency type:");
    println!("1. BasicNeeds");
    println!("2. Education");
    println!("3. Environmental");
    println!("4. Community");
    let choice: u32 = get_input("Enter choice (1-4): ")?.parse()?;
    match choice {
        1 => Ok(CurrencyType::BasicNeeds),
        2 => Ok(CurrencyType::Education),
        3 => Ok(CurrencyType::Environmental),
        4 => Ok(CurrencyType::Community),
        _ => Err("Invalid currency type choice".into()),
    }
}

fn get_proposal_type() -> Result<ProposalType, Box<dyn std::error::Error>> {
    println!("Select proposal type:");
    println!("1. Constitutional");
    println!("2. EconomicAdjustment");
    println!("3. NetworkUpgrade");
    let choice: u32 = get_input("Enter choice (1-3): ")?.parse()?;
    match choice {
        1 => Ok(ProposalType::Constitutional),
        2 => Ok(ProposalType::EconomicAdjustment),
        3 => Ok(ProposalType::NetworkUpgrade),
        _ => Err("Invalid proposal type choice".into()),
    }
}

fn get_proposal_category() -> Result<ProposalCategory, Box<dyn std::error::Error>> {
    println!("Select proposal category:");
    println!("1. Economic");
    println!("2. Technical");
    println!("3. Social");
    let choice: u32 = get_input("Enter choice (1-3): ")?.parse()?;
    match choice {
        1 => Ok(ProposalCategory::Economic),
        2 => Ok(ProposalCategory::Technical),
        3 => Ok(ProposalCategory::Social),
        _ => Err("Invalid proposal category choice".into()),
    }
}