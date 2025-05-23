use crate::HandlerTable;
use crate::{gdt, println};
use lazy_static::lazy_static;
use pic8259::ChainedPics;
use spin::Mutex;
use x86_64::structures::idt::{InterruptDescriptorTable, InterruptStackFrame};

// This code is largely Copyright (c) 2019 Philipp Oppermann.
// Gabriel Ferrer added:
// - HANDLERS variable.
// - Use of HANDLERS in init_idt, timer_interrupt_handler, keyboard_interrupt_handler
// - enum WhichInterrupt and the variable to hold its value

#[derive(Copy, Clone, Debug)]
pub enum WhichInterrupt {
    Timer, Keyboard, Breakpoint,
}

lazy_static! {
    static ref LAST_INTERRUPT: Mutex<Option<WhichInterrupt>> = Mutex::new(None);
}

lazy_static! {
    static ref HANDLERS: Mutex<Option<HandlerTable>> = Mutex::new(None);
}

lazy_static! {
    static ref IDT: InterruptDescriptorTable = {
        let mut idt = InterruptDescriptorTable::new();
        idt.breakpoint.set_handler_fn(breakpoint_handler);
        unsafe {
            idt.double_fault
                .set_handler_fn(double_fault_handler)
                .set_stack_index(gdt::DOUBLE_FAULT_IST_INDEX);
        }
        idt[InterruptIndex::Timer.as_u8()].set_handler_fn(timer_interrupt_handler);
        idt[InterruptIndex::Keyboard.as_u8()].set_handler_fn(keyboard_interrupt_handler);
        idt
    };
}

/// Initializes the interrupt table with the given interrupt handlers.
pub fn init_idt(handlers: HandlerTable) {
    *(HANDLERS.lock()) = Some(handlers);
    IDT.load();
}

extern "x86-interrupt" fn breakpoint_handler(stack_frame: InterruptStackFrame) {
    *(LAST_INTERRUPT.lock()) = Some(WhichInterrupt::Breakpoint);
    println!("EXCEPTION: BREAKPOINT\n{:#?}", stack_frame);
}

extern "x86-interrupt" fn double_fault_handler(
    stack_frame: InterruptStackFrame,
    _error_code: u64,
) -> ! {
    let last = LAST_INTERRUPT.lock();
    panic!("EXCEPTION: DOUBLE FAULT (last interrupt: {:?})\n{:#?}", last, stack_frame);
}

const PIC_1_OFFSET: u8 = 32;
const PIC_2_OFFSET: u8 = PIC_1_OFFSET + 8;

pub static PICS: Mutex<ChainedPics> =
    Mutex::new(unsafe { ChainedPics::new(PIC_1_OFFSET, PIC_2_OFFSET) });

#[derive(Debug, Clone, Copy)]
#[repr(u8)]
enum InterruptIndex {
    Timer = PIC_1_OFFSET,
    Keyboard,
}

impl InterruptIndex {
    fn as_u8(self) -> u8 {
        self as u8
    }
}

extern "x86-interrupt" fn timer_interrupt_handler(_stack_frame: InterruptStackFrame) {
    *(LAST_INTERRUPT.lock()) = Some(WhichInterrupt::Timer);
    let h = &*HANDLERS.lock();
    if let Some(handler) = h {
        handler.handle_timer();
    }
    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Timer.as_u8());
    }
}

extern "x86-interrupt" fn keyboard_interrupt_handler(_stack_frame: InterruptStackFrame) {
    *(LAST_INTERRUPT.lock()) = Some(WhichInterrupt::Keyboard);
    use pc_keyboard::{layouts, HandleControl, Keyboard, ScancodeSet1};
    use x86_64::instructions::port::Port;

    lazy_static! {
        static ref KEYBOARD: Mutex<Keyboard<layouts::Us104Key, ScancodeSet1>> =
            Mutex::new(Keyboard::new(
                ScancodeSet1::new(),
                layouts::Us104Key,
                HandleControl::Ignore
            ));
    }

    let mut keyboard = KEYBOARD.lock();
    let mut port = Port::new(0x60);

    let scancode: u8 = unsafe { port.read() };
    if let Ok(Some(key_event)) = keyboard.add_byte(scancode) {
        if let Some(key) = keyboard.process_keyevent(key_event) {
            let h = &*HANDLERS.lock();
            if let Some(handler) = h {
                handler.handle_keyboard(key);
            }
        }
    }

    unsafe {
        PICS.lock()
            .notify_end_of_interrupt(InterruptIndex::Keyboard.as_u8());
    }
}
