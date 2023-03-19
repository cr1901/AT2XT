use bitflags::bitflags;
use msp430g2211::generic::{Readable, Reg, RegisterSpec, Writable};
use msp430g2211::port_1_2::*;

bitflags! {
    #[derive(Clone, Copy)]
    pub struct Pins: u8 {
        const AT_CLK = 0b0000_0001;
        const AT_DATA = 0b0001_0000;
        const XT_CLK = 0b0000_0100;
        const XT_DATA = 0b0000_1000;
        const XT_SENSE = 0b0000_0010;
        const UNUSED_5 = 0b0010_0000;
        const UNUSED_6 = 0b0100_0000;
        const UNUSED_7 = 0b1000_0000;
        const AT_MASK = Self::AT_CLK.bits() | Self::AT_DATA.bits();
        const XT_MASK = Self::XT_CLK.bits() | Self::XT_DATA.bits();
    }
}

macro_rules! from_impl_for_pins {
    ($t:ty) => {
        impl From<$t> for Pins {
            fn from(r: $t) -> Self {
                Pins::from_bits_retain(r.bits())
            }
        }
    };
}

from_impl_for_pins! { &p1in::R }
from_impl_for_pins! { &p1out::R }
from_impl_for_pins! { &p1dir::R }
from_impl_for_pins! { &p1ifg::R }
from_impl_for_pins! { &p1ie::R }
from_impl_for_pins! { &p1ies::R }

trait PortWrite {
    fn bits_w(&mut self, bits: u8) -> &mut Self;
}

macro_rules! impl_port_write {
    ($t:ty, $f:ident) => {
        impl PortWrite for $t {
            fn bits_w(&mut self, bits: u8) -> &mut Self {
                self.$f().bits(bits)
            }
        }
    };
}

impl_port_write! { p1in::W, p1in }
impl_port_write! { p1out::W, p1out }
impl_port_write! { p1dir::W, p1dir }
impl_port_write! { p1ifg::W, p1ifg }
impl_port_write! { p1ie::W, p1ie }
impl_port_write! { p1ies::W, p1ies }

fn set_port_reg<REG>(reg: &Reg<REG>, pins: Pins)
where
    <REG as Writable>::Writer: PortWrite,
    REG: RegisterSpec + Readable + Writable,
    Pins: for<'a> From<&'a <REG as Readable>::Reader>,
{
    reg.modify(|r, w| {
        let p = Pins::from(r) | pins;
        <<REG as Writable>::Writer as PortWrite>::bits_w(w, p.bits())
    });
}

fn clear_port_reg<REG>(reg: &Reg<REG>, pins: Pins)
where
    <REG as Writable>::Writer: PortWrite,
    REG: RegisterSpec + Readable + Writable,
    Pins: for<'a> From<&'a <REG as Readable>::Reader>,
{
    reg.modify(|r, w| {
        let p = Pins::from(r) & !pins;
        <<REG as Writable>::Writer as PortWrite>::bits_w(w, p.bits())
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
    p.p1dir.write(|w| w.p1dir().bits(0x00));
    clear_port_reg(&p.p1ifg, Pins::AT_CLK);
    set_port_reg(&p.p1ies, Pins::AT_CLK);
    set_port_reg(&p.p1ie, Pins::AT_CLK);
}

pub fn disable_at_clk_int(p: &msp430g2211::PORT_1_2) {
    clear_port_reg(&p.p1ie, Pins::AT_CLK);
}

// Spurious pin interrupts are undesireable, but should not cause memory
// safety issues (data races) due to the various Cells.
pub fn enable_at_clk_int(p: &msp430g2211::PORT_1_2) {
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
