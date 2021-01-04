#![no_std]
#![feature(abi_x86_interrupt)]

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;
pub mod handler_table;

use core::panic::PanicInfo;
use crate::handler_table::HandlerTable;

pub fn init(handlers: HandlerTable) {
    gdt::init();
    interrupts::init_idt(handlers);
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

pub fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}