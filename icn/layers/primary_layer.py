# icn/layers/primary_layer.py

from .base_layer import BaseLayer
from ..blockchain.block import Block
import time

class PrimaryLayer(BaseLayer):
    def __init__(self, blockchain):
        super().__init__(blockchain)
        self.transactions = []
        self.blocks = []

    def initialize(self):
        # Initialize the genesis block
        genesis_block = Block(0, [], int(time.time()), "0")
        self.blocks.append(genesis_block)

    def process_event(self, event):
        if event['type'] == 'transaction':
            return self.add_transaction(event['data'])
        elif event['type'] == 'block':
            return self.add_block(event['data'])
        return False

    def add_transaction(self, transaction):
        # Validate transaction
        if self.validate_transaction(transaction):
            self.transactions.append(transaction)
            return True
        return False

    def validate_transaction(self, transaction):
        # Add transaction validation logic here
        return True

    def add_block(self, block):
        # Validate block
        if self.validate_block(block):
            self.blocks.append(block)
            self.transactions = []  # Clear pending transactions
            return True
        return False

    def validate_block(self, block):
        # Add block validation logic here
        return True

    def create_block(self, miner_address):
        # Create a new block from pending transactions
        new_block = Block(
            index=len(self.blocks),
            transactions=self.transactions,
            timestamp=int(time.time()),
            previous_hash=self.blocks[-1].hash if self.blocks else "0",
            miner=miner_address
        )
        new_block.mine_block(self.blockchain.difficulty)
        return new_block

    def get_balance(self, address):
        # Calculate balance for an address
        balance = 0
        for block in self.blocks:
            for tx in block.transactions:
                if tx['recipient'] == address:
                    balance += tx['amount']
                if tx['sender'] == address:
                    balance -= tx['amount']
        return balance