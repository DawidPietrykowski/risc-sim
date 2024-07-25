#include <stdio.h>
#include <string.h>
#include <unistd.h>
#include <stdlib.h>

#define BUFFER_SIZE 256

void custom_print(const char* str) {
    write(1, str, strlen(str));
}

int main() {
    char buffer[BUFFER_SIZE];
    int a = 42;
    int b = 73;
    int result;

    // Dynamic memory allocation
    custom_print("Testing dynamic memory allocation:\n");
    
    int* dynamic_array = (int*)malloc(5 * sizeof(int));
    if (dynamic_array == NULL) {
        custom_print("Memory allocation failed\n");
        return 1;
    }

    for (int i = 0; i < 5; i++) {
        dynamic_array[i] = i * 10;
    }

    custom_print("Dynamic array contents: ");
    for (int i = 0; i < 5; i++) {
        sprintf(buffer, "%d ", dynamic_array[i]);
        custom_print(buffer);
    }
    custom_print("\n");

    // Realloc test
    custom_print("Reallocating memory:\n");
    int* resized_array = (int*)realloc(dynamic_array, 10 * sizeof(int));
    if (resized_array == NULL) {
        custom_print("Memory reallocation failed\n");
        free(dynamic_array);
        return 1;
    }
    dynamic_array = resized_array;

    for (int i = 5; i < 10; i++) {
        dynamic_array[i] = i * 10;
    }

    custom_print("Resized array contents: ");
    for (int i = 0; i < 10; i++) {
        sprintf(buffer, "%d ", dynamic_array[i]);
        custom_print(buffer);
    }
    custom_print("\n");

    // Free memory
    free(dynamic_array);

    // Pointer arithmetic
    custom_print("Testing pointer arithmetic:\n");
    int arr[5] = {10, 20, 30, 40, 50};
    int* ptr = arr;

    custom_print("Array contents using pointer: ");
    for (int i = 0; i < 5; i++) {
        sprintf(buffer, "%d ", *ptr);
        custom_print(buffer);
        ptr++;
    }
    custom_print("\n");

    // Function pointer
    custom_print("Testing function pointer:\n");
    void (*print_func)(const char*) = &custom_print;
    print_func("This is printed using a function pointer\n");

    // Bit manipulation
    custom_print("Testing bit manipulation:\n");
    unsigned int num = 0b10101010;
    sprintf(buffer, "Original number: %u\n", num);
    custom_print(buffer);

    // Set a bit
    num |= (1 << 2);
    sprintf(buffer, "After setting bit 2: %u\n", num);
    custom_print(buffer);

    // Clear a bit
    num &= ~(1 << 4);
    sprintf(buffer, "After clearing bit 4: %u\n", num);
    custom_print(buffer);

    // Toggle a bit
    num ^= (1 << 6);
    sprintf(buffer, "After toggling bit 6: %u\n", num);
    custom_print(buffer);

    return 0;
}
