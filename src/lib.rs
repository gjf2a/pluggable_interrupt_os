//! This crate enables the user to create a simple operating system by supplying interrupt handlers
//! for the timer and the keyboard. As time and energy permits, I may add other interrupt handlers
//! that seem useful.
//!
//! I developed this crate to support assignments in my operating systems course at Hendrix
//! College. It provides a nice introduction to bare-metal programming. It has not been
//! "battle-tested" in a production domain.
//!
//! The code is heavily derivative of the examples from the outstanding resource
//! [Writing an Operating System in Rust](https://os.phil-opp.com/). I would like to gratefully
//! acknowledge Philipp Oppermann's efforts to create this resource. Comments in each source file
//! specify which code elements I have adopted from him.
//!
//! Before attempting to use this crate, it is essential to read the following tutorials. In fact,
//! don't just read the tutorials; work through them! In particular, make sure to set up the
//! [x86_64-blog_os.json](https://os.phil-opp.com/minimal-rust-kernel/) file and the
//! [.cargo/config.toml](https://os.phil-opp.com/minimal-rust-kernel/) file as described therein.
//! - [A Freestanding Rust Binary](https://os.phil-opp.com/freestanding-rust-binary/)
//! - [A Minimal Rust Kernel](https://os.phil-opp.com/minimal-rust-kernel/)
//! - [VGA Text Mode](https://os.phil-opp.com/vga-text-mode/)
//! - [CPU Exceptions](https://os.phil-opp.com/cpu-exceptions/)
//! - [Double Faults](https://os.phil-opp.com/double-fault-exceptions/)
//! - [Hardware Interrupts](https://os.phil-opp.com/hardware-interrupts/)
//!
//! Having read and understood the ideas from the above tutorials, you can use this crate to create
//! your own Pluggable Interrupt Operating System (PIOS).
//!
//! Here is a very basic example (found in main.rs in this crate):
//! ```
//! #![no_std]
//! #![no_main]
//!
//! use pc_keyboard::DecodedKey;
//! use pluggable_interrupt_os::HandlerTable;
//!
//! fn tick() {
//!     print!(".");
//! }
//!
//! fn key(key: DecodedKey) {
//!     match key {
//!         DecodedKey::Unicode(character) => print!("{}", character),
//!         DecodedKey::RawKey(key) => print!("{:?}", key),
//!     }
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn _start() -> ! {
//!     HandlerTable::new()
//!         .keyboard(key)
//!         .timer(tick)
//!         .start()
//! }
//! ```
//!
//! In this example, we begin with our interrupt handlers. The **tick()** handler prints a period
//! on every timer event, and the **key()** handler displays the character typed whenever the
//! key is pressed. The **_start()** function kicks everything off by placing references to these
//! two functions in a **HandlerTable** object. Invoking **.start()** on the **HandlerTable**
//! starts execution. The PIOS sits back and loops endlessly, relying on the event handlers to
//! perform any events of interest or importance.
//!
//! As we can see from this example, the capabilities of your PIOS will be
//! limited to handling keyboard events and displaying text in the VGA buffer. Within that scope,
//! however, you can achieve quite a lot. I personally enjoyed recreating a version of a
//! well-known 1980s [arcade classic](https://github.com/gjf2a/ghost_hunter).
//!
//! Here is the main.rs from that program:
//! ```
//! #![no_std]
//! #![no_main]
//!
//! use lazy_static::lazy_static;
//! use spin::Mutex;
//! use ghost_hunter_core::GhostHunterGame;
//! use ghost_hunter::MainGame;
//! use pluggable_interrupt_os::HandlerTable;
//! use pc_keyboard::DecodedKey;
//!
//! lazy_static! {
//!     static ref GAME: Mutex<MainGame> = Mutex::new(GhostHunterGame::new());
//! }
//!
//! fn tick() {
//!     ghost_hunter::tick(&mut GAME.lock());
//! }
//!
//! fn key(key: DecodedKey) {
//!     GAME.lock().key(key);
//! }
//!
//! #[no_mangle]
//! pub extern "C" fn _start() -> ! {
//!     HandlerTable::new()
//!         .keyboard(key)
//!         .timer(tick)
//!         .start()
//! }
//! ```
//!
//! For this program, I created a
//! [GhostHunterGame struct](https://github.com/gjf2a/ghost_hunter_core/blob/master/src/lib.rs)
//! to represent the state of the game. It is wrapped in a **Mutex** and initialized using
//! [lazy_static!](https://docs.rs/lazy_static/1.4.0/lazy_static/) to ensure safe access. Nearly
//! any nontrivial program will need to make use of this design pattern.
//!
//! The **tick()** function calls a special
//! [ghost_hunter::tick()](https://github.com/gjf2a/ghost_hunter/blob/master/src/lib.rs) function
//! that handles details of drawing the game state in the VGA buffer. It also advances the ghosts
//! by one position.
//!
//! The **key()** function calls the **GhostHunterGame::key()** method to convey updates to game
//! state resulting from keypresses.
//!
//! This is a pedagogical experiment. I would be interested to hear from anyone who
//! finds this useful or has suggestions.

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

/// Table of interrupt handlers. This struct uses the
/// [Builder pattern](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
/// Start by calling new() to create a new Handler table. Then use the appropriate methods to set
/// up the handlers. When ready, call the **.start()** method to start up your pluggable
/// interrupt operating system.
///
/// For now, it only includes timer and keyboard handlers.
/// I will add more if it seems useful to do so.
/// Double-fault handling is addressed "behind the scenes".
pub struct HandlerTable {
    timer: Option<fn()>, keyboard: Option<fn(DecodedKey)>, startup: Option<fn()>, foreground: fn()
}

impl HandlerTable {
    /// Creates a new HandlerTable with no handlers.
    pub fn new() -> Self {
        HandlerTable {timer: None, keyboard: None, startup: None, foreground: x86_64::instructions::hlt}
    }

    /// Starts up a simple operating system using the specified handlers.
    pub fn start(self) -> ! {
        self.startup.map(|f| f());
        let fore = self.foreground;
        init(self);
        loop {
            (fore)()
        }
    }

    /// Sets the timer handler.
    /// Returns Self for chained [Builder pattern construction](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
    pub fn timer(mut self, timer_handler: fn()) -> Self {
        self.timer = Some(timer_handler);
        self
    }

    /// Called by the low-level interrupt routines to handle a timer event.
    pub fn handle_timer(&self) {
        if let Some(timer) = self.timer {
            (timer)()
        }
    }

    /// Sets the keyboard handler. The [DecodedKey](https://docs.rs/pc-keyboard/0.5.1/pc_keyboard/enum.DecodedKey.html)
    /// enum comes from the [pc_keyboard](https://crates.io/crates/pc-keyboard) crate.
    ///
    /// Returns Self for chained [Builder pattern construction](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
    pub fn keyboard(mut self, keyboard_handler: fn(DecodedKey)) -> Self {
        self.keyboard = Some(keyboard_handler);
        self
    }

    /// Called by the low-level interrupt routines to handle a keyboard event.
    pub fn handle_keyboard(&self, key: DecodedKey) {
        if let Some(keyboard) = self.keyboard {
            (keyboard)(key)
        }
    }

    /// Sets the startup handler.
    /// Returns Self for chained [Builder pattern construction](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
    pub fn startup(mut self, startup_handler: fn()) -> Self {
        self.startup = Some(startup_handler);
        self
    }

    /// Sets the foreground loop handler.
    /// This function is called indefinitely.
    /// Returns Self for chained [Builder pattern construction](https://doc.rust-lang.org/1.0.0/style/ownership/builders.html).
    pub fn foreground_loop(mut self, foreground_loop: fn()) -> Self {
        self.foreground = foreground_loop;
        self
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