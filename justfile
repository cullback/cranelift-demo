build:
    cargo run
    clang -c main.c -o platform.o
    clang platform.o tempo.o -o main
