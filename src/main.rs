#![no_std]
#![no_main]

mod serial;
mod vga_buffer;

use pc_keyboard::DecodedKey;
use pluggable_interrupt_os::HandlerTable;
use no_panic::no_panic;

#[no_panic]
fn start() {
    println!("Hello, world!");
}

#[no_panic]
fn tick() {
    print!(".");
}

#[no_panic]
fn key(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(character) => print!("{}", character),
        DecodedKey::RawKey(key) => print!("{:?}", key),
    }
}

#[no_mangle]
#[no_panic]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .startup(start)
        .start()
}
