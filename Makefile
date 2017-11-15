ARCH ?= x86_64
kernel := build/kernel-$(ARCH).bin
iso := build/os-$(ARCH).iso
target ?= $(ARCH)-weclaw_os
rust_os := target/$(target)/debug/libweclaw_os.a

linker_script := src/arch/$(ARCH)/boot/layout.ld
grub_cfg := src/arch/$(ARCH)/boot/grub.cfg
assembly_object_file := build/arch/$(ARCH)/boot/boot.o

.PHONY: all clean run iso cargo

all: $(kernel)

clean:
	@cargo clean
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

$(kernel): cargo $(rust_os) $(assembly_object_file) $(linker_script)
	@ld -n --gc-sections -T $(linker_script) -o $(kernel) $(assembly_object_file) $(rust_os)

cargo:
	@xargo build --target $(target)