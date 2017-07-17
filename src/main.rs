#![no_std]
#![feature(asm)]
#![feature(proc_macro)]
#![feature(abi_msp430_interrupt)]
#![feature(const_fn)]

use core::cell::{Cell, RefCell};

extern crate bare_metal;
use bare_metal::{Mutex};

extern crate volatile_register;

extern crate msp430;
use msp430::interrupt::{enable, free};
use msp430::asm;

extern crate bit_reverse;
use bit_reverse::ParallelReverse;

extern crate msp430g2211;

#[macro_use(task)]
extern crate msp430_rtfm as rtfm;
use rtfm::app;

mod keyfsm;
use keyfsm::{Cmd, ProcReply, Fsm};

mod keybuffer;
use keybuffer::{KeycodeBuffer, KeyIn, KeyOut};

mod driver;
use driver::KeyboardPins;


macro_rules! set_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => { $w.bits($r.bits() | $m) };
}

macro_rules! clear_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => { $w.bits($r.bits() & !$m) };
}



app! {
    device: msp430g2211,

    idle: {
        resources: [KEYBOARD_PINS, PORT_1_2],
    },

    resources: {
        KEYBOARD_PINS : KeyboardPins = KeyboardPins::new();
    },

    tasks: {
        PORT1: {
            resources: [PORT_1_2],
        },
    },
}


/* interrupt!(TIMERA0, timer0_handler);
fn timer0_handler() {
    // you can do something here
} */


//task!(PORT1)


//task!(PORT1, porta_handler);
/* fn porta_handler(r: PORT1::Resources) {
    // Interrupts already disabled, and doesn't make sense to nest them, since bits need
    // to be received in order. Just wrap whole block.
    free(|cs| {
        if HOST_MODE.borrow(cs).get() {
            let mut key_out = KEY_OUT.borrow(cs).get();
            if !key_out.is_empty() {
                if key_out.shift_out() {
                    //KEYBOARD_PINS.at_data.set(&cs);
                } else{
                    //KEYBOARD_PINS.at_data.unset(&cs);
                }

                // Immediately after sending out the Stop Bit, we should release the lines.
                if key_out.is_empty() {
                    //KEYBOARD_PINS.at_idle(&cs);
                }
            } else {
                if //KEYBOARD_PINS.at_data.is_unset() {
                    DEVICE_ACK.borrow(cs).set(true);
                    key_out.clear();
                }
            }

            KEY_OUT.borrow(cs).set(key_out);
            //KEYBOARD_PINS.clear_at_clk_int(&cs);
        } else {
            let full : bool;
            let mut key_in = KEY_IN.borrow(cs).get();

            // Are the buffer functions safe in nested interrupts? Is it possible to use tokens/manual
            // sync for nested interrupts while not giving up safety?
            // Example: Counter for nest level when updating buffers. If it's ever more than one, panic.
            key_in.shift_in(//KEYBOARD_PINS.at_data.is_set());
            full = key_in.is_full();

            if full {
                //KEYBOARD_PINS.at_inhibit(&cs); // Ask keyboard to not send anything while processing keycode.

                IN_BUFFER.borrow(cs)
                    .borrow_mut()
                    .put(key_in.take().unwrap());
                key_in.clear();

                //KEYBOARD_PINS.at_idle(&cs);
            }

            KEY_IN.borrow(cs).set(key_in);
            //KEYBOARD_PINS.clear_at_clk_int(&cs);
        }
    });
} */

static IN_BUFFER : Mutex<RefCell<KeycodeBuffer>> = Mutex::new(RefCell::new(KeycodeBuffer::new()));
static KEY_IN : Mutex<Cell<KeyIn>> = Mutex::new(Cell::new(KeyIn::new()));
static KEY_OUT : Mutex<Cell<KeyOut>> = Mutex::new(Cell::new(KeyOut::new()));
static HOST_MODE : Mutex<Cell<bool>> = Mutex::new(Cell::new(false));
static DEVICE_ACK : Mutex<Cell<bool>> = Mutex::new(Cell::new(false));

const AT_CLK: u8 = 1;
const XT_SENSE: u8 = 1 << 1;
const XT_CLK: u8 = 1 << 2;
const XT_DATA: u8 = 1 << 3;
const AT_DATA: u8 = 1 << 4;


fn init(p: init::Peripherals, r: init::Resources) {
    p.WATCHDOG_TIMER.wdtctl.write(|w| unsafe {
        const PASSWORD: u16 = 0x5A00;
        w.bits(PASSWORD).wdthold().set_bit()
    });

    // Make port idle
    r.KEYBOARD_PINS.idle(p.PORT_1_2);

    p.SYSTEM_CLOCK.bcsctl1.write(|w| w.xt2off().set_bit()
        .rsel3().set_bit()); // XT2 off, Range Select 7.
    p.SYSTEM_CLOCK.bcsctl2.write(|w| w.divs().divs_2()); // Divide submain clock by 4.
}

fn idle(r: idle::Resources) -> ! {
    //send_byte_to_at_keyboard(0xFF);

    loop {
        // NOTE it seems this infinite loop gets optimized to `undef` if the NOP
        // is removed
        asm::nop()
    }
}


#[no_mangle]
pub extern "C" fn used_to_be_main() -> ! {
    send_byte_to_at_keyboard(0xFF);

    let mut loop_cmd : Cmd;
    let mut loop_reply : ProcReply = ProcReply::init();
    let mut fsm_driver : Fsm = Fsm::start();

    'get_command: loop {
        // Run state machine/send reply. Receive new cmd.
        loop_cmd = fsm_driver.run(&loop_reply).unwrap();

        loop_reply = match loop_cmd {
            Cmd::ClearBuffer => {
                free(|cs| {
                    IN_BUFFER.borrow(cs)
                        .borrow_mut()
                        .flush();
                });
                ProcReply::ClearedBuffer
            },
            Cmd::ToggleLed(m) => {
                toggle_leds(m);
                ProcReply::LedToggled(m)
            }
            Cmd::SendXTKey(k) => {
                send_byte_to_pc(k);
                ProcReply::SentKey(k)
            },
            Cmd::WaitForKey => {
                // The micro spends the majority of its life idle. It is possible for the host PC and
                // the keyboard to send data to the micro at the same time. To keep control flow simple,
                // the micro will only respond to host PC acknowledge requests if its idle.
                let mut xt_reset : bool = false;
                'idle: while free(|cs| { IN_BUFFER.borrow(cs).borrow().is_empty() }) {
                    // If host computer wants to reset
                    /* if //KEYBOARD_PINS.xt_sense.is_unset() {
                        send_byte_to_at_keyboard(0xFF);
                        send_byte_to_pc(0xAA);
                        xt_reset = true;
                        break;
                    } */
                }

                if xt_reset {
                    ProcReply::KeyboardReset
                } else {
                    let mut bits_in = free(|cs|{
                        IN_BUFFER.borrow(cs)
                            .borrow_mut()
                            .take().unwrap()
                    });

                    bits_in = bits_in & !(0x4000 + 0x0001); // Mask out start/stop bit.
                    bits_in = bits_in >> 2; // Remove stop bit and parity bit (FIXME: Check parity).
                    ProcReply::GrabbedKey((bits_in as u8).swap_bits())
                }
            },

        }
    }
}

pub fn send_xt_bit(bit : u8) -> () {
    free(|cs| {
        if bit == 1 {
            //KEYBOARD_PINS.xt_data.set(&cs);
        } else {
            //KEYBOARD_PINS.xt_data.unset(&cs);
        }

        //KEYBOARD_PINS.xt_clk.unset(&cs);
    });

    delay(88); // 55 microseconds at 1.6 MHz

    free(|cs| {
        //KEYBOARD_PINS.xt_clk.set(&cs);
    });
}

pub fn send_byte_to_pc(mut byte : u8) -> () {
    // The host cannot send data; the only communication it can do with the micro is pull
    // the CLK (reset) and DATA (shift register full) low.
    // Wait for the host to release the lines.

    /* while //KEYBOARD_PINS.xt_clk.is_unset() || //KEYBOARD_PINS.xt_data.is_unset() {

    } */

    free(|cs| {
        //KEYBOARD_PINS.xt_out(&cs);
    });

    send_xt_bit(0);
    send_xt_bit(1);

    for _ in 0..8 {
        send_xt_bit((byte & 0x01)); /* Send data... */
		byte = byte >> 1;
    }

    free(|cs| {
        //KEYBOARD_PINS.xt_in(&cs);
    });
}

fn send_byte_to_at_keyboard(byte : u8) -> () {
    free(|cs| {
        let mut key_out = KEY_OUT.borrow(cs).get();
        key_out.put(byte).unwrap();
        // Safe outside of critical section: As long as HOST_MODE is
        // not set, it's not possible for the interrupt
        // context to touch this variable.
        KEY_OUT.borrow(cs).set(key_out);
        //KEYBOARD_PINS.disable_at_clk_int();
    });

    /* while //KEYBOARD_PINS.at_clk.is_unset() {

    } */

    free(|cs| {
        //KEYBOARD_PINS.at_inhibit(&cs);
    });

    delay(160); // 100 microseconds

    free(|cs| {
        //KEYBOARD_PINS.at_data.unset(cs);
    });

    delay(53); // 33 microseconds

    free(|cs| {
        //KEYBOARD_PINS.at_clk.set(cs);
        //KEYBOARD_PINS.at_clk.mk_in(cs);
        //KEYBOARD_PINS.clear_at_clk_int(cs);
        unsafe {
            //KEYBOARD_PINS.enable_at_clk_int();
        }
        HOST_MODE.borrow(cs).set(true);
        DEVICE_ACK.borrow(cs).set(false);
    });

    while free(|cs| {
        !DEVICE_ACK.borrow(cs).get()
    }) { }

    free(|cs| {
        HOST_MODE.borrow(cs).set(false);
    })
}

fn toggle_leds(mask : u8) -> () {
    send_byte_to_at_keyboard(0xED);
    delay(5000);
    send_byte_to_at_keyboard(mask);
}

fn delay(n: u16) {
    unsafe {
        asm!(r#"
1:
    dec $0
    jne 1b
    "# :: "{r12}"(n) : "r12" : "volatile");
    }
}
