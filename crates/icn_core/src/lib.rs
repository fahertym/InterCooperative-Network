use icn_common::{Error, Result, CurrencyType};
use icn_blockchain::{Blockchain, Transaction};
use icn_vm::smart_contract::{AssetTokenContract, BondContract};
use chrono::Utc;
use std::io;

pub struct IcnNode {
    pub blockchain: Blockchain,
    pub asset_contract: AssetTokenContract,
    pub bond_contract: BondContract,
}

impl IcnNode {
    pub fn new() -> Self {
        IcnNode {
            blockchain: Blockchain::new(),
            asset_contract: AssetTokenContract::new(),
            bond_contract: BondContract::new(),
        }
    }

    pub fn process_transaction(&mut self, transaction: Transaction) -> Result<()> {
        self.blockchain.add_transaction(transaction)?;
        self.blockchain.create_block("Miner".to_string())?;
        Ok(())
    }

    pub fn get_balance(&self, address: &str) -> f64 {
        self.blockchain.get_balance(address)
    }
}

pub fn main() {
    let mut node = IcnNode::new();

    loop {
        println!("Enter a command (balance, transaction, exit):");
        let mut input = String::new();
        io::stdin().read_line(&mut input).expect("Failed to read line");

        match input.trim() {
            "balance" => {
                println!("Enter address:");
                let mut address = String::new();
                io::stdin().read_line(&mut address).expect("Failed to read line");
                let balance = node.get_balance(address.trim());
                println!("Balance: {}", balance);
            }
            "transaction" => {
                println!("Enter from address:");
                let mut from = String::new();
                io::stdin().read_line(&mut from).expect("Failed to read line");
                println!("Enter to address:");
                let mut to = String::new();
                io::stdin().read_line(&mut to).expect("Failed to read line");
                println!("Enter amount:");
                let mut amount = String::new();
                io::stdin().read_line(&mut amount).expect("Failed to read line");

                let transaction = Transaction::new(
                    from.trim().to_string(),
                    to.trim().to_string(),
                    amount.trim().parse().expect("Invalid amount"),
                    CurrencyType::BasicNeeds,
                    1000,
                );

                node.process_transaction(transaction).expect("Transaction failed");
                println!("Transaction processed");
            }
            "exit" => break,
            _ => println!("Unknown command"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use icn_blockchain::{Transaction};

    #[test]
    fn test_process_transaction() {
        let mut node = IcnNode::new();
        let transaction = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        assert!(node.process_transaction(transaction).is_ok());
        assert_eq!(node.blockchain.pending_transactions.len(), 0);
    }

    #[test]
    fn test_get_balance() {
        let mut node = IcnNode::new();
        let transaction1 = Transaction::new(
            "Alice".to_string(),
            "Bob".to_string(),
            100.0,
            CurrencyType::BasicNeeds,
            1000,
        );
        let transaction2 = Transaction::new(
            "Bob".to_string(),
            "Alice".to_string(),
            50.0,
            CurrencyType::BasicNeeds,
            1000,
        );

        node.process_transaction(transaction1).unwrap();
        node.process_transaction(transaction2).unwrap();

        assert_eq!(node.get_balance("Alice"), -50.0);
        assert_eq!(node.get_balance("Bob"), 50.0);
    }
}
