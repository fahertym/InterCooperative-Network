from icn.blockchain.chain import Blockchain
from icn.blockchain.transaction import Transaction

def test_blockchain():
    # Create a new blockchain
    bc = Blockchain()

    # Create some DIDs
    alice_did = bc.create_did()
    bob_did = bc.create_did()
    charlie_did = bc.create_did()

    # Add validators
    bc.add_validator(alice_did, 200)
    bc.add_validator(bob_did, 150)

    # Create and add transactions
    tx1 = Transaction(alice_did, bob_did, 50)
    tx1.sign_transaction(bc.did_manager)
    bc.add_transaction(tx1)

    tx2 = Transaction(bob_did, charlie_did, 30)
    tx2.sign_transaction(bc.did_manager)
    bc.add_transaction(tx2)

    # Mine a block
    bc.mine_pending_transactions(alice_did)

    # Check balances
    print(f"Alice's balance: {bc.get_balance(alice_did)}")
    print(f"Bob's balance: {bc.get_balance(bob_did)}")
    print(f"Charlie's balance: {bc.get_balance(charlie_did)}")

    # Verify the blockchain
    print(f"Is blockchain valid? {bc.is_chain_valid()}")

    # Try to add an invalid transaction
    invalid_tx = Transaction(alice_did, bob_did, 1000)
    try:
        bc.add_transaction(invalid_tx)
    except ValueError as e:
        print(f"Invalid transaction caught: {str(e)}")

    # Mine another block
    bc.mine_pending_transactions(bob_did)

    # Final balance check
    print(f"Alice's final balance: {bc.get_balance(alice_did)}")
    print(f"Bob's final balance: {bc.get_balance(bob_did)}")
    print(f"Charlie's final balance: {bc.get_balance(charlie_did)}")

    # Test DAO functionality
    dao = bc.create_dao("TestDAO")
    dao.add_member(alice_did)
    dao.add_member(bob_did)

    proposal_id = dao.create_proposal(alice_did, "Test Proposal", 3600)
    print(f"Proposal created with ID: {proposal_id}")

    dao.vote_on_proposal(proposal_id, alice_did, True)
    dao.vote_on_proposal(proposal_id, bob_did, True)

    # Fast-forward time (in a real scenario, you'd wait for the voting period to end)
    dao.proposals[proposal_id].start_time -= 3601

    result = dao.execute_proposal(proposal_id)
    print(f"Proposal execution result: {result}")

if __name__ == "__main__":
    test_blockchain()