#!/bin/bash

# Name of the output file
OUTPUT_FILE="all_code_files_content.txt"

# Clear the output file if it already exists
> $OUTPUT_FILE

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

# Process files in the src directory
process_files "src"

# Include the Cargo.toml file in the root directory
if [ -f "Cargo.toml" ]; then
  echo "===== START OF Cargo.toml =====" >> $OUTPUT_FILE
  cat "Cargo.toml" >> $OUTPUT_FILE
  echo "===== END OF Cargo.toml =====" >> $OUTPUT_FILE
  echo >> $OUTPUT_FILE
fi

echo "All relevant files have been processed and concatenated into $OUTPUT_FILE."
