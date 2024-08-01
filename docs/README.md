# InterCooperative Network (ICN) Node

## Overview

The InterCooperative Network (ICN) Node is a comprehensive implementation of a decentralized cooperative management system. This project integrates various components such as blockchain, consensus mechanisms, smart contracts, decentralized identity, and a democratic governance model to support a scalable and democratic cooperative network.

## Features

- **Blockchain**: Secure and immutable ledger for recording transactions and smart contracts.
- **Consensus Mechanisms**: Proof of Contribution (PoC) to ensure fair and transparent block validation and proposer selection.
- **Smart Contracts**: Support for deploying and executing smart contracts within the blockchain.
- **Decentralized Identity**: Management of decentralized identities (DIDs) for secure and verifiable user interactions.
- **Democratic Governance**: Proposal and voting system for decentralized decision-making.
- **Multi-Currency System**: Support for various types of currencies to represent different economic activities.
- **Network Communication**: Robust network layer for node communication and data packet management.

## Project Structure

```
/home/matt/InterCooperative-Network
├── build_and_run_demo.sh
├── Cargo.lock
├── Cargo.toml
├── CHANGELOG.md
├── cliff.toml
├── crates
│   ├── icn_api
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── lib.rs
│   │       └── web.rs
│   ├── icn_blockchain
│   │   ├── Cargo.toml
│   │   ├── README.md
│   │   └── src
│   │       ├── asset_tokenization.rs
│   │       ├── blockchain.rs
│   │       ├── lib.rs
│   │       └── transaction_validator.rs
│   ├── icn_common
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── bit_utils.rs
│   │       ├── error.rs
│   │       └── lib.rs
│   ├── icn_consensus
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── bft_poc.rs
│   │       ├── consensus.rs
│   │       ├── lib.rs
│   │       └── proof_of_cooperation.rs
│   ├── icn_core
│   │   ├── Cargo.toml
│   │   ├── src
│   │   │   ├── cli
│   │   │   │   └── mod.rs
│   │   │   ├── config.rs
│   │   │   ├── error.rs
│   │   │   ├── lib.rs
│   │   │   ├── logging
│   │   │   │   └── mod.rs
│   │   │   ├── main.rs
│   │   │   └── security
│   │   │       └── mod.rs
│   │   └── tests
│   │       ├── blockchain_and_consensus_tests.rs
│   │       ├── blockchain_tests.rs
│   │       ├── icn_node_tests.rs
│   │       ├── integration_tests.rs
│   │       ├── mod.rs
│   │       └── smart_contract_tests.rs
│   ├── icn_currency
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── asset_token.rs
│   │       ├── bond.rs
│   │       ├── currency.rs
│   │       ├── lib.rs
│   │       └── wallet.rs
│   ├── icn_dao
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   ├── icn_demo
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── main.rs
│   ├── icn_governance
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── democracy.rs
│   │       ├── governance.rs
│   │       ├── lib.rs
│   │       ├── proposal.rs
│   │       └── voting.rs
│   ├── icn_identity
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── did.rs
│   │       ├── identity_manager.rs
│   │       └── lib.rs
│   ├── icn_incentives
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   ├── icn_language
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── bytecode.rs
│   │       └── lib.rs
│   ├── icn_market
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── entities.rs
│   │       ├── lib.rs
│   │       ├── market.rs
│   │       ├── market_tests.rs
│   │       └── transaction.rs
│   ├── icn_network
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── discovery.rs
│   │       ├── lib.rs
│   │       ├── naming.rs
│   │       ├── network.rs
│   │       ├── node.rs
│   │       ├── packet.rs
│   │       ├── protocol.rs
│   │       ├── routing.rs
│   │       └── security.rs
│   ├── icn_node_management
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── content_store.rs
│   │       ├── fib.rs
│   │       ├── icn_node.rs
│   │       ├── lib.rs
│   │       ├── node.rs
│   │       └── pit.rs
│   ├── icn_reputation
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── lib.rs
│   ├── icn_sharding
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── cross_shard_communication.rs
│   │       ├── cross_shard_sync.rs
│   │       ├── cross_shard_transaction_manager.rs
│   │       └── lib.rs
│   ├── icn_smart_contracts
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── contract.pest
│   │       └── lib.rs
│   ├── icn_storage
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── lib.rs
│   │       ├── storage_manager.rs
│   │       └── storage_node.rs
│   ├── icn_testnet
│   │   ├── Cargo.toml
│   │   └── src
│   │       └── main.rs
│   ├── icn_utils
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── error.rs
│   │       ├── lib.rs
│   │       ├── types
│   │       │   ├── block.rs
│   │       │   ├── mod.rs
│   │       │   └── transaction.rs
│   │       └── utils.rs
│   ├── icn_vm
│   │   ├── Cargo.toml
│   │   └── src
│   │       ├── compiler.rs
│   │       ├── coop_vm.rs
│   │       ├── lib.rs
│   │       ├── opcode.rs
│   │       ├── smart_contract.rs
│   │       ├── templates.rs
│   │       └── vm.rs
│   └── icn_zkp
│       ├── Cargo.toml
│       └── src
│           ├── circuits.rs
│           └── lib.rs
├── docs
│   ├── API.md
│   ├── ARCHITECTURE.md
│   ├── CONTRIBUTING.md
│   ├── index.html
│   ├── README.md
│   ├── script.js
│   └── style.css
├── examples
│   └── voting_contract.coop
├── frontend
│   ├── app.js
│   └── index.html
├── PROJECT_STRUCTURE_AND_CODE_CONTENTS.txt
├── README.md
└── update_project.sh
```

## Getting Started

### Prerequisites

- Rust and Cargo installed on your system. You can follow the instructions [here](https://www.rust-lang.org/tools/install) to install them.

### Building the Project

To build the project, navigate to the project directory and run:

```sh
cargo build
```

### Running the Project

To run the project, use the following command:

```sh
cargo run
```

This will start the ICN node and execute the main function defined in `src/main.rs`.

## Usage

### Command Line Interface (CLI)

The project includes a command line interface for interacting with the ICN node. The CLI provides options for deploying and executing smart contracts, viewing the blockchain state, and more. To run the CLI, execute:

```sh
cargo run --bin icn_node_cli
```

### Smart Contracts

You can deploy and execute smart contracts using the CLI. Follow the prompts to enter the details of your smart contract.

### Governance

Create and vote on proposals through the CLI to engage in the democratic decision-making process. The system supports various proposal types such as constitutional changes, economic adjustments, and network upgrades.

### Identity Management

Register and manage decentralized identities (DIDs) using the provided interfaces. The DIDs ensure secure and verifiable interactions within the network.

### Currency System

The multi-currency system supports different types of currencies representing various economic activities. You can manage and transact using these currencies through the available interfaces.

## Modules

### Blockchain

- **block.rs**: Defines the structure and functions for a block in the blockchain.
- **blockchain.rs**: Implements the blockchain, including block creation, validation, and transaction management.
- **transaction.rs**: Defines the structure and functions for transactions.
- **transaction_validator.rs**: Implements transaction validation logic.

### Consensus

- **consensus.rs**: Implements the Proof of Contribution (PoC) consensus mechanism.
- **mod.rs**: Groups and re-exports consensus components.

### Currency

- **currency.rs**: Defines different currency types and implements the currency system.
- **mod.rs**: Groups and re-exports currency components.

### Governance

- **democracy.rs**: Implements the democratic system for proposal creation, voting, and execution.
- **mod.rs**: Groups and re-exports governance components.

### Identity

- **did.rs**: Implements decentralized identity (DID) management.
- **mod.rs**: Groups and re-exports identity components.

### Network

- **network.rs**: Implements network node and communication infrastructure.
- **packet.rs**: Defines packet structures for network communication.
- **mod.rs**: Groups and re-exports network components.

### Node

- **content_store.rs**: Implements the content store for caching data.
- **fib.rs**: Implements the Forwarding Information Base (FIB) for managing forwarding entries.
- **icn_node.rs**: Main implementation for the ICN node.

### Smart Contracts

- **smart_contract.rs**: Implements smart contract functionality.
- **mod.rs**: Groups and re-exports smart contract components.

### Virtual Machine (VM)

- **vm.rs**: Implements the virtual machine for executing smart contracts.
- **compiler.rs**: Implements the compiler for the custom smart contract language.
- **mod.rs**: Groups and re-exports VM components.

## Contributing

Contributions are welcome! Please submit a pull request or open an issue to discuss any changes or additions.

## Discord

Join our community on Discord for discussions, support, and collaboration: [https://discord.gg/885kUVUhDg](https://discord.gg/885kUVUhDg)

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
