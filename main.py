import asyncio
import sys
import aiohttp
import argparse
import os
import signal
import psutil
from icn.blockchain.chain import Blockchain
from icn.network.node import Node
from icn.storage.file_storage import FileStorage

BOOTSTRAP_NODES = [
    "http://localhost:8000",
    "http://localhost:8001",
]

def is_port_in_use(port):
    for conn in psutil.net_connections():
        if conn.laddr.port == port:
            return True
    return False

async def main(port):
    if is_port_in_use(port):
        print(f"Error: Port {port} is already in use")
        return

    storage = FileStorage(f"./data_{port}")
    blockchain = Blockchain()
    storage.load_blockchain(blockchain)

    bootstrap_nodes = [node for node in BOOTSTRAP_NODES if f":{port}" not in node]
    node = Node("localhost", port, blockchain, storage, bootstrap_nodes)
    
    def signal_handler(sig, frame):
        print(f"Stopping node on port {port}")
        asyncio.create_task(node.stop())

    signal.signal(signal.SIGINT, signal_handler)
    
    await node.start()

    # Keep the main coroutine running
    while True:
        await asyncio.sleep(1)

async def check_status(port):
    if not is_port_in_use(port):
        print(f"No node is running on port {port}")
        return

    async with aiohttp.ClientSession() as session:
        try:
            async with session.get(f"http://localhost:{port}/status", timeout=5) as response:
                if response.status == 200:
                    status = await response.json()
                    print(f"Node Status (Port {port}):")
                    print(f"  Node Address: {status['node_address']}")
                    print(f"  Connected Peers: {len(status['peers'])}")
                    print(f"  Blockchain Length: {status['blockchain_length']}")
                    print(f"  Pending Transactions: {status['pending_transactions']}")
                else:
                    print(f"Failed to get status from node on port {port}. Status code: {response.status}")
        except aiohttp.ClientError as e:
            print(f"Error connecting to node on port {port}: {str(e)}")

if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="ICN Node")
    parser.add_argument("port", type=int, help="Port to run the node on")
    parser.add_argument("--status", action="store_true", help="Check node status")
    args = parser.parse_args()

    if args.status:
        asyncio.run(check_status(args.port))
    else:
        try:
            asyncio.run(main(args.port))
        except KeyboardInterrupt:
            print(f"Stopping node on port {args.port}")