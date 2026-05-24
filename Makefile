# Paths
BOOT_SRC   := src/boot/boot.asm
BOOT_BIN   := build/boot.bin
BUILD_DIR  := build

# Tools
NASM  := nasm
QEMU  := qemu-system-x86_64

.PHONY: all run clean

all: $(BOOT_BIN)

# Assemble the bootloader
$(BOOT_BIN): $(BOOT_SRC) | $(BUILD_DIR)
	$(NASM) -f bin $< -o $@

# Create build dir if it doesn't exist
$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

# Run in QEMU
run: $(BOOT_BIN)
	$(QEMU) -drive format=raw,file=$(BOOT_BIN) -no-reboot

# Clean build artifacts
clean:
	rm -rf $(BUILD_DIR)