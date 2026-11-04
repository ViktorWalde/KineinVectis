#include "sample.h"
#include "sample_config.h"

int main(void) {
    const int result = sample_sum(3, 4);
    return result == SAMPLE_EXPECTED && sample_scale(2) == 4 ? 0 : 1;
}
