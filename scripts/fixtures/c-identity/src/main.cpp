#include "cpp_api.h"
#include "shared.h"
int main()
{
    return c_value() + cpp_api::value() == 10 ? 0 : 1;
}
