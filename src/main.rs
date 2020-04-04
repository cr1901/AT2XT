#![no_main]
#![no_std]
#![feature(abi_msp430_interrupt)]

extern crate panic_msp430;

use bare_metal::{CriticalSection, Mutex};
use bit_reverse::BitwiseReverse;
use core::cell::{Cell, RefCell};
use msp430::interrupt as mspint;
use msp430_atomic::AtomicBool;
use msp430_rt::entry;
use msp430g2211::{interrupt, Peripherals};

mod keyfsm;
use keyfsm::{Cmd, Fsm, LedMask, ProcReply};

mod keybuffer;
use keybuffer::{KeyIn, KeyOut, KeycodeBuffer};

mod driver;
use driver::Pins;

mod peripheral;
use peripheral::At2XtPeripherals;

macro_rules! delay_us {
    ($u:expr) => {
        // Timer is 100000 Hz, thus granularity of 10us.
        delay(($u / 10) + 1)
    };
}

static TIMEOUT: AtomicBool = AtomicBool::new(false);
static HOST_MODE: AtomicBool = AtomicBool::new(false);
static DEVICE_ACK: AtomicBool = AtomicBool::new(false);

static IN_BUFFER: Mutex<RefCell<KeycodeBuffer>> = Mutex::new(RefCell::new(KeycodeBuffer::new()));
static KEY_IN: Mutex<Cell<KeyIn>> = Mutex::new(Cell::new(KeyIn::new()));
static KEY_OUT: Mutex<Cell<KeyOut>> = Mutex::new(Cell::new(KeyOut::new()));

#[interrupt]
fn TIMERA0(cs: CriticalSection) {
    TIMEOUT.store(true);

    // Use unwrap b/c within interrupt handlers, if we can't get access to
    // peripherals right away, there's no point in continuing.
    let timer: &msp430g2211::TIMER_A2 = At2XtPeripherals::periph_ref_map(&cs).unwrap();
    // Writing 0x0000 stops Timer in MC1.
    timer.taccr0.write(|w| unsafe { w.bits(0x0000) });
    // CCIFG will be reset when entering interrupt; no need to clear it.
    // Nesting is disabled, and chances of receiving second CCIFG in the ISR
    // are nonexistant.
}

#[interrupt]
fn PORT1(cs: CriticalSection) {
    let port = At2XtPeripherals::periph_ref_map(&cs).unwrap();

    if HOST_MODE.load() {
        let mut keyout = KEY_OUT.borrow(&cs).get();

        if let Some(k) = keyout.shift_out() {
            if k {
                driver::set(port, Pins::AT_DATA);
            } else {
                driver::unset(port, Pins::AT_DATA);
            }

            // Immediately after sending out the Stop Bit, we should release the lines.
            if keyout.is_empty() {
                driver::at_idle(port);
            }
        } else {
            // TODO: Is it possible to get a spurious clock interrupt and
            // thus skip this logic?
            if driver::is_unset(port, Pins::AT_DATA) {
                DEVICE_ACK.store(true);
                keyout.clear();
            }
        }

        KEY_OUT.borrow(&cs).set(keyout);
        driver::clear_at_clk_int(port);
    } else {
        let mut keyin = KEY_IN.borrow(&cs).get();

        // Are the buffer functions safe in nested interrupts? Is it possible to use tokens/manual
        // sync for nested interrupts while not giving up safety?
        // Example: Counter for nest level when updating buffers. If it's ever more than one, panic.
        if keyin.shift_in(driver::is_set(port, Pins::AT_DATA)).is_err() {
            driver::at_inhibit(port); // Ask keyboard to not send anything while processing keycode.

            if let Some(k) = keyin.take() {
                if let Ok(mut b) = IN_BUFFER.borrow(&cs).try_borrow_mut() {
                    // Dropping keys when the buffer is full is in line
                    // with what AT/XT hosts do. Saves 2 bytes on panic :)!
                    #[allow(clippy::let_underscore_must_use)]
                    {
                        let _ = b.put(k);
                    }
                }
            }

            keyin.clear();

            driver::at_idle(port);
        }

        KEY_IN.borrow(&cs).set(keyin);
        driver::clear_at_clk_int(port);
    }
}

fn init(cs: CriticalSection) {
    let p = Peripherals::take().unwrap();

    p.WATCHDOG_TIMER.wdtctl.write(|w| unsafe {
        const PASSWORD: u16 = 0x5A00;
        w.bits(PASSWORD).wdthold().set_bit()
    });

    driver::idle(&p.PORT_1_2);

    p.SYSTEM_CLOCK
        .bcsctl1
        .write(|w| w.xt2off().set_bit().rsel3().set_bit()); // XT2 off, Range Select 7.
    p.SYSTEM_CLOCK.bcsctl2.write(|w| w.divs().divs_2()); // Divide submain clock by 4.

    p.TIMER_A2.taccr0.write(|w| unsafe { w.bits(0x0000) });
    p.TIMER_A2
        .tactl
        .write(|w| w.tassel().tassel_2().id().id_2().mc().mc_1());
    p.TIMER_A2.tacctl0.write(|w| w.ccie().set_bit());

    let shared = At2XtPeripherals {
        port: p.PORT_1_2,
        timer: p.TIMER_A2,
    };

    At2XtPeripherals::init(shared, &cs).unwrap();

    drop(cs);
    unsafe {
        mspint::enable();
    }
}

#[entry]
fn main(cs: CriticalSection) -> ! {
    init(cs);

    send_byte_to_at_keyboard(Cmd::RESET).unwrap();

    let mut loop_cmd: Cmd;
    let mut loop_reply: ProcReply = ProcReply::init();
    let mut fsm_driver: Fsm = Fsm::start();

    loop {
        // Run state machine/send reply. Receive new cmd.
        loop_cmd = fsm_driver.run(&loop_reply).unwrap();

        loop_reply = match loop_cmd {
            Cmd::ClearBuffer => {
                mspint::free(|cs| {
                    // XXX: IN_BUFFER.borrow(cs).borrow_mut() and
                    // IN_BUFFER.borrow(cs).try_borrow_mut().unwrap()
                    // bring in dead formatting code! Use explicit
                    // if-let for now and handle errors by doing nothing.

                    if let Ok(mut b) = IN_BUFFER.borrow(cs).try_borrow_mut() {
                        b.flush()
                    }
                });
                ProcReply::ClearedBuffer
            }
            Cmd::ToggleLed(m) => {
                toggle_leds(m).unwrap();
                ProcReply::LedToggled(m)
            }
            Cmd::SendXTKey(k) => {
                send_byte_to_pc(k).unwrap();
                ProcReply::SentKey(k)
            }
            Cmd::WaitForKey => {
                // The micro spends the majority of its life idle. It is possible for the host PC and
                // the keyboard to send data to the micro at the same time. To keep control flow simple,
                // the micro will only respond to host PC acknowledge requests if its idle.
                fn reset_requested() -> bool {
                    mspint::free(|cs| {
                        let port = At2XtPeripherals::periph_ref_map(cs).unwrap();

                        driver::is_unset(port, Pins::XT_SENSE)
                    })
                };

                let mut xt_reset: bool = false;

                while mspint::free(|cs| {
                    IN_BUFFER
                        .borrow(cs)
                        .try_borrow_mut()
                        .map_or(true, |b| b.is_empty())
                }) {
                    // If host computer wants to reset
                    if reset_requested() {
                        send_byte_to_at_keyboard(Cmd::RESET).unwrap();
                        send_byte_to_pc(Cmd::SELF_TEST_PASSED).unwrap();
                        xt_reset = true;
                        break;
                    }
                }

                if xt_reset {
                    ProcReply::KeyboardReset
                } else {
                    let mut bits_in = mspint::free(|cs| {
                        IN_BUFFER
                            .borrow(cs)
                            .try_borrow_mut()
                            .map_or(0, |mut b| b.take().unwrap_or(0))
                    });

                    bits_in &= !(0x4000 + 0x0001); // Mask out start/stop bit.
                    bits_in >>= 2; // Remove stop bit and parity bit (FIXME: Check parity).

                    // Truncate is in fact what I want here, so allow lint.
                    #[allow(clippy::as_conversions)]
                    ProcReply::GrabbedKey((bits_in as u8).swap_bits())
                }
            }
        }
    }
}

pub fn send_xt_bit(bit: u8) -> Result<(), ()> {
    mspint::free(|cs| {
        let port = At2XtPeripherals::periph_ref(&cs)?;

        if bit == 1 {
            driver::set(port, Pins::XT_DATA);
        } else {
            driver::unset(port, Pins::XT_DATA);
        }

        driver::unset(port, Pins::XT_CLK);

        Ok(())
    })?;

    delay_us!(55)?;

    mspint::free(|cs| {
        let port = At2XtPeripherals::periph_ref(&cs)?;

        driver::set(port, Pins::XT_CLK);
        Ok(())
    })?;

    Ok(())
}

pub fn send_byte_to_pc(mut byte: u8) -> Result<(), ()> {
    fn wait_for_host() -> Result<bool, ()> {
        mspint::free(|cs| {
            let port = At2XtPeripherals::periph_ref(&cs)?;

            let clk_or_data_unset =
                driver::is_unset(port, Pins::XT_CLK) || driver::is_unset(port, Pins::XT_DATA);

            if !clk_or_data_unset {
                driver::xt_out(port);
            }

            Ok(clk_or_data_unset)
        })
    };

    // The host cannot send data; the only communication it can do with the micro is pull
    // the CLK (reset) and DATA (shift register full) low.
    // Wait for the host to release the lines.
    while wait_for_host()? {}

    send_xt_bit(0)?;
    send_xt_bit(1)?;

    for _ in 0..8 {
        send_xt_bit(byte & 0x01)?; /* Send data... */
        byte >>= 1;
    }

    mspint::free(|cs| {
        let port = At2XtPeripherals::periph_ref(&cs)?;

        driver::xt_in(port);
        Ok(())
    })?;

    Ok(())
}

fn send_byte_to_at_keyboard(byte: u8) -> Result<(), ()> {
    // TODO: What does the AT keyboard protocol say about retrying xfers
    // when inhibiting communication? Does the keyboard retry from the beginning
    // or from the interrupted bit? Right now, we don't flush KeyIn, so
    // we do it from the interrupted bit. This seems to work fine.
    fn wait_for_at_keyboard() -> Result<bool, ()> {
        mspint::free(|cs| {
            let port = At2XtPeripherals::periph_ref(&cs)?;

            let unset = driver::is_unset(port, Pins::AT_CLK);

            if !unset {
                driver::at_inhibit(port);
            }

            Ok(unset)
        })
    }

    mspint::free(|cs| {
        let port = At2XtPeripherals::periph_ref(&cs)?;

        let mut key_out = KEY_OUT.borrow(cs).get();

        key_out.put(byte).unwrap();

        // Safe outside of critical section: As long as HOST_MODE is
        // not set, it's not possible for the interrupt
        // context to touch this variable.
        KEY_OUT.borrow(cs).set(key_out);
        driver::disable_at_clk_int(port);
        Ok(())
    })?;

    /* If/when timer int is enabled, this loop really needs to allow preemption during
    I/O read. Can it be done without overhead of CriticalSection? */
    while wait_for_at_keyboard()? {}

    delay_us!(100)?;

    mspint::free(|cs| {
        let port = At2XtPeripherals::periph_ref(&cs)?;

        driver::unset(port, Pins::AT_DATA);
        Ok(())
    })?;

    delay_us!(33)?;

    mspint::free(|cs| {
        let port = At2XtPeripherals::periph_ref(&cs)?;

        driver::set(port, Pins::AT_CLK);
        driver::mk_in(port, Pins::AT_CLK);
        driver::clear_at_clk_int(port);

        unsafe {
            driver::enable_at_clk_int(port);
        }
        HOST_MODE.store(true);
        DEVICE_ACK.store(false);
        Ok(())
    })?;

    while !DEVICE_ACK.load() {}

    HOST_MODE.store(false);

    Ok(())
}

fn toggle_leds(mask: LedMask) -> Result<(), ()> {
    send_byte_to_at_keyboard(Cmd::SET_LEDS)?;
    delay_us!(3000)?;
    send_byte_to_at_keyboard(mask.bits())?;
    Ok(())
}

fn delay(time: u16) -> Result<(), ()> {
    start_timer(time)?;
    while !TIMEOUT.load() {}

    Ok(())
}

fn start_timer(time: u16) -> Result<(), ()> {
    mspint::free(|cs| {
        let timer: &msp430g2211::TIMER_A2 = At2XtPeripherals::periph_ref(&cs)?;

        TIMEOUT.store(false);
        timer.taccr0.write(|w| unsafe { w.bits(time) });
        Ok(())
    })
}
