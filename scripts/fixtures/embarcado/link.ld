/* Cortex-M3 da maquina lm3s6965evb do QEMU: flash em 0x0, SRAM em 0x20000000
   (fonte: qemu-system-arm -machine lm3s6965evb; Stellaris LM3S6965 datasheet:
   256 KiB flash, 64 KiB SRAM). */
MEMORY
{
  FLASH (rx) : ORIGIN = 0x00000000, LENGTH = 256K
  SRAM  (rwx): ORIGIN = 0x20000000, LENGTH = 64K
}
ENTRY(Reset_Handler)
_estack = ORIGIN(SRAM) + LENGTH(SRAM);
SECTIONS
{
  .isr_vector : { KEEP(*(.isr_vector)) } > FLASH
  .text : { *(.text*) *(.rodata*) } > FLASH
  _sidata = LOADADDR(.data);
  .data : { _sdata = .; *(.data*) _edata = .; } > SRAM AT > FLASH
  .bss : { _sbss = .; *(.bss*) *(COMMON) _ebss = .; } > SRAM
}
