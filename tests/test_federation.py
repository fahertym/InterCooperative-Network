# tests/test_federation.py

import unittest
from icn.blockchain.chain import Blockchain
from icn.dao.governance import CooperativeManager
from icn.federation.federation import FederationManager

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

if __name__ == '__main__':
    unittest.main()