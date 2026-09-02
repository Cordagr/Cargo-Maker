#include <cstdio>
#include "sample_crate.h"

int main() {
    int result = sample_crate_add(2, 3);
    std::printf("2 + 3 = %d\n", result);
    return result == 5 ? 0 : 1;
}
