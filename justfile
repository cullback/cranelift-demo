# Default program to compile if not specified
DEFAULT_PROGRAM := program/basic.rb

# Build recipe:
# 1. Run the Rust compiler (cargo run) which takes the .rb program path as an argument
#    and produces tempo.o (the compiled object code for the .rb program).
# 2. Compile main.c into platform.o.
# 3. Link platform.o (our C host) with tempo.o (the compiled .rb program) to create the final executable 'main'.
build PROGRAM_PATH=DEFAULT_PROGRAM:
    cargo run -- {{PROGRAM_PATH}}
    clang -c main.c -o platform.o
    clang platform.o tempo.o -o main

# Run recipe:
# 1. First, ensure the program is built using the same PROGRAM_PATH.
# 2. Then, execute the compiled C program './main'.
run PROGRAM_PATH=DEFAULT_PROGRAM:
    just build PROGRAM_PATH={{PROGRAM_PATH}}
    ./main
