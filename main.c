#include <stdio.h>
#include <stdint.h>
#include <stdlib.h>

extern int64_t tempo_entry(int64_t arg);

int64_t get_two_from_c() {
    return 2;
}

int64_t parse_int64(char* str) {
    char *endptr;
    int64_t value = strtoll(str, &endptr, 10);

    if (endptr == str || *endptr != '\0') {
        fprintf(stderr, "Fatal error: Invalid integer '%s'\n", str);
        exit(1);
    }
    return value;
}

int main(int argc, char *argv[]) {
    if (argc != 2) {
        fprintf(stderr, "Usage: %s <integer>\n", argv[0]);
        return 1;
    }

    int64_t value = parse_int64(argv[1]);
    int64_t result = tempo_entry(value);

    printf("%lld\n", result);
    return 0;

}
