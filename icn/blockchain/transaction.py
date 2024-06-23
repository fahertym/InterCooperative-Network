import time
import json
import hashlib

class Transaction:
    def __init__(self, sender_did, recipient_did, amount):
        self.sender_did = sender_did
        self.recipient_did = recipient_did
        self.amount = amount
        self.timestamp = time.time()
        self.signature = None

    def to_dict(self):
        return {
            "sender_did": self.sender_did,
            "recipient_did": self.recipient_did,
            "amount": self.amount,
            "timestamp": self.timestamp,
            "signature": self.signature.hex() if self.signature else None
        }

    def calculate_hash(self):
        # Create a copy of the transaction data without the signature
        transaction_data = {
            "sender_did": self.sender_did,
            "recipient_did": self.recipient_did,
            "amount": self.amount,
            "timestamp": self.timestamp
        }
        transaction_string = json.dumps(transaction_data, sort_keys=True)
        return hashlib.sha256(transaction_string.encode()).hexdigest()

    def sign_transaction(self, did_manager):
        transaction_hash = self.calculate_hash()
        self.signature = did_manager.sign_message(self.sender_did, transaction_hash)
        print(f"Transaction signed. Hash: {transaction_hash}, Signature: {self.signature.hex()}")

    def is_valid(self, did_manager):
        if self.sender_did == "0":  # Mining reward transaction
            return True
        
        if not self.signature:
            print("Transaction is invalid: No signature")
            return False

        transaction_hash = self.calculate_hash()
        print(f"Verifying transaction. Hash: {transaction_hash}, Signature: {self.signature.hex()}")
        if not did_manager.verify_message(self.sender_did, transaction_hash, self.signature):
            print("Transaction is invalid: Signature verification failed")
            return False

        return True