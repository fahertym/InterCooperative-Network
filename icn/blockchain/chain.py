from .block import Block
from .transaction import Transaction
import time

class Blockchain:
    def __init__(self):
        self.chain = [self.create_genesis_block()]
        self.difficulty = 4
        self.pending_transactions = []
        self.mining_reward = 10

    def create_genesis_block(self):
        return Block(0, [], int(time.time()), "0")

    def get_latest_block(self):
        return self.chain[-1]

    def mine_pending_transactions(self, miner_reward_address):
        block = Block(len(self.chain), self.pending_transactions, int(time.time()), self.get_latest_block().hash)
        block.mine_block(self.difficulty)

        print("Block successfully mined!")
        self.chain.append(block)

        self.pending_transactions = [
            Transaction("0", miner_reward_address, self.mining_reward)
        ]

    def add_transaction(self, transaction):
        if not transaction.sender or not transaction.recipient:
            raise ValueError("Transaction must include sender and recipient")
        
        if not transaction.is_valid():
            raise ValueError("Cannot add invalid transaction to chain")
        
        self.pending_transactions.append(transaction)

    def get_balance(self, address):
        balance = 0
        for block in self.chain:
            for tx in block.transactions:
                if tx.sender == address:
                    balance -= tx.amount
                if tx.recipient == address:
                    balance += tx.amount
        return balance

    def is_chain_valid(self):
        for i in range(1, len(self.chain)):
            current_block = self.chain[i]
            previous_block = self.chain[i-1]

            if current_block.hash != current_block.calculate_hash():
                return False

            if current_block.previous_hash != previous_block.hash:
                return False

        return True