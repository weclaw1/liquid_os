arch ?= x86_64
kernel := build/kernel-$(arch).bin
iso := build/os-$(arch).iso
target ?= $(arch)-weclaw_os
rust_os := target/$(target)/debug/libweclaw_os.a

linker_script := src/arch/$(arch)/boot/linker.ld
grub_cfg := src/arch/$(arch)/boot/grub.cfg
assembly_source_files := $(wildcard src/arch/$(arch)/boot/*.asm)
assembly_object_files := $(patsubst src/arch/$(arch)/boot/%.asm, \
	build/arch/$(arch)/boot/%.o, $(assembly_source_files))

.PHONY: all clean run iso kernel

all: $(kernel)

clean:
	@rm -r build

run: $(iso)
	@qemu-system-x86_64 -cdrom $(iso)

iso: $(iso)

$(iso): $(kernel) $(grub_cfg)
	@mkdir -p build/isofiles/boot/grub
	@cp $(kernel) build/isofiles/boot/kernel.bin
	@cp $(grub_cfg) build/isofiles/boot/grub
	@grub-mkrescue -o $(iso) build/isofiles 2> /dev/null
	@rm -r build/isofiles

$(kernel): kernel $(rust_os) $(assembly_object_files) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) \
		$(assembly_object_files) $(rust_os)

kernel:
	@xargo build --target $(target)

# compile assembly files
build/arch/$(arch)/boot/%.o: src/arch/$(arch)/boot/%.asm
	@mkdir -p $(shell dirname $@)
	@nasm -felf64 $< -o $@