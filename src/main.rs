#![no_std]
#![no_main]
#![feature(asm)]
#![feature(used)]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(abi_msp430_interrupt)]
#![feature(const_fn)]

extern crate volatile_register;
use volatile_register::RW;
use volatile_register::RO;

mod keymap;
use keymap::to_xt;

mod keybuffer;
use keybuffer::KeycodeBuffer;


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
    // you can do something here
}

extern "C" {
    static mut WDTCTL: RW<u16>;
    static P1IN: RO<u8>;
    static mut P1IE: RW<u8>;
    static mut P1IES: RW<u8>;
    static mut P1IFG: RW<u8>;
    static mut P1DIR: RW<u8>;
    static mut P1OUT: RW<u8>;
}

static mut IN_BUFFER : KeycodeBuffer = KeycodeBuffer::new();

// AT Keyboard to micro
const PORT1_DATA: u8 = 0b0001_0000;
const PORT1_CLK: u8 = 0b0000_0001;

// Sample clock from host- host can pull it low
const CAPTURE_P2CLK: u8 = 0b0000_0010;

// Micro to host
const PORT2_DATA: u8 = 0b0000_1000;
const PORT2_DATA_PIN: u8 = 3;
const PORT2_CLK: u8 = 0b0000_0100;


#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    WDTCTL.write(0x5A00 + 0x80); // WDTPW + WDTHOLD
    P1DIR.write(0x00); // Sense lines as input for now. P1.0 is output to keyboard. P1.1 is input from keyboard.
    //P1IFG.modify(|x| x & !PORT1_CLK);
    //P1IES.modify(|x| x | PORT1_CLK);
    //P1IE.modify(|x| x | PORT1_CLK);

    'get_command: loop {
        // P1OUT.modify(|x| !x);
        // delay(40000);

        // Run state machine/send reply. Receive new cmd.

        // The micro spends the majority of its life idle. It is possible for the host PC and
        // the keyboard to send data to the micro at the same time. To keep control flow simple,
        // the micro will only respond to host PC acknowledge requests if its idle.
        'idle: while IN_BUFFER.is_empty() {

            // if host computer wants to reset
            continue 'get_command;
        }
    }
}

pub unsafe fn send_xt_bit(bit : u8) -> () {
    P1OUT.modify(|x| x & !PORT2_DATA);
    P1OUT.modify(|x| x | (bit << PORT2_DATA_PIN));
    P1OUT.modify(|x| x & !PORT2_CLK);
    // PAUSE
    P1OUT.modify(|x| x | PORT2_CLK);
}

pub fn send_byte_to_pc(mut byte : u8) -> () {
    unsafe {
        // The host cannot send data; the only communication it can do with the micro is pull
        // the CLK (reset) and DATA (shift register full) low.
        // Wait for the host to release the lines.
        while ((P1IN.read() & PORT2_CLK) == 0) || ((P1IN.read() & PORT2_DATA) == 0) {

        }

        P1OUT.modify(|x| x | (PORT2_CLK + PORT2_DATA));
        P1DIR.modify(|x| x | (PORT2_CLK + PORT2_DATA));
        send_xt_bit(0);
        send_xt_bit(1);
    }

    let mut bit_count : u8 = 0;
    while bit_count < 8 {
        unsafe {
            send_xt_bit((byte & 0x01)); /* Send data... */
        }
		byte = byte >> 1;
        bit_count = bit_count + 1;
    }

    unsafe {
        P1OUT.modify(|x| x | PORT2_DATA);
        P1DIR.modify(|x| x & !(PORT2_CLK + PORT2_DATA));
    }
}





unsafe fn delay(n: u16) {
    asm!(r#"
1:
    dec $0
    jne 1b
    "# :: "{r12}"(n) : "r12" : "volatile");
}

#[used]
#[lang = "panic_fmt"]
extern "C" fn panic_fmt() -> ! {
    loop {}
}
