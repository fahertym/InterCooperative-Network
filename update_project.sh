#!/bin/bash

# Name of the output file
OUTPUT_FILE="all_code_files_content.txt"

# Function to update changelog
update_changelog() {
    git-cliff -o CHANGELOG.md
    git add CHANGELOG.md
}

# Function to process files and append their contents to the output file
process_files() {
    for file in "$1"/*; do
        if [ -f "$file" ] && [[ "$file" == *.rs || "$file" == *.toml ]]; then
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
    # Clear the output file if it already exists
    > $OUTPUT_FILE

    # Process files in the crates directory
    process_files "crates"

    # Include the workspace Cargo.toml file
    if [ -f "Cargo.toml" ]; then
        echo "===== START OF Cargo.toml =====" >> $OUTPUT_FILE
        cat "Cargo.toml" >> $OUTPUT_FILE
        echo "===== END OF Cargo.toml =====" >> $OUTPUT_FILE
        echo >> $OUTPUT_FILE
    fi

    echo "All relevant files have been processed and concatenated into $OUTPUT_FILE."
    git add $OUTPUT_FILE
}

# Main script execution
main() {
    # Check if there are any changes to commit
    if [[ -z $(git status -s) ]]; then
        echo "No changes to commit."
        exit 0
    fi

    # Prompt for commit message
    echo "Enter your commit message:"
    read commit_message

    # Generate LLM file
    generate_llm_file

    # Update changelog
    update_changelog

    # Git operations
    git add .
    git commit -m "$commit_message"
    git push origin main

    echo "Changes have been committed and pushed to the repository."
}

# Execute the main function
main