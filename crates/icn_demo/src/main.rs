// File: icn_demo/src/main.rs

use icn_core::{IcnNode, Config};
use icn_common::{Transaction, Proposal, CurrencyType, ProposalStatus, ProposalType, ProposalCategory};
use tokio;
use std::io::{self, Write};
use chrono::{Duration, Utc};
use log::{info, warn, error};
use uuid::Uuid;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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
            "create_identity" => create_identity(&node).await?,
            "mint_currency" => mint_currency(&node).await?,
            "transaction" => process_transaction(&node).await?,
            "proposal" => create_proposal(&node).await?,
            "vote" => vote_on_proposal(&node).await?,
            "finalize_proposal" => finalize_proposal(&node).await?,
            "balance" => check_balance(&node).await?,
            "blockchain" => print_blockchain(&node).await?,
            "network_stats" => print_network_stats(&node).await?,
            "allocate_resource" => allocate_resource(&node).await?,
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
    println!("  help               - Show this help message");
    println!("  create_identity    - Create a new identity");
    println!("  mint_currency      - Mint currency for an address");
    println!("  transaction        - Create a new transaction");
    println!("  proposal           - Create a new proposal");
    println!("  vote               - Vote on a proposal");
    println!("  finalize_proposal  - Finalize a proposal");
    println!("  balance            - Check account balance");
    println!("  blockchain         - Print the current blockchain");
    println!("  network_stats      - Get network statistics");
    println!("  allocate_resource  - Allocate a resource");
    println!("  exit               - Exit the application");
}

async fn create_identity(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
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

async fn mint_currency(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    println!("Minting currency...");
    
    print!("Enter address: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    let address = address.trim();

    print!("Enter currency type (BasicNeeds, Education, Environmental, Community): ");
    io::stdout().flush()?;
    let mut currency_type_str = String::new();
    io::stdin().read_line(&mut currency_type_str)?;
    let currency_type = match currency_type_str.trim() {
        "BasicNeeds" => CurrencyType::BasicNeeds,
        "Education" => CurrencyType::Education,
        "Environmental" => CurrencyType::Environmental,
        "Community" => CurrencyType::Community,
        _ => return Err("Invalid currency type".into()),
    };

    print!("Enter amount: ");
    io::stdout().flush()?;
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount: f64 = amount_str.trim().parse()?;

    node.mint_currency(address, &currency_type, amount).await?;
    println!("Currency minted successfully");
    Ok(())
}

async fn process_transaction(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
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
        _ => return Err("Invalid currency type".into()),
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

async fn create_proposal(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
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

    let proposal = Proposal {
        id: Uuid::new_v4().to_string(),
        title: title.trim().to_string(),
        description: description.trim().to_string(),
        proposer: proposer.trim().to_string(),
        created_at: Utc::now(),
        voting_ends_at: Utc::now() + Duration::days(7),
        status: ProposalStatus::Active,
        proposal_type: ProposalType::Constitutional,
        category: ProposalCategory::Economic,
        required_quorum: 0.66,
        execution_timestamp: None,
    };

    let proposal_id = node.create_proposal(proposal).await?;
    println!("Proposal created successfully. ID: {}", proposal_id);
    Ok(())
}

async fn vote_on_proposal(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    println!("Voting on a proposal...");

    print!("Proposal ID: ");
    io::stdout().flush()?;
    let mut proposal_id = String::new();
    io::stdin().read_line(&mut proposal_id)?;

    print!("Voter: ");
    io::stdout().flush()?;
    let mut voter = String::new();
    io::stdin().read_line(&mut voter)?;

    print!("In favor? (yes/no): ");
    io::stdout().flush()?;
    let mut in_favor_str = String::new();
    io::stdin().read_line(&mut in_favor_str)?;
    let in_favor = in_favor_str.trim().to_lowercase() == "yes";

    print!("Weight: ");
    io::stdout().flush()?;
    let mut weight_str = String::new();
    io::stdin().read_line(&mut weight_str)?;
    let weight: f64 = weight_str.trim().parse()?;

    node.vote_on_proposal(proposal_id.trim(), voter.trim().to_string(), in_favor, weight).await?;
    println!("Vote recorded successfully");
    Ok(())
}

async fn finalize_proposal(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    println!("Finalizing a proposal...");

    print!("Proposal ID: ");
    io::stdout().flush()?;
    let mut proposal_id = String::new();
    io::stdin().read_line(&mut proposal_id)?;

    let status = node.finalize_proposal(proposal_id.trim()).await?;
    println!("Proposal finalized. Final status: {:?}", status);
    Ok(())
}

async fn check_balance(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
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
        _ => return Err("Invalid currency type".into()),
    };
    
    let balance = node.get_balance(address.trim(), &currency_type).await?;
    println!("Balance: {} {:?}", balance, currency_type);
    Ok(())
}

async fn print_blockchain(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    let blockchain = node.get_blockchain().await?;
    println!("Current Blockchain:");
    for (i, block) in blockchain.iter().enumerate() {
        println!("Block {}: {:?}", i, block);
    }
    Ok(())
}

async fn print_network_stats(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    let stats = node.get_network_stats().await?;
    println!("Network Statistics:");
    println!("  Connected Peers: {}", stats.connected_peers);
    println!("  Total Transactions: {}", stats.total_transactions);
    println!("  Active Proposals: {}", stats.active_proposals);
    Ok(())
}

async fn allocate_resource(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
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