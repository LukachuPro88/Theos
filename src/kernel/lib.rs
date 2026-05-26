#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

pub mod drivers;
pub mod globals;
pub mod idt;
pub mod interrupts;
pub mod io;

use core::panic::PanicInfo;

/// Handles Theos kernel panic
///
/// Triggers when unrecoverable error happens within the kernel
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

    // Stable bare-metal method to extract and display the panic message payload
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

    interrupts::init_idt();

    // This opens up the CPU gates so it starts listening to the PIC.
    unsafe {
        core::arch::asm!("sti");
    }

    // 3. Safe infinite execution parking lot
    loop {
        unsafe {
            core::arch::asm!("hlt");
        }
    }
}
