#ifndef C_API_H
#define C_API_H
static inline int c_kind(int value)
{
    return _Generic(value, int: 1, default: 0);
}
int c_value(void);
#endif
