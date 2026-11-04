#ifndef SAMPLE_H
#define SAMPLE_H

#ifdef __cplusplus
#error "C mode required"
#endif

_Static_assert(_Generic(0, int: 1, default: 0), "C11 required");
int sample_sum(int left, int right);
int sample_scale(int value);

#endif
