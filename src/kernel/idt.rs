//! # Interrupt Descriptor Table (IDT) Entry Module
//!
//! Defines the architectural memory layout for a single 64-bit Long Mode
//! Interrupt Gate. This structure communicates directly with x86-64 CPU hardware
//! registers to point the processor to specific Interrupt Service Routines (ISRs)
//! when an exception or hardware interrupt occurs.

#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtEntry {
    /// Bits 0–15 of the 64-bit target handler function pointer address.
    pointer_low: u16,
    /// The GDT code segment selector index (e.g., `0x18` for `GDT_64_CODE_SEG`).
    gdt_selector: u16,
    /// Type and attributes bitmask (e.g., Present flag, Privilege level, Gate type).
    options: u16,
    /// Bits 16–31 of the 64-bit target handler function pointer address.
    pointer_middle: u16,
    /// Bits 32–63 of the 64-bit target handler function pointer address.
    pointer_high: u32,
    /// Explicitly zeroed space reserved by Intel and AMD hardware specifications.
    reserved: u32,
}

/// The 64-bit IDT register structure (IDTR) format required by the native `lidt` assembly instruction.
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct IdtPtr {
    /// The maximum size limit of the complete IDT array in bytes minus 1.
    limit: u16,
    /// The exact 64-bit linear virtual memory address where the IDT array begins.
    base: u64,
}

/// The main structure representing all 256 execution vectors available to the x86-64 processor.
#[repr(C, align(16))]
pub struct InterruptDescriptorTable {
    entries: [IdtEntry; 256],
}

impl IdtEntry {
    /// Creates a zero-initialized, inactive IDT gate placeholder.
    ///
    /// If an interrupt fires on a vector mapped to a missing gate, the CPU
    /// will automatically escalate to a General Protection Fault or a Double Fault.
    pub const fn missing() -> Self {
        Self {
            pointer_low: 0,
            gdt_selector: 0,
            options: 0,
            pointer_middle: 0,
            pointer_high: 0,
            reserved: 0,
        }
    }

    /// Dynamically binds a 64-bit function pointer address to this interrupt gate.
    ///
    /// Slices the address across the hardware-specified `low`, `middle`, and `high`
    /// bit slots, hooks it to the kernel's 64-bit code segment, and flags it with
    /// `0x8E00` (Present, Ring 0, 64-bit Interrupt Gate).
    pub fn set_handler(&mut self, handler_addr: u64) {
        self.pointer_low = handler_addr as u16;
        self.gdt_selector = 0x18; // Matches your CS = 0018 from the QEMU trace log

        self.options = 0x8E00;

        self.pointer_middle = (handler_addr >> 16) as u16;
        self.pointer_high = (handler_addr >> 32) as u32;
        self.reserved = 0;
    }
}

impl InterruptDescriptorTable {
    /// Instantiates a completely clear, missing IDT table matrix.
    pub const fn new() -> Self {
        Self {
            entries: [IdtEntry::missing(); 256],
        }
    }

    /// Accesses a mutable reference to a specific gate index vector (0..255).
    pub fn edit_entry(&mut self, index: usize) -> &mut IdtEntry {
        &mut self.entries[index]
    }

    /// Generates a hardware-compatible pointer structure and executes the native `lidt` instruction.
    ///
    /// # Safety
    /// This function directly manipulates CPU control registers. An incorrectly configured IDT
    /// will result in an immediate unrecoverable Triple Fault upon the next hardware interrupt.
    pub unsafe fn load(&self) {
        let ptr = IdtPtr {
            limit: (core::mem::size_of::<Self>() - 1) as u16,
            base: self as *const Self as u64,
        };

        core::arch::asm!(
            "lidt [{}]",
            in(reg) &ptr,
            options(readonly, nostack, preserves_flags)
        );
    }
}
