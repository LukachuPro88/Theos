# Theos

> A lightweight operating system written in Rust.

Theos is a hobby operating system built from scratch, designed to explore low-level systems programming through the safety and expressiveness of Rust. It boots on bare metal via a hand-crafted MBR bootloader and transitions through multiple CPU modes before handing off to a Rust kernel.

---

## Features

- Custom 512-byte MBR bootloader written in x86 assembly
- Full transition from Real Mode → Protected Mode → Long Mode
- Rust kernel with no standard library (`#![no_std]`)
- Minimal, auditable codebase with no external runtime dependencies

---

## Architecture

The boot process of Theos transitions the CPU across three distinct architectural eras to prepare the environment for the Rust kernel:

### 1. Real Mode (16-bit)

The BIOS loads the 512-byte MBR bootloader at address `0x7C00`. At this stage the CPU operates in 16-bit Real Mode with direct access to BIOS interrupts. The bootloader is responsible for:

- Setting up the initial stack
- Loading additional sectors from disk into memory
- Preparing to enter Protected Mode

### 2. Protected Mode (32-bit)

The bootloader enables Protected Mode by setting up the Global Descriptor Table (GDT) and flipping the PE bit in `CR0`. This unlocks:

- 32-bit addressing
- Hardware-enforced memory segmentation
- A stable environment for setting up paging

### 3. Long Mode (64-bit)

With paging structures in place, Theos transitions to 64-bit Long Mode — the native mode for modern x86-64 processors. Control is then transferred to the Rust kernel entry point.

---

## Building

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (nightly toolchain)
- `nasm` — for assembling the bootloader
- `qemu-system-x86_64` — for running the OS in a virtual machine
- `binutils` — for linking and creating the disk image

### Build & Run

```bash
# Clone the repository
git clone https://github.com/yourname/theos
cd theos

# Build the kernel
cargo build --target x86_64-unknown-none

# Assemble the bootloader
nasm -f bin src/boot/boot.asm -o boot.bin

# Create a disk image and run
make run
```

---

## Project Structure

```
theos/
├── src/
│   ├── boot/          # MBR bootloader (x86 assembly)
│   └── kernel/        # Rust kernel entry point and core logic
│         └── drivers/ # Hardware drivers (VGA, keyboard, etc.)
├── Cargo.toml
├── Makefile
└── README.md
```

---

## License

MIT — see [LICENSE](LICENSE) for details.