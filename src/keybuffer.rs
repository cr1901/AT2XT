use portable_atomic::{AtomicU8, AtomicU16, Ordering::SeqCst};

pub struct KeycodeBuffer {
    head: AtomicU8,
    tail: AtomicU8,
    contents: [AtomicU16; 16],
}

impl KeycodeBuffer {
    pub const fn new() -> KeycodeBuffer {
        const ZERO_ATOMIC_U16: AtomicU16 = AtomicU16::new(0);

        KeycodeBuffer {
            head: AtomicU8::new(0),
            tail: AtomicU8::new(0),
            contents: [ZERO_ATOMIC_U16; 16],
        }
    }

    pub fn flush(&mut self) {
        self.tail.store(self.head.load(SeqCst), SeqCst);
    }

    fn is_empty(&self) -> bool {
        (self.head.load(SeqCst) - self.tail.load(SeqCst)) == 0
    }

    pub fn put(&self, in_key: u16) -> Result<(), ()> {
        // if self.tail.wrapping_sub(self.head) >= 16 might be possible!
        if (self.tail.load(SeqCst) - self.head.load(SeqCst)) >= 15 {
            Err(())
        } else {
            /* The most space-efficient way to add/remove queue elements is to
            force the array access to be within bounds by ignoring the top bits
            (equivalent to "% power_of_two"). This will optimize out the bounds
            check. */
            if let Some(buf_ref) = self.contents.get(usize::from(self.tail.load(SeqCst) % 16)) {
                buf_ref.store(in_key, SeqCst);
                self.tail.fetch_add(1, SeqCst);
                Ok(())
            } else {
                Err(())
            }
        }
    }

    pub fn take(&self) -> Option<u16> {
        if self.is_empty() {
            None
        } else {
            // Same logic applies as with tail.
            let out_key = self.contents.get(usize::from(self.head.load(SeqCst) % 16));

            if out_key.is_some() {
                self.head.fetch_add(1, SeqCst);
            }

            out_key.map(|k| k.load(SeqCst))
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
        if self.is_full() {
            Err(())
        } else {
            self.contents = (self.contents << 1) | u16::from(bit);
            self.pos += 1;

            if self.is_full() {
                Err(())
            } else {
                Ok(())
            }
        }
    }

    pub fn take(&mut self) -> Option<u16> {
        if self.is_full() {
            self.pos = 0;
            Some(self.contents)
        } else {
            None
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
        if self.is_empty() {
            None
        } else {
            let cast_bit: bool = (self.contents & 0x01) == 1;
            self.contents >>= 1;
            self.pos += 1;
            Some(cast_bit)
        }
    }

    pub fn put(&mut self, byte: u8) -> Result<(), ()> {
        if !self.is_empty() {
            return Err(());
        }

        let num_ones = byte.count_ones();
        let stop_bit: u16 = 1 << 9;
        let parity_bit: u16 = if num_ones % 2 == 0 { 1 << 8 } else { 0 };
        self.contents = u16::from(byte) | parity_bit | stop_bit;
        self.pos = 0;
        Ok(())
    }
}
