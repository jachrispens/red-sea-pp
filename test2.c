#define con(a, b, ...) a ## b ## __VA_ARGS__

int main(int argc, char** argv) con(,<, %)
    
con(return 0; %, , >)