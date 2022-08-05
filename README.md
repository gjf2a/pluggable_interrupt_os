# Overview

This crate enables the user to create a simple operating system by supplying interrupt handlers
for the timer and the keyboard. As time and energy permits, I may add other interrupt handlers
that seem useful.

I developed this crate to support assignments in my operating systems course at Hendrix
College. It provides a nice introduction to bare-metal programming. It has not been
"battle-tested" in a production domain.

The code is heavily derivative of the examples from the outstanding resource
[Writing an Operating System in Rust](https://os.phil-opp.com/). I would like to gratefully
acknowledge Philipp Oppermann's efforts to create this resource. Comments in each source file
specify which code elements I have adopted from him.

Before attempting to use this crate:
* Read the following tutorials. In fact, don't just read them; work through them!
  * [A Freestanding Rust Binary](https://os.phil-opp.com/freestanding-rust-binary/)
  * [A Minimal Rust Kernel](https://os.phil-opp.com/minimal-rust-kernel/)
  * [VGA Text Mode](https://os.phil-opp.com/vga-text-mode/)
  * [CPU Exceptions](https://os.phil-opp.com/cpu-exceptions/)
  * [Double Faults](https://os.phil-opp.com/double-fault-exceptions/)
  * [Hardware Interrupts](https://os.phil-opp.com/hardware-interrupts/)
* Install the following software:
  * [Qemu](https://www.qemu.org/)
  * Nightly Rust:
    * `rustup default nightly`
  * `llvm-tools-preview`:
    * `rustup component add llvm-tools-preview`
  * The [bootimage](https://github.com/rust-osdev/bootimage) tool:
    * `cargo install bootimage`
* Set up the following files as described in the tutorials:
  * [x86_64-blog_os.json](https://os.phil-opp.com/minimal-rust-kernel/#a-minimal-kernel)
  * [.cargo/config.toml](https://os.phil-opp.com/minimal-rust-kernel/#building-our-kernel)
  * [Cargo.toml](https://os.phil-opp.com/freestanding-rust-binary/#summary) 
    (and also [here](https://os.phil-opp.com/minimal-rust-kernel/#creating-a-bootimage))

Having read and understood the ideas from the above tutorials, you can use this crate to create
your own Pluggable Interrupt Operating System (PIOS).

Here is a very basic example (found in `main.rs` in this crate):
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

I have created a 
[simple but more elaborate example](https://github.com/gjf2a/pluggable_interrupt_template) 
that you can use as a template for your own projects. It includes the 
[`.cargo/config`](https://github.com/gjf2a/pluggable_interrupt_template/blob/master/.cargo/config.toml),
[`Cargo.toml`](https://github.com/gjf2a/pluggable_interrupt_template/blob/master/Cargo.toml),
and [`x86_64-blog_os.json`](https://github.com/gjf2a/pluggable_interrupt_template/blob/master/x86_64-blog_os.json) 
files described in the tutorials. Once the other components are installed, it should be ready
to run. 

It demonstrates a simple interactive program that uses both keyboard and timer interrupts.
When the user types a viewable key, it is added to a string in the middle of the screen.
When the user types an arrow key, the string begins moving in the indicated direction.
Here is its [`main.rs`](https://github.com/gjf2a/pluggable_interrupt_template/blob/master/src/main.rs):

```
#![no_std]
#![no_main]

use lazy_static::lazy_static;
use spin::Mutex;
use pc_keyboard::DecodedKey;
use pluggable_interrupt_template::LetterMover;
use pluggable_interrupt_os::HandlerTable;

#[no_mangle]
pub extern "C" fn _start() -> ! {
    HandlerTable::new()
        .keyboard(key)
        .timer(tick)
        .start()
}

lazy_static! {
    static ref LETTERS: Mutex<LetterMover> = Mutex::new(LetterMover::new());
}


fn tick() {
    LETTERS.lock().tick();
}

fn key(key: DecodedKey) {
    LETTERS.lock().key(key);
}
```

I created the [`LetterMover`](https://github.com/gjf2a/pluggable_interrupt_template/blob/master/src/lib.rs)
`struct` to represent the application state. It is wrapped in a **Mutex** and initialized using
[lazy_static!](https://docs.rs/lazy_static/1.4.0/lazy_static/) to ensure safe access. Nearly
any nontrivial program will need to make use of this design pattern.

The **tick()** function calls the `LetterMover::tick()` method after unlocking the object. 
Similarly, the **key()** function calls the `LetterMover::key()` method, again after unlocking
the object.

Here is the rest of its code, found in its [`lib.rs`](https://github.com/gjf2a/pluggable_interrupt_template/blob/master/src/lib.rs) file:
```
#![cfg_attr(not(test), no_std)]

use bare_metal_modulo::{ModNum, ModNumIterator};
use pluggable_interrupt_os::vga_buffer::{BUFFER_WIDTH, BUFFER_HEIGHT, plot, ColorCode, Color, is_drawable};
use pc_keyboard::{DecodedKey, KeyCode};
use num::traits::SaturatingAdd;

#[derive(Copy,Debug,Clone,Eq,PartialEq)]
pub struct LetterMover {
    letters: [char; BUFFER_WIDTH],
    num_letters: ModNum<usize>,
    next_letter: ModNum<usize>,
    col: ModNum<usize>,
    row: ModNum<usize>,
    dx: ModNum<usize>,
    dy: ModNum<usize>
}

impl LetterMover {
    pub fn new() -> Self {
        LetterMover {
            letters: ['A'; BUFFER_WIDTH],
            num_letters: ModNum::new(1, BUFFER_WIDTH),
            next_letter: ModNum::new(1, BUFFER_WIDTH),
            col: ModNum::new(BUFFER_WIDTH / 2, BUFFER_WIDTH),
            row: ModNum::new(BUFFER_HEIGHT / 2, BUFFER_HEIGHT),
            dx: ModNum::new(0, BUFFER_WIDTH),
            dy: ModNum::new(0, BUFFER_HEIGHT)
        }
    }
```

This data structure represents the letters the user has typed, the total number of typed letters,
the position of the next letter to type, the position of the string, and its motion. Initially,
the string consists of the letter `A`, motionless, and situated in the middle of the screen.

The [`ModNum` data type](https://crates.io/crates/bare_metal_modulo) represents an integer 
(modulo m). It is very useful for keeping all of these values within the constraints of the 
VGA buffer.

```
    fn letter_columns(&self) -> impl Iterator<Item=usize> {
        ModNumIterator::new(self.col)
            .take(self.num_letters.a())
            .map(|m| m.a())
    }
```

Also from the [bare_metal_modulo](https://crates.io/crates/bare_metal_modulo) crate, the 
`ModNumIterator` data type starts at the specified value and loops around through the ring.
In this case, it takes just enough values to represent all of the columns to use when plotting
our string. Using `ModNum` ensures that all the column values are legal and wrap around 
appropriately. 

```
    pub fn tick(&mut self) {
        self.clear_current();
        self.update_location();
        self.draw_current();
    }

    fn clear_current(&self) {
        for x in self.letter_columns() {
            plot(' ', x, self.row.a(), ColorCode::new(Color::Black, Color::Black));
        }
    }
    
    fn update_location(&mut self) {
        self.col += self.dx;
        self.row += self.dy;
    }
    
    fn draw_current(&self) {
        for (i, x) in self.letter_columns().enumerate() {
            plot(self.letters[i], x, self.row.a(), ColorCode::new(Color::Cyan, Color::Black));
        }
    }
```

On each tick:
* Clear the current string.
* Update its position.
* Redraw the string in its new location.

```
    pub fn key(&mut self, key: DecodedKey) {
        match key {
            DecodedKey::RawKey(code) => self.handle_raw(code),
            DecodedKey::Unicode(c) => self.handle_unicode(c)
        }
    }

    fn handle_raw(&mut self, key: KeyCode) {
        match key {
            KeyCode::ArrowLeft => {
                self.dx -= 1;
            }
            KeyCode::ArrowRight => {
                self.dx += 1;
            }
            KeyCode::ArrowUp => {
                self.dy -= 1;
            }
            KeyCode::ArrowDown => {
                self.dy += 1;
            }
            _ => {}
        }
    }

    fn handle_unicode(&mut self, key: char) {
        if is_drawable(key) {
            self.letters[self.next_letter.a()] = key;
            self.next_letter += 1;
            self.num_letters = self.num_letters.saturating_add(&ModNum::new(1, self.num_letters.m()));
        }
    }
}
```

The keyboard handler receives each character as it is typed. Keys representable as a `char`
are added to the moving string. The arrow keys change how the string is moving.

As we can see from this example, the capabilities of your PIOS will be
limited to handling keyboard events and displaying text in the VGA buffer. Within that scope,
however, you can achieve quite a lot. I personally enjoyed recreating a version of a
well-known 1980s [arcade classic](https://github.com/gjf2a/ghost_hunter).

This is a pedagogical experiment. I would be interested to hear from anyone who
finds this useful or has suggestions.

Updates:
* **0.4**
  * Updated to pic8259 version 0.10
  * Fixed corresponding maintenance errors.
* **0.2**
  * Added `is_drawable()` function to determine whether a given `char` can be rendered in the 
    VGA buffer.
  * Rewrote README.md to describe the [Pluggable Interrupt Template](https://github.com/gjf2a/pluggable_interrupt_template)   