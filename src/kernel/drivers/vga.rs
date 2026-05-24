//! # VGA Module
//!
//! Provides a structured, bare-metal screen writer (`ScreenWriter`) that interacts directly
//! with x86 hardware VGA memory. Features include automatic wrapping at row bounds, newline 
//! parsing, and memory-copying row shifts to achieve full screen scrolling.

const BUFFER_HEIGHT: usize = 25;
const BUFFER_WIDTH: usize = 80;
const VGA_ADDRESS: *mut u16 = 0xB8000 as *mut u16;

/// Standard 16-color palette supported natively by classic x86 VGA hardware text modes.
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color {
    Black = 0,
    Blue = 1,
    Green = 2,
    Cyan = 3,
    Red = 4,
    Magenta = 5,
    Brown = 6,
    LightGray = 7,
    DarkGray = 8,
    LightBlue = 9,
    LightGreen = 10,
    LightCyan = 11,
    LightRed = 12,
    Pink = 13,
    Yellow = 14,
    White = 15,
}

/// A hardware tracking state layout that abstracts coordinate management and color mapping
/// into standard, structured safe methods.
pub struct ScreenWriter {
    column_position: usize,
    row_position: usize,
    color_attribute: u16,
}

impl ScreenWriter {
    /// Creates a new `ScreenWriter` targeted at top-left index origin `(0,0)`.
    pub const fn new(fg: Color, bg: Color) -> Self {
        Self {
            column_position: 0,
            row_position: 0,
            color_attribute: ((((bg as u8) << 4) | (fg as u8)) as u16) << 8,
        }
    }

    /// Completely wipes the screen with blank space characters using the active color code.
    pub fn clear_screen(&mut self) {
        unsafe {
            let blank = ' ' as u16 | self.color_attribute;
            for i in 0..(BUFFER_HEIGHT * BUFFER_WIDTH) {
                *VGA_ADDRESS.add(i) = blank;
            }
        }
        self.column_position = 0;
        self.row_position = 0;
    }

    /// Writes a single byte to the active cursor layout.
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_position >= BUFFER_WIDTH {
                    self.new_line();
                }

                let offset = (self.row_position * BUFFER_WIDTH) + self.column_position;
                unsafe {
                    *VGA_ADDRESS.add(offset) = (byte as u16) | self.color_attribute;
                }
                self.column_position += 1;
            }
        }
    }

    /// Breaks string references into a sequential series of plain bytes for text processing.
    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            self.write_byte(byte);
        }
    }

    /// Moves the internal column position tracker back to index zero and steps downward.
    fn new_line(&mut self) {
        if self.row_position < BUFFER_HEIGHT - 1 {
            self.row_position += 1;
        } else {
            unsafe {
                for row in 1..BUFFER_HEIGHT {
                    for col in 0..BUFFER_WIDTH {
                        let src_offset = (row * BUFFER_WIDTH) + col;
                        let dest_offset = ((row - 1) * BUFFER_WIDTH) + col;
                        *VGA_ADDRESS.add(dest_offset) = *VGA_ADDRESS.add(src_offset);
                    }
                }
                let blank = ' ' as u16 | self.color_attribute;
                let bottom_row_start = (BUFFER_HEIGHT - 1) * BUFFER_WIDTH;
                for i in 0..BUFFER_WIDTH {
                    *VGA_ADDRESS.add(bottom_row_start + i) = blank;
                }
            }
        }
        self.column_position = 0;
    }
}