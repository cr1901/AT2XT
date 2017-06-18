pub struct KeycodeBuffer {
    head : u8,
    tail : u8,
    contents : [u16; 16],
}

impl KeycodeBuffer {
    pub const fn new() -> KeycodeBuffer {
        KeycodeBuffer {
            head : 0,
            tail : 0,
            contents : [0; 16],
        }
    }

    pub fn flush(&mut self) -> () {
        // TODO: For a debug build, reset all fields to 0?
        self.head = self.tail;
    }

    pub fn is_empty(&self) -> bool {
        (self.head - self.tail == 0)
    }

    pub fn put(&mut self, in_key : u16) -> () {
        // TODO: A full buffer is an abnormal condition worth a panic/reset.

        // Critical Section
        self.contents[self.tail as usize] = in_key;
        self.tail = (self.tail + 1) % 16;
        // End critical section
    }

    pub fn take(&mut self) -> Option<u16> {
        if self.is_empty() {
            None
        } else {
            // Critical Section
            let out_key = self.contents[self.head as usize];
            self.head = (self.head + 1) % 16;
            // End critical section
            Some(out_key)
        }
    }
}


// There is no need for a KeyOut struct because shifting out on either the keyboard or
// host side can have its scope limited to a single function.
pub struct KeyIn {
    pos : u8,
    contents : u16,
}

impl KeyIn {
    pub const fn new() -> KeyIn {
        KeyIn {
            pos : 0,
            contents : 0,
        }
    }

    pub fn is_full(&self) -> bool {
        self.pos < 11
    }

    pub fn shift_in(&mut self, bit : bool) -> () {
        let cast_bit : u16 = if bit {
                1
            } else {
                0
            };
        // Critical Section
        self.contents = (self.contents << 1) & cast_bit;
        self.pos = self.pos + 1;
        // End critical section
    }

    pub fn take(&mut self) -> Option<u16> {
        if !self.is_full() {
            None
        } else {
            // Critical Section
            self.pos = 0;
            Some(self.contents)
            // End critical section
        }
    }
}
