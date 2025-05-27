#include <stdio.h>
#include <stdint.h> // int64_t

// Declare the external function from main.o
// The signature must match what Cranelift generated:
extern char* tempo_entry(char* arg);

int main() {
    char* input_str = "Hello from C!";
    char* result_str;

    printf("C: Calling Cranelift-generated function tempo_entry with \"%s\"\n", input_str);
    result_str = tempo_entry(input_str); // Call the function
    printf("C: Received result: \"%s\"\n", result_str);

    char* input_str2 = "Another test string.";
    printf("C: Calling Cranelift-generated function tempo_entry with \"%s\"\n", input_str2);
    result_str = tempo_entry(input_str2); // Call it again
    printf("C: Received result: \"%s\"\n", result_str);

    return 0;
}
