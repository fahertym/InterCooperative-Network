# icn/blockchain/chain.py

import time
from .block import Block
from .transaction import Transaction
from ..consensus.pocos import PoCoS
from ..identity.did import DIDManager
from ..dao.governance import CooperativeManager
from ..federation.federation import FederationManager
from ..smartcontracts.language import SmartContractLanguage
from ..smartcontracts.vm import SmartContractVM

class Blockchain:
    def __init__(self):
        self.chain = [self.create_genesis_block()]
        self.difficulty = 4
        self.pending_transactions = []
        self.mining_reward = 10
        self.consensus = PoCoS(self)
        self.did_manager = DIDManager()
        self.cooperative_manager = CooperativeManager(self)
        self.federation_manager = FederationManager()
        self.contracts = {}
        self.contract_states = {}
        self.vm = SmartContractVM(self)
        self.prices = {}
        self.balances = {}
        self.votes = {}

    def create_genesis_block(self):
        return Block(0, [], int(time.time()), "0")

    def get_latest_block(self):
        return self.chain[-1]

    def add_block(self, block):
        if self.is_block_valid(block):
            self.chain.append(block)
            return True
        return False

    def is_block_valid(self, block):
        if block.index != len(self.chain):
            return False
        if block.previous_hash != self.get_latest_block().hash:
            return False
        if block.hash != block.calculate_hash():
            return False
        if block.hash[:self.difficulty] != "0" * self.difficulty:
            return False
        return True

    def set_price(self, item, price):
        self.prices[item] = price

    def get_price(self, item):
        return self.prices.get(item, 0)

    def trade(self, buyer, seller, item, quantity):
        price = self.get_price(item)
        total_cost = price * quantity
        if self.balances.get(buyer, 0) >= total_cost:
            self.balances[buyer] = self.balances.get(buyer, 0) - total_cost
            self.balances[seller] = self.balances.get(seller, 0) + total_cost
            return True
        return False

    def get_balance(self, account):
        return self.balances.get(account, 0)

    def transfer(self, from_account, to, amount):
        if self.balances.get(from_account, 0) >= amount:
            self.balances[from_account] = self.balances.get(from_account, 0) - amount
            self.balances[to] = self.balances.get(to, 0) + amount
            return True
        return False

    def vote(self, voter, proposal, vote):
        if proposal not in self.votes:
            self.votes[proposal] = {}
        self.votes[proposal][voter] = vote

    def get_vote_result(self, proposal):
        if proposal not in self.votes:
            return None
        votes = self.votes[proposal]
        yes_votes = sum(1 for v in votes.values() if v)
        return yes_votes / len(votes) if votes else 0

    def create_transaction(self, sender, recipient, amount_or_message):
        if isinstance(amount_or_message, (int, float)):
            transaction = Transaction(sender, recipient, amount_or_message)
        else:
            transaction = Transaction(sender, recipient, 0, message=amount_or_message)
        transaction.sign_transaction(self.did_manager)
        return transaction

    def add_transaction(self, transaction):
        if not transaction.sender_did or not transaction.recipient_did:
            raise ValueError("Transaction must include sender and recipient DIDs")
        
        if not transaction.is_valid(self.did_manager):
            raise ValueError(f"Cannot add invalid transaction to chain: {transaction}")
        
        self.pending_transactions.append(transaction)

    def mine_pending_transactions(self, miner_did):
        if not self.consensus.is_validator(miner_did):
            raise ValueError("Miner is not a valid validator")

        block = Block(len(self.chain), self.pending_transactions, int(time.time()), self.get_latest_block().hash)
        block.mine_block(self.difficulty)

        if self.add_block(block):
            self.pending_transactions = [
                Transaction("NETWORK", miner_did, self.mining_reward, is_mining_reward=True)
            ]
            return True
        return False

    def create_did(self):
        return self.did_manager.create_did()

    def create_cooperative(self, name):
        return self.cooperative_manager.create_cooperative(name)

    def get_cooperative(self, name):
        return self.cooperative_manager.get_cooperative(name)

    def create_federation(self, name, cooperative_names):
        cooperatives = [self.get_cooperative(coop_name) for coop_name in cooperative_names]
        if all(cooperatives):
            return self.federation_manager.create_federation(name, cooperatives)
        return None

    def get_federation(self, name):
        return self.federation_manager.get_federation(name)

    def list_federations(self):
        return self.federation_manager.list_federations()

    def add_cooperative_to_federation(self, federation_name, cooperative_name):
        cooperative = self.get_cooperative(cooperative_name)
        if cooperative:
            return self.federation_manager.add_cooperative_to_federation(federation_name, cooperative)
        return False

    def remove_cooperative_from_federation(self, federation_name, cooperative_name):
        cooperative = self.get_cooperative(cooperative_name)
        if cooperative:
            return self.federation_manager.remove_cooperative_from_federation(federation_name, cooperative)
        return False

    def deploy_contract(self, code):
        contract = SmartContractLanguage.parse(code)
        contract_id = f"contract_{len(self.contracts)}"
        self.contracts[contract_id] = contract
        self.contract_states[contract_id] = {}
        return contract_id

    def execute_contract(self, contract_id, *args):
        if contract_id not in self.contracts:
            return False
        contract = self.contracts[contract_id]
        self.vm.execute(contract)
        return True

    @classmethod
    def is_chain_valid(cls, chain):
        for i in range(1, len(chain)):
            current_block = chain[i]
            previous_block = chain[i-1]

            if current_block.hash != current_block.calculate_hash():
                return False

            if current_block.previous_hash != previous_block.hash:
                return False

        return True