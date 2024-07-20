use icn_core::{IcnNode, Config};
use icn_types::{Transaction, Proposal, ProposalType, ProposalCategory, CurrencyType, ProposalStatus};
use std::io::{self, Write};
use chrono::{Duration, Utc};
use log::{info, warn, error};
use uuid::Uuid;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let config = Config {
        shard_count: 4,
        consensus_threshold: 0.66,
        consensus_quorum: 0.51,
        network_port: 8080,
    };

    info!("Starting InterCooperative Network testnet...");
    let node = IcnNode::new(config)?;
    node.start()?;

    info!("Node started successfully. Type 'help' for available commands.");

    loop {
        print!("> ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        match input {
            "help" => print_help(),
            "exit" => break,
            "transaction" => process_transaction(&node)?,
            "proposal" => create_proposal(&node)?,
            "balance" => check_balance(&node)?,
            "identity" => create_identity(&node)?,
            "allocate" => allocate_resource(&node)?,
            "network" => get_network_stats(&node)?,
            _ => println!("Unknown command. Type 'help' for available commands."),
        }
    }

    info!("Stopping node...");
    node.stop()?;
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

fn process_transaction(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Processing a new transaction");
    
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

    let transaction = Transaction {
        from: from.trim().to_string(),
        to: to.trim().to_string(),
        amount,
        currency_type: CurrencyType::BasicNeeds,
        timestamp: Utc::now().timestamp(),
        signature: None,
    };

    node.process_transaction(transaction)?;
    info!("Transaction processed successfully");
    Ok(())
}

fn create_proposal(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating a new proposal");
    
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

    let proposal_id = node.create_proposal(proposal)?;
    info!("Proposal created successfully with ID: {}", proposal_id);
    Ok(())
}

fn check_balance(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Checking balance");
    
    print!("Address: ");
    io::stdout().flush()?;
    let mut address = String::new();
    io::stdin().read_line(&mut address)?;
    
    let balance = node.get_balance(address.trim(), &CurrencyType::BasicNeeds)?;
    println!("Balance: {}", balance);
    Ok(())
}

fn create_identity(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Creating a new identity");
    
    print!("Name: ");
    io::stdout().flush()?;
    let mut name = String::new();
    io::stdin().read_line(&mut name)?;

    print!("Email: ");
    io::stdout().flush()?;
    let mut email = String::new();
    io::stdin().read_line(&mut email)?;

    let mut attributes = std::collections::HashMap::new();
    attributes.insert("name".to_string(), name.trim().to_string());
    attributes.insert("email".to_string(), email.trim().to_string());

    let identity = node.create_identity(attributes)?;
    info!("Identity created successfully with ID: {}", identity.id);
    Ok(())
}

fn allocate_resource(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Allocating a resource");
    
    print!("Resource ID: ");
    io::stdout().flush()?;
    let mut resource_id = String::new();
    io::stdin().read_line(&mut resource_id)?;

    print!("Amount: ");
    io::stdout().flush()?;
    let mut amount_str = String::new();
    io::stdin().read_line(&mut amount_str)?;
    let amount: u64 = amount_str.trim().parse()?;

    node.allocate_resource(resource_id.trim(), amount)?;
    info!("Resource allocated successfully");
    Ok(())
}

fn get_network_stats(node: &IcnNode) -> Result<(), Box<dyn std::error::Error>> {
    info!("Getting network statistics");
    
    let stats = node.get_network_stats()?;
    println!("Connected peers: {}", stats.connected_peers);
    println!("Total transactions: {}", stats.total_transactions);
    println!("Network uptime: {} seconds", stats.uptime.as_secs());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_node_creation_and_basic_operations() {
        let config = Config {
            shard_count: 4,
            consensus_threshold: 0.66,
            consensus_quorum: 0.51,
            network_port: 8080,
        };

        let node = IcnNode::new(config).unwrap();
        assert!(node.start().is_ok());

        // Test create identity
        let mut attributes = std::collections::HashMap::new();
        attributes.insert("name".to_string(), "Alice".to_string());
        attributes.insert("email".to_string(), "alice@example.com".to_string());
        let identity = node.create_identity(attributes).unwrap();
        assert_eq!(identity.attributes.get("name"), Some(&"Alice".to_string()));

        // Test process transaction
        let transaction = Transaction {
            from: "Alice".to_string(),
            to: "Bob".to_string(),
            amount: 50.0,
            currency_type: CurrencyType::BasicNeeds,
            timestamp: Utc::now().timestamp(),
            signature: None,
        };
        assert!(node.process_transaction(transaction).is_ok());

        // Test create proposal
        let proposal = Proposal {
            id: Uuid::new_v4().to_string(),
            title: "Test Proposal".to_string(),
            description: "This is a test proposal".to_string(),
            proposer: "Alice".to_string(),
            created_at: Utc::now(),
            voting_ends_at: Utc::now() + Duration::days(7),
            status: ProposalStatus::Active,
            proposal_type: ProposalType::Constitutional,
            category: ProposalCategory::Economic,
            required_quorum: 0.66,
            execution_timestamp: None,
        };
        assert!(node.create_proposal(proposal).is_ok());

        // Test get network stats
        let stats = node.get_network_stats().unwrap();
        assert!(stats.connected_peers >= 0);

        assert!(node.stop().is_ok());
    }
}