#include <stdio.h>
#include <stdint.h> // int64_t

// Declare the external function from main.o
// The signature must match what Cranelift generated:
extern int64_t tempo_entry(int64_t arg);

int main() {
    int64_t input_val = 10;
    int64_t result;

    printf("C: Calling Cranelift-generated function _main with %lld\n", (long long)input_val);
    result = tempo_entry(input_val); // Call the function from main.o
    printf("C: Received result: %lld\n", (long long)result); // Expected: 10 + 5 = 15

    int64_t input_val2 = 42;
    printf("C: Calling Cranelift-generated function _main with %lld\n", (long long)input_val2);
    result = tempo_entry(input_val2); // Call it again
    printf("C: Received result: %lld\n", (long long)result); // Expected: 42 + 5 = 47

    return 0;
}
