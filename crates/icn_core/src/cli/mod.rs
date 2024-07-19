use crate::IcnNode;
use icn_blockchain::Blockchain;
use icn_vm::smart_contract::{AssetTokenContract, BondContract};
use chrono::Utc;
use std::io::{self, Write};

pub fn run_cli(node: &mut IcnNode) {
    loop {
        print_menu();
        let choice = get_user_input("Enter your choice: ");

        match choice.trim() {
            "1" => deploy_contract(node),
            "2" => execute_contracts(node),
            "3" => view_blockchain_state(node),
            "4" => break,
            "9" => create_asset_token(node),
            "10" => transfer_asset_token(node),
            "11" => view_asset_token(node),
            "12" => create_bond(node),
            "13" => transfer_bond(node),
            "14" => view_bond(node),
            _ => println!("Invalid choice. Please try again."),
        }
    }
}

fn print_menu() {
    println!("\n--- Smart Contract CLI ---");
    println!("1. Deploy a new smart contract");
    println!("2. Execute smart contracts in the latest block");
    println!("3. View blockchain state");
    println!("4. Exit");
    println!("9. Create a new asset token");
    println!("10. Transfer an asset token");
    println!("11. View asset token details");
    println!("12. Create a new bond");
    println!("13. Transfer a bond");
    println!("14. View bond details");
}

fn get_user_input(prompt: &str) -> String {
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    input.trim().to_string()
}

fn deploy_contract(node: &mut IcnNode) {
    // Add logic to deploy a smart contract
    println!("Deploying a new smart contract...");
}

fn execute_contracts(node: &mut IcnNode) {
    node.with_blockchain(|blockchain| {
        blockchain.execute_smart_contracts()
    }).unwrap_or_else(|e| println!("Failed to execute smart contracts: {}", e));
}

fn view_blockchain_state(node: &IcnNode) {
    node.with_blockchain(|blockchain| {
        println!("{:#?}", blockchain);
        Ok(())
    }).unwrap_or_else(|e| println!("Failed to view blockchain state: {}", e));
}

fn create_asset_token(node: &mut IcnNode) {
    let asset_id = get_user_input("Enter asset ID: ");
    let name = get_user_input("Enter asset name: ");
    let description = get_user_input("Enter asset description: ");
    let owner = get_user_input("Enter owner ID: ");
    let value: f64 = get_user_input("Enter asset value: ").parse().unwrap_or(0.0);

    let contract = AssetTokenContract::new(asset_id, name, description, owner, value);
    node.with_blockchain(|blockchain| {
        blockchain.deploy_smart_contract(Box::new(contract))
    }).unwrap_or_else(|e| println!("Failed to create asset token: {}", e));
}

fn transfer_asset_token(node: &mut IcnNode) {
    let asset_id = get_user_input("Enter asset ID: ");
    let new_owner = get_user_input("Enter new owner ID: ");

    node.with_blockchain(|blockchain| {
        blockchain.transfer_asset_token(&asset_id, &new_owner)
    }).unwrap_or_else(|e| println!("Failed to transfer asset token: {}", e));
}

fn view_asset_token(node: &IcnNode) {
    let asset_id = get_user_input("Enter asset ID: ");
    node.with_blockchain(|blockchain| {
        if let Some(token) = blockchain.get_asset_token(&asset_id) {
            println!("{:?}", token);
        } else {
            println!("Asset token not found");
        }
        Ok(())
    }).unwrap_or_else(|e| println!("Failed to view asset token: {}", e));
}

fn create_bond(node: &mut IcnNode) {
    let bond_id = get_user_input("Enter bond ID: ");
    let name = get_user_input("Enter bond name: ");
    let description = get_user_input("Enter bond description: ");
    let issuer = get_user_input("Enter issuer ID: ");
    let face_value: f64 = get_user_input("Enter face value: ").parse().unwrap_or(0.0);
    let maturity_date = get_user_input("Enter maturity date (YYYY-MM-DD): ");
    let interest_rate: f64 = get_user_input("Enter interest rate: ").parse().unwrap_or(0.0);
    let owner = get_user_input("Enter owner ID: ");

    let maturity_date = chrono::NaiveDate::parse_from_str(&maturity_date, "%Y-%m-%d")
        .map(|date| date.and_hms(0, 0, 0))
        .map(|naive| chrono::DateTime::<Utc>::from_naive_utc_and_offset(naive, Utc))
        .unwrap_or_else(|_| Utc::now());

    let contract = BondContract::new(bond_id, name, description, issuer, face_value, maturity_date, interest_rate, owner);
    node.with_blockchain(|blockchain| {
        blockchain.deploy_smart_contract(Box::new(contract))
    }).unwrap_or_else(|e| println!("Failed to create bond: {}", e));
}

fn transfer_bond(node: &mut IcnNode) {
    let bond_id = get_user_input("Enter bond ID: ");
    let new_owner = get_user_input("Enter new owner ID: ");

    node.with_blockchain(|blockchain| {
        blockchain.transfer_bond(&bond_id, &new_owner)
    }).unwrap_or_else(|e| println!("Failed to transfer bond: {}", e));
}

fn view_bond(node: &IcnNode) {
    let bond_id = get_user_input("Enter bond ID: ");
    node.with_blockchain(|blockchain| {
        if let Some(bond) = blockchain.get_bond(&bond_id) {
            println!("{:?}", bond);
        } else {
            println!("Bond not found");
        }
        Ok(())
    }).unwrap_or_else(|e| println!("Failed to view bond: {}", e));
}