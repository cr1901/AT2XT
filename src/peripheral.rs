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

    pub fn periph_ref_map<'a, T, F>(cs: &'a CriticalSection, f: F) -> Option<&'a T>
    where
        &'a T: private::Sealed,
        F: FnOnce(&'a Self) -> &'a T,
    {
        PERIPHERALS.borrow(cs).get().map(f)
    }

    pub fn periph_ref<'a, T>(cs: &'a CriticalSection) -> Result<&'a T, ()>
    where
        &'a T: private::Sealed + From<&'a super::At2XtPeripherals>
    {
        Self::periph_ref_map(cs, |p| From::from(p)).ok_or(())
    }
}

mod private {
    pub trait Sealed {}

    impl Sealed for &msp430g2211::PORT_1_2 {}
    impl Sealed for &msp430g2211::TIMER_A2 {}

    impl<'a> From<&'a super::At2XtPeripherals> for &'a msp430g2211::PORT_1_2 {
        fn from(p: &'a super::At2XtPeripherals) -> Self {
            &p.port
        }
    }

    impl<'a> From<&'a super::At2XtPeripherals> for &'a msp430g2211::TIMER_A2 {
        fn from(p: &'a super::At2XtPeripherals) -> Self {
            &p.timer
        }
    }
}
