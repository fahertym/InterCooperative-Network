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
src
├── blockchain
│   ├── block.rs
│   ├── blockchain.rs
│   ├── mod.rs
│   ├── transaction.rs
│   ├── transaction_validator.rs
├── cli.rs
├── consensus
│   ├── consensus.rs
│   ├── mod.rs
├── currency
│   ├── currency.rs
│   ├── mod.rs
├── governance
│   ├── democracy.rs
│   ├── mod.rs
├── identity
│   ├── did.rs
│   ├── mod.rs
├── lib.rs
├── main.rs
├── mod.rs
├── network
│   ├── data_packet.rs
│   ├── interest_packet.rs
│   ├── mod.rs
│   ├── network.rs
│   ├── packet.rs
├── node
│   ├── content_store.rs
│   ├── fib.rs
│   ├── icn_node.rs
├── smart_contract
│   ├── mod.rs
│   ├── smart_contract.rs
├── vm
│   ├── compiler.rs
│   ├── mod.rs
│   ├── vm.rs
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

## License

This project is licensed under the MIT License. See the [LICENSE](LICENSE) file for details.
