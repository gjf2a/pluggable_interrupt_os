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

This is a pedagogical experiment. I would be interested to hear from anyone who 
finds this useful or has suggestions. 