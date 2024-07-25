#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>
#include <math.h>

#define BUFFER_SIZE 256

void custom_print(const char* str) {
    write(1, str, strlen(str));
}

// Recursive function to calculate factorial
unsigned long long factorial(unsigned int n) {
    if (n == 0 || n == 1) return 1;
    return n * factorial(n - 1);
}

// Struct definition
struct Point {
    int x;
    int y;
};

// Union definition
union Data {
    int i;
    float f;
    char str[20];
};

int main() {
    char buffer[BUFFER_SIZE];

    // Recursion test
    custom_print("Testing recursion (factorial):\n");
    unsigned int n = 5;
    unsigned long long fact = factorial(n);
    sprintf(buffer, "Factorial of %u is %llu\n", n, fact);
    custom_print(buffer);

    // Nested loops
    custom_print("Testing nested loops:\n");
    for (int i = 0; i < 3; i++) {
        for (int j = 0; j < 3; j++) {
            sprintf(buffer, "(%d, %d) ", i, j);
            custom_print(buffer);
        }
        custom_print("\n");
    }

    // Switch statement
    custom_print("Testing switch statement:\n");
    int choice = 2;
    switch(choice) {
        case 1:
            custom_print("You chose 1\n");
            break;
        case 2:
            custom_print("You chose 2\n");
            break;
        case 3:
            custom_print("You chose 3\n");
            break;
        default:
            custom_print("Invalid choice\n");
    }

    // Struct test
    custom_print("Testing structs:\n");
    struct Point p1 = {10, 20};
    sprintf(buffer, "Point coordinates: (%d, %d)\n", p1.x, p1.y);
    custom_print(buffer);

    // Union test
    custom_print("Testing unions:\n");
    union Data data;
    data.i = 10;
    sprintf(buffer, "data.i: %d\n", data.i);
    custom_print(buffer);
    data.f = 220.5;
    sprintf(buffer, "data.f: %.2f\n", data.f);
    custom_print(buffer);
    strcpy(data.str, "C Programming");
    custom_print(data.str);
    custom_print("\n");

    // Floating-point operations (if supported)
    custom_print("Testing floating-point operations:\n");
    float f1 = 10.5;
    float f2 = 5.2;
    float f_result = f1 * f2;
    sprintf(buffer, "%.2f * %.2f = %.2f\n", f1, f2, f_result);
    custom_print(buffer);

    // More complex pointer operations
    custom_print("Testing complex pointer operations:\n");
    int arr[2][3] = {{1, 2, 3}, {4, 5, 6}};
    int (*ptr)[3] = arr;
    
    for (int i = 0; i < 2; i++) {
        for (int j = 0; j < 3; j++) {
            sprintf(buffer, "%d ", *(*(ptr + i) + j));
            custom_print(buffer);
        }
        custom_print("\n");
    }

    // Inline assembly (if supported by your simulator)
    custom_print("Testing inline assembly:\n");
    int x = 10, y = 20, sum;
    __asm__ (
        "add %0, %1, %2"
        : "=r" (sum)
        : "r" (x), "r" (y)
    );
    sprintf(buffer, "Sum calculated using inline assembly: %d\n", sum);
    custom_print(buffer);

    return 0;
}
