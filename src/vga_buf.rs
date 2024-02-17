use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;

// vga buffer
// |1bytes|       1bytes         |
// -------------------------------
// | 0-7  | 8-11 | 12-14 | 15    |
// | char | fg   | bg    | blink |
// -------------------------------

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {  // TODO: why pattern match is required
        col_pos: 0,
        style: Style::new(Color::Yellow, Color::Black, true, false),
        vga_buf: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

// http://www.brackeen.com/vga/basics.html
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
    Yellow = 6,
    Gray = 7, // Light Gray is White
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Style(u8);

impl Default for Style {
    fn default() -> Self {
        Self(0)
    }
}

impl Style {
    pub fn new(fg: Color, bg: Color, light: bool, blink: bool) -> Self {
        let mut code = (bg as u8) << 4 | fg as u8;
        if light {
            code |= 1u8 << 3;
        }

        if blink {
            code |= 1u8 << 7;
        }
        Self(code)
    }

    pub fn code(&self) -> u8 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
pub struct ScreenChar {
    pub ch: u8,
    pub style: Style,
}

#[repr(transparent)]
pub struct Buffer {
    pub chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    pub col_pos: usize,
    pub style: Style,
    pub vga_buf: &'static mut Buffer,
}

impl Writer {
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.col_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                // for now, print byte to the bottom of vga buffer(screen)
                let row = BUFFER_HEIGHT - 1;
                let col = self.col_pos;

                self.vga_buf.chars[row][col].write(ScreenChar {
                    ch: byte,
                    style: self.style,
                });
                self.col_pos += 1;
            }
        }
    }

    pub fn write_string(&mut self, bytes: &str) {
        for byte in bytes.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
        }
    }

    pub fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let ch = self.vga_buf.chars[row][col].read();
                self.vga_buf.chars[row - 1][col].write(ch);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.col_pos = 0;
    }

    fn clear_row(&mut self, row: usize) {
        for col in 0..BUFFER_WIDTH {
            self.vga_buf.chars[row][col].write(ScreenChar {
                ch: b' ',
                style: Default::default(),
            });
        }
    }
}

impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}

#[macro_export]
macro_rules! println {
    () => (print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buf::_print(format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use fmt::Write;
    use x86_64::instructions::interrupts;
    // NOTE: avoid deadlock, this is just a temporary solution.
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}
