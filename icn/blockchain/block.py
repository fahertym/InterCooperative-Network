import time
import json
import hashlib

class Block:
    def __init__(self, index, transactions, timestamp, previous_hash):
        self.index = index
        self.transactions = transactions
        self.timestamp = timestamp
        self.previous_hash = previous_hash
        self.nonce = 0
        self.hash = None
        self.calculate_hash()

    def calculate_hash(self):
        block_dict = {
            "index": self.index,
            "transactions": [tx.to_dict() for tx in self.transactions],
            "timestamp": self.timestamp,
            "previous_hash": self.previous_hash,
            "nonce": self.nonce
        }
        block_string = json.dumps(block_dict, sort_keys=True)
        self.hash = hashlib.sha256(block_string.encode()).hexdigest()
        return self.hash

    def mine_block(self, difficulty):
        target = "0" * difficulty
        while self.hash[:difficulty] != target:
            self.nonce += 1
            self.calculate_hash()
        print(f"Block mined: {self.hash}")

    def to_dict(self):
        return {
            "index": self.index,
            "transactions": [tx.to_dict() for tx in self.transactions],
            "timestamp": self.timestamp,
            "previous_hash": self.previous_hash,
            "nonce": self.nonce,
            "hash": self.hash
        }

    @classmethod
    def from_dict(cls, block_dict):
        from ..blockchain.transaction import Transaction  # Import here to avoid circular import
        block = cls(
            block_dict['index'],
            [Transaction.from_dict(tx) for tx in block_dict['transactions']],
            block_dict['timestamp'],
            block_dict['previous_hash']
        )
        block.nonce = block_dict['nonce']
        block.hash = block_dict['hash']
        return block