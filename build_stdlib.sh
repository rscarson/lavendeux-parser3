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

cargo run --bin compiler -- -F -f "stdlib/src/stdlib.lav" -o "stdlib/stdlib.lbc" --allow-syscalld $DEBUG_FLAG
