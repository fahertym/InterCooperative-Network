# ICN API Crate

## Overview

The `icn_api` crate is a part of the InterCooperative Network (ICN) project, designed to provide an HTTP API layer for interacting with the ICN blockchain and associated modules. This crate is built using the `warp` web framework and is intended to facilitate communication between clients and the ICN network by exposing a set of RESTful endpoints.

## Features

- **Proposal Management:** Create, vote on, and finalize proposals within the ICN network.
- **Transaction Handling:** Submit transactions, query balances, and manage currency within the network.
- **Identity Management:** Create and manage decentralized identities (DIDs) within the ICN.
- **Resource Allocation:** Allocate and manage resources within the network.
- **Network Statistics:** Retrieve statistics and status information about the ICN network.

## Directory Structure

    icn_api/
    ├── Cargo.toml
    └── src/
        ├── lib.rs
        └── web.rs

- **`lib.rs`**: This file contains the core implementation of the API handlers, defining the logic for handling various requests and interacting with the underlying ICN modules.
- **`web.rs`**: This file contains the web server setup using the `warp` framework, defining the routing for different API endpoints.

## Dependencies

The `icn_api` crate relies on the following dependencies:

- **`warp`**: A lightweight, composable web framework for building HTTP services in Rust.
- **`tokio`**: An asynchronous runtime for the Rust programming language, used to handle concurrency within the API.
- **`icn_core`**: Core utilities and configurations shared across different ICN crates.
- **`icn_common`**: Common types, error handling, and utility functions used across the ICN project.
- **`serde` and `serde_json`**: Libraries for serializing and deserializing data structures.

## Usage

To use the ICN API, set up and run the web server using the `start_web_server` function provided in the `web.rs` file. This will launch the API server on the specified address and port, allowing clients to interact with the ICN network.

Example:

    use std::sync::Arc;
    use icn_api::web::start_web_server;
    use icn_common::ApiLayer;

    #[tokio::main]
    async fn main() {
        let api_layer = Arc::new(ApiLayer::new());
        start_web_server(api_layer).await;
    }

## API Endpoints

The ICN API exposes several endpoints, including but not limited to:

- **`POST /transaction`**: Submit a transaction to the network.
- **`POST /proposal`**: Create a new proposal.
- **`POST /vote`**: Vote on an existing proposal.
- **`POST /finalize`**: Finalize a proposal.
- **`GET /balance`**: Retrieve the balance for a specific address.
- **`POST /identity`**: Create a new decentralized identity.

## Testing

The crate includes a set of unit tests that can be run using the following command:

    cargo test

These tests cover various API functionalities, ensuring that the endpoints work correctly and handle different scenarios, such as successful transactions, voting, and error handling.

## Contributing

Contributions to the `icn_api` crate are welcome. Please follow the guidelines in the main ICN repository's `CONTRIBUTING.md` file.

## License

This project is licensed under the MIT License. See the `LICENSE` file for more details.
