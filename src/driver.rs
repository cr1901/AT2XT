use bitflags::bitflags;

macro_rules! set_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => {
        unsafe { $w.bits($r.bits() | $m) }
    };
}

macro_rules! clear_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => {
        unsafe { $w.bits($r.bits() & !$m) }
    };
}

bitflags! {
    pub struct Pins: u8 {
        const AT_CLK = 0b00000001;
        const AT_DATA = 0b00010000;
        const XT_CLK = 0b00000100;
        const XT_DATA = 0b00001000;
        const XT_SENSE = 0b00000010;
        const AT_MASK = Self::AT_CLK.bits | Self::AT_DATA.bits;
        const XT_MASK = Self::XT_CLK.bits | Self::XT_DATA.bits;
    }
}

pub fn set(p: &msp430g2211::PORT_1_2, pins: Pins) {
    p.p1out.modify(|r, w| set_bits_with_mask!(r, w, pins.bits()));
}

pub fn unset(p: &msp430g2211::PORT_1_2, pins: Pins) {
    p.p1out
        .modify(|r, w| clear_bits_with_mask!(r, w, pins.bits()));
}

pub fn mk_in(p: &msp430g2211::PORT_1_2, pins: Pins) {
    p.p1dir
        .modify(|r, w| clear_bits_with_mask!(r, w, pins.bits()));
}

// No side effects from reading pins- these fcns are safe.
pub fn is_set(p: &msp430g2211::PORT_1_2, pins: Pins) -> bool {
    (p.p1in.read().bits() & pins.bits()) != 0
}

pub fn is_unset(p: &msp430g2211::PORT_1_2, pins: Pins) -> bool {
    (p.p1in.read().bits() & pins.bits()) == 0
}

pub fn idle(p: &msp430g2211::PORT_1_2) {
    p.p1dir.write(|w| unsafe { w.bits(0x00) });
    p.p1ifg
        .modify(|r, w| clear_bits_with_mask!(r, w, Pins::AT_CLK.bits()));
    p.p1ies
        .modify(|r, w| set_bits_with_mask!(r, w, Pins::AT_CLK.bits()));
    p.p1ie
        .modify(|r, w| set_bits_with_mask!(r, w, Pins::AT_CLK.bits()));
}

pub fn disable_at_clk_int(p: &msp430g2211::PORT_1_2) {
    p.p1ie
        .modify(|r, w| clear_bits_with_mask!(r, w, Pins::AT_CLK.bits()));
}

// Unsafe because can be used in contexts where it's assumed pin ints can't occur.
#[allow(unused_unsafe)]
pub unsafe fn enable_at_clk_int(p: &msp430g2211::PORT_1_2) {
    p.p1ie
        .modify(|r, w| set_bits_with_mask!(r, w, Pins::AT_CLK.bits()));
}

pub fn clear_at_clk_int(p: &msp430g2211::PORT_1_2) {
    p.p1ifg
        .modify(|r, w| clear_bits_with_mask!(r, w, Pins::AT_CLK.bits()));
}

pub fn at_idle(p: &msp430g2211::PORT_1_2) {
    set(p, Pins::AT_CLK);
    set(p, Pins::AT_DATA);
    {
        let at_mask: u8 = Pins::AT_CLK.bits() | Pins::AT_DATA.bits();
        p.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, at_mask));
    }
}

pub fn at_inhibit(p: &msp430g2211::PORT_1_2) {
    unset(p, Pins::AT_CLK);
    set(p, Pins::AT_DATA);
    {
        let at_mask: u8 = Pins::AT_CLK.bits() | Pins::AT_DATA.bits();
        p.p1dir.modify(|r, w| set_bits_with_mask!(r, w, at_mask));
    }
}

pub fn xt_out(p: &msp430g2211::PORT_1_2) {
    let xt_mask: u8 = Pins::XT_CLK.bits() | Pins::XT_DATA.bits();
    p.p1out.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
    p.p1dir.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
}

pub fn xt_in(p: &msp430g2211::PORT_1_2) {
    let xt_mask: u8 = Pins::XT_CLK.bits() | Pins::XT_DATA.bits();
    p.p1out
        .modify(|r, w| set_bits_with_mask!(r, w, Pins::XT_DATA.bits()));
    p.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, xt_mask));
}
