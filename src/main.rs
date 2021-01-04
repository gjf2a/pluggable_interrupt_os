#![no_std]
#![no_main]

// Based on Writing an OS in Rust (https://os.phil-opp.com/)
// Adapted by Gabriel Ferrer

mod vga_buffer;
mod serial;

use pc_keyboard::DecodedKey;
use pluggable_interrupt_os::HandlerTable;

fn tick() {
    print!(".");
}

fn key(key: DecodedKey) {
    match key {
        DecodedKey::Unicode(character) => print!("{}", character),
        DecodedKey::RawKey(key) => print!("{:?}", key),
    }
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    let handlers = HandlerTable::new()
        .keyboard(key)
        .timer(tick);
    pluggable_interrupt_os::init(handlers);
    pluggable_interrupt_os::hlt_loop();
}
