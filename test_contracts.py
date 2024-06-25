# tests/test_contracts.py

from icn.blockchain.chain import Blockchain
from icn.blockchain.contract import SmartContract, SmartContractParser

def test_smart_contracts():
    blockchain = Blockchain()

    # Test a simple transfer contract
    transfer_contract = """
    transfer alice to bob amount 50
    transfer bob to charlie amount 30
    """
    contract_id = blockchain.deploy_contract(transfer_contract)
    assert contract_id is not None, "Failed to deploy transfer contract"

    # Test contract execution
    result = blockchain.execute_contract(contract_id)
    assert result, "Failed to execute transfer contract"

    # Test a voting contract
    voting_contract = """
    vote alice proposal_1
    vote bob proposal_1
    vote charlie proposal_2
    """
    contract_id = blockchain.deploy_contract(voting_contract)
    assert contract_id is not None, "Failed to deploy voting contract"

    # Test contract execution
    result = blockchain.execute_contract(contract_id)
    assert result, "Failed to execute voting contract"

    print("All smart contract tests passed!")

if __name__ == "__main__":
    test_smart_contracts()