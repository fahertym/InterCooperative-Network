import unittest
from icn.blockchain.chain import Blockchain
from icn.dao.governance import CooperativeManager, Cooperative

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

if __name__ == '__main__':
    unittest.main()