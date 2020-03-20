use bitflags::bitflags;

bitflags! {
    #[derive(Default)]
    pub struct LedMask: u8 {
        const SCROLL = 0b0000_0001;
        const NUM = 0b0000_0010;
        const CAPS = 0b0000_0100;
    }
}
