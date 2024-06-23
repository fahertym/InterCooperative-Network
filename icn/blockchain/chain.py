from .block import Block
from .transaction import Transaction
from ..consensus.pocos import PoCoS
from ..identity.did import DIDManager
from ..dao.governance import DAOManager
import time

class Blockchain:
    def __init__(self):
        self.chain = [self.create_genesis_block()]
        self.difficulty = 4
        self.pending_transactions = []
        self.mining_reward = 10
        self.consensus = PoCoS(self)
        self.did_manager = DIDManager()
        self.dao_manager = DAOManager(self)

    def create_genesis_block(self):
        return Block(0, [], int(time.time()), "0")

    def get_latest_block(self):
        return self.chain[-1]

    def mine_pending_transactions(self, miner_did):
        if not self.consensus.is_validator(miner_did):
            print("Miner is not a valid validator")
            return False

        # Add mining reward transaction
        reward_tx = Transaction("0", miner_did, self.mining_reward)
        self.pending_transactions.append(reward_tx)

        block = Block(len(self.chain), self.pending_transactions, int(time.time()), self.get_latest_block().hash)
        block.mine_block(self.difficulty)

        if self.consensus.validate_block(block):
            print("Block successfully mined and validated!")
            self.chain.append(block)
            self.pending_transactions = []
            return True
        else:
            print("Block validation failed")
            return False

    def add_transaction(self, transaction):
        if not transaction.sender_did or not transaction.recipient_did:
            raise ValueError("Transaction must include sender and recipient DIDs")
        
        if not transaction.is_valid(self.did_manager):
            raise ValueError(f"Cannot add invalid transaction to chain: {transaction.to_dict()}")
        
        self.pending_transactions.append(transaction)

    def get_balance(self, did):
        balance = 0
        for block in self.chain:
            for tx in block.transactions:
                if tx.sender_did == did:
                    balance -= tx.amount
                if tx.recipient_did == did:
                    balance += tx.amount
        return balance

    def is_chain_valid(self):
        for i in range(1, len(self.chain)):
            current_block = self.chain[i]
            previous_block = self.chain[i-1]

            if current_block.hash != current_block.calculate_hash():
                print(f"Invalid hash for block {i}")
                return False

            if current_block.previous_hash != previous_block.hash:
                print(f"Invalid previous hash for block {i}")
                return False

            if not self.consensus.validate_block(current_block):
                print(f"Consensus validation failed for block {i}")
                return False

            for transaction in current_block.transactions:
                if not transaction.is_valid(self.did_manager):
                    print(f"Invalid transaction in block {i}")
                    return False

        return True

    def add_validator(self, did, stake):
        return self.consensus.add_validator(did, stake)

    def remove_validator(self, did):
        return self.consensus.remove_validator(did)

    def update_validator_stake(self, did, stake):
        return self.consensus.update_stake(did, stake)

    def update_validator_cooperation_score(self, did, score):
        return self.consensus.update_cooperation_score(did, score)

    def create_did(self):
        return self.did_manager.create_did()

    def create_dao(self, name):
        return self.dao_manager.create_dao(name)

    def get_dao(self, name):
        return self.dao_manager.get_dao(name)