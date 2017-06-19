use interrupt::critical_section;

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

        critical_section(| | {
            let ptr = &mut self.contents[0] as * mut u16; // Why does this work?
            unsafe { *(ptr.offset(self.tail as isize)) = in_key; } // self.contents[self.tail as usize] = in_key; brings in too much code.
            self.tail = (self.tail + 1) % 16;
        });
    }

    pub fn take(&mut self) -> Option<u16> {
        if self.is_empty() {
            None
        } else {
            let mut out_key : u16 = 0;
            critical_section(| | {
                let ptr = &self.contents[0] as * const u16;
                out_key = unsafe { *(ptr.offset(self.head as isize)) }; // let out_key = self.contents[self.head as usize]; brings in too much code.
                self.head = (self.head + 1) % 16;
            });
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

    pub fn clear(&mut self) {
        self.pos = 0;
        self.contents = 0;
    }

    pub fn shift_in(&mut self, bit : bool) -> () {
        // TODO: A nonzero start value (when self.pos == 0) is a runtime invariant violation.
        let cast_bit : u16 = if bit {
                1
            } else {
                0
            };
        critical_section(| | {
            self.contents = (self.contents << 1) & cast_bit;
            self.pos = self.pos + 1;
        })
    }

    pub fn take(&mut self) -> Option<u16> {
        if !self.is_full() {
            None
        } else {
            critical_section(| | {
                self.pos = 0;
            });
            Some(self.contents)
        }
    }
}

// https://doc.rust-lang.org/src/core/panicking.rs.html#54-58
// panic_fmt is used from core::panicking even though I declared my own, and additionally
// the number of args don't match! Why?
// I can't prevent bringing in core::fmt::Display with bounds checking, so I'll have to
// do it manually.
/* #[used]
#[lang = "panic_bounds_check"]
extern "C" fn panic_bounds_check() -> ! {
    loop {}
} */
