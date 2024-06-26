# tests/test_blockchain.py

import unittest
from icn.blockchain.chain import Blockchain
from icn.blockchain.transaction import Transaction

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

if __name__ == '__main__':
    unittest.main()