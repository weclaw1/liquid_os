global long_mode_start

section .text
bits 64
long_mode_start:
    ; load 0 into all data segment registers
    mov ax, 0
    mov ss, ax
    mov ds, ax
    mov es, ax
    mov fs, ax
    mov gs, ax

    call clear_screen
    
    ; call the rust main
    extern kmain
    call kmain

    ; print `OKAY` to screen
    mov rax, 0x2f592f412f4b2f4f
    mov qword [0xb8000], rax
    hlt

clear_screen:
    mov eax, 0xb8000 ;start of vga buffer
    mov ecx, 2000 ;loop 2000 times - 80 columns x 25 rows
    .start_loop:
    mov word [eax], 0x0020 ; 0x0020 - blank space
    add eax, 2 ;increment vga buffer address, each character takes 2 bytes
    loop .start_loop
    ret