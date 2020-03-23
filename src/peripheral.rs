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

    pub fn periph_ref_unwrap(cs: &CriticalSection) -> &Self {
        PERIPHERALS.borrow(cs).get().unwrap()
    }

    fn periph_ref(cs: &CriticalSection) -> Result<&Self, ()> {
        PERIPHERALS.borrow(cs).get().ok_or(())
    }

    pub fn port_ref(cs: &CriticalSection) -> Result<&msp430g2211::PORT_1_2, ()> {
        Ok(&Self::periph_ref(cs)?.port)
    }

    pub fn timer_ref(cs: &CriticalSection) -> Result<&msp430g2211::TIMER_A2, ()> {
        Ok(&Self::periph_ref(cs)?.timer)
    }
}
