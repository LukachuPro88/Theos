# Paths
BOOT_SRC    := src/boot/boot.asm
STAGE2_SRC  := src/boot/stage2.asm
BOOT_BIN    := build/boot.bin
STAGE2_BIN  := build/stage2.bin
DISK_IMG    := build/theos.img
BUILD_DIR   := build

NASM := nasm
QEMU := qemu-system-x86_64

.PHONY: all run clean

all: $(DISK_IMG)

$(BUILD_DIR):
	mkdir -p $(BUILD_DIR)

$(BOOT_BIN): $(BOOT_SRC) | $(BUILD_DIR)
	$(NASM) -f bin $< -o $@

$(STAGE2_BIN): $(STAGE2_SRC) | $(BUILD_DIR)
	$(NASM) -f bin $< -o $@

# Combine both stages into one disk image
$(DISK_IMG): $(BOOT_BIN) $(STAGE2_BIN)
	cat $(BOOT_BIN) $(STAGE2_BIN) > $(DISK_IMG)

run: $(DISK_IMG)
	$(QEMU) -drive format=raw,file=$(DISK_IMG) -no-reboot

clean:
	rm -rf $(BUILD_DIR)