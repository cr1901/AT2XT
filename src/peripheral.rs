use bare_metal::{CriticalSection, Mutex};
use once_cell::unsync::OnceCell;

static PERIPHERALS: Mutex<OnceCell<At2XtPeripherals>> = Mutex::new(OnceCell::new());

pub struct At2XtPeripherals {
    pub port: msp430g2211::PORT_1_2,
    pub timer: msp430g2211::TIMER_A2,
}

impl At2XtPeripherals {
    pub fn init(self, cs: &CriticalSection) -> Result<(), ()> {
        // We want to consume our Peripherals struct so interrupts
        // and the main thread can access the peripherals; OnceCell
        // returns the data to you on error.
        PERIPHERALS.borrow(cs).set(self).map_err(|_e| {})
    }

    fn periph_ref(cs: &CriticalSection) -> Option<&Self> {
        PERIPHERALS.borrow(cs).get()
    }

    pub fn periph_ref_map<T, F>(cs: &CriticalSection, f: F) -> Option<&T>
    where
        T: private::Sealed,
        F: FnOnce(&Self) -> &T,
    {
        Self::periph_ref(cs).map(f)
    }

    pub fn port_ref(cs: &CriticalSection) -> Result<&msp430g2211::PORT_1_2, ()> {
        Self::periph_ref_map(cs, |p| &p.port).ok_or(())
    }

    pub fn timer_ref(cs: &CriticalSection) -> Result<&msp430g2211::TIMER_A2, ()> {
        Self::periph_ref_map(cs, |p| &p.timer).ok_or(())
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for msp430g2211::PORT_1_2 {}
    impl Sealed for msp430g2211::TIMER_A2 {}
}
