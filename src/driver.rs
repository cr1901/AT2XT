use bitflags::bitflags;
use msp430g2211::generic::{Readable, Reg, Writable, R};

bitflags! {
    pub struct Pins: u8 {
        const AT_CLK = 0b00000001;
        const AT_DATA = 0b00010000;
        const XT_CLK = 0b00000100;
        const XT_DATA = 0b00001000;
        const XT_SENSE = 0b00000010;
        const UNUSED_5 = 0b00100000;
        const UNUSED_6 = 0b01000000;
        const UNUSED_7 = 0b10000000;
        const AT_MASK = Self::AT_CLK.bits | Self::AT_DATA.bits;
        const XT_MASK = Self::XT_CLK.bits | Self::XT_DATA.bits;
    }
}

impl<'a, REG> From<&'a Reg<u8, REG>> for Pins
where
    Reg<u8, REG>: Readable,
{
    fn from(reg: &'a Reg<u8, REG>) -> Self {
        Pins::from_bits_truncate(reg.read().bits())
    }
}

impl<T> From<R<u8, T>> for Pins {
    fn from(r: R<u8, T>) -> Self {
        Pins::from_bits_truncate(r.bits())
    }
}

impl<'a, T> From<&'a R<u8, T>> for Pins {
    fn from(r: &'a R<u8, T>) -> Self {
        Pins::from_bits_truncate(r.bits())
    }
}

fn set_port_reg<'a, REG>(reg: &'a Reg<u8, REG>, pins: Pins)
where
    Reg<u8, REG>: Readable + Writable,
{
    reg.modify(|r, w| unsafe { w.bits((Pins::from(r) | pins).bits()) });
}

fn clear_port_reg<'a, REG>(reg: &'a Reg<u8, REG>, pins: Pins)
where
    Reg<u8, REG>: Readable + Writable,
{
    reg.modify(|r, w| unsafe { w.bits((Pins::from(r) & !pins).bits()) });
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

// No side effects from reading pins- these fcns are safe.
pub fn is_set(p: &msp430g2211::PORT_1_2, pins: Pins) -> bool {
    Pins::from(&p.p1in).contains(pins)
}

pub fn is_unset(p: &msp430g2211::PORT_1_2, pins: Pins) -> bool {
    !Pins::from(&p.p1in).intersects(pins)
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
    clear_port_reg(&p.p1dir, Pins::AT_CLK | Pins::AT_DATA);
}

pub fn at_inhibit(p: &msp430g2211::PORT_1_2) {
    unset(p, Pins::AT_CLK);
    set(p, Pins::AT_DATA);
    set_port_reg(&p.p1dir, Pins::AT_CLK | Pins::AT_DATA);
}

pub fn xt_out(p: &msp430g2211::PORT_1_2) {
    set_port_reg(&p.p1out, Pins::XT_CLK | Pins::XT_DATA);
    set_port_reg(&p.p1dir, Pins::XT_CLK | Pins::XT_DATA);
}

pub fn xt_in(p: &msp430g2211::PORT_1_2) {
    set_port_reg(&p.p1out, Pins::XT_DATA);
    clear_port_reg(&p.p1dir, Pins::XT_CLK | Pins::XT_DATA);
}
