// vga_buffer.rs - Memory Mapped IO to the VGA Buffer


use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use volatile::Volatile;


lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_pos: 0,
        // TODO allow user to choose the VGA color code
        color_code: ColorCode::new(Color::Green, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}


#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum Color { // Standard VGA Color Palette
    Black       = 0,
    Blue        = 1,
    Green       = 2,
    Cyan        = 3,
    Red         = 4,
    Magenta     = 5,
    Brown       = 6,
    LightGray   = 7,
    DarkGray    = 8,
    LightBlue   = 9,
    LightGreen  = 10,
    LightCyan   = 11,
    LightRed    = 12,
    Pink        = 13,
    Yellow      = 14,
    White       = 15,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
struct ColorCode(u8);


impl ColorCode {
    fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}


// A screen character in the VGA text buffer, consisting of an ASCII character and a `ColorCode`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}


// The height of the text buffer (normally 25 lines).
const BUFFER_HEIGHT: usize = 25;
// The width of the text buffer (normally 80 columns).
const BUFFER_WIDTH: usize = 80;


struct Buffer { // Represents VGA Text buffer
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}


// Printed when a character code is invalid
const UNRECOGNIZED_CHAR: u8 = 0xfe;

pub struct Writer {
    column_pos: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    // Writes a single byte to the VGA Text Buffer, wraps the line if needed
    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => {
                if self.column_pos >= BUFFER_WIDTH {
                    self.new_line();
                }

                let row = BUFFER_HEIGHT - 1;
                let col = self.column_pos;

                let color_code = self.color_code;
                self.buffer.chars[row][col].write(ScreenChar {
                    ascii_character: byte,
                    color_code,
                });
                self.column_pos += 1;
            }
        }
    }

    // Writes a string and does line wrapping if string exceeds buffer width
    fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                // printable ASCII byte or newline
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                // not part of printable ASCII range
                _ => self.write_byte(UNRECOGNIZED_CHAR),
            }
        }
    }

    // Shifts all lines one line up and clears the last row.
    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_pos = 0;
    }

    // Clears a row by overwriting it with blank characters.
    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
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
macro_rules! print {
    ($($arg:tt)*) => ($crate::vga_buffer::_print(format_args!($($arg)*)));
}


#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}


#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    use core::fmt::Write;
    WRITER.lock().write_fmt(args).unwrap();
}


#[test_case]
fn test_println_simple_no_panic() {
    println!("test_println_simple_no_panic");
}


#[test_case]
fn test_println_many_no_panic() {
    for _ in 0..200 {
        println!("test_println_many_no_panic");
    }
}


#[test_case]
fn test_println_output() {
    let s = "Lorem ipsum dolor sit amet";
    println!("{}", s);
    for (i, c) in s.chars().enumerate() {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), c);
    }
}


#[test_case]
fn test_print_line_wrap() {
    for _ in 0..BUFFER_WIDTH {
        print!("X");
    }
    print!("Y");
    for i in 0..BUFFER_WIDTH {
        let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][i].read();
        assert_eq!(char::from(screen_char.ascii_character), 'X');
    }
    let screen_char = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 1][0].read();
    assert_eq!(char::from(screen_char.ascii_character), 'Y');
    println!(""); // Clear the line for the next test
}


#[test_case]
fn test_non_printable_char() {
    let s = "💖";
    println!("{}", s);

    let screen_char1 = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][0].read();
    let screen_char2 = WRITER.lock().buffer.chars[BUFFER_HEIGHT - 2][1].read();
    assert_eq!(screen_char1.ascii_character, UNRECOGNIZED_CHAR);
    assert_eq!(screen_char2.ascii_character, UNRECOGNIZED_CHAR);
}
