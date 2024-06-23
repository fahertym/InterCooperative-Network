import json
import os
from ..blockchain.block import Block

class FileStorage:
    def __init__(self, directory):
        self.directory = directory
        os.makedirs(directory, exist_ok=True)

    def save_blockchain(self, blockchain):
        chain_data = [block.to_dict() for block in blockchain.chain]
        with open(os.path.join(self.directory, 'blockchain.json'), 'w') as f:
            json.dump(chain_data, f)

    def load_blockchain(self, blockchain):
        try:
            with open(os.path.join(self.directory, 'blockchain.json'), 'r') as f:
                chain_data = json.load(f)
            blockchain.chain = [Block.from_dict(block_data) for block_data in chain_data]
        except FileNotFoundError:
            pass  # If the file doesn't exist, we'll use the existing genesis block

    def save_peers(self, peers):
        with open(os.path.join(self.directory, 'peers.json'), 'w') as f:
            json.dump(list(peers), f)

    def load_peers(self):
        try:
            with open(os.path.join(self.directory, 'peers.json'), 'r') as f:
                return set(json.load(f))
        except FileNotFoundError:
            return set()