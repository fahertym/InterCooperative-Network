# icn/identity/did.py

import uuid
from cryptography.hazmat.primitives import hashes
from cryptography.hazmat.primitives.asymmetric import rsa, padding

class DIDManager:
    def __init__(self):
        self.dids = {}

    def create_did(self):
        did = str(uuid.uuid4())
        private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048
        )
        public_key = private_key.public_key()
        self.dids[did] = {
            'private_key': private_key,
            'public_key': public_key
        }
        return did

    def sign_message(self, did, message):
        if did not in self.dids:
            raise ValueError("DID not found")
        private_key = self.dids[did]['private_key']
        signature = private_key.sign(
            message.encode(),
            padding.PSS(
                mgf=padding.MGF1(hashes.SHA256()),
                salt_length=padding.PSS.MAX_LENGTH
            ),
            hashes.SHA256()
        )
        return signature

    def verify_message(self, did, message, signature):
        if did not in self.dids:
            raise ValueError("DID not found")
        public_key = self.dids[did]['public_key']
        try:
            public_key.verify(
                signature,
                message.encode(),
                padding.PSS(
                    mgf=padding.MGF1(hashes.SHA256()),
                    salt_length=padding.PSS.MAX_LENGTH
                ),
                hashes.SHA256()
            )
            return True
        except:
            return False