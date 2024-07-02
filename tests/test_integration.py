# tests/test_integration.py

from icn.blockchain.chain import Blockchain

def test_integrated_operation(blockchain):
    # Create identities
    alice_id = blockchain.create_identity({"name": "Alice"})
    bob_id = blockchain.create_identity({"name": "Bob"})

    # Add validators
    blockchain.add_validator(alice_id, 100)
    blockchain.add_validator(bob_id, 100)

    # Create a transaction
    tx = blockchain.create_transaction(alice_id, bob_id, 50)

    # Mine a block
    blockchain.mine_block(bob_id)

    # Check balances
    assert blockchain.get_balance(alice_id) == 50
    assert blockchain.get_balance(bob_id) == 150

    # Deploy a simple contract
    contract_code = "PUSH 5\nPUSH 3\nADD"
    contract_id = blockchain.deploy_contract(contract_code, alice_id)

    # Execute the contract
    result = blockchain.execute_contract(contract_id, "main", [], alice_id)
    assert result == 8

    # Create a governance proposal
    proposal_data = {
        "title": "Increase mining reward",
        "description": "Increase mining reward to 15 tokens",
        "type": "CHANGE_REWARD"
    }
    proposal_id = blockchain.create_proposal(proposal_data)

    # Vote on the proposal
    blockchain.vote_on_proposal(proposal_id, alice_id, True)
    blockchain.vote_on_proposal(proposal_id, bob_id, True)

    # Execute the proposal
    blockchain.execute_proposal(proposal_id)

    # Verify the change
    assert blockchain.mining_reward == 15

    print("All operations completed successfully!")

if __name__ == "__main__":
    blockchain = Blockchain()
    blockchain.initialize()
    test_integrated_operation(blockchain)