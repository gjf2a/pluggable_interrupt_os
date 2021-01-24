# Overview

This crate enables the user to create a simple operating system by supplying interrupt handlers
for the timer and the keyboard. As time and energy permits, I may add other interrupt handlers
that seem useful.

I developed this crate to support two assignments in my operating systems course at Hendrix
College. It provides a nice introduction to bare-metal programming. It has not been
"battle-tested" in a production domain.

The code is heavily derivative of the examples from the outstanding resource
[Writing an Operating System in Rust](https://os.phil-opp.com/). I would like to gratefully
acknowledge Philipp Oppermann's efforts to create this resource. Comments in each source file
specify which code elements I have adopted from him.

Before attempting to use this crate, it is essential to read the following tutorials. In fact,
don't just read the tutorials; work through them! In particular, make sure to set up the
[x86_64-blog_os.json](https://os.phil-opp.com/minimal-rust-kernel/) file and the
[.cargo/config.toml](https://os.phil-opp.com/minimal-rust-kernel/) file as described therein.
- [A Freestanding Rust Binary](https://os.phil-opp.com/freestanding-rust-binary/)
- [A Minimal Rust Kernel](https://os.phil-opp.com/minimal-rust-kernel/)
- [VGA Text Mode](https://os.phil-opp.com/vga-text-mode/)
- [CPU Exceptions](https://os.phil-opp.com/cpu-exceptions/)
- [Double Faults](https://os.phil-opp.com/double-fault-exceptions/)
- [Hardware Interrupts](https://os.phil-opp.com/hardware-interrupts/)

Having read and understood the ideas from the above tutorials, you can use this crate to create
your own Pluggable Interrupt Operating System (PIOS).

Here is a very basic example (found in main.rs in this crate):
```
#![no_std]
#![no_main]

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
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .start()
}
```

In this example, we begin with our interrupt handlers. The **tick()** handler prints a period
on every timer event, and the **key()** handler displays the character typed whenever the
key is pressed. The **_start()** function kicks everything off by placing references to these
two functions in a **HandlerTable** object. Invoking **.start()** on the **HandlerTable**
starts execution. The PIOS sits back and loops endlessly, relying on the event handlers to
perform any events of interest or importance.

As we can see from this example, the capabilities of your PIOS will be
limited to handling keyboard events and displaying text in the VGA buffer. Within that scope,
however, you can achieve quite a lot. I personally enjoyed recreating a version of a
well-known 1980s [arcade classic](https://github.com/gjf2a/ghost_hunter).

Here is the main.rs from that program:
```
#![no_std]
#![no_main]

use lazy_static::lazy_static;
use spin::Mutex;
use ghost_hunter_core::GhostHunterGame;
use ghost_hunter::MainGame;
use pluggable_interrupt_os::HandlerTable;
use pc_keyboard::DecodedKey;

lazy_static! {
    static ref GAME: Mutex<MainGame> = Mutex::new(GhostHunterGame::new());
}

fn tick() {
    ghost_hunter::tick(&mut GAME.lock());
}

fn key(key: DecodedKey) {
    GAME.lock().key(key);
}

#[no_mangle]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .start()
}
```

For this program, I created a
[GhostHunterGame struct](https://github.com/gjf2a/ghost_hunter_core/blob/master/src/lib.rs)
to represent the state of the game. It is wrapped in a **Mutex** and initialized using
[lazy_static!](https://docs.rs/lazy_static/1.4.0/lazy_static/) to ensure safe access. Nearly
any nontrivial program will need to make use of this design pattern.

The **tick()** function calls a special
[ghost_hunter::tick()](https://github.com/gjf2a/ghost_hunter/blob/master/src/lib.rs) function
that handles details of drawing the game state in the VGA buffer. It also advances the ghosts
by one position.

The **key()** function calls the **GhostHunterGame::key()** method to convey updates to game
state resulting from keypresses.

To help you get started on your own projects, I have created a 
[project template](https://github.com/gjf2a/pluggable_interrupt_template) 
that includes most of what you need. To get the project template working, you'll 
also need to install the following:
* [Qemu](https://www.qemu.org/)
* Nightly Rust. To install:
    * `rustup default nightly`
* `llvm-tools-preview`. To install:
    * `rustup component add llvm-tools-preview`
* The [bootimage](https://github.com/rust-osdev/bootimage) tool. To install it:
    * `cargo install bootimage`

This is a pedagogical experiment. I would be interested to hear from anyone who
finds this useful or has suggestions.

