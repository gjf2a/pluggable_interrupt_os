// Code in this file is largely Copyright (c) 2019 Philipp Oppermann.
//
// Gabriel Ferrer added these elements:
// - Writer::{plot, peek, write_char}
// - clear_row(), clear_screen(), plot_str(), plot(), plot_num(), peek()
// - clear(), plot_num_right_justified(), num_str_len()
// - ColorCode::{foreground(), background()}
// - Plot enum
// - impl From for Color

use volatile::Volatile;
use core::fmt;
use lazy_static::lazy_static;
use spin::Mutex;
use core::ops::RangeInclusive;

pub const DRAWABLE: RangeInclusive<u8> = 0x20..=0x7e;

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
/// Represents a 4-bit x86 color code
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

impl From<u8> for Color {
    // I attempted to use the enum-repr crate instead of doing this, but it did not compile.
    fn from(n: u8) -> Self {
        use Color::*;
        match n {
            0 => Black, 1 => Blue, 2 => Green, 3 => Cyan, 4 => Red, 5 => Magenta, 6 => Brown,
            7 => LightGray, 8 => DarkGray, 9 => LightBlue, 10 => LightGreen, 11 => LightCyan,
            12 => LightRed, 13 => Pink, 14 => Yellow, 15 => White,
            _ => panic!("Undefined color value: {}", n)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
/// Represents two 4-bit x86 colors: a foreground and a background
pub struct ColorCode(u8);

#[allow(dead_code)]
impl ColorCode {
    pub fn new(foreground: Color, background: Color) -> ColorCode {
        ColorCode((background as u8) << 4 | (foreground as u8))
    }

    pub fn foreground(&self) -> Color {
        Color::from(self.0 & 0xF)
    }

    pub fn background(&self) -> Color {
        Color::from((self.0 & 0xF0) >> 4)
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

#[allow(dead_code)]
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
                DRAWABLE | b'\n' => self.write_byte(byte),
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

#[allow(dead_code)]
/// Clears one row of the VGA buffer, setting everything to the background color specified.
/// It will **panic** on an illegal row.
pub fn clear_row(row: usize, background: Color) {
    let color = ColorCode::new(background, background);
    for col in 0..BUFFER_WIDTH {
        plot(' ', col, row, color);
    }
}

#[allow(dead_code)]
/// Sets all rows of the VGA buffer to Black.
pub fn clear_screen() {
    for row in 0..BUFFER_HEIGHT {
        clear_row(row, Color::Black);
    }
}

#[allow(dead_code)]
/// Displays the specified string at the given coordinates.
/// If the string exceeds the width of the buffer, it will be truncated.
/// An illegal row will **panic**.
pub fn plot_str(s: &str, col: usize, row: usize, color: ColorCode) -> usize {
    use crate::serial_println;
    let end = BUFFER_WIDTH.min(col + s.len());
    for (c, chr) in (col..end).zip(s.chars()) {
        serial_println!("Plotting {} ({},{})", chr, c, row);
        plot(chr, c, row, color);
    }
    end % BUFFER_WIDTH
}

#[allow(dead_code)]
/// Clears a certain number of spaces.
/// Returns the next column to use after the call.
/// It will **panic** on an illegal row.
pub fn clear(num_spaces: usize, col: usize, row: usize, color: ColorCode) -> usize {
    let end = BUFFER_WIDTH.min(col + num_spaces);
    for c in col..end {
        plot(' ', c, row, color);
    }
    end % BUFFER_WIDTH
}

#[allow(dead_code)]
/// Plots the given character at the given location with the given color.
/// It will **panic** on an illegal row or column.
pub fn plot(c: char, col: usize, row: usize, color: ColorCode) {
    WRITER.lock().plot(col, row, ScreenChar { ascii_character: c as u8, color_code: color });
}

#[allow(dead_code)]
/// Returns the length **num** would have when plotted.
pub fn num_str_len(num: isize) -> usize {
    if num == 0 {
        1
    } else if num < 0 {
        1 + num_str_len(-num)
    } else {
        let mut num = num;
        let mut c = 0;
        while num > 0 {
            num /= 10;
            c += 1;
        }
        c
    }
}

#[allow(dead_code)]
/// Displays the given number at the specified coordinates.
/// Returns the next column to use after the call.
///
/// It will pad the number with spaces to its left so that it occupies **total_space** columns.
/// If the number requires more columns than **total_spaces**, it will spill over to the right,
/// thus foiling right-justification.
///
/// If the number exceeds the width of the buffer, it will be truncated.
///
/// It will **panic** if an illegal row is given.
pub fn plot_num_right_justified(total_space: usize, num: isize, col: usize, row: usize, color: ColorCode) -> usize {
    let space_needed = num_str_len(num);
    let leading_spaces = if space_needed < total_space {total_space - space_needed} else {0};
    if leading_spaces > 0 {
        clear(leading_spaces, col, row, ColorCode::new(color.background(), color.background()));
    }
    plot_num(num, col + leading_spaces, row, color)
}

#[allow(dead_code)]
/// Displays the given number at the specified coordinates.
/// Returns the next column to use after the call.
///
/// If the number exceeds the width of the buffer, it will be truncated.
///
/// It will **panic** if an illegal row is given.
pub fn plot_num(num: isize, col: usize, row: usize, color: ColorCode) -> usize {
    if num == 0 {
        plot('0', col, row, color);
        (col + 1) % BUFFER_WIDTH
    } else if num < 0 {
        plot('-', col, row, color);
        plot_num(-num, col + 1, row, color)
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
        (col + c) % BUFFER_WIDTH
    }
}

#[allow(dead_code)]
/// Returns the character and color at the specified coordinates.
///
/// It will **panic** given an illegal row or column.
pub fn peek(col: usize, row: usize) -> (char, ColorCode) {
    let result = WRITER.lock().peek(col, row);
    (result.ascii_character as char, result.color_code)
}

#[allow(dead_code)]
/// Represents different options for plotting data.
pub enum Plot<'a>  {
    Str(&'a str), USize(usize), USizeRightJustified(usize,usize), ISize(isize), ISizeRightJustified(isize, usize), Clear(usize)
}

#[allow(dead_code)]
impl <'a> Plot<'a> {
    /// Calls the corresponding plot function given the data.
    ///
    /// It will **panic** on an illegal row.
    pub fn plot(&self, col: usize, row: usize, color: ColorCode) -> usize {
        match self {
            Plot::Str(s) => plot_str(s, col, row, color),
            Plot::Clear(num_spaces) => clear(*num_spaces, col, row, color),
            Plot::ISize(num) => plot_num(*num, col, row, color),
            Plot::ISizeRightJustified(num, total_space) =>
                plot_num_right_justified(*total_space, *num, col, row, color),
            Plot::USize(num) => plot_num(*num as isize, col, row, color),
            Plot::USizeRightJustified(num, total_space) =>
                plot_num_right_justified(*total_space, *num as isize, col, row, color)
        }
    }

    /// Calls the plot functions for all of the elements of **plots**, returning the final
    /// column number when complete. It will display each element in left-to-right order.
    ///
    /// It will **panic** on an illegal row.
    pub fn plot_all(col: usize, row: usize, color: ColorCode, plots: &[Plot]) -> usize {
        let mut col = col;
        for plot in plots {
            col = plot.plot(col, row, color);
        }
        col
    }
}