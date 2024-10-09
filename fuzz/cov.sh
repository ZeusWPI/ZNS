#!/bin/env bash
set -e

# Check if the correct number of arguments is provided
if [ "$#" -ne 2 ]; then
    echo "Usage: $0 <llvm-cov> <fuzz_target>"
    exit 1
fi

# Assign the first argument to the fuzz_target variable
COMMAND=$1
FUZZ_TARGET=$2


if ! command -v $(COMMAND) &> /dev/null; then
    echo "llvm-cov could not be found, please install LLVM."
    exit 1
fi

if ! command -v rustfilt &> /dev/null; then
    echo "rustfilt could not be found, please install rustfilt."
    exit 1
fi

cargo fuzz coverage "$FUZZ_TARGET"

TARGET_DIR="target/x86_64-unknown-linux-gnu/coverage/x86_64-unknown-linux-gnu/release"
PROF_DATA="coverage/$FUZZ_TARGET/coverage.profdata"
OUTPUT_FILE="coverage/index.html"

if [ ! -f "$PROF_DATA" ]; then
    echo "Coverage data file $PROF_DATA not found."
    exit 1
fi

$COMMAND show "$TARGET_DIR/$FUZZ_TARGET" --format=html \
                                         -Xdemangler=rustfilt \
                                         --ignore-filename-regex="\.cargo" \
                                         -instr-profile="$PROF_DATA" \
                                         > "$OUTPUT_FILE"

echo "Coverage report generated as $OUTPUT_FILE"
