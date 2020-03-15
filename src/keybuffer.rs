pub struct KeycodeBuffer {
    head: u8,
    tail: u8,
    contents: [u16; 16],
}

impl KeycodeBuffer {
    pub const fn new() -> KeycodeBuffer {
        KeycodeBuffer {
            head: 0,
            tail: 0,
            contents: [0; 16],
        }
    }

    pub fn flush(&mut self) {
        self.tail = 0;
        self.head = 0;
    }

    pub fn is_empty(&self) -> bool {
        self.head.wrapping_sub(self.tail) == 0
    }

    pub fn put(&mut self, in_key: u16) -> Result<(), ()> {
        // if self.tail.wrapping_sub(self.head) >= 16 might be possible!
        if self.tail.wrapping_sub(self.head) >= 15 {
            Err(())
        } else {
            /* The most space-efficient way to add/remove queue elements is to
            force the array access to be within bounds by ignoring the top bits
            (equivalent to "% power_of_two"). This will optimize out the bounds
            check. */
            self.contents[(self.tail % 16) as usize] = in_key;
            self.tail = self.tail.wrapping_add(1);
            Ok(())
        }
    }

    pub fn take(&mut self) -> Option<u16> {
        if self.is_empty() {
            None
        } else {
            // Same logic applies as with tail.
            let out_key: u16 = self.contents[(self.head % 16) as usize];
            self.head = self.head.wrapping_add(1);
            Some(out_key)
        }
    }
}

#[derive(Clone, Copy)]
pub struct KeyIn {
    pos: u8,
    contents: u16,
}

impl KeyIn {
    pub const fn new() -> KeyIn {
        KeyIn {
            pos: 0,
            contents: 0,
        }
    }

    fn is_full(self) -> bool {
        self.pos >= 11
    }

    pub fn clear(&mut self) {
        self.pos = 0;
        self.contents = 0;
    }

    pub fn shift_in(&mut self, bit: bool) -> Result<(), ()> {
        // TODO: A nonzero start value (when self.pos == 0) is a runtime invariant violation.
        let cast_bit: u16 = if bit { 1 } else { 0 };
        self.contents = (self.contents << 1) | cast_bit;
        self.pos += 1;

        if !self.is_full() {
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn take(&mut self) -> Option<u16> {
        if !self.is_full() {
            None
        } else {
            self.pos = 0;
            Some(self.contents)
        }
    }
}

#[derive(Clone, Copy)]
pub struct KeyOut {
    pos: u8,
    contents: u16,
}

impl KeyOut {
    pub const fn new() -> KeyOut {
        KeyOut {
            pos: 10,
            contents: 0,
        }
    }

    pub fn is_empty(self) -> bool {
        self.pos > 9 // Data 0-7, Parity, and Stop. Start bit has to be handled specially b/c
                     // it's part of keyboard negotiation.
    }

    pub fn clear(&mut self) {
        self.pos = 10;
        self.contents = 0;
    }

    pub fn shift_out(&mut self) -> Option<bool> {
        // TODO: A nonzero start value (when self.pos == 0) is a runtime invariant violation.
        if !self.is_empty() {
            let cast_bit: bool = (self.contents & 0x01) == 1;
            self.contents >>= 1;
            self.pos += 1;
            Some(cast_bit)
        } else {
            None
        }
    }

    pub fn put(&mut self, byte: u8) -> Result<(), ()> {
        if !self.is_empty() {
            Err(())
        } else {
            let mut sout = byte;
            let mut num_ones: u8 = 0;

            for _ in 0..8 {
                num_ones += sout & 0x01;
                sout <<= 1;
            }

            let stop_bit: u16 = 1 << 9;
            let parity_bit: u16 = if num_ones % 2 == 0 { 1 << 8 } else { 0 };
            self.contents = (byte as u16) | parity_bit | stop_bit;
            self.pos = 0;
            Ok(())
        }
    }
}
