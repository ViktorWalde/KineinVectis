ENTRY(reset)
PHDRS { text PT_LOAD FLAGS(5); data PT_LOAD FLAGS(6); bss PT_LOAD FLAGS(6); }
MEMORY {
    FLASH (rx) : ORIGIN = 0x08004000, LENGTH = 64K
    RAM (rwx) : ORIGIN = 0x20000000, LENGTH = 16K
}
SECTIONS {
    .text : { KEEP(*(.vectors)) *(.text*) *(.rodata*) . = ALIGN(4); } > FLASH :text
    .data : { *(.data*) } > RAM AT> FLASH :data
    .bss (NOLOAD) : { *(.bss*) } > RAM :bss
    /DISCARD/ : { *(.ARM.exidx*) }
}
