import random
import time

class PoCoS:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.validators = {}
        self.stake_threshold = 100
        self.cooperation_score_threshold = 50
        self.last_active_time = {}

    def add_validator(self, did, stake):
        if stake >= self.stake_threshold:
            self.validators[did] = {
                'stake': stake,
                'cooperation_score': 100,
                'successful_validations': 0,
                'total_validations': 0,
                'last_validation_time': time.time()
            }
            return True
        return False

    def remove_validator(self, did):
        if did in self.validators:
            del self.validators[did]
            return True
        return False

    def update_stake(self, did, stake):
        if did in self.validators:
            self.validators[did]['stake'] = stake
            return True
        return False

    def mine_block(self, block, miner_did):
        if self.is_validator(miner_did):
            block.mine_block(self.blockchain.difficulty)
            self.update_cooperation_score(miner_did, True)
            return True
        return False

    def is_validator(self, did):
        if did in self.validators:
            return (self.validators[did]['stake'] >= self.stake_threshold and
                    self.validators[did]['cooperation_score'] >= self.cooperation_score_threshold)
        return False

    def select_validator(self):
        eligible_validators = [v for v in self.validators if self.is_validator(v)]
        if not eligible_validators:
            return None
        
        total_stake = sum(self.validators[v]['stake'] for v in eligible_validators)
        selection = random.uniform(0, total_stake)
        
        current_stake = 0
        for validator in eligible_validators:
            current_stake += self.validators[validator]['stake']
            if current_stake > selection:
                return validator
        
        return None

    def update_cooperation_score(self, did, success):
        if did in self.validators:
            validator = self.validators[did]
            validator['total_validations'] += 1
            if success:
                validator['successful_validations'] += 1
            
            # Calculate cooperation score based on successful validations and activity
            success_ratio = validator['successful_validations'] / validator['total_validations']
            time_since_last_validation = time.time() - validator['last_validation_time']
            activity_factor = max(0, 1 - (time_since_last_validation / (24 * 60 * 60)))  # Decays over 24 hours
            
            validator['cooperation_score'] = (success_ratio * 70 + activity_factor * 30)
            validator['last_validation_time'] = time.time()

    def validate_block(self, block):
        # Implement block validation logic here
        return True  # Placeholder implementation