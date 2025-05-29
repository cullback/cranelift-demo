build PROGRAM_PATH:
    cargo run -- {{PROGRAM_PATH}}
    clang -c main.c -o platform.o
    clang platform.o tempo.o -o main

run PROGRAM_PATH arg:
    just build {{PROGRAM_PATH}}
    ./main {{arg}}
