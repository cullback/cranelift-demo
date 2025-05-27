#include <stdio.h>
#include <stdint.h> // int64_t

// Declare the external function from main.o
// The signature must match what Cranelift generated:
extern int64_t tempo_entry(char* arg);

// Function to be called by Cranelift-generated code
int64_t get_two_from_c() {
    printf("C: get_two_from_c() called\n");
    return 2;
}

int main() {
    char* input_str = "Hello from C (this string will be unused by tempo_entry now)!";
    int64_t result_val;

    printf("C: Calling Cranelift-generated function tempo_entry with string: \"%s\"\n", input_str);
    result_val = tempo_entry(input_str); // Call the function
    printf("C: Received integer result: %lld\n", (long long)result_val);

    // Call it again to ensure repeatability
    char* input_str2 = "Another test string (also unused).";
    printf("C: Calling Cranelift-generated function tempo_entry with string: \"%s\"\n", input_str2);
    result_val = tempo_entry(input_str2); // Call it again
    printf("C: Received integer result: %lld\n", (long long)result_val);

    return 0;
}
