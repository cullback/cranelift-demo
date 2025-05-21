# Cranelift demo

Compile a function in cranelift and call it from C.


```shell
otool -tV main.o # view assembly
clang -c main.c -o caller.o
clang caller.o main.o -o final_program
```
