use crate::idt::InterruptDescriptorTable;
use crate::io::{inb, outb};

/// Represents the fundamental execution context pushed onto the stack
/// by x86-64 hardware before routing execution to an exception handler.
#[repr(C)]
pub struct InterruptStackFrame {
    /// The memory instruction pointer pointing to the instruction that caused the exception.
    pub instruction_pointer: u64,
    /// The code segment selector index register.
    pub code_segment: u64,
    /// The EFLAGS register containing processor status and control flags.
    pub cpu_flags: u64,
    /// The active stack pointer register at the time the exception was caught.
    pub stack_pointer: u64,
    /// The stack segment selector index register.
    pub stack_segment: u64,
}

/// The global instance of our kernel's interrupt vector layout.
///
/// Kept statically in memory for the duration of execution runtime. Marked as
/// mutable to allow early-boot runtime registration of handler functions.
static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

/// Remaps the legacy Programmable Interrupt Controllers (PIC) so hardware interrupts
/// do not overlap with native CPU exceptions (0x00 - 0x1F).
unsafe fn remap_pic() {
    // ICW1: Start initialization sequence for both PICs
    outb(0x20, 0x11);
    outb(0xA0, 0x11);

    // ICW2: Set vector offsets (Master PIC = 0x20, Slave PIC = 0x28)
    outb(0x21, 0x20);
    outb(0xA1, 0x28);

    // ICW3: Tell Master PIC that there is a slave PIC at IRQ2 (0x04)
    outb(0x21, 0x04);
    // Tell Slave PIC its cascade identity (0x02)
    outb(0xA1, 0x02);

    // ICW4: Set 8086/88 mode
    outb(0x21, 0x01);
    outb(0xA1, 0x01);

    // Clear masks to enable hardware interrupts
    outb(0x21, 0x00);
    outb(0xA1, 0x00);
}

/// Initializes the IDT matrix, registers exception handlers, remaps the PIC, and loads the table register.
///
/// This safely structures the vector mappings before communicating the table address
/// directly to the CPU control registers.
#[allow(static_mut_refs)]
pub fn init_idt() {
    unsafe {
        // Breakpoint Handler (Vector 3)
        let bp_address = breakpoint_handler as *const () as u64;
        IDT.edit_entry(3).set_handler(bp_address);

        // Keyboard Handler (Vector 33 / 0x21)
        let kb_address = keyboard_handler as *const () as u64;
        IDT.edit_entry(0x21).set_handler(kb_address);

        // Instruct the CPU to activate this table
        IDT.load();

        // Configure hardware routing
        remap_pic();
    }
}

/// The specific handler invoked automatically by vector 3 breakpoint traps.
///
/// Uses the unstable "x86-interrupt" ABI calling convention to safely clean
/// up the custom hardware stack frame upon execution completion.
pub extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    loop {}
}

/// The Keyboard Interrupt Service Routine (IRQ 1 / Vector 0x21).
pub extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    unsafe {
        let scancode = inb(0x60);

        // A basic look-up array where index = scancode, value = ASCII character
        // This covers a few basic keys on a standard US keyboard map
        const SCANCODE_TO_ASCII: [char; 58] = [
            '\0', '\x1B', '1', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '=', '\x08', '\t',
            'q', 'w', 'e', 'r', 't', 'y', 'u', 'i', 'o', 'p', '[', ']', '\n', '\0', 'a', 's', 'd',
            'f', 'g', 'h', 'j', 'k', 'l', ';', '\'', '`', '\0', '\\', 'z', 'x', 'c', 'v', 'b', 'n',
            'm', ',', '.', '/', '\0', '*', '\0', ' ',
        ];

        // Ensure the keypress is a "make" code (pressed down) and within our array bounds
        if scancode & 0x80 == 0 && (scancode as usize) < SCANCODE_TO_ASCII.len() {
            let character = SCANCODE_TO_ASCII[scancode as usize];

            if character != '\0' {
                // For a raw test directly to the top-left of the VGA screen:
                let vga_buffer = 0xB8000 as *mut u8;
                *vga_buffer = character as u8; // Writes the actual letter
                *vga_buffer.offset(1) = 0x0A; // Green text attribute
            }
        }

        // Inform the PIC that the interrupt traffic clear is complete
        outb(0x20, 0x20);
    }
}
