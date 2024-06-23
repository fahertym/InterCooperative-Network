import random
from ..blockchain.transaction import Transaction

class PoCoS:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.validators = {}
        self.stake_threshold = 100
        self.cooperation_score_threshold = 50

    def add_validator(self, address, stake):
        if stake >= self.stake_threshold:
            self.validators[address] = {
                'stake': stake,
                'cooperation_score': 100  # Initial perfect score
            }
            return True
        return False

    def remove_validator(self, address):
        if address in self.validators:
            del self.validators[address]
            return True
        return False

    def update_stake(self, address, stake):
        if address in self.validators:
            self.validators[address]['stake'] = stake
            return True
        return False

    def update_cooperation_score(self, address, score):
        if address in self.validators:
            self.validators[address]['cooperation_score'] = max(0, min(100, score))
            return True
        return False

    def is_validator(self, address):
        return address in self.validators and \
               self.validators[address]['stake'] >= self.stake_threshold and \
               self.validators[address]['cooperation_score'] >= self.cooperation_score_threshold

    def select_validator(self):
        eligible_validators = [v for v in self.validators if self.is_validator(v)]
        if not eligible_validators:
            return None
        
        # Weight by stake and cooperation score
        weights = [self.validators[v]['stake'] * self.validators[v]['cooperation_score'] for v in eligible_validators]
        return random.choices(eligible_validators, weights=weights)[0]

    def validate_block(self, block):
        # Simple validation for now. We'll expand this later.
        return block.hash.startswith('0' * self.blockchain.difficulty)

    def distribute_rewards(self, miner_address):
        if self.is_validator(miner_address):
            reward = Transaction("0", miner_address, self.blockchain.mining_reward)
            self.blockchain.add_transaction(reward)
            return True
        return False