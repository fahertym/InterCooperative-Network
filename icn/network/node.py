import json
import asyncio
import aiohttp
from aiohttp import web
import random
import logging

from ..blockchain.chain import Blockchain
from ..blockchain.block import Block
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

    async def start(self):
        app = web.Application()
        app.router.add_get('/blocks', self.get_blocks)
        app.router.add_post('/transactions/new', self.new_transaction)
        app.router.add_post('/nodes/register', self.register_nodes)
        app.router.add_get('/nodes/resolve', self.consensus)
        app.router.add_get('/nodes/peers', self.get_peers)
        app.router.add_get('/status', self.get_status)

        self.runner = web.AppRunner(app)
        await self.runner.setup()
        self.site = web.TCPSite(self.runner, self.host, self.port)
        await self.site.start()
        self.logger.info(f"Node started on {self.address}")

        self.is_running = True
        asyncio.create_task(self.periodic_tasks())

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
            await asyncio.sleep(60)  # Run every minute

    async def discover_peers(self):
        all_peers = set(self.peers) | set(self.bootstrap_nodes)
        if not all_peers:
            return

        async with aiohttp.ClientSession() as session:
            for peer in all_peers:
                if peer == self.address:
                    continue
                try:
                    async with session.get(f'{peer}/nodes/peers') as response:
                        if response.status == 200:
                            new_peers = await response.json()
                            self.peers.update(new_peers)
                            self.logger.info(f"Discovered new peers: {new_peers}")
                    # Register ourselves with the peer
                    await self.register_with_node(session, peer)
                except aiohttp.ClientError:
                    if peer in self.peers:
                        self.peers.remove(peer)
                        self.logger.warning(f"Removed unresponsive peer: {peer}")

        self.storage.save_peers(self.peers)

    async def register_with_node(self, session, node_address):
        try:
            async with session.post(f'{node_address}/nodes/register', json={'nodes': [self.address]}) as response:
                if response.status == 200:
                    self.logger.info(f"Successfully registered with node: {node_address}")
                else:
                    self.logger.warning(f"Failed to register with node: {node_address}")
        except aiohttp.ClientError:
            self.logger.warning(f"Failed to connect to node: {node_address}")

    async def sync_blockchain(self):
        replaced = await self.resolve_conflicts()
        if replaced:
            self.logger.info("Blockchain was replaced with a longer one from the network")
        else:
            self.logger.info("Our blockchain is up to date")

    async def get_blocks(self, request):
        blocks = [block.to_dict() for block in self.blockchain.chain]
        return web.json_response(blocks)

    async def new_transaction(self, request):
        data = await request.json()
        required = ['sender_did', 'recipient_did', 'amount']
        if not all(k in data for k in required):
            return web.json_response({"message": "Missing values"}, status=400)

        transaction = self.blockchain.create_transaction(data['sender_did'], data['recipient_did'], data['amount'])
        self.blockchain.add_transaction(transaction)

        return web.json_response({"message": "Transaction added to pool"})

    async def register_nodes(self, request):
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

    async def consensus(self, request):
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

    async def resolve_conflicts(self):
        neighbours = self.peers
        new_chain = None

        max_length = len(self.blockchain.chain)

        async with aiohttp.ClientSession() as session:
            for node in neighbours:
                try:
                    async with session.get(f'{node}/blocks') as response:
                        if response.status == 200:
                            data = await response.json()
                            length = len(data)
                            chain = [Block.from_dict(block) for block in data]

                            if length > max_length and self.blockchain.is_chain_valid(chain):
                                max_length = length
                                new_chain = chain
                except aiohttp.ClientError:
                    self.logger.warning(f"Failed to connect to peer: {node}")

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
            "pending_transactions": len(self.blockchain.pending_transactions)
        }
        return web.json_response(status)