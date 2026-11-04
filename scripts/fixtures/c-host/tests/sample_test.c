#include "sample.h"
#include "sample_config.h"

int main(void) {
    return sample_sum(3, 4) == SAMPLE_EXPECTED && sample_scale(-2) == -4 ? 0 : 1;
}
