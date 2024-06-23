import uuid
import json
from cryptography.hazmat.primitives.asymmetric import rsa, padding
from cryptography.hazmat.primitives import hashes, serialization
from cryptography.exceptions import InvalidSignature

class DID:
    def __init__(self):
        self.id = str(uuid.uuid4())
        self.private_key = rsa.generate_private_key(
            public_exponent=65537,
            key_size=2048
        )
        self.public_key = self.private_key.public_key()

    def sign(self, message):
        signature = self.private_key.sign(
            message.encode(),
            padding.PSS(
                mgf=padding.MGF1(hashes.SHA256()),
                salt_length=padding.PSS.MAX_LENGTH
            ),
            hashes.SHA256()
        )
        return signature

    def verify(self, message, signature):
        try:
            self.public_key.verify(
                signature,
                message.encode(),
                padding.PSS(
                    mgf=padding.MGF1(hashes.SHA256()),
                    salt_length=padding.PSS.MAX_LENGTH
                ),
                hashes.SHA256()
            )
            return True
        except InvalidSignature:
            print(f"Invalid signature for message: {message}")
            return False
        except Exception as e:
            print(f"Signature verification failed: {str(e)}")
            return False

    def to_dict(self):
        return {
            "id": self.id,
            "public_key": self.public_key.public_bytes(
                encoding=serialization.Encoding.PEM,
                format=serialization.PublicFormat.SubjectPublicKeyInfo
            ).decode()
        }

class DIDManager:
    def __init__(self):
        self.dids = {}

    def create_did(self):
        did = DID()
        self.dids[did.id] = did
        return did.id

    def get_did(self, did_id):
        return self.dids.get(did_id)

    def sign_message(self, did_id, message):
        did = self.get_did(did_id)
        if did:
            return did.sign(message)
        return None

    def verify_message(self, did_id, message, signature):
        did = self.get_did(did_id)
        if did:
            return did.verify(message, signature)
        return False

    def get_public_key(self, did_id):
        did = self.get_did(did_id)
        if did:
            return did.public_key
        return None