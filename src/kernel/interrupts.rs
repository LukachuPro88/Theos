use crate::idt::InterruptDescriptorTable;
use crate::io::inb;

/// Represents the fundamental execution context context pushed onto the stack
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

/// Initializes the IDT matrix, registers exception handlers, and loads the table register.
///
/// This safely structures the vector mappings before communicating the table address
/// directly to the CPU control registers.
/// Initializes the IDT matrix, registers exception handlers, and loads the table register.
#[allow(static_mut_refs)]
pub fn init_idt() {
    unsafe {
        // Fixes function_casts_as_integer by passing through a raw pointer first
        let handler_address = breakpoint_handler as *const () as u64;
        IDT.edit_entry(3).set_handler(handler_address);

        // Instruct the CPU to activate this table
        IDT.load();
    }
}

/// The specific handler invoked automatically by vector 3 breakpoint traps.
///
/// Uses the unstable "x86-interrupt" ABI calling convention to safely clean
/// up the custom hardware stack frame upon execution completion.
pub extern "x86-interrupt" fn breakpoint_handler(_frame: InterruptStackFrame) {
    // Access your vga::ScreenWriter here to print a diagnostic message!
    // For now, we trap the CPU to prevent it from wandering off.
    loop {}
}

/// The Keyboard Interrupt Service Routine (IRQ 1 / Vector 0x21).
pub extern "x86-interrupt" fn keyboard_handler(_frame: InterruptStackFrame) {
    unsafe {
        // Read the raw scancode from the keyboard controller data port
        let scancode = inb(0x60);

        // TODO: Map the raw scancode to an ASCII byte.
        // For example, scancode 0x1E is the 'A' key press.
        if scancode == 0x1E {
            // Call your VGA driver to write the character here!
            // crate::drivers::vga::print_char('A');
        }

        // CRITICAL FOR HARDWARE INTERRUPTS:
        // We must send an End of Interrupt (EOI) signal to the PIC controllers
        // so they know they can send the next keypress signal.
        inb(0x20); // Sending a dummy read or explicit EOI command port write
    }
}
