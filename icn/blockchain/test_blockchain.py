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
    invalid_tx.sign_transaction(bc.did_manager)
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

if __name__ == "__main__":
    test_blockchain()