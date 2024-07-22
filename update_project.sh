#!/bin/bash

# Constants
OUTPUT_FILE="PROJECT_STRUCTURE_AND_CODE_CONTENTS.txt"
PROJECT_DIR="/home/matt/InterCooperative-Network"
IGNORE_FILES="CHANGELOG.md cliff.toml"

# Function to update changelog
update_changelog() {
    echo "Updating changelog..."
    if ! git-cliff -o CHANGELOG.md; then
        echo "Failed to update changelog."
        exit 1
    fi
    git add CHANGELOG.md
}

# Function to process files and append their contents to the output file
process_files() {
    local dir="$1"
    for file in "$dir"/*; do
        if [[ "$IGNORE_FILES" =~ $(basename "$file") ]]; then
            echo "Ignoring file: $file"
            continue
        fi
        if [ -f "$file" ] && [[ "$file" == *.rs || "$file" == *.toml ]]; then
            echo "Processing file: $file"
            echo "===== START OF $file =====" >> $OUTPUT_FILE
            cat "$file" >> $OUTPUT_FILE
            echo "===== END OF $file =====" >> $OUTPUT_FILE
            echo >> $OUTPUT_FILE
        elif [ -d "$file" ] && [[ "$file" != *"target"* ]]; then
            process_files "$file"
        fi
    done
}

# Function to generate LLM file
generate_llm_file() {
    echo "Generating LLM file..."

    # Clear the output file if it already exists
    > $OUTPUT_FILE

    # Generate file structure tree
    echo "Generating file structure tree..."
    echo "===== START OF FILE STRUCTURE =====" >> $OUTPUT_FILE
    if ! tree -I 'target|node_modules' $PROJECT_DIR >> $OUTPUT_FILE; then
        echo "Failed to generate file structure tree."
        exit 1
    fi
    echo "===== END OF FILE STRUCTURE =====" >> $OUTPUT_FILE
    echo >> $OUTPUT_FILE

    # Process files in the project directory
    process_files "$PROJECT_DIR"

    # Include the workspace Cargo.toml file if it exists
    if [ -f "$PROJECT_DIR/Cargo.toml" ]; then
        echo "Processing workspace Cargo.toml..."
        echo "===== START OF $PROJECT_DIR/Cargo.toml =====" >> $OUTPUT_FILE
        cat "$PROJECT_DIR/Cargo.toml" >> $OUTPUT_FILE
        echo "===== END OF $PROJECT_DIR/Cargo.toml =====" >> $OUTPUT_FILE
        echo >> $OUTPUT_FILE
    fi

    echo "All relevant files have been processed and concatenated into $OUTPUT_FILE."
    git add $OUTPUT_FILE
}

# Function to prompt for a commit message
get_commit_message() {
    echo "Enter your commit message (end with an empty line):"
    commit_message=""
    while IFS= read -r line; do
        [[ $line ]] || break
        commit_message+="$line"$'\n'
    done
    echo "$commit_message"
}

# Main script execution
main() {
    set -e
    echo "Starting script..."

    # Check if PROJECT_DIR exists
    if [ ! -d "$PROJECT_DIR" ]; then
        echo "Project directory $PROJECT_DIR does not exist."
        exit 1
    fi

    echo "Navigating to project directory..."
    # Navigate to the project directory
    cd $PROJECT_DIR

    echo "Checking for changes to commit..."
    # Check if there are any changes to commit
    if [[ -z $(git status -s) ]]; then
        echo "No changes to commit."
        exit 0
    fi

    echo "Prompting for commit message..."
    # Prompt for commit message
    commit_message=$(get_commit_message)

    echo "Generating LLM file..."
    # Generate LLM file
    generate_llm_file

    echo "Updating changelog..."
    # Update changelog
    update_changelog

    echo "Committing changes..."
    # Git operations
    git add .
    git commit -m "$commit_message"
    git push origin main

    echo "Changes have been committed and pushed to the repository."
}

# Execute the main function
main
