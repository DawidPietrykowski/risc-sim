#include <stdio.h>
#include <string.h>
#include <unistd.h>

#define BUFFER_SIZE 256

void custom_print(const char* str) {
    write(1, str, strlen(str));
}

int main() {
    char buffer[BUFFER_SIZE];
    int a = 42;
    int b = 73;
    int result;

    // Basic arithmetic operations
    result = a + b;
    sprintf(buffer, "Addition: %d + %d = %d\n", a, b, result);
    custom_print(buffer);

    result = b - a;
    sprintf(buffer, "Subtraction: %d - %d = %d\n", b, a, result);
    custom_print(buffer);

    result = a * b;
    sprintf(buffer, "Multiplication: %d * %d = %d\n", a, b, result);
    custom_print(buffer);

    result = b / a;
    sprintf(buffer, "Division: %d / %d = %d\n", b, a, result);
    custom_print(buffer);

    // Bitwise operations
    result = a & b;
    sprintf(buffer, "Bitwise AND: %d & %d = %d\n", a, b, result);
    custom_print(buffer);

    result = a | b;
    sprintf(buffer, "Bitwise OR: %d | %d = %d\n", a, b, result);
    custom_print(buffer);

    result = a ^ b;
    sprintf(buffer, "Bitwise XOR: %d ^ %d = %d\n", a, b, result);
    custom_print(buffer);

    result = ~a;
    sprintf(buffer, "Bitwise NOT: ~%d = %d\n", a, result);
    custom_print(buffer);

    // Shift operations
    result = a << 2;
    sprintf(buffer, "Left shift: %d << 2 = %d\n", a, result);
    custom_print(buffer);

    result = b >> 2;
    sprintf(buffer, "Right shift: %d >> 2 = %d\n", b, result);
    custom_print(buffer);

    // Loop and conditional
    custom_print("Counting from 1 to 5:\n");
    for (int i = 1; i <= 5; i++) {
        sprintf(buffer, "%d ", i);
        custom_print(buffer);
    }
    custom_print("\n");

    // Array manipulation
    int arr[5] = {10, 20, 30, 40, 50};
    custom_print("Array elements: ");
    for (int i = 0; i < 5; i++) {
        sprintf(buffer, "%d ", arr[i]);
        custom_print(buffer);
    }
    custom_print("\n");

    return 0;
}
