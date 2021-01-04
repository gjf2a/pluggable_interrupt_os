use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;

lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer {
        column_position: 0,
        color_code: ColorCode::new(Color::Yellow, Color::Black),
        buffer: unsafe { &mut *(0xb8000 as *mut Buffer) },
    });
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct ColorCode(u8);

impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(C)]
struct ScreenChar {
    ascii_character: u8,
    color_code: ColorCode,
}

pub const BUFFER_HEIGHT: usize = 25;
pub const BUFFER_WIDTH: usize = 80;

#[repr(transparent)]
struct Buffer {
    chars: [[Volatile<ScreenChar>; BUFFER_WIDTH]; BUFFER_HEIGHT],
}

pub struct Writer {
    column_position: usize,
    color_code: ColorCode,
    buffer: &'static mut Buffer,
}

impl Writer {
    fn plot(&mut self, col: usize, row: usize, content: ScreenChar) {
        self.buffer.chars[row][col].write(content);
    }

    fn peek(&self, col: usize, row: usize) -> ScreenChar {
        self.buffer.chars[row][col].read()
    }

    pub fn write_byte(&mut self, byte: u8) {
        match byte {
            b'\n' => self.new_line(),
            byte => self.write_char(byte)
        }
    }

    // GJF - Refactored out of write_byte()
    fn write_char(&mut self, byte: u8) {
        if self.column_position >= BUFFER_WIDTH {
            self.new_line();
        }

        let row = BUFFER_HEIGHT - 1;
        let col = self.column_position;

        self.plot(col, row, ScreenChar {
            ascii_character: byte,
            color_code: self.color_code,
        });
        self.column_position += 1;
    }

    fn new_line(&mut self) {
        for row in 1..BUFFER_HEIGHT {
            for col in 0..BUFFER_WIDTH {
                let character = self.buffer.chars[row][col].read();
                self.buffer.chars[row - 1][col].write(character);
            }
        }
        self.clear_row(BUFFER_HEIGHT - 1);
        self.column_position = 0;
    }

    fn clear_row(&mut self, row: usize) {
        let blank = ScreenChar {
            ascii_character: b' ',
            color_code: self.color_code,
        };
        for col in 0..BUFFER_WIDTH {
            self.buffer.chars[row][col].write(blank);
        }
    }

    pub fn write_string(&mut self, s: &str) {
        for byte in s.bytes() {
            match byte {
                0x20..=0x7e | b'\n' => self.write_byte(byte),
                _ => self.write_byte(0xfe),
            }
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
    use x86_64::instructions::interrupts;

    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}

pub fn clear_row(row: usize, background: Color) {
    let color = ColorCode::new(background, background);
    for col in 0..BUFFER_WIDTH {
        plot(' ', col, row, color);
    }
}

pub fn clear_screen() {
    for row in 0..BUFFER_HEIGHT {
        clear_row(row, Color::Black);
    }
}

pub fn plot_str(s: &str, col: usize, row: usize, color: ColorCode) {
    use crate::serial_println;
    let end = BUFFER_WIDTH.min(col + s.len());
    for (c, chr) in (col..end).zip(s.chars()) {
        serial_println!("Plotting {} ({},{})", chr, c, row);
        plot(chr, c, row, color);
    }
}

pub fn plot(c: char, col: usize, row: usize, color: ColorCode) {
    WRITER.lock().plot(col, row, ScreenChar { ascii_character: c as u8, color_code: color });
}

pub fn plot_num(num: isize, col: usize, row: usize, color: ColorCode) {
    if num == 0 {
        plot('0', col, row, color);
    } else if num < 0 {
        plot('-', col, row, color);
        plot_num(-num, col + 1, row, color);
    } else {
        let mut buffer = [' '; BUFFER_WIDTH];
        let mut c = 0;
        let mut num = num;
        while num > 0 && c + col < buffer.len() {
            buffer[c] = ((num % 10 + '0' as isize) as u8) as char;
            num /= 10;
            c += 1;
        }
        for i in 0..c {
            plot(buffer[i], col + c - i - 1, row, color);
        }
    }
}

pub fn peek(col: usize, row: usize) -> (char, ColorCode) {
    let result = WRITER.lock().peek(col, row);
    (result.ascii_character as char, result.color_code)
}
