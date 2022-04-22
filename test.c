#define G FUNC
#define FUNC H
#define H G
#define B A
#define A(x, y) (x y)
#define C (a b)

#define bc e
#define h(x, y) x ## y

h(a + b(), c + d)