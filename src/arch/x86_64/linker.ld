ENTRY(start)

SECTIONS {
    . = 1M;

    .multiboot_header :
    {
        /* ensure that the multiboot header is at the beginning */
        *(.multiboot_header)
    }

    .text :
    {
        *(.text .text.*)
    }

    .data :
    {
        *(.data .data.*)
    }

    .bss :
    {
        *(.bss .bss.*)
    }
}