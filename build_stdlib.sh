#!bin/sh

# This script is used to build the standard library for the lavendeux project.
# The standard library is a collection of functions available to the user
# Because the library itself is a dependency of the project, it must be built
# before the project can be built.

cargo run --bin compiler -- -F -f stdlib/src/math.lav -o stdlib/math.bin --allow-syscalld
cargo run --bin compiler -- -F -f stdlib/src/system.lav -o stdlib/system.bin --allow-syscalld