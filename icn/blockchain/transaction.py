import time
import json
import hashlib

class Transaction:
    def __init__(self, sender_did, recipient_did, amount, is_mining_reward=False):
        self.sender_did = sender_did
        self.recipient_did = recipient_did
        self.amount = amount
        self.timestamp = time.time()
        self.signature = None
        self.is_mining_reward = is_mining_reward

    def sign(self, signature):
        if not self.is_mining_reward:
            self.signature = signature

    def to_dict(self):
        return {
            "sender_did": self.sender_did,
            "recipient_did": self.recipient_did,
            "amount": self.amount,
            "timestamp": self.timestamp,
            "is_mining_reward": self.is_mining_reward
        }

    def calculate_hash(self):
        transaction_string = json.dumps(self.to_dict(), sort_keys=True)
        return hashlib.sha256(transaction_string.encode()).hexdigest()

    def is_valid(self, did_manager):
        if self.is_mining_reward:
            return self.sender_did == "NETWORK" and self.signature is None
        
        if not self.signature:
            print("Transaction is invalid: No signature")
            return False

        return did_manager.verify_message(self.sender_did, self.calculate_hash(), self.signature)

    def sign_transaction(self, did_manager):
        if not self.is_mining_reward:
            transaction_hash = self.calculate_hash()
            self.signature = did_manager.sign_message(self.sender_did, transaction_hash)
            print(f"Transaction signed. Hash: {transaction_hash}, Signature: {self.signature.hex()}")

    def __str__(self):
        return f"Transaction(sender: {self.sender_did}, recipient: {self.recipient_did}, amount: {self.amount}, is_mining_reward: {self.is_mining_reward})"