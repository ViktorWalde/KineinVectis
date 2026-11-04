#include "c_api.h"
static_assert(sizeof(int) >= 2);
int c_value(void)
{
    typeof(1) value = 2;
    return c_kind(value) + value;
}
