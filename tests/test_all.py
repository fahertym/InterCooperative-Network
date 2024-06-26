# tests/test_all.py

import unittest
from icn.blockchain.chain import Blockchain
from icn.blockchain.transaction import Transaction
from icn.dao.governance import CooperativeManager, Cooperative
from icn.federation.federation import FederationManager
from icn.consensus.pocos import PoCoS
from icn.identity.did import DIDManager
from icn.smartcontracts.language import SmartContractLanguage
from icn.smartcontracts.vm import SmartContractVM

class TestBlockchain(unittest.TestCase):
    def setUp(self):
        self.blockchain = Blockchain()

    def test_create_genesis_block(self):
        self.assertEqual(len(self.blockchain.chain), 1)
        self.assertEqual(self.blockchain.chain[0].index, 0)
        self.assertEqual(self.blockchain.chain[0].previous_hash, "0")

    def test_add_transaction(self):
        sender_did = self.blockchain.create_did()
        recipient_did = self.blockchain.create_did()
        tx = self.blockchain.create_transaction(sender_did, recipient_did, 10)
        self.blockchain.add_transaction(tx)
        self.assertEqual(len(self.blockchain.pending_transactions), 1)

    def test_mine_block(self):
        sender_did = self.blockchain.create_did()
        recipient_did = self.blockchain.create_did()
        tx = self.blockchain.create_transaction(sender_did, recipient_did, 10)
        self.blockchain.add_transaction(tx)
        
        miner_did = self.blockchain.create_did()
        self.blockchain.add_validator(miner_did, 100)
        
        self.assertTrue(self.blockchain.mine_pending_transactions(miner_did))
        self.assertEqual(len(self.blockchain.chain), 2)
        self.assertEqual(len(self.blockchain.pending_transactions), 1)  # Mining reward transaction

    def test_get_balance(self):
        sender_did = self.blockchain.create_did()
        recipient_did = self.blockchain.create_did()
        self.blockchain.add_validator(sender_did, 100)
        
        tx = self.blockchain.create_transaction(sender_did, recipient_did, 50)
        self.blockchain.add_transaction(tx)
        self.blockchain.mine_pending_transactions(sender_did)
        
        self.assertEqual(self.blockchain.get_balance(sender_did), 60)  # 100 (initial) - 50 (sent) + 10 (mining reward)
        self.assertEqual(self.blockchain.get_balance(recipient_did), 50)

    def test_create_did(self):
        did = self.blockchain.create_did()
        self.assertIsNotNone(did)
        self.assertIsInstance(did, str)

    def test_add_validator(self):
        did = self.blockchain.create_did()
        self.assertTrue(self.blockchain.add_validator(did, 100))
        self.assertTrue(self.blockchain.consensus.is_validator(did))

class TestCooperative(unittest.TestCase):
    def setUp(self):
        self.blockchain = Blockchain()
        self.coop_manager = CooperativeManager(self.blockchain)

    def test_create_cooperative(self):
        coop = self.coop_manager.create_cooperative("TestCoop")
        self.assertIsInstance(coop, Cooperative)
        self.assertEqual(coop.name, "TestCoop")
        self.assertEqual(len(self.coop_manager.cooperatives), 1)

    def test_add_member(self):
        coop = self.coop_manager.create_cooperative("TestCoop")
        member_did = self.blockchain.create_did()
        coop.add_member(member_did)
        self.assertIn(member_did, coop.members)

    def test_create_and_vote_proposal(self):
        coop = self.coop_manager.create_cooperative("TestCoop")
        member1 = self.blockchain.create_did()
        member2 = self.blockchain.create_did()
        coop.add_member(member1)
        coop.add_member(member2)

        proposal_id = coop.create_proposal(member1, "Test Proposal", "general", 3600)
        self.assertIsNotNone(proposal_id)

        self.assertTrue(coop.vote_on_proposal(proposal_id, member1, True))
        self.assertTrue(coop.vote_on_proposal(proposal_id, member2, True))

        proposal = coop.proposals[proposal_id]
        self.assertTrue(proposal.get_result())

    def test_execute_proposal(self):
        coop = self.coop_manager.create_cooperative("TestCoop")
        member1 = self.blockchain.create_did()
        member2 = self.blockchain.create_did()
        coop.add_member(member1)

        proposal_id = coop.create_proposal(member1, member2, "add_member", 0)
        coop.vote_on_proposal(proposal_id, member1, True)

        self.assertTrue(coop.execute_proposal(proposal_id))
        self.assertIn(member2, coop.members)

    def test_create_proposal_with_voting_strategy(self):
        coop = self.coop_manager.create_cooperative("TestCoop")
        member = self.blockchain.create_did()
        coop.add_member(member)
        proposal_id = coop.create_proposal(member, "Test Proposal", "general", 3600, "SIMPLE_MAJORITY", 0.5)
        self.assertIsNotNone(proposal_id)
        self.assertEqual(coop.proposals[proposal_id].voting_strategy, "SIMPLE_MAJORITY")

class TestFederation(unittest.TestCase):
    def setUp(self):
        self.blockchain = Blockchain()
        self.coop_manager = CooperativeManager(self.blockchain)
        self.federation_manager = FederationManager()

    def test_create_federation(self):
        coop1 = self.coop_manager.create_cooperative("Coop1")
        coop2 = self.coop_manager.create_cooperative("Coop2")
        federation = self.federation_manager.create_federation("TestFed", [coop1, coop2])
        self.assertIsNotNone(federation)
        self.assertEqual(federation.name, "TestFed")
        self.assertEqual(len(federation.member_cooperatives), 2)

    def test_add_cooperative_to_federation(self):
        coop1 = self.coop_manager.create_cooperative("Coop1")
        coop2 = self.coop_manager.create_cooperative("Coop2")
        federation = self.federation_manager.create_federation("TestFed", [coop1])
        self.assertTrue(self.federation_manager.add_cooperative_to_federation("TestFed", coop2))
        self.assertIn(coop2, federation.member_cooperatives)

    def test_create_inter_federation_agreement(self):
        coop1 = self.coop_manager.create_cooperative("Coop1")
        coop2 = self.coop_manager.create_cooperative("Coop2")
        coop3 = self.coop_manager.create_cooperative("Coop3")
        coop4 = self.coop_manager.create_cooperative("Coop4")
        
        fed1 = self.federation_manager.create_federation("Fed1", [coop1, coop2])
        fed2 = self.federation_manager.create_federation("Fed2", [coop3, coop4])
        
        agreement_id = self.federation_manager.create_inter_federation_agreement("Fed1", "Fed2", "Collaboration Agreement")
        self.assertIsNotNone(agreement_id)
        
        agreement = self.federation_manager.get_inter_federation_agreement(agreement_id)
        self.assertEqual(agreement['federation1'], fed1)
        self.assertEqual(agreement['federation2'], fed2)
        self.assertEqual(agreement['terms'], "Collaboration Agreement")

    def test_remove_cooperative_from_federation(self):
        coop1 = self.coop_manager.create_cooperative("Coop1")
        coop2 = self.coop_manager.create_cooperative("Coop2")
        federation = self.federation_manager.create_federation("TestFed", [coop1, coop2])
        self.assertTrue(self.federation_manager.remove_cooperative_from_federation("TestFed", coop2))
        self.assertNotIn(coop2, federation.member_cooperatives)

class TestConsensus(unittest.TestCase):
    def setUp(self):
        self.blockchain = Blockchain()
        self.consensus = self.blockchain.consensus

    def test_add_and_remove_validator(self):
        did = self.blockchain.create_did()
        self.assertTrue(self.consensus.add_validator(did, 100))
        self.assertTrue(self.consensus.is_validator(did))
        self.assertTrue(self.consensus.remove_validator(did))
        self.assertFalse(self.consensus.is_validator(did))

    def test_update_validator_stake(self):
        did = self.blockchain.create_did()
        self.consensus.add_validator(did, 100)
        self.assertTrue(self.consensus.update_stake(did, 200))
        self.assertEqual(self.consensus.validators[did]['stake'], 200)

class TestDIDManager(unittest.TestCase):
    def setUp(self):
        self.did_manager = DIDManager()

    def test_create_and_verify_did(self):
        did = self.did_manager.create_did()
        self.assertIsNotNone(did)
        self.assertIsInstance(did, str)
        
        message = "Test message"
        signature = self.did_manager.sign_message(did, message)
        self.assertTrue(self.did_manager.verify_message(did, message, signature))

class TestSmartContract(unittest.TestCase):
    def setUp(self):
        self.blockchain = Blockchain()
        self.vm = SmartContractVM(self.blockchain)

    def test_parse_and_execute_contract(self):
        contract_code = """
        PUSH 5
        PUSH 3
        ADD
        """
        instructions = SmartContractLanguage.parse(contract_code)
        self.vm.execute(instructions)
        self.assertEqual(self.vm.stack[-1], 8)

if __name__ == '__main__':
    unittest.main()