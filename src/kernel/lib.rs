#![no_std]
#![no_main]


pub mod drivers;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

#[no_mangle]
pub extern "C" fn kernel_main() -> ! {
    // Instantiate our documented screen writer module
    let mut writer = drivers::vga::ScreenWriter::new(
        drivers::vga::Color::LightGreen,
        drivers::vga::Color::Black,
    );

    // Completely clear out messy legacy BIOS boot artifacts
    writer.clear_screen();

    // Safely print strings using architectural hardware abstraction
    writer.write_string("Theos Kernel v0.1.0 Successfully Loaded!\n");
    writer.write_string("Executing core Rust layers on top of 4GB mapped memory bounds.");

    loop {}
}
