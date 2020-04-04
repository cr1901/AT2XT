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

    pub fn periph_ref_map<'a, T>(cs: &'a CriticalSection) -> Option<&'a T>
    where
        &'a T: private::Sealed,
        Self: AsRef<T>,
    {
        PERIPHERALS.borrow(cs).get().map(|p| p.as_ref())
    }

    pub fn periph_ref<'a, T>(cs: &'a CriticalSection) -> Result<&'a T, ()>
    where
        &'a T: private::Sealed,
        Self: AsRef<T>,
    {
        Self::periph_ref_map(cs).ok_or(())
    }
}

mod private {
    // Not sure if this is necessary, but keep around to restrict the types
    // that periph_ref can return.
    pub trait Sealed {}

    impl Sealed for &msp430g2211::PORT_1_2 {}
    impl Sealed for &msp430g2211::TIMER_A2 {}

    impl AsRef<msp430g2211::PORT_1_2> for super::At2XtPeripherals {
        fn as_ref(&self) -> &msp430g2211::PORT_1_2 {
            &self.port
        }
    }

    impl AsRef<msp430g2211::TIMER_A2> for super::At2XtPeripherals {
        fn as_ref(&self) -> &msp430g2211::TIMER_A2 {
            &self.timer
        }
    }
}
