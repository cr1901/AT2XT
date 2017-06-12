#![no_std]
#![no_main]
#![feature(asm)]
#![feature(used)]
#![feature(lang_items)]
#![feature(global_asm)]
#![feature(abi_msp430_interrupt)]

extern crate volatile_register;
use volatile_register::RW;

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

extern "C" {
    static mut WDTCTL: RW<u16>;
    static mut P1DIR: RW<u8>;
    static mut P1OUT: RW<u8>;
}

#[no_mangle]
pub unsafe extern "C" fn main() -> ! {
    WDTCTL.write(0x5A00 + 0x80);
    P1DIR.write(0b0100_0001);
    P1OUT.write(0x01);
    loop {
        P1OUT.modify(|x| !x);
        delay(40000);
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
