use bitflags::bitflags;
use msp430g2211::generic::{Readable, Reg, Writable, R};

bitflags! {
    pub struct Pins: u8 {
        const AT_CLK = 0b0000_0001;
        const AT_DATA = 0b0001_0000;
        const XT_CLK = 0b0000_0100;
        const XT_DATA = 0b0000_1000;
        const XT_SENSE = 0b0000_0010;
        const UNUSED_5 = 0b0010_0000;
        const UNUSED_6 = 0b0100_0000;
        const UNUSED_7 = 0b1000_0000;
        const AT_MASK = Self::AT_CLK.bits | Self::AT_DATA.bits;
        const XT_MASK = Self::XT_CLK.bits | Self::XT_DATA.bits;
    }
}

impl<T> From<&R<u8, T>> for Pins {
    fn from(r: &R<u8, T>) -> Self {
        Pins::from_bits_truncate(r.bits())
    }
}

fn set_port_reg<REG>(reg: &Reg<u8, REG>, pins: Pins)
where
    Reg<u8, REG>: Readable + Writable,
{
    reg.modify(|r, w| {
        let p = Pins::from(r) | pins;
        unsafe { w.bits(p.bits()) }
    });
}

fn clear_port_reg<REG>(reg: &Reg<u8, REG>, pins: Pins)
where
    Reg<u8, REG>: Readable + Writable,
{
    reg.modify(|r, w| {
        let p = Pins::from(r) & !pins;
        unsafe { w.bits(p.bits()) }
    });
}

pub fn set(p: &msp430g2211::PORT_1_2, pins: Pins) {
    set_port_reg(&p.p1out, pins);
}

pub fn unset(p: &msp430g2211::PORT_1_2, pins: Pins) {
    clear_port_reg(&p.p1out, pins)
}

pub fn mk_in(p: &msp430g2211::PORT_1_2, pins: Pins) {
    clear_port_reg(&p.p1dir, pins)
}

// The following two functions are only meant to be used to test one pin at a time,
// although multiple pins should work ("if all are set", "if all are unset").
pub fn is_set(p: &msp430g2211::PORT_1_2, pins: Pins) -> bool {
    Pins::from(&p.p1in.read()).contains(pins)
}

pub fn is_unset(p: &msp430g2211::PORT_1_2, pins: Pins) -> bool {
    !Pins::from(&p.p1in.read()).intersects(pins)
}

pub fn idle(p: &msp430g2211::PORT_1_2) {
    p.p1dir.write(|w| unsafe { w.bits(0x00) });
    clear_port_reg(&p.p1ifg, Pins::AT_CLK);
    set_port_reg(&p.p1ies, Pins::AT_CLK);
    set_port_reg(&p.p1ie, Pins::AT_CLK);
}

pub fn disable_at_clk_int(p: &msp430g2211::PORT_1_2) {
    clear_port_reg(&p.p1ie, Pins::AT_CLK);
}

// Unsafe because can be used in contexts where it's assumed pin ints can't occur.
#[allow(unused_unsafe)]
pub unsafe fn enable_at_clk_int(p: &msp430g2211::PORT_1_2) {
    set_port_reg(&p.p1ie, Pins::AT_CLK);
}

pub fn clear_at_clk_int(p: &msp430g2211::PORT_1_2) {
    clear_port_reg(&p.p1ifg, Pins::AT_CLK);
}

pub fn at_idle(p: &msp430g2211::PORT_1_2) {
    set(p, Pins::AT_CLK);
    set(p, Pins::AT_DATA);
    clear_port_reg(&p.p1dir, Pins::AT_MASK);
}

pub fn at_inhibit(p: &msp430g2211::PORT_1_2) {
    unset(p, Pins::AT_CLK);
    set(p, Pins::AT_DATA);
    set_port_reg(&p.p1dir, Pins::AT_MASK);
}

pub fn xt_out(p: &msp430g2211::PORT_1_2) {
    set_port_reg(&p.p1out, Pins::XT_MASK);
    set_port_reg(&p.p1dir, Pins::XT_MASK);
}

pub fn xt_in(p: &msp430g2211::PORT_1_2) {
    set_port_reg(&p.p1out, Pins::XT_DATA);
    clear_port_reg(&p.p1dir, Pins::XT_MASK);
}
