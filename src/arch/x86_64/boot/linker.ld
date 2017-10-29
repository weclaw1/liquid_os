ENTRY(start)

SECTIONS {
    . = 1M;

    .multiboot_header :
    {
        /* ensure that the multiboot header is at the beginning */
        KEEP(*(.multiboot_header))
        . = ALIGN(4K);
    }

    .text :
    {
        *(.text .text.*)
        . = ALIGN(4K);
    }

    .data :
    {
        *(.data .data.*)
        . = ALIGN(4K);
    }

    .bss :
    {
        *(.bss .bss.*)
        . = ALIGN(4K);
    }
}