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

pub static AT_CLK: Pin = Pin::new(0);
pub static AT_DATA: Pin = Pin::new(4);
pub static XT_CLK: Pin = Pin::new(2);
pub static XT_DATA: Pin = Pin::new(3);
pub static XT_SENSE: Pin = Pin::new(1);

pub fn idle(p: &msp430g2211::PORT_1_2) {
    p.p1dir.write(|w| unsafe { w.bits(0x00) });
    p.p1ifg
        .modify(|r, w| clear_bits_with_mask!(r, w, AT_CLK.bitmask()));
    p.p1ies
        .modify(|r, w| set_bits_with_mask!(r, w, AT_CLK.bitmask()));
    p.p1ie
        .modify(|r, w| set_bits_with_mask!(r, w, AT_CLK.bitmask()));
}

pub fn disable_at_clk_int(p: &msp430g2211::PORT_1_2) {
    p.p1ie
        .modify(|r, w| clear_bits_with_mask!(r, w, AT_CLK.bitmask()));
}

// Unsafe because can be used in contexts where it's assumed pin ints can't occur.
#[allow(unused_unsafe)]
pub unsafe fn enable_at_clk_int(p: &msp430g2211::PORT_1_2) {
    p.p1ie
        .modify(|r, w| set_bits_with_mask!(r, w, AT_CLK.bitmask()));
}

pub fn clear_at_clk_int(p: &msp430g2211::PORT_1_2) {
    p.p1ifg
        .modify(|r, w| clear_bits_with_mask!(r, w, AT_CLK.bitmask()));
}

pub fn at_idle(p: &msp430g2211::PORT_1_2) {
    AT_CLK.set(p);
    AT_DATA.set(p);
    {
        let at_mask: u8 = AT_CLK.bitmask() | AT_DATA.bitmask();
        p.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, at_mask));
    }
}

pub fn at_inhibit(p: &msp430g2211::PORT_1_2) {
    AT_CLK.unset(p);
    AT_DATA.set(p);
    {
        let at_mask: u8 = AT_CLK.bitmask() | AT_DATA.bitmask();
        p.p1dir.modify(|r, w| set_bits_with_mask!(r, w, at_mask));
    }
}

pub fn xt_out(p: &msp430g2211::PORT_1_2) {
    let xt_mask: u8 = XT_CLK.bitmask() | XT_DATA.bitmask();
    p.p1out.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
    p.p1dir.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
}

pub fn xt_in(p: &msp430g2211::PORT_1_2) {
    let xt_mask: u8 = XT_CLK.bitmask() | XT_DATA.bitmask();
    p.p1out
        .modify(|r, w| set_bits_with_mask!(r, w, XT_DATA.bitmask()));
    p.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, xt_mask));
}

pub struct Pin {
    loc: u8,
}

impl Pin {
    pub const fn new(pin_no: u8) -> Pin {
        Pin { loc: pin_no }
    }

    fn bitmask(&self) -> u8 {
        1 << self.loc
    }

    pub fn set(&self, p: &msp430g2211::PORT_1_2) {
        p.p1out
            .modify(|r, w| set_bits_with_mask!(r, w, self.bitmask()));
    }

    pub fn unset(&self, p: &msp430g2211::PORT_1_2) {
        p.p1out
            .modify(|r, w| clear_bits_with_mask!(r, w, self.bitmask()));
    }

    pub fn mk_in(&self, p: &msp430g2211::PORT_1_2) {
        p.p1dir
            .modify(|r, w| clear_bits_with_mask!(r, w, self.bitmask()));
    }

    // No side effects from reading pins- these fcns are safe.
    pub fn is_set(&self, p: &msp430g2211::PORT_1_2) -> bool {
        (p.p1in.read().bits() & self.bitmask()) != 0
    }

    pub fn is_unset(&self, p: &msp430g2211::PORT_1_2) -> bool {
        (p.p1in.read().bits() & self.bitmask()) == 0
    }
}
