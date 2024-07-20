# ICN Blockchain

The `icn_blockchain` crate is a core component of the InterCooperative Network (ICN) project. It provides the fundamental blockchain structure and functionality, including block creation, transaction processing, and asset management.

## Features

- Block creation and validation
- Transaction processing and verification
- Multi-currency support
- Asset tokenization
- Balance management for different currency types
- Genesis block generation

## Usage

To use the `icn_blockchain` crate in your project, add the following to your `Cargo.toml`:

```toml
[dependencies]
icn_blockchain = { path = "../icn_blockchain" }
```

Then, in your Rust code:

```rust
use icn_blockchain::{Blockchain, Transaction, Block, CurrencyType, AssetToken};

// Create a new blockchain
let mut blockchain = Blockchain::new();

// Add a transaction
let transaction = Transaction::new(
    "Alice".to_string(),
    "Bob".to_string(),
    100.0,
    CurrencyType::BasicNeeds,
);
blockchain.add_transaction(transaction).unwrap();

// Create a new block
let new_block = blockchain.create_block("Miner1".to_string()).unwrap();

// Add the block to the chain
blockchain.add_block(new_block).unwrap();

// Check balance
let balance = blockchain.get_balance("Bob", &CurrencyType::BasicNeeds);

// Create an asset token
let token = blockchain.create_asset_token(
    "Token1".to_string(),
    "Test Token".to_string(),
    "Alice".to_string(),
).unwrap();

// Transfer an asset token
blockchain.transfer_asset_token(&token.id, "Bob").unwrap();
```

## Structure

The main components of the `icn_blockchain` crate are:

- `Blockchain`: The core structure that manages the chain of blocks, transactions, and asset registry.
- `Block`: Represents a single block in the blockchain.
- `Transaction`: Represents a transaction between two parties.
- `CurrencyType`: Enum representing different types of currencies in the system.
- `AssetToken`: Represents a tokenized asset.
- `AssetRegistry`: Manages the creation and transfer of asset tokens.

## Error Handling

This crate uses the `IcnError` type from the `icn_utils` crate for error handling. All public functions that can fail return `IcnResult<T>`, which is an alias for `Result<T, IcnError>`.

## Testing

The crate includes a set of unit tests to ensure the correct functionality of its components. You can run the tests using:

```
cargo test
```

## Contributing

Contributions to the `icn_blockchain` crate are welcome. Please ensure that your code adheres to the project's coding standards and is well-documented. Don't forget to add tests for any new functionality.

## License

This project is licensed under [INSERT LICENSE HERE].