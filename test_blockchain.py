from icn.blockchain.chain import Blockchain
from icn.blockchain.transaction import Transaction

def test_blockchain():
    # Create a new blockchain
    bc = Blockchain()

    # Create some DIDs
    alice_did = bc.create_did()
    bob_did = bc.create_did()
    charlie_did = bc.create_did()

    print(f"Alice DID: {alice_did}")
    print(f"Bob DID: {bob_did}")
    print(f"Charlie DID: {charlie_did}")

    # Add validators
    bc.add_validator(alice_did, 200)
    bc.add_validator(bob_did, 150)
    bc.add_validator(charlie_did, 100)

    # Create and add transactions
    tx1 = Transaction(alice_did, bob_did, 50)
    tx1.sign_transaction(bc.did_manager)
    bc.add_transaction(tx1)

    tx2 = Transaction(bob_did, charlie_did, 30)
    tx2.sign_transaction(bc.did_manager)
    bc.add_transaction(tx2)

    # Mine blocks
    for i in range(5):
        miner = bc.consensus.select_validator()
        if bc.mine_pending_transactions(miner):
            print(f"Block {i+1} mined by {miner}")
        else:
            print(f"Failed to mine block {i+1}")

    # Check balances
    print(f"Alice's balance: {bc.get_balance(alice_did)}")
    print(f"Bob's balance: {bc.get_balance(bob_did)}")
    print(f"Charlie's balance: {bc.get_balance(charlie_did)}")

    # Verify the blockchain
    print(f"Is blockchain valid? {bc.is_chain_valid()}")

    # Test DAO functionality
    dao = bc.create_dao("TestDAO")
    dao.add_member(alice_did)
    dao.add_member(bob_did)
    dao.add_member(charlie_did)

    # Create different types of proposals
    add_member_proposal = dao.create_proposal(alice_did, "david_did", "add_member", 3600)
    remove_member_proposal = dao.create_proposal(bob_did, "charlie_did", "remove_member", 3600)
    transfer_proposal = dao.create_proposal(charlie_did, f"{bob_did},25", "transfer_funds", 3600)

    # Vote on proposals
    for proposal_id in [add_member_proposal, remove_member_proposal, transfer_proposal]:
        dao.vote_on_proposal(proposal_id, alice_did, True)
        dao.vote_on_proposal(proposal_id, bob_did, True)
        dao.vote_on_proposal(proposal_id, charlie_did, False)

    # Fast-forward time and execute proposals
    for proposal in dao.proposals.values():
        proposal.start_time -= 3601

    for proposal_id in [add_member_proposal, remove_member_proposal, transfer_proposal]:
        result = dao.execute_proposal(proposal_id)
        print(f"Proposal {proposal_id} execution result: {result}")

    # Check final state
    print(f"DAO members: {dao.members}")
    print(f"Alice's final balance: {bc.get_balance(alice_did)}")
    print(f"Bob's final balance: {bc.get_balance(bob_did)}")
    print(f"Charlie's final balance: {bc.get_balance(charlie_did)}")

if __name__ == "__main__":
    test_blockchain()