import asyncio
import sys
import aiohttp
import argparse
import os
import signal
from icn.blockchain.chain import Blockchain
from icn.network.node import Node
from icn.storage.file_storage import FileStorage

BOOTSTRAP_NODES = [
    "http://localhost:8000",
    "http://localhost:8001",
]

def write_pid(port):
    pid = os.getpid()
    with open(f"node_{port}.pid", "w") as f:
        f.write(str(pid))

def read_pid(port):
    try:
        with open(f"node_{port}.pid", "r") as f:
            return int(f.read().strip())
    except FileNotFoundError:
        return None

def is_port_in_use(port):
    pid = read_pid(port)
    if pid:
        try:
            os.kill(pid, 0)
            return True
        except OSError:
            return False
    return False

async def main(port):
    if is_port_in_use(port):
        print(f"Error: A node is already running on port {port}")
        return

    write_pid(port)

    storage = FileStorage(f"./data_{port}")
    blockchain = Blockchain()
    storage.load_blockchain(blockchain)

    bootstrap_nodes = [node for node in BOOTSTRAP_NODES if f":{port}" not in node]
    node = Node("localhost", port, blockchain, storage, bootstrap_nodes)
    
    def signal_handler(sig, frame):
        print(f"Stopping node on port {port}")
        os.remove(f"node_{port}.pid")
        sys.exit(0)

    signal.signal(signal.SIGINT, signal_handler)
    
    await node.start()

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
            os.remove(f"node_{args.port}.pid")