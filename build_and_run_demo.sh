#!/bin/bash

# Navigate to the project root
cd /home/matt/InterCooperative-Network

# Clean previous build artifacts
cargo clean

# Build the project
echo "Building the project..."
cargo build --release

# Check if the build was successful
if [ $? -eq 0 ]; then
    echo "Build successful. Running the demo..."
    cargo run --release --bin icn_demo
else
    echo "Build failed. Please check the error messages above."
fi