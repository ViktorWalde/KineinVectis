/* Golden fixture: GNU Arm ELF and objcopy BIN, never executed on hardware. */
__attribute__((section(".vectors"), used))
const unsigned int vectors[] = {0x20001000, 0x08004009};
volatile unsigned int initialized = 0x12345678;
volatile unsigned int zeroed[3];
void reset(void) {
    zeroed[0] = initialized;
    for (;;) {}
}
