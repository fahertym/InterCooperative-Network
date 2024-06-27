# icn/consensus/pocos.py

import random
import time
import math

class PoCoS:
    def __init__(self, blockchain):
        self.blockchain = blockchain
        self.validators = {}
        self.stake_threshold = 100
        self.cooperation_score_threshold = 50
        self.last_active_time = {}
        self.total_blocks_created = 0

    def add_validator(self, did, stake):
        if stake >= self.stake_threshold:
            self.validators[did] = {
                'stake': stake,
                'cooperation_score': 100,
                'blocks_created': 0,
                'last_active_time': time.time(),
                'total_uptime': 0
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

    def is_validator(self, did):
        if did in self.validators:
            return (self.validators[did]['stake'] >= self.stake_threshold and
                    self.validators[did]['cooperation_score'] >= self.cooperation_score_threshold)
        return False

    def select_validator(self):
        eligible_validators = [v for v in self.validators if self.is_validator(v)]
        if not eligible_validators:
            return None
        
        total_weight = sum(self.calculate_validator_weight(v) for v in eligible_validators)
        selection = random.uniform(0, total_weight)
        
        current_weight = 0
        for validator in eligible_validators:
            current_weight += self.calculate_validator_weight(validator)
            if current_weight > selection:
                return validator
        
        return None

    def calculate_validator_weight(self, did):
        validator = self.validators[did]
        stake_weight = validator['stake']
        cooperation_weight = validator['cooperation_score']
        activity_weight = self.calculate_activity_weight(did)
        return stake_weight * cooperation_weight * activity_weight

    def calculate_activity_weight(self, did):
        validator = self.validators[did]
        time_since_last_active = time.time() - validator['last_active_time']
        activity_decay = math.exp(-time_since_last_active / (24 * 60 * 60))  # Decay over 24 hours
        return 1 + (validator['total_uptime'] / (24 * 60 * 60)) * activity_decay

    def update_validator_metrics(self, did):
        if did in self.validators:
            validator = self.validators[did]
            current_time = time.time()
            
            validator['blocks_created'] += 1
            self.total_blocks_created += 1

            time_since_last_active = current_time - validator['last_active_time']
            validator['total_uptime'] += time_since_last_active
            validator['last_active_time'] = current_time

            expected_blocks = self.total_blocks_created * (validator['stake'] / sum(v['stake'] for v in self.validators.values()))
            cooperation_ratio = validator['blocks_created'] / max(expected_blocks, 1)
            validator['cooperation_score'] = min(100, max(0, int(cooperation_ratio * 100)))

    def validate_block(self, block):
        # Implement block validation logic here
        # This is a placeholder implementation
        return True

    def get_validator_info(self, did):
        if did in self.validators:
            validator = self.validators[did]
            return {
                'stake': validator['stake'],
                'cooperation_score': validator['cooperation_score'],
                'blocks_created': validator['blocks_created'],
                'total_uptime': validator['total_uptime']
            }
        return None