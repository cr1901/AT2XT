#![no_main]
#![no_std]
#![feature(asm)]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use core::cell::{Cell, RefCell};
use bare_metal::Mutex;
use bit_reverse::BitwiseReverse;
use msp430::interrupt as mspint;
use msp430_rt::entry;
use msp430g2211::{interrupt, Peripherals};
use msp430_atomic::AtomicBool;
use once_cell::unsync::OnceCell;

mod keyfsm;
use keyfsm::{Cmd, ProcReply, Fsm};

mod keybuffer;
use keybuffer::{KeycodeBuffer, KeyIn, KeyOut};

mod driver;
use driver::KeyboardPins;

#[cfg(feature = "use-timer")]
macro_rules! us_to_ticks {
    ($u:expr) => {
        // Timer is 100000 Hz, thus granularity of 10us.
        ($u / 10) + 1
    }
}

#[cfg(not(feature = "use-timer"))]
macro_rules! us_to_ticks {
    ($u:expr) => {
        // Delay is approx clock speed, thus granularity of 0.625us.
        ($u * 16) / 10
    }
}

struct At2XtPeripherals {
    port: msp430g2211::PORT_1_2,
    timer: msp430g2211::TIMER_A2
}


#[cfg(feature = "use-timer")]
static TIMEOUT : AtomicBool = AtomicBool::new(false);
static HOST_MODE : AtomicBool = AtomicBool::new(false);
static DEVICE_ACK : AtomicBool = AtomicBool::new(false);

static IN_BUFFER : Mutex<RefCell<KeycodeBuffer>> = Mutex::new(RefCell::new(KeycodeBuffer::new()));
static KEYBOARD_PINS : KeyboardPins = KeyboardPins::new();
static KEY_IN : Mutex<Cell<KeyIn>> = Mutex::new(Cell::new(KeyIn::new()));
static KEY_OUT : Mutex<Cell<KeyOut>> = Mutex::new(Cell::new(KeyOut::new()));
static PERIPHERALS : Mutex<OnceCell<At2XtPeripherals>> = Mutex::new(OnceCell::new());


#[cfg(feature = "use-timer")]
#[interrupt]
fn TIMERA0() {
    TIMEOUT.store(true);

    mspint::free(|cs| {
        let p = PERIPHERALS.borrow(cs).get().unwrap();
        // Writing 0x0000 stops Timer in MC1.
        p.timer.taccr0.write(|w| unsafe { w.bits(0x0000) });
        // CCIFG will be reset when entering interrupt; no need to clear it.
        // Nesting is disabled, and chances of receiving second CCIFG in the ISR
        // are nonexistant. */
    });
}


#[interrupt]
fn PORT1() {
    mspint::free(|cs| {
        let pins = &PERIPHERALS.borrow(cs).get().unwrap().port;

        if HOST_MODE.load() {
            let mut keyout = KEY_OUT.borrow(cs).get();

            if keyout.is_empty() {
                if keyout.shift_out() {
                    KEYBOARD_PINS.at_data.set(pins);
                } else{
                    KEYBOARD_PINS.at_data.unset(pins);
                }

                // Immediately after sending out the Stop Bit, we should release the lines.
                if keyout.is_empty() {
                    KEYBOARD_PINS.at_idle(pins);
                }
            } else {
                // TODO: Is it possible to get a spurious clock interrupt and
                // thus skip this logic?
                if KEYBOARD_PINS.at_data.is_unset(pins) {
                    DEVICE_ACK.store(true);
                    keyout.clear();
                }
            }

            KEY_OUT.borrow(cs).set(keyout);
            KEYBOARD_PINS.clear_at_clk_int(pins);
        } else {
            let full : bool;
            let mut keyin = KEY_IN.borrow(cs).get();

            // Are the buffer functions safe in nested interrupts? Is it possible to use tokens/manual
            // sync for nested interrupts while not giving up safety?
            // Example: Counter for nest level when updating buffers. If it's ever more than one, panic.
            keyin.shift_in(KEYBOARD_PINS.at_data.is_set(pins));
            full = keyin.is_full();

            if full {
                KEYBOARD_PINS.at_inhibit(pins); // Ask keyboard to not send anything while processing keycode.

                match keyin.take() {
                    Some(k) => {
                        match IN_BUFFER.borrow(cs).try_borrow_mut() {
                            Ok(mut b) => { b.put(k) },
                            Err(_) => { }
                        }
                    },
                    None => { },
                }

                keyin.clear();

                KEYBOARD_PINS.at_idle(pins);
            }

            KEY_IN.borrow(cs).set(keyin);
            KEYBOARD_PINS.clear_at_clk_int(pins);
        }
    });
}


fn init() {
    let p = Peripherals::take().unwrap();

    p.WATCHDOG_TIMER.wdtctl.write(|w| unsafe {
        const PASSWORD: u16 = 0x5A00;
        w.bits(PASSWORD).wdthold().set_bit()
    });

    // Make port idle
    mspint::free(|cs| {
        KEYBOARD_PINS.idle(&p.PORT_1_2);
    });

    p.SYSTEM_CLOCK.bcsctl1.write(|w| w.xt2off().set_bit()
        .rsel3().set_bit()); // XT2 off, Range Select 7.
    p.SYSTEM_CLOCK.bcsctl2.write(|w| w.divs().divs_2()); // Divide submain clock by 4.

    #[cfg(feature = "use-timer")]
    {
        p.TIMER_A2.taccr0.write(|w| unsafe { w.bits(0x0000) });
        p.TIMER_A2.tactl.write(|w| w.tassel().tassel_2()
            .id().id_2().mc().mc_1());
        p.TIMER_A2.tacctl0.write(|w| w.ccie().set_bit());
    }

    mspint::free(|cs| {
        let shared = At2XtPeripherals {
            port : p.PORT_1_2,
            timer: p.TIMER_A2
        };

        PERIPHERALS.borrow(cs).set(shared).ok().unwrap();
    });

    unsafe { mspint::enable(); }
}

#[entry]
fn main() -> ! {
    init();

    send_byte_to_at_keyboard(0xFF);

    let mut loop_cmd : Cmd;
    let mut loop_reply : ProcReply = ProcReply::init();
    let mut fsm_driver : Fsm = Fsm::start();

    loop {
        // Run state machine/send reply. Receive new cmd.
        loop_cmd = fsm_driver.run(&loop_reply).unwrap();

        loop_reply = match loop_cmd {
            Cmd::ClearBuffer => {
                mspint::free(|cs| {
                    // XXX: IN_BUFFER.borrow(cs).borrow_mut() and
                    // IN_BUFFER.borrow(cs).try_borrow_mut().unwrap()
                    // bring in dead formatting code! Use explicit
                    // match for now and handle errors by doing nothing.

                    match IN_BUFFER.borrow(cs).try_borrow_mut() {
                        Ok(mut b) => { b.flush() },
                        Err(_) => { }
                    }
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
                while mspint::free(|cs| {
                    match IN_BUFFER.borrow(cs).try_borrow_mut() {
                        Ok(b) => { b.is_empty() },
                        Err(_) => { true }
                    }
                }) {
                    // If host computer wants to reset
                    if mspint::free(|cs| {
                        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

                        KEYBOARD_PINS
                            .xt_sense.is_unset(port)
                    }) {
                        send_byte_to_at_keyboard(0xFF);
                        send_byte_to_pc(0xAA);
                        xt_reset = true;
                        break;
                    }
                }

                if xt_reset {
                    ProcReply::KeyboardReset
                } else {
                    let mut bits_in = mspint::free(|cs|{
                        match IN_BUFFER.borrow(cs).try_borrow_mut() {
                            Ok(mut b) => {
                                match b.take() {
                                    Some(k) => { k },
                                    None => { 0 },
                                }
                            },
                            Err(_) => { 0 },
                        }
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
    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        if bit == 1 {
            KEYBOARD_PINS.xt_data.set(port);
        } else {
            KEYBOARD_PINS.xt_data.unset(port);
        }

        KEYBOARD_PINS.xt_clk.unset(port);
    });

    delay(us_to_ticks!(55));

    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.xt_clk.set(port);
    });
}

pub fn send_byte_to_pc(mut byte : u8) -> () {
    // The host cannot send data; the only communication it can do with the micro is pull
    // the CLK (reset) and DATA (shift register full) low.
    // Wait for the host to release the lines.

    while mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.xt_clk.is_unset(port) || KEYBOARD_PINS.xt_data.is_unset(port)
    }) { }

    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.xt_out(port);
    });

    send_xt_bit(0);
    send_xt_bit(1);

    for _ in 0..8 {
        send_xt_bit(byte & 0x01); /* Send data... */
		byte = byte >> 1;
    }

    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.xt_in(port);
    });
}

fn send_byte_to_at_keyboard(byte : u8) -> () {
    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;
        let mut key_out = KEY_OUT.borrow(cs).get();

        // XXX: key_out.put(byte).unwrap() is misoptimized
        // and brings in unused panic strings.
        match key_out.put(byte) {
            Ok(_) => { },
            Err(_) => { }
            // Err(_) => { panic!() } // Even this brings in unused panic strings.
        }

        // Safe outside of critical section: As long as HOST_MODE is
        // not set, it's not possible for the interrupt
        // context to touch this variable.
        KEY_OUT.borrow(cs).set(key_out);
        KEYBOARD_PINS.disable_at_clk_int(port);
    });

    /* If/when timer int is enabled, this loop really needs to allow preemption during
    I/O read. Can it be done without overhead of CriticalSection? */
    while mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.at_clk.is_unset(port)
    }) { }


    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.at_inhibit(port);
    });

    delay(us_to_ticks!(100));

    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.at_data.unset(port);
    });

    delay(us_to_ticks!(33));

    mspint::free(|cs| {
        let port = &PERIPHERALS.borrow(cs).get().unwrap().port;

        KEYBOARD_PINS.at_clk.set(port);
        KEYBOARD_PINS.at_clk.mk_in(port);
        KEYBOARD_PINS.clear_at_clk_int(port);

        unsafe {
            KEYBOARD_PINS.enable_at_clk_int(port);
        }
        HOST_MODE.store(true);
        DEVICE_ACK.store(false);
    });

    while !DEVICE_ACK.load() { }

    HOST_MODE.store(false);
}

fn toggle_leds(mask : u8) -> () {
    send_byte_to_at_keyboard(0xED);
    delay(us_to_ticks!(3000));
    send_byte_to_at_keyboard(mask);
}

#[cfg(not(feature = "use-timer"))]
fn delay(n : u16) {
    unsafe {
        asm!(r#"
1:
    dec $0
    jne 1b
    "# :: "{r12}"(n) : "r12" : "volatile");
    }
}

#[cfg(feature = "use-timer")]
fn delay(time : u16) {
    start_timer(time);
    while !TIMEOUT.load() {

    }
}

#[cfg(feature = "use-timer")]
fn start_timer(time : u16) -> () {
    mspint::free(|cs| {
        let timer = &PERIPHERALS.borrow(cs).get().unwrap().timer;
        TIMEOUT.store(false);
        timer.taccr0.write(|w| unsafe { w.bits(time) });
    })
}
