default: run

.PHONY: clean

build/multiboot_header.o: multiboot_header.asm
	mkdir -p build
	nasm -f elf64 multiboot_header.asm -o build/multiboot_header.o

build/boot.o: boot.asm
	mkdir -p build
	nasm -f elf64 boot.asm -o build/boot.o

build/long_mode_init.o: long_mode_init.asm
	mkdir -p build
	nasm -f elf64 long_mode_init.asm -o build/long_mode_init.o

build/kernel.bin: build/multiboot_header.o build/boot.o build/long_mode_init.o linker.ld
	ld -n -o build/kernel.bin -T linker.ld build/multiboot_header.o build/boot.o build/long_mode_init.o

build/os.iso: build/kernel.bin grub.cfg
	mkdir -p build/isofiles/boot/grub
	cp grub.cfg build/isofiles/boot/grub
	cp build/kernel.bin build/isofiles/boot/
	grub-mkrescue -o build/os.iso build/isofiles

run: build/os.iso
	qemu-system-x86_64 -cdrom build/os.iso

build: build/os.iso

clean:
	rm -rf build