BOOT_SRC    := src/boot/boot.asm
STAGE2_SRC  := src/boot/stage2.asm
KERNEL_SRC  := src/kernel/lib.rs
BOOT_BIN    := build/boot.bin
STAGE2_OBJ  := build/stage2.o
STAGE2_BIN  := build/stage2.bin
DISK_IMG    := build/theos.img
BUILD_DIR   := build
LINKER_SCR  := linker.ld

# Rust Configuration
RUST_LIB    := target/x86_64-unknown-none/release/libkernel.a

# Tools
NASM := nasm
LD   := ld
QEMU := qemu-system-x86_64

.PHONY: all run clean

all: $(DISK_IMG)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# 1. Compile STAGE2_BIN first so we can calculate its size
$(STAGE2_OBJ): $(STAGE2_SRC) | $(BUILD_DIR)
	$(NASM) -f elf64 $< -o $@

$(RUST_LIB): $(KERNEL_SRC) Cargo.toml
	cargo build --release

$(STAGE2_BIN): $(STAGE2_OBJ) $(RUST_LIB) $(LINKER_SCR)
	$(LD) -T $(LINKER_SCR) $(STAGE2_OBJ) $(RUST_LIB) -o $@

# 2. Dynamically pass the calculated sector count to NASM when building the boot sector
$(BOOT_BIN): $(BOOT_SRC) $(STAGE2_BIN) | $(BUILD_DIR)
	@$(eval FILE_SIZE=$(shell stat -c%s $(STAGE2_BIN)))
	@$(eval SECTORS=$(shell echo $$(( ($(FILE_SIZE) + 511) / 512 )) ))
	@echo "Kernel size is $(FILE_SIZE) bytes. Injecting SECTOR_COUNT=$(SECTORS) into bootloader."
	$(NASM) -f bin -dSECTOR_COUNT=$(SECTORS) $< -o $@

$(DISK_IMG): $(BOOT_BIN) $(STAGE2_BIN)
	cat $(BOOT_BIN) $(STAGE2_BIN) > $(DISK_IMG)

run: $(DISK_IMG)
	$(QEMU) -drive format=raw,file=$(DISK_IMG) -no-reboot

clean:
	rm -rf $(BUILD_DIR) target