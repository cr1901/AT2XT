use msp430g2211;
use bare_metal::CriticalSection;

macro_rules! set_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => { $w.bits($r.bits() | $m) };
}

macro_rules! clear_bits_with_mask {
    ($r:ident, $w:ident, $m:expr) => { $w.bits($r.bits() & !$m) };
}

pub struct KeyboardPins {
    pub at_clk : Pin,
    pub at_data : Pin,
    pub xt_clk : Pin,
    pub xt_data : Pin,
    pub xt_sense : Pin,
    // was_initialized : bool
}

impl KeyboardPins {
    // Safe as long as only one copy exists in memory (and it doesn't make sense for two copies to
    // exist); P1DIR can only be accessed from within this module, and never from an interrupt.
    pub const fn new() -> KeyboardPins {
        KeyboardPins {
            at_clk : Pin::new(0),
            at_data : Pin::new(4),
            xt_clk : Pin::new(2),
            xt_data : Pin::new(3),
            xt_sense : Pin::new(1)
        }
    }

    // Not safe in the general case, but in my code base, I only call this once during
    // initialization before the only interrupts that touches these registers is enabled.
    // Option 1: Possible to make fully safe using was_initialized?
    // Pitfall 1: Does globally enable
    pub fn idle(&self, ctx : &CriticalSection)  -> () {
        let _ = ctx;
        let port = unsafe { &*msp430g2211::PORT_1_2.get() };
        port.p1dir.write(|w| w.bits(0x00));
        port.p1ifg.modify(|r, w| clear_bits_with_mask!(r, w, self.at_clk.bitmask()));
        port.p1ies.modify(|r, w| set_bits_with_mask!(r, w, self.at_clk.bitmask()));
        port.p1ie.modify(|r, w| set_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    pub fn disable_at_clk_int(&self) -> () {
        let port = unsafe { &*msp430g2211::PORT_1_2.get() };
        port.p1ie.modify(|r, w| clear_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    // Unsafe because can be used in contexts where it's assumed pin ints can't occur.
    pub unsafe fn enable_at_clk_int(&self) -> () {
        let port = &*msp430g2211::PORT_1_2.get();
        port.p1ie.modify(|r, w| set_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    pub fn clear_at_clk_int(&self, ctx : &CriticalSection) -> () {
        let port = msp430g2211::PORT_1_2.borrow(ctx);
        port.p1ifg.modify(|r, w| clear_bits_with_mask!(r, w, self.at_clk.bitmask()));
    }

    pub fn at_idle(&self, ctx : &CriticalSection) -> () {
        // XXX: Mutable borrow happens twice if we borrow port first and then call these
        // fns?
        self.at_clk.set(ctx);
        self.at_data.set(ctx);
        {
            let port = msp430g2211::PORT_1_2.borrow(ctx);
            let at_mask : u8 = self.at_clk.bitmask() | self.at_data.bitmask();
            port.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, at_mask));
        }
    }

    pub fn at_inhibit(&self, ctx : &CriticalSection) -> () {
        // XXX: Mutable borrow happens twice if we borrow port first and then call these
        // fns?
        self.at_clk.unset(ctx);
        self.at_data.set(ctx);
        {
            let port = msp430g2211::PORT_1_2.borrow(ctx);
            let at_mask : u8 = self.at_clk.bitmask() | self.at_data.bitmask();
            port.p1dir.modify(|r, w| set_bits_with_mask!(r, w, at_mask));
        }
    }

    #[allow(dead_code)]
    pub fn at_send(&self, ctx : &CriticalSection) -> () {
        self.at_clk.set(ctx);
        self.at_data.set(ctx);
        self.at_clk.mk_in(ctx);
        self.at_data.mk_out(ctx);
    }

    // Why in japaric's closures access to the pins for an actual write aren't wrapped in unsafe?
    pub fn xt_out(&self, ctx : &CriticalSection) -> () {
        let port = msp430g2211::PORT_1_2.borrow(ctx);
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        port.p1out.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
        port.p1dir.modify(|r, w| set_bits_with_mask!(r, w, xt_mask));
    }

    pub fn xt_in(&self, ctx : &CriticalSection) -> () {
        let port = msp430g2211::PORT_1_2.borrow(ctx);
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        port.p1out.modify(|r, w| set_bits_with_mask!(r, w, self.xt_data.bitmask()));
        port.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, xt_mask));
    }
}


pub struct Pin {
    loc : u8
}

impl Pin {
    pub const fn new(pin_no : u8) -> Pin {
        Pin { loc : pin_no }
    }

    fn bitmask(&self) -> u8 {
        (1 << self.loc)
    }

    pub fn set(&self, ctx : &CriticalSection) -> () {
        let port = msp430g2211::PORT_1_2.borrow(ctx);
        port.p1out.modify(|r, w| set_bits_with_mask!(r, w, self.bitmask()));
    }

    pub fn unset(&self, ctx : &CriticalSection) -> () {
        let port = msp430g2211::PORT_1_2.borrow(ctx);
        port.p1out.modify(|r, w| clear_bits_with_mask!(r, w, self.bitmask()));
    }

    pub fn mk_in(&self, ctx : &CriticalSection) -> () {
        let port = msp430g2211::PORT_1_2.borrow(ctx);
        port.p1dir.modify(|r, w| clear_bits_with_mask!(r, w, self.bitmask()));
    }

    #[allow(dead_code)]
    pub fn mk_out(&self, ctx : &CriticalSection) -> () {
        let port = msp430g2211::PORT_1_2.borrow(ctx);
        port.p1dir.modify(|r, w| set_bits_with_mask!(r, w, self.bitmask()));
    }


    // No side effects from reading pins- these fcns are safe.
    pub fn is_set(&self) ->  bool {
        let port = unsafe { &*msp430g2211::PORT_1_2.get() };
        (port.p1in.read().bits() & self.bitmask()) != 0
    }

    pub fn is_unset(&self) -> bool {
        let port = unsafe { &*msp430g2211::PORT_1_2.get() };
        (port.p1in.read().bits() & self.bitmask()) == 0
    }
}
