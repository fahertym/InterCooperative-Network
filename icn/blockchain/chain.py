from .block import Block
from .transaction import Transaction
from ..consensus.pocos import PoCoS
import time

class Blockchain:
    def __init__(self):
        self.chain = [self.create_genesis_block()]
        self.difficulty = 4
        self.pending_transactions = []
        self.mining_reward = 10
        self.consensus = PoCoS(self)

    def create_genesis_block(self):
        return Block(0, [], int(time.time()), "0")

    def get_latest_block(self):
        return self.chain[-1]

    def mine_pending_transactions(self, miner_address):
        if not self.consensus.is_validator(miner_address):
            print("Miner is not a valid validator")
            return False

        block = Block(len(self.chain), self.pending_transactions, int(time.time()), self.get_latest_block().hash)
        block.mine_block(self.difficulty)

        if self.consensus.validate_block(block):
            print("Block successfully mined and validated!")
            self.chain.append(block)
            self.consensus.distribute_rewards(miner_address)
            self.pending_transactions = []
            return True
        else:
            print("Block validation failed")
            return False

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

            if not self.consensus.validate_block(current_block):
                return False

        return True

    def add_validator(self, address, stake):
        return self.consensus.add_validator(address, stake)

    def remove_validator(self, address):
        return self.consensus.remove_validator(address)

    def update_validator_stake(self, address, stake):
        return self.consensus.update_stake(address, stake)

    def update_validator_cooperation_score(self, address, score):
        return self.consensus.update_cooperation_score(address, score)