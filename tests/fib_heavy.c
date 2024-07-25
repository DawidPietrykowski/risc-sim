#include <stdio.h>
#include <time.h>

#define ITERATIONS 1000 // 1 thousand iterations

unsigned long long fibonacci(unsigned int n) {
    if (n <= 1) return n;
    unsigned long long a = 0, b = 1, temp;
    for (unsigned int i = 2; i <= n; i++) {
        temp = a + b;
        a = b;
        b = temp;
    }
    return b;
}

int main() {
    unsigned long long result = 0;

    for (unsigned int i = 0; i < ITERATIONS; i++) {
        result += fibonacci(i % 50); // Calculate Fibonacci numbers up to 50
        result *= 1103515245;        // Linear congruential generator
        result += 12345;
        result &= 0xFFFFFFFF;        // Limit to 32 bits
    }
    
    printf("%llu\n", result);
    
    return 0;
}
