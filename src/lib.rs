#![no_std]
#![feature(abi_x86_interrupt)]

pub mod serial;
pub mod vga_buffer;
pub mod interrupts;
pub mod gdt;

use core::panic::PanicInfo;

use pc_keyboard::DecodedKey;

pub struct HandlerTable {
    timer: Option<fn()>, keyboard: Option<fn(DecodedKey)>
}

impl HandlerTable {
    pub fn new() -> Self {
        HandlerTable {timer: None, keyboard: None}
    }

    pub fn timer(mut self, timer_handler: fn()) -> Self {
        self.timer = Some(timer_handler);
        self
    }

    pub fn handle_timer(&self) {
        if let Some(timer) = self.timer {
            (timer)()
        }
    }

    pub fn keyboard(mut self, keyboard_handler: fn(DecodedKey)) -> Self {
        self.keyboard = Some(keyboard_handler);
        self
    }

    pub fn handle_keyboard(&self, key: DecodedKey) {
        if let Some(keyboard) = self.keyboard {
            (keyboard)(key)
        }
    }
}

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