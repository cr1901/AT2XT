#![no_std]
#![no_main]
#![feature(asm)]
#![feature(used)]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(abi_msp430_interrupt)]
#![feature(const_fn)]

extern crate bare_metal;

extern crate volatile_register;
use volatile_register::RW;

extern crate msp430;
use msp430::interrupt::{enable, free};

extern crate bit_reverse;
use bit_reverse::ParallelReverse;

mod keyfsm;
use keyfsm::{Cmd, ProcReply, Fsm};

mod keymap;

mod keybuffer;
use keybuffer::{KeycodeBuffer, KeyIn, KeyOut};

mod driver;
use driver::KeyboardPins;


global_asm!(r#"
    .globl reset_handler
reset_handler:
    mov #__stack, r1
    br #main
"#);

#[used]
#[link_section = "__interrupt_vector_reset"]
static RESET_VECTOR: unsafe extern "msp430-interrupt" fn() = reset_handler;

extern "msp430-interrupt" {
    fn reset_handler();
}

#[used]
#[link_section = "__interrupt_vector_timer0_a0"]
static TIM0_VECTOR: unsafe extern "msp430-interrupt" fn() = timer0_handler;

unsafe extern "msp430-interrupt" fn timer0_handler() {
    // you can do something here
}

#[used]
#[link_section = "__interrupt_vector_port1"]
static PORTA_VECTOR: unsafe extern "msp430-interrupt" fn() = porta_handler;

unsafe extern "msp430-interrupt" fn porta_handler() {
    if HOST_MODE {
        free(|cs| {
            //
            if !KEY_OUT.is_empty() {
                if KEY_OUT.shift_out(&cs) {
                    KEYBOARD_PINS.at_data.set(&cs);
                } else{
                    KEYBOARD_PINS.at_data.unset(&cs);
                }

                // Immediately after sending out the Stop Bit, we should release the lines.
                if KEY_OUT.is_empty() {
                    KEYBOARD_PINS.at_idle(&cs);
                }
            } else {
                if KEYBOARD_PINS.at_data.is_unset() {
                    DEVICE_ACK = true;
                    KEY_OUT.clear(&cs);
                }
            }

            KEYBOARD_PINS.clear_at_clk_int(&cs);
        });
    } else {
        // Interrupts already disabled, and doesn't make sense to nest them, since bits need
        // to be received in order. Just wrap whole block.
        free(|cs| {
            let full : bool;

            // Are the buffer functions safe in nested interrupts? Is it possible to use tokens/manual
            // sync for nested interrupts while not giving up safety?
            // Example: Counter for nest level when updating buffers. If it's ever more than one, panic.
            KEY_IN.shift_in(KEYBOARD_PINS.at_data.is_set(), &cs);
            full = KEY_IN.is_full();

            if full {
                KEYBOARD_PINS.at_inhibit(&cs); // Ask keyboard to not send anything while processing keycode.

                IN_BUFFER.put(KEY_IN.take(&cs).unwrap(), &cs);
                KEY_IN.clear(&cs);

                KEYBOARD_PINS.at_idle(&cs);
            }
            KEYBOARD_PINS.clear_at_clk_int(&cs);
        });
    }
}

extern "C" {
    static mut WDTCTL: RW<u16>;
    static mut BCSCTL1: RW<u8>;
    static mut BCSCTL2: RW<u8>;
    // TACCR0
    // TACTL
    // TACCTL0
}

static mut IN_BUFFER : KeycodeBuffer = KeycodeBuffer::new();
static mut KEY_IN : KeyIn = KeyIn::new();
static mut KEY_OUT : KeyOut = KeyOut::new();
static mut HOST_MODE : bool = false;
static mut DEVICE_ACK : bool = false;
static KEYBOARD_PINS : KeyboardPins = KeyboardPins::new();

#[no_mangle]
pub extern "C" fn main() -> ! {
    unsafe {
        WDTCTL.write(0x5A00 + 0x80); // WDTPW + WDTHOLD
    }

    free(|cs| {
        KEYBOARD_PINS.idle(&cs); // FIXME: Can we make this part of new()?
        unsafe {
            KEY_OUT.clear(&cs); // Currently, no support for DATA section, so what should be a const fn
            // to initialize KEY_OUT buffer to empty (pos > 10, i.e. nonzero) must be done
            // manually.
            // Upon further bug flushing: All const fns do not currently work; assumes runtime
            // support which I do not currently have.
            IN_BUFFER.flush(&cs);
            KEY_IN.clear(&cs);
            HOST_MODE = false;
            DEVICE_ACK = false;

        }
    });

    unsafe {
        BCSCTL1.write(0x88); // XT2 off, Range Select 7.
        BCSCTL2.write(0x04); // Divide submain clock by 4.
        enable(); // Enable interrupts.
    }

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
                    unsafe { IN_BUFFER.flush(&cs); }
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
                unsafe {
                    'idle: while free(|cs| { IN_BUFFER.is_empty(&cs) }) {
                        // If host computer wants to reset
                        if KEYBOARD_PINS.xt_sense.is_unset() {
                            send_byte_to_at_keyboard(0xFF);
                            send_byte_to_pc(0xAA);
                            xt_reset = true;
                            break;
                        }
                    }

                    if xt_reset {
                        ProcReply::KeyboardReset
                    } else {
                        let mut bits_in = free(|cs|{
                            IN_BUFFER.take(&cs).unwrap()
                        });

                        bits_in = bits_in & !(0x4000 + 0x0001); // Mask out start/stop bit.
                        bits_in = bits_in >> 2; // Remove stop bit and parity bit (FIXME: Check parity).
                        ProcReply::GrabbedKey((bits_in as u8).swap_bits())
                    }
                }
            },

        }
    }
}

pub fn send_xt_bit(bit : u8) -> () {
    free(|cs| {
        if bit == 1 {
            KEYBOARD_PINS.xt_data.set(&cs);
        } else {
            KEYBOARD_PINS.xt_data.unset(&cs);
        }

        KEYBOARD_PINS.xt_clk.unset(&cs);
    });

    unsafe { delay(88); } // 55 microseconds at 1.6 MHz

    free(|cs| {
        KEYBOARD_PINS.xt_clk.set(&cs);
    });
}

pub fn send_byte_to_pc(mut byte : u8) -> () {
    // The host cannot send data; the only communication it can do with the micro is pull
    // the CLK (reset) and DATA (shift register full) low.
    // Wait for the host to release the lines.

    while KEYBOARD_PINS.xt_clk.is_unset() || KEYBOARD_PINS.xt_data.is_unset() {

    }

    free(|cs| {
        KEYBOARD_PINS.xt_out(&cs);
    });

    send_xt_bit(0);
    send_xt_bit(1);

    for _ in 0..8 {
        send_xt_bit((byte & 0x01)); /* Send data... */
		byte = byte >> 1;
    }

    free(|cs| {
        KEYBOARD_PINS.xt_in(&cs);
    });
}

fn send_byte_to_at_keyboard(byte : u8) -> () {
    free(|cs| {
        unsafe {
            KEY_OUT.put(byte, &cs).unwrap();
        }   // Safe outside of critical section: As long as HOST_MODE is
            // not set, it's not possible for the interrupt
            // context to touch this variable.
        KEYBOARD_PINS.disable_at_clk_int();
    });

    while KEYBOARD_PINS.at_clk.is_unset() {

    }

    free(|cs| {
        KEYBOARD_PINS.at_inhibit(&cs);
    });

    unsafe { delay(160); } // 100 microseconds

    free(|cs| {
        KEYBOARD_PINS.at_data.unset(cs);
    });

    unsafe { delay(53); } // 33 microseconds

    free(|cs| {
        KEYBOARD_PINS.at_clk.set(cs);
        KEYBOARD_PINS.at_clk.mk_in(cs);
        KEYBOARD_PINS.clear_at_clk_int(cs);
        unsafe {
            KEYBOARD_PINS.enable_at_clk_int();
            HOST_MODE = true;
            DEVICE_ACK= false;
        }
    });

    // FIXME: Truly unsafe until I create a mutex later. Data race can occur (but unlikely, for
    // the sake of testing).
    unsafe {
        while free(|cs| {
            let _ = cs;
            !DEVICE_ACK
        }) { }

        free(|cs| {
            let _ = cs;
            HOST_MODE = false;
        })
    }
}

fn toggle_leds(mask : u8) -> () {
    send_byte_to_at_keyboard(0xED);
    unsafe { delay(5000); }
    send_byte_to_at_keyboard(mask);
}


unsafe fn delay(n: u16) {
    asm!(r#"
1:
    dec $0
    jne 1b
    "# :: "{r12}"(n) : "r12" : "volatile");
}

#[used]
#[no_mangle]
#[lang = "panic_fmt"]
#[allow(private_no_mangle_fns)]
extern "C" fn panic_fmt() -> ! {
    loop {
        unsafe { asm!("nop" ::: "memory" : "volatile"); }
    }
}
