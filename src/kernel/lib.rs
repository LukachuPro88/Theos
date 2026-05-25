#![feature(abi_x86_interrupt)]
#![no_std]
#![no_main]

pub mod drivers;
pub mod idt;
pub mod interrupts;
pub mod io;

use core::panic::PanicInfo;

/// Handles Theos kernel panic
/// 
/// Triggers when unrecoverable error happens within the kernel
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    // Disable interrupts immediately so no hardware events disrupt the panic screen
    unsafe {
        core::arch::asm!("cli");
    }

    // Create a dedicated ScreenWriter for the crash screen (White text on Red background)
    let mut panic_writer =
        drivers::vga::ScreenWriter::new(drivers::vga::Color::White, drivers::vga::Color::Red);

    // Flash the entire screen red to wipe away the working state
    panic_writer.clear_screen();

    // Print a dramatic, scary kernel panic header
    panic_writer.write_string(
        "================================================================================\n",
    );
    panic_writer.write_string(
        "               !!! THEOS KERNEL PANIC !!! CAUGHT UNRECOVERABLE ERROR !!!        \n",
    );
    panic_writer.write_string(
        "================================================================================\n\n",
    );
    panic_writer.write_string(
        "The operating system dropped execution parameters to prevent hardware damage.\n\n",
    );

    // Print the file location where the panic originated
    panic_writer.write_string("CRASH LOCATION:\n  ");
    if let Some(location) = info.location() {
        panic_writer.write_string(location.file());
        panic_writer.write_string(" at line ");
        // Since we can't format integers easily without standard library macros yet,
        // this points us directly to the source file.
    } else {
        panic_writer.write_string("Unknown Location");
    }
    panic_writer.write_string("\n\n");

    panic_writer.write_string("SYSTEM STATUS: Halted.\n");
    panic_writer.write_string("Please restart your machine manually.");

    // Lock the CPU cores permanently
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

    // 1. Load the IDT structure and remap the hardware PICs
    interrupts::init_idt();

    // 2. CRITICAL: Fire the "Set Interrupt Flag" assembly command.
    // This opens up the CPU gates so it starts listening to the PIC.
    unsafe {
        core::arch::asm!("sti");
    }

    // 3. Safe infinite execution parking lot
    loop {
        unsafe {
            // "hlt" pauses the CPU until an external hardware interrupt (like a keypress) wakes it up.
            // This keeps your system stable and stops it from eating 100% of your host machine's CPU thread.
            core::arch::asm!("hlt");
        }
    }
}
