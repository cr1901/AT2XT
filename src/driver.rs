use volatile_register::RW;
use volatile_register::RO;

use ::interrupt::CriticalSectionToken;
use ::interrupt::critical_section;

extern "C" {
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
    pub fn idle(&self, ctx : &CriticalSectionToken)  -> () {
        unsafe {
            P1DIR.write(0x00);
            P1IFG.modify(|x| x & !self.at_clk.bitmask());
            P1IES.modify(|x| x | self.at_clk.bitmask());
            P1IE.modify(|x| x | self.at_clk.bitmask());
        }
    }

    pub fn clear_at_clk_int(&self, ctx : &CriticalSectionToken) -> () {
        unsafe {
            P1IFG.modify(|x| x & !self.at_clk.bitmask());
        }
    }

    pub fn at_idle(&self, ctx : &CriticalSectionToken) -> () {
        let at_mask : u8 = self.at_clk.bitmask() | self.at_data.bitmask();
        self.at_clk.set(ctx);
        self.at_data.set(ctx);
        unsafe {
            P1DIR.modify(|x| x & !at_mask);
        }
    }

    pub fn at_inhibit(&self, ctx : &CriticalSectionToken) -> () {
        let at_mask : u8 = self.at_clk.bitmask() | self.at_data.bitmask();
        self.at_clk.unset(ctx);
        self.at_data.set(ctx);
        unsafe {
            P1DIR.modify(|x| x | at_mask);
        }
    }

    // Why in japaric's closures access to the pins for an actual write aren't wrapped in unsafe?
    pub fn xt_out(&self, ctx : &CriticalSectionToken) -> () {
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        unsafe {
            P1OUT.modify(|x| x | xt_mask);
            P1DIR.modify(|x| x | xt_mask);
        }
    }

    pub fn xt_in(&self, ctx : &CriticalSectionToken) -> () {
        let xt_mask : u8 = self.xt_clk.bitmask() | self.xt_data.bitmask();
        unsafe {
            P1OUT.modify(|x| x | self.xt_data.bitmask());
            P1DIR.modify(|x| x & !xt_mask)
        }
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
    pub fn set(&self, ctx : &CriticalSectionToken) -> () {
        unsafe { P1OUT.modify(|x| x | self.bitmask()) }
    }

    // unsafe b/c P1OUT can be modified from another thread w/o synchronization.
    pub fn unset(&self, ctx : &CriticalSectionToken) -> () {
        unsafe { P1OUT.modify(|x| x & !self.bitmask()); }
    }

    pub fn mk_in(&self, ctx : &CriticalSectionToken) -> () {
        unsafe { P1DIR.modify(|x| x & !self.bitmask()); }
    }

    pub fn mk_out(&self, ctx : &CriticalSectionToken) -> () {
        unsafe { P1DIR.modify(|x| x | self.bitmask()); }
    }


    // No side effects from reading pins- these fcns are safe.
    pub fn is_set(&self) ->  bool {
        (unsafe { P1IN.read() } & self.bitmask()) != 0
    }

    pub fn is_unset(&self) -> bool {
        (unsafe { P1IN.read() } & self.bitmask()) == 0
    }
}
