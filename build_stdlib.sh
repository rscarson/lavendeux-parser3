#!/bin/bash
# Copyright Richard Carson, 2024
# Licensed under the MIT License

# This script compiles the Lavendeux standard library.
# The library will not include debug information unless the --debug flag is passed.
# The production build should NOT include debug information.
# Usage: compile.sh [--debug]

DEBUG_FLAG=""

# Check if --debug flag is passed
if [ "$1" == "--debug" ]; then
    DEBUG_FLAG="-D"
fi

# Define source file paths
SOURCE_PATHS="stdlib/src/math.lav stdlib/src/system.lav"

for p in $SOURCE_PATHS; do
    echo "Compiling $p..."
    cargo run --bin compiler -- -F -f "$p" -o "$(dirname "$p")/$(basename -s .lav "$p").bin" --allow-syscalld $DEBUG_FLAG
    echo
done
