#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

//! # Theos Kernel Entry Point
//!
//! This is the root crate of the Theos kernel. It wires together all kernel
//! subsystems — VGA display, interrupt handling, and hardware I/O — and
//! defines the entry point jumped to by the bootloader.
//!
//! ## Boot Flow
//! ```text
//! MBR (boot.asm)
//!   └─> Stage 2 (stage2.asm)
//!         └─> Long mode setup + page tables
//!               └─> kernel_main()
//! ```

pub mod drivers;
pub mod globals;
pub mod idt;
pub mod interrupts;
pub mod io;

use core::panic::PanicInfo;

/// A single entry from the BIOS `INT 0x15, EAX=0xE820` memory map.
///
/// The bootloader queries this table in real mode and stores it at a known
/// physical address. The kernel reads it to discover which regions of RAM
/// are safe to use for allocation.
///
/// ## Region Types
/// | `region_type` | Meaning                        |
/// |---------------|-------------------------------|
/// | `1`           | Usable RAM — safe to allocate |
/// | `2`           | Reserved — do not touch       |
/// | `3`           | ACPI reclaimable               |
/// | `4`           | ACPI NVS — must be preserved  |
/// | `5`           | Bad memory — unusable         |
///
/// ## Layout
/// This struct is `#[repr(C)]` to match the exact 24-byte layout the BIOS
/// writes into the buffer. Do not reorder or add fields without updating
/// the bootloader as well.
#[repr(C)]
struct E820Entry {
    /// Physical base address of this memory region.
    pub base: u64,
    /// Length of this memory region in bytes.
    pub length: u64,
    /// Classification of this region. `1` means usable RAM.
    pub region_type: u32,
    /// Extended ACPI 3.0 attributes. Safe to ignore for basic allocation.
    pub acpi_attrs: u32,
}

/// Kernel panic handler — called by the Rust runtime on any unrecoverable error.
///
/// This function is invoked automatically whenever a `panic!()`, a failed
/// `unwrap()`, an out-of-bounds index, or any other unrecoverable condition
/// occurs inside the kernel.
///
/// ## Behaviour
/// 1. Immediately disables CPU interrupts via `cli` to freeze all hardware activity.
/// 2. Clears the VGA screen and renders a structured panic report including:
///    - The source file and line number where the panic originated.
///    - The panic message, if it can be extracted as a static string.
/// 3. Halts the CPU in an infinite `hlt` loop — the machine must be restarted.
///
/// ## Safety
/// The `cli` instruction is `unsafe` because it is a privileged x86 operation
/// that affects global CPU interrupt state. It is safe to call here because
/// a panic is already an unrecoverable situation; no further interrupt
/// handling is meaningful.
///
/// This function diverges (`-> !`) and never returns.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    unsafe {
        core::arch::asm!("cli"); // Disable interrupts immediately
    }

    let mut w =
        drivers::vga::ScreenWriter::new(drivers::vga::Color::LightRed, drivers::vga::Color::Black);
    w.clear_screen();

    w.write_string(
        "================================================================================\n",
    );

    w.write_string("panicpanicpanic - KERNEL PANIC - panicpanicpanic");
    w.write_string("\n");
    w.write_string(
        "--------------------------------------------------------------------------------\n",
    );
    w.write_string("\n");
    w.write_string("  THEOS has encountered a fatal exception and cannot continue.\n");
    w.write_string("  The kernel has been halted to prevent damage to your system.\n");
    w.write_string("\n");
    w.write_string(
        "--------------------------------------------------------------------------------\n",
    );

    // Print panic location if available
    if let Some(location) = info.location() {
        w.write_string("  FAULT LOCATION : ");
        w.write_string(location.file());
        w.write_string("\n");
    }

    // Stable bare-metal method to extract and display the panic message payload.
    // `info.message().as_str()` only succeeds for static string literals;
    // formatted panic messages (e.g. `panic!("{}", x)`) fall through to the else branch.
    w.write_string("  FAULT REASON   : ");
    if let Some(msg) = info.message().as_str() {
        w.write_string(msg);
    } else {
        w.write_string(
            "A formatted runtime assertion failed. (Check source code for macro specifics)",
        );
    }
    w.write_string("\n");

    w.write_string("\n");
    w.write_string(
        "--------------------------------------------------------------------------------\n",
    );
    w.write_string("\n");
    w.write_string("  PROCESSOR STATE : HALTED\n");
    w.write_string("  INTERRUPTS      : DISABLED\n");
    w.write_string("  RECOVERY        : NOT POSSIBLE\n");
    w.write_string("\n");
    w.write_string("  If this is your first time seeing this screen, restart your machine.\n");
    w.write_string("  If this screen appears again, your kernel is corrupted.\n");
    w.write_string("\n");
    w.write_string(
        "================================================================================\n",
    );
    w.write_string("  * SYSTEM HALTED *\n");
    w.write_string(
        "================================================================================\n",
    );

    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}

/// Primary kernel entry point — jumped to by the stage 2 bootloader.
///
/// This function is the first Rust code to execute after the CPU transitions
/// into 64-bit long mode. The bootloader sets up the GDT, enables paging over
/// the first 2MB, and transfers control here via a `call kernel_main`.
///
/// ## Initialisation Order
/// The sequence below is load-order sensitive. Do not reorder without
/// understanding the dependencies between subsystems.
///
/// 1. **VGA** — initialised first so all subsequent steps can print diagnostics.
/// 2. **IDT** — installs interrupt/exception handlers before `sti` is called.
/// 3. **STI** — opens the CPU to hardware interrupts only after the IDT is ready.
///
/// ## ABI
/// Marked `extern "C"` and `#[no_mangle]` so the bootloader's `call kernel_main`
/// resolves correctly via the linker. The System V AMD64 ABI is assumed; the
/// bootloader aligns `rsp` to a 16-byte boundary before the call.
///
/// ## Future Parameters
/// Once the bootloader is updated to query `INT 0x15, E820`, this signature
/// should become:
/// ```rust
/// pub extern "C" fn kernel_main(e820_count: u64, e820_addr: u64) -> !
/// ```
/// where `e820_addr` is the physical address of the memory map buffer and
/// `e820_count` is the number of [`E820Entry`] records it contains.
///
/// This function diverges (`-> !`) — it parks the CPU in a `hlt` loop and
/// never returns to the bootloader.
#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Instantiate our documented screen writer module
    let mut writer =
        drivers::vga::ScreenWriter::new(drivers::vga::Color::White, drivers::vga::Color::Black);

    // Completely clear out messy legacy BIOS boot artifacts
    writer.clear_screen();

    // Safely print strings using architectural hardware abstraction
    writer.write_string("Theos Kernel v0.1.0 Successfully Loaded!\n");

    // Test kernel panic — Uncomment to panic
    // panic!("Testing kernel panic system");

    // Install exception and hardware interrupt handlers into the IDT.
    // Must happen before `sti` — if an interrupt fires with no handler
    // registered, the CPU will triple-fault and reset.
    interrupts::init_idt();

    // Open the CPU interrupt gate. From this point forward, hardware
    // events (keyboard, timer, etc.) will fire their registered handlers.
    unsafe {
        core::arch::asm!("sti");
    }

    // Park the CPU. `hlt` suspends execution until the next interrupt,
    // keeping power consumption low while the kernel waits for events.
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
