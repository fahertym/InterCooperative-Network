# icn/blockchain/chain.py

from ..layers.primary_layer import PrimaryLayer
from ..layers.identity_layer import IdentityLayer
from ..layers.data_layer import DataLayer
from ..layers.contract_layer import ContractLayer
from ..layers.governance_layer import GovernanceLayer
from ..consensus.pocos import PoCoS

class Blockchain:
    def __init__(self):
        self.primary_layer = PrimaryLayer(self)
        self.identity_layer = IdentityLayer(self)
        self.data_layer = DataLayer(self)
        self.contract_layer = ContractLayer(self)
        self.governance_layer = GovernanceLayer(self)
        
        self.consensus = PoCoS(self)
        self.difficulty = 4
        self.mining_reward = 10

    def initialize(self):
        self.primary_layer.initialize()
        self.identity_layer.initialize()
        self.data_layer.initialize()
        self.contract_layer.initialize()
        self.governance_layer.initialize()

    def process_event(self, event):
        layer = self.get_layer_for_event(event)
        return layer.process_event(event)

    def get_layer_for_event(self, event):
        event_type = event['type']
        if event_type in ['transaction', 'block']:
            return self.primary_layer
        elif event_type in ['create_identity', 'verify_identity']:
            return self.identity_layer
        elif event_type in ['store_data', 'retrieve_data']:
            return self.data_layer
        elif event_type in ['deploy_contract', 'execute_contract']:
            return self.contract_layer
        elif event_type in ['create_proposal', 'vote_proposal', 'execute_proposal']:
            return self.governance_layer
        else:
            raise ValueError(f"Unknown event type: {event_type}")

    def create_transaction(self, sender, recipient, amount, message=None):
        transaction = {
            'type': 'transaction',
            'data': {
                'sender': sender,
                'recipient': recipient,
                'amount': amount,
                'message': message
            }
        }
        return self.process_event(transaction)

    def mine_block(self, miner_address):
        if not self.consensus.is_validator(miner_address):
            raise ValueError("Miner is not a valid validator")
        
        block = self.primary_layer.create_block(miner_address)
        if self.primary_layer.add_block(block):
            reward_tx = {
                'type': 'transaction',
                'data': {
                    'sender': "NETWORK",
                    'recipient': miner_address,
                    'amount': self.mining_reward,
                    'message': "Mining Reward"
                }
            }
            self.process_event(reward_tx)
            return True
        return False

    def get_balance(self, address):
        return self.primary_layer.get_balance(address)

    def create_identity(self, identity_data):
        event = {
            'type': 'create_identity',
            'data': identity_data
        }
        return self.process_event(event)

    def verify_identity(self, verification_data):
        event = {
            'type': 'verify_identity',
            'data': verification_data
        }
        return self.process_event(event)

    def store_data(self, data, access_control):
        event = {
            'type': 'store_data',
            'data': {
                'content': data,
                'access_control': access_control
            }
        }
        return self.process_event(event)

    def retrieve_data(self, data_id, requester):
        event = {
            'type': 'retrieve_data',
            'data': {
                'data_id': data_id,
                'requester': requester
            }
        }
        return self.process_event(event)

    def deploy_contract(self, contract_code, deployer):
        event = {
            'type': 'deploy_contract',
            'data': {
                'code': contract_code,
                'deployer': deployer
            }
        }
        return self.process_event(event)

    def execute_contract(self, contract_id, function, params, caller):
        event = {
            'type': 'execute_contract',
            'data': {
                'contract_id': contract_id,
                'function': function,
                'params': params,
                'caller': caller
            }
        }
        return self.process_event(event)

    def create_proposal(self, proposal_data):
        event = {
            'type': 'create_proposal',
            'data': proposal_data
        }
        return self.process_event(event)

    def vote_on_proposal(self, proposal_id, voter, vote):
        event = {
            'type': 'vote_proposal',
            'data': {
                'proposal_id': proposal_id,
                'voter': voter,
                'vote': vote
            }
        }
        return self.process_event(event)

    def execute_proposal(self, proposal_id):
        event = {
            'type': 'execute_proposal',
            'data': {
                'proposal_id': proposal_id
            }
        }
        return self.process_event(event)

    def add_validator(self, address, stake):
        return self.consensus.add_validator(address, stake)

    def remove_validator(self, address):
        return self.consensus.remove_validator(address)

    def is_validator(self, address):
        return self.consensus.is_validator(address)

    def get_validators(self):
        return self.consensus.get_validators()

    def update_validator_stake(self, address, new_stake):
        return self.consensus.update_stake(address, new_stake)