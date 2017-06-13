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
