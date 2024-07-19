use crate::blockchain::Blockchain;
use crate::smart_contract::{AssetTokenContract, BondContract};
use chrono::Utc;
use std::io::{self, Write};

fn run_cli(blockchain: &mut Blockchain) {
    loop {
        print_menu();
        let choice = get_user_input("Enter your choice: ");

        match choice.trim() {
            "1" => deploy_contract(blockchain),
            "2" => execute_contracts(blockchain),
            "3" => view_blockchain_state(blockchain),
            "4" => break,
            "9" => create_asset_token(blockchain),
            "10" => transfer_asset_token(blockchain),
            "11" => view_asset_token(blockchain),
            "12" => create_bond(blockchain),
            "13" => transfer_bond(blockchain),
            "14" => view_bond(blockchain),
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

fn deploy_contract(blockchain: &mut Blockchain) {
    // Add logic to deploy a smart contract
}

fn execute_contracts(blockchain: &mut Blockchain) {
    match blockchain.execute_smart_contracts() {
        Ok(_) => println!("Executed smart contracts in the latest block successfully."),
        Err(e) => println!("Failed to execute smart contracts: {}", e),
    }
}

fn view_blockchain_state(blockchain: &Blockchain) {
    println!("{:#?}", blockchain);
}

fn create_asset_token(blockchain: &mut Blockchain) {
    let asset_id = get_user_input("Enter asset ID: ");
    let name = get_user_input("Enter asset name: ");
    let description = get_user_input("Enter asset description: ");
    let owner = get_user_input("Enter owner ID: ");
    let value = get_user_input("Enter asset value: ").parse().unwrap();

    let contract = AssetTokenContract::new(asset_id, name, description, owner, value);
    match blockchain.deploy_smart_contract(Box::new(contract)) {
        Ok(_) => println!("Asset token created successfully!"),
        Err(e) => println!("Failed to create asset token: {}", e),
    }
}

fn transfer_asset_token(blockchain: &mut Blockchain) {
    let asset_id = get_user_input("Enter asset ID: ");
    let new_owner = get_user_input("Enter new owner ID: ");

    match blockchain.transfer_asset_token(&asset_id, &new_owner) {
        Ok(_) => println!("Asset token transferred successfully!"),
        Err(e) => println!("Failed to transfer asset token: {}", e),
    }
}

fn view_asset_token(blockchain: &Blockchain) {
    let asset_id = get_user_input("Enter asset ID: ");
    match blockchain.get_asset_token(&asset_id) {
        Some(token) => println!("{:?}", token),
        None => println!("Asset token not found"),
    }
}

fn create_bond(blockchain: &mut Blockchain) {
    let bond_id = get_user_input("Enter bond ID: ");
    let name = get_user_input("Enter bond name: ");
    let description = get_user_input("Enter bond description: ");
    let issuer = get_user_input("Enter issuer ID: ");
    let face_value = get_user_input("Enter face value: ").parse().unwrap();
    let maturity_date = get_user_input("Enter maturity date (YYYY-MM-DD): ");
    let interest_rate = get_user_input("Enter interest rate: ").parse().unwrap();
    let owner = get_user_input("Enter owner ID: ");

    let maturity_date = DateTime::parse_from_rfc3339(&format!("{}T00:00:00Z", maturity_date)).unwrap().with_timezone(&Utc);

    let contract = BondContract::new(bond_id, name, description, issuer, face_value, maturity_date, interest_rate, owner);
    match blockchain.deploy_smart_contract(Box::new(contract)) {
        Ok(_) => println!("Bond created successfully!"),
        Err(e) => println!("Failed to create bond: {}", e),
    }
}

fn transfer_bond(blockchain: &mut Blockchain) {
    let bond_id = get_user_input("Enter bond ID: ");
    let new_owner = get_user_input("Enter new owner ID: ");

    match blockchain.transfer_bond(&bond_id, &new_owner) {
        Ok(_) => println!("Bond transferred successfully!"),
        Err(e) => println!("Failed to transfer bond: {}", e),
    }
}

fn view_bond(blockchain: &Blockchain) {
    let bond_id = get_user_input("Enter bond ID: ");
    match blockchain.get_bond(&bond_id) {
        Some(bond) => println!("{:?}", bond),
        None => println!("Bond not found"),
    }
}
