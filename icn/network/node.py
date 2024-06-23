import json
import asyncio
import aiohttp
from aiohttp import web
import random
import logging
import time

from ..blockchain.chain import Blockchain
from ..blockchain.block import Block
from ..blockchain.transaction import Transaction
from ..storage.file_storage import FileStorage

logging.basicConfig(level=logging.INFO, format='%(asctime)s - %(name)s - %(levelname)s - %(message)s')

class Node:
    def __init__(self, host, port, blockchain, storage, bootstrap_nodes=None):
        self.host = host
        self.port = port
        self.address = f"http://{host}:{port}"
        self.blockchain = blockchain
        self.storage = storage
        self.peers = self.storage.load_peers()
        self.bootstrap_nodes = bootstrap_nodes or []
        self.logger = logging.getLogger(f"Node:{port}")
        self.runner = None
        self.site = None
        self.is_running = False
        self.last_peer_discovery = 0
        self.last_blockchain_sync = 0
        self.last_peer_cleanup = 0

    async def start(self):
        app = web.Application()
        app.router.add_get('/blocks', self.get_blocks)
        app.router.add_post('/transactions/new', self.new_transaction)
        app.router.add_post('/nodes/register', self.register_nodes)
        app.router.add_get('/nodes/resolve', self.consensus)
        app.router.add_get('/nodes/peers', self.get_peers)
        app.router.add_get('/status', self.get_status)
        app.router.add_post('/mine', self.mine)
        app.router.add_post('/blocks/new', self.receive_new_block)

        self.runner = web.AppRunner(app)
        await self.runner.setup()
        self.site = web.TCPSite(self.runner, self.host, self.port)
        await self.site.start()
        self.logger.info(f"Node started on {self.address}")

        self.is_running = True
        asyncio.create_task(self.periodic_tasks())

        # Immediate peer discovery and blockchain sync
        await self.discover_peers()
        await self.sync_blockchain()

    async def stop(self):
        self.is_running = False
        if self.site:
            await self.site.stop()
        if self.runner:
            await self.runner.cleanup()
        self.logger.info(f"Node stopped on {self.address}")

    async def periodic_tasks(self):
        while self.is_running:
            await self.discover_peers()
            await self.sync_blockchain()
            await self.cleanup_peers()
            await asyncio.sleep(60)  # Run every minute

    async def discover_peers(self):
        self.last_peer_discovery = time.time()
        all_peers = set(self.peers) | set(self.bootstrap_nodes)
        all_peers.discard(self.address)  # Remove self from peers
        if not all_peers:
            return

        async with aiohttp.ClientSession() as session:
            for peer in all_peers:
                try:
                    async with session.get(f'{peer}/nodes/peers', timeout=5) as response:
                        if response.status == 200:
                            new_peers = await response.json()
                            self.peers.update(new_peers)
                            self.peers.discard(self.address)  # Ensure self is not in peers
                            self.logger.info(f"Discovered new peers: {new_peers}")
                    # Register ourselves with the peer
                    await self.register_with_node(session, peer)
                except (aiohttp.ClientError, asyncio.TimeoutError) as e:
                    self.logger.warning(f"Error discovering peers from {peer}: {str(e)}")
                    if peer in self.peers:
                        self.peers.remove(peer)
                        self.logger.warning(f"Removed unresponsive peer: {peer}")

        self.storage.save_peers(self.peers)

    async def register_with_node(self, session, node_address):
        try:
            async with session.post(f'{node_address}/nodes/register', json={'nodes': [self.address]}, timeout=5) as response:
                if response.status == 200:
                    self.logger.info(f"Successfully registered with node: {node_address}")
                else:
                    self.logger.warning(f"Failed to register with node: {node_address}. Status: {response.status}")
        except (aiohttp.ClientError, asyncio.TimeoutError) as e:
            self.logger.warning(f"Failed to connect to node: {node_address}. Error: {str(e)}")

    async def sync_blockchain(self):
        self.last_blockchain_sync = time.time()
        try:
            replaced = await self.resolve_conflicts()
            if replaced:
                self.logger.info("Blockchain was replaced with a longer one from the network")
            else:
                self.logger.info("Our blockchain is up to date")
        except Exception as e:
            self.logger.error(f"Error during blockchain synchronization: {str(e)}")

    async def cleanup_peers(self):
        current_time = time.time()
        if current_time - self.last_peer_cleanup < 300:  # Run every 5 minutes
            return

        self.last_peer_cleanup = current_time
        inactive_peers = set()

        async with aiohttp.ClientSession() as session:
            for peer in self.peers:
                try:
                    async with session.get(f'{peer}/status', timeout=5) as response:
                        if response.status != 200:
                            inactive_peers.add(peer)
                except (aiohttp.ClientError, asyncio.TimeoutError):
                    inactive_peers.add(peer)

        for peer in inactive_peers:
            self.peers.remove(peer)
            self.logger.info(f"Removed inactive peer: {peer}")

        self.storage.save_peers(self.peers)

    async def get_blocks(self, request):
        blocks = [block.to_dict() for block in self.blockchain.chain]
        return web.json_response(blocks)

    async def new_transaction(self, request):
        try:
            data = await request.json()
            required = ['sender_did', 'recipient_did', 'amount']
            if not all(k in data for k in required):
                return web.json_response({"message": "Missing values"}, status=400)

            transaction = await self.create_transaction(data['sender_did'], data['recipient_did'], data['amount'])
            return web.json_response({"message": "Transaction added to pool", "transaction": transaction.to_dict()})
        except Exception as e:
            self.logger.error(f"Error processing new transaction: {str(e)}")
            return web.json_response({"message": "Error processing transaction"}, status=500)

    async def register_nodes(self, request):
        try:
            data = await request.json()
            nodes = data.get('nodes')
            if nodes is None:
                return web.json_response({"message": "Error: Please supply a valid list of nodes"}, status=400)

            for node in nodes:
                if node != self.address:
                    self.peers.add(node)

            self.storage.save_peers(self.peers)

            return web.json_response({
                "message": "New nodes have been added",
                "total_nodes": list(self.peers)
            })
        except Exception as e:
            self.logger.error(f"Error registering nodes: {str(e)}")
            return web.json_response({"message": "Error registering nodes"}, status=500)

    async def consensus(self, request):
        try:
            replaced = await self.resolve_conflicts()

            if replaced:
                response = {
                    'message': 'Our chain was replaced',
                    'new_chain': [block.to_dict() for block in self.blockchain.chain]
                }
            else:
                response = {
                    'message': 'Our chain is authoritative',
                    'chain': [block.to_dict() for block in self.blockchain.chain]
                }

            return web.json_response(response)
        except Exception as e:
            self.logger.error(f"Error during consensus: {str(e)}")
            return web.json_response({"message": "Error during consensus"}, status=500)

    async def resolve_conflicts(self):
        neighbours = self.peers
        new_chain = None

        max_length = len(self.blockchain.chain)

        async with aiohttp.ClientSession() as session:
            for node in neighbours:
                try:
                    async with session.get(f'{node}/blocks', timeout=5) as response:
                        if response.status == 200:
                            data = await response.json()
                            length = len(data)
                            chain = [Block.from_dict(block) for block in data]

                            if length > max_length and self.blockchain.is_chain_valid(chain):
                                max_length = length
                                new_chain = chain
                except (aiohttp.ClientError, asyncio.TimeoutError) as e:
                    self.logger.warning(f"Failed to connect to peer: {node}. Error: {str(e)}")

        if new_chain:
            self.blockchain.chain = new_chain
            self.storage.save_blockchain(self.blockchain)
            return True

        return False

    async def get_peers(self, request):
        return web.json_response(list(self.peers))

    async def get_status(self, request):
        status = {
            "node_address": self.address,
            "peers": list(self.peers),
            "blockchain_length": len(self.blockchain.chain),
            "pending_transactions": len(self.blockchain.pending_transactions),
            "last_peer_discovery": self.last_peer_discovery,
            "last_blockchain_sync": self.last_blockchain_sync,
            "last_peer_cleanup": self.last_peer_cleanup
        }
        return web.json_response(status)

    async def create_transaction(self, sender_did, recipient_did, amount):
        transaction = self.blockchain.create_transaction(sender_did, recipient_did, amount)
        self.blockchain.add_transaction(transaction)
        await self.broadcast_transaction(transaction)
        return transaction

    async def broadcast_transaction(self, transaction):
        async with aiohttp.ClientSession() as session:
            for peer in self.peers:
                try:
                    async with session.post(f'{peer}/transactions/new', json=transaction.to_dict(), timeout=5) as response:
                        if response.status == 200:
                            self.logger.info(f"Transaction broadcast to {peer} successful")
                        else:
                            self.logger.warning(f"Failed to broadcast transaction to {peer}. Status: {response.status}")
                except (aiohttp.ClientError, asyncio.TimeoutError) as e:
                    self.logger.warning(f"Error broadcasting transaction to {peer}: {str(e)}")

    async def mine(self, request):
        try:
            data = await request.json()
            miner_did = data.get('miner_did')
            if not miner_did:
                return web.json_response({"message": "Miner DID is required"}, status=400)

            success = await self.mine_block(miner_did)
            if success:
                return web.json_response({"message": "New block mined and broadcast successfully"})
            else:
                return web.json_response({"message": "Failed to mine new block"}, status=500)
        except Exception as e:
            self.logger.error(f"Error mining new block: {str(e)}")
            return web.json_response({"message": "Error mining new block"}, status=500)

    async def mine_block(self, miner_did):
        success = self.blockchain.mine_pending_transactions(miner_did)
        if success:
            await self.broadcast_new_block()
        return success

    async def broadcast_new_block(self):
        latest_block = self.blockchain.get_latest_block()
        async with aiohttp.ClientSession() as session:
            for peer in self.peers:
                try:
                    async with session.post(f'{peer}/blocks/new', json=latest_block.to_dict(), timeout=5) as response:
                        if response.status == 200:
                            self.logger.info(f"New block broadcast to {peer} successful")
                        else:
                            self.logger.warning(f"Failed to broadcast new block to {peer}. Status: {response.status}")
                except (aiohttp.ClientError, asyncio.TimeoutError) as e:
                    self.logger.warning(f"Error broadcasting new block to {peer}: {str(e)}")

    async def receive_new_block(self, request):
        try:
            block_data = await request.json()
            new_block = Block.from_dict(block_data)
            if self.blockchain.add_block(new_block):
                self.logger.info(f"Received and added new block: {new_block.index}")
                return web.json_response({"message": "New block added successfully"})
            else:
                return web.json_response({"message": "Failed to add new block"}, status=400)
        except Exception as e:
            self.logger.error(f"Error processing new block: {str(e)}")
            return web.json_response({"message": "Error processing new block"}, status=500)