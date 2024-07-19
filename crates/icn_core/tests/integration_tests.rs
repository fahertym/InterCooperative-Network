use icn_node::blockchain::Blockchain;
use icn_node::smart_contract::{parse_contract, ContractType};

#[test]
fn test_smart_contract_integration() {
    let mut blockchain = Blockchain::new();

    // Deploy an asset transfer contract
    let contract_input = "Asset Transfer
Creator: Alice
From: Alice
To: Bob
Asset: ICN_TOKEN
Amount: 100.0";

    let contract = parse_contract(contract_input).unwrap();
    blockchain.deploy_smart_contract(contract).unwrap();

    // Deploy a proposal contract
    let proposal_input = "Proposal
Creator: Charlie
Title: New Community Project
Description: Implement a recycling program
Voting Period: 604800
Option 1: Approve
Option 2: Reject
Quorum: 0.5";

    let proposal_contract = parse_contract(proposal_input).unwrap();
    blockchain.deploy_smart_contract(proposal_contract).unwrap();

    // Execute smart contracts
    blockchain.execute_smart_contracts().unwrap();

    // Verify blockchain state
    assert_eq!(blockchain.chain.len(), 2); // Genesis block + 1 block with contracts
    assert_eq!(blockchain.chain.last().unwrap().smart_contracts.len(), 2);

    // Verify contract execution results
    let env = &blockchain.execution_environment;
    assert_eq!(env.balances.get("Bob").unwrap().get("ICN_TOKEN").unwrap(), &100.0);
    assert!(env.votes.contains_key(&blockchain.chain.last().unwrap().smart_contracts[1].id));
}