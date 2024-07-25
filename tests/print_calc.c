#include <stdio.h>
#include <string.h>
#include <unistd.h>

int main() {

    printf("Hello, World!\n");

    int a = 10;
    int b = 23;
    int c = a + b + 35;
    printf("The sum of %d and %d+35 is %d\n", a, b, c);
    
    char str[64];
    sprintf(str, "sprintf test: %d\n", c);

    int strl = strlen(str);
    write(1, str, strl);

    return 0;
}