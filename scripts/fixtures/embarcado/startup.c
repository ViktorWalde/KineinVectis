extern unsigned long _estack, _sidata, _sdata, _edata, _sbss, _ebss;
int main(void);
void Reset_Handler(void);
void Default_Handler(void) { for (;;) {} }
__attribute__((section(".isr_vector"), used))
const void *const vetores[] = {
    &_estack, Reset_Handler, Default_Handler, Default_Handler,
};
void Reset_Handler(void) {
    unsigned long *src = &_sidata, *dst = &_sdata;
    while (dst < &_edata) *dst++ = *src++;
    for (dst = &_sbss; dst < &_ebss;) *dst++ = 0;
    main();
    for (;;) {}
}
