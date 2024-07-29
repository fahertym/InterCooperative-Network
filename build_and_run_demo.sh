#!/bin/bash

# Navigate to the project root
cd /home/matt/InterCooperative-Network

# Clean previous build artifacts
cargo clean

# Lint the project
echo "Linting the project..."
cargo clippy --all-targets --all-features -- -D warnings
if [ $? -ne 0 ]; then
    echo "Linting failed. Please check the error messages above."
    exit 1
fi

# Build the project
echo "Building the project..."
cargo build

# Check if the build was successful
if [ $? -eq 0 ]; then
    echo "Build successful. Running the tests..."
    cargo test
    if [ $? -eq 0 ]; then
        echo "Tests passed. Running the demo..."
        cargo run --bin icn_demo
    else
        echo "Tests failed. Please check the error messages above."
    fi
else
    echo "Build failed. Please check the error messages above."
fi
