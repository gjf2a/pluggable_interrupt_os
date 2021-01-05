#![no_std]
#![feature(abi_x86_interrupt)]

// hlt_loop() and panic() are Copyright (c) 2019 Philipp Oppermann.
// Everything else is written by Gabriel Ferrer.

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

    pub fn start(self) -> ! {
        init(self);
        println!("Starting up...");
        hlt_loop();
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

fn init(handlers: HandlerTable) {
    gdt::init();
    interrupts::init_idt(handlers);
    unsafe { interrupts::PICS.lock().initialize() };
    x86_64::instructions::interrupts::enable();
}

fn hlt_loop() -> ! {
    loop {
        x86_64::instructions::hlt();
    }
}

#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    println!("{}", info);
    hlt_loop();
}