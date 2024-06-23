import time
import json
import hashlib

class Transaction:
    def __init__(self, sender, recipient, amount):
        self.sender = sender
        self.recipient = recipient
        self.amount = amount
        self.timestamp = time.time()
        self.signature = None

    def to_dict(self):
        return {
            "sender": self.sender,
            "recipient": self.recipient,
            "amount": self.amount,
            "timestamp": self.timestamp,
            "signature": self.signature
        }

    def calculate_hash(self):
        transaction_string = json.dumps(self.to_dict(), sort_keys=True)
        return hashlib.sha256(transaction_string.encode()).hexdigest()

    def sign_transaction(self, private_key):
        # This is a placeholder. We'll implement actual signing later with the identity module
        self.signature = "signed"

    def is_valid(self):
        # This is a placeholder. We'll implement actual validation later
        return self.signature is not None