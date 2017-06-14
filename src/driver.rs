use volatile_register::RW;
use volatile_register::RO;

extern "C" {
    static mut WDTCTL: RW<u16>;
    static P1IN: RO<u8>;
    static mut P1IE: RW<u8>;
    static mut P1IES: RW<u8>;
    static mut P1IFG: RW<u8>;
    static mut P1DIR: RW<u8>;
    static mut P1OUT: RW<u8>;
}

pub struct KeyboardPins {
    pub at_clk : Pin,
    pub at_data : Pin,
    pub xt_clk : Pin,
    pub xt_data : Pin,
    pub xt_sense : Pin
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

    pub unsafe fn idle(&self)  -> () {
        P1DIR.write(0x00);
    }

    pub unsafe fn xt_out(&self) -> () {
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        P1OUT.modify(|x| x | xt_mask);
        P1DIR.modify(|x| x | xt_mask);
    }

    pub unsafe fn xt_in(&self) -> () {
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        P1OUT.modify(|x| x | self.xt_data.bitmask());
        P1DIR.modify(|x| x & !xt_mask);
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

    // unsafe b/c P1OUT can be modified from another thread w/o synchronization.
    pub unsafe fn set(&self) -> () {
        P1OUT.modify(|x| x | self.bitmask());
    }

    // unsafe b/c P1OUT can be modified from another thread w/o synchronization.
    pub unsafe fn unset(&self) -> () {
        P1OUT.modify(|x| x & !self.bitmask());
    }

    pub unsafe fn mk_in(&self) -> () {
        P1DIR.modify(|x| x & !self.bitmask());
    }

    pub unsafe fn mk_out(&self) -> () {
        P1DIR.modify(|x| x | self.bitmask());
    }


    // No side effects from reading pins- these fcns are safe.
    pub fn is_set(&self) ->  bool {
        (unsafe { P1IN.read() } & self.bitmask()) != 0
    }

    pub fn is_unset(&self) -> bool {
        (unsafe { P1IN.read() } & self.bitmask()) == 0
    }
}
