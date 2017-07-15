mod keymap {
    static KEYCODE_LUT : [u8; 132] =
    // 0    1    2    3    4    5    6    7    8    9    A    B    C    D    E    F
    [0x00,0x43,0x00,0x3F,0x3D,0x3B,0x3C,0x58,0x00,0x44,0x42,0x40,0x3E,0x0F,0x29,0x00,
    0x00,0x38,0x2A,0x00,0x1D,0x10,0x02,0x00,0x00,0x00,0x2C,0x1F,0x1E,0x11,0x03,0x00,
    0x00,0x2E,0x2D,0x20,0x12,0x05,0x04,0x00,0x00,0x39,0x2F,0x21,0x14,0x13,0x06,0x00,
    0x00,0x31,0x30,0x23,0x22,0x15,0x07,0x00,0x00,0x00,0x32,0x24,0x16,0x08,0x09,0x00,
    0x00,0x33,0x25,0x17,0x18,0x0B,0x0A,0x00,0x00,0x34,0x35,0x26,0x27,0x19,0x0C,0x00,
    0x00,0x00,0x28,0x00,0x1A,0x0D,0x00,0x00,0x3A,0x36,0x1C,0x1B,0x00,0x2B,0x00,0x00,
    0x00,0x00,0x00,0x00,0x00,0x00,0x0E,0x00,0x00,0x4F,0x00,0x4B,0x47,0x00,0x00,0x00,
    0x52,0x53,0x50,0x4C,0x4D,0x48,0x01,0x45,0x57,0x4E,0x51,0x4A,0x37,0x49,0x46,0x00,
    0x00,0x00,0x00,0x41];

    #[inline(always)]
    pub fn to_xt(at_in : u8) -> u8 {
        let ptr = &KEYCODE_LUT[0] as * const u8;
        if at_in < 132 {
            unsafe { *ptr.offset(at_in as isize) }
        } else {
            0
        }
    }
}

/* #[derive(Debug)]
pub enum Leds {
    CapsLock,
    NumLock,
    ScrollLock,
} */

#[derive(Debug)]
pub enum Cmd {
    WaitForKey,
    ClearBuffer, // If Reset Occurs.
    ToggleLed(u8),
    SendXTKey(u8),
}

/* impl Cmd {
    pub fn init() -> Cmd {
        Cmd::WaitForKey
    }
} */


pub enum ProcReply {
    // JustInitialized,
    NothingToDo,
    GrabbedKey(u8),
    SentKey(u8),
    ClearedBuffer,
    LedToggled(u8),
    KeyboardReset,
    //SentEcho,
}

impl ProcReply {
    pub fn init() -> ProcReply {
        ProcReply::NothingToDo
    }
}

#[derive(Debug)]
enum State {
    NotInKey,
    SimpleKey(u8),
    PossibleBreakCode,
    KnownBreakCode(u8),
    UnmodifiedKey(u8),
    ToggleLedFirst(u8),
    // InPause(u8), // Number of keycodes in pause left to handle- alternate impl.
    Inconsistent,
    ExpectingBufferClear,
}

pub struct Fsm {
    curr_state : State,
    expecting_pause : bool,
    led_mask : u8
}

impl Fsm {
    pub fn start() -> Fsm {
        Fsm { curr_state : State::NotInKey, expecting_pause : false, led_mask : 0 }
    }

    pub fn run(&mut self, curr_reply : &ProcReply) -> Cmd {
        let next_state = self.next_state(curr_reply);

        let next_cmd = match &next_state {
            &State::NotInKey => { Cmd::WaitForKey },
            &State::SimpleKey(k) => { Cmd::SendXTKey(keymap::to_xt(k)) },
            &State::PossibleBreakCode => { Cmd::WaitForKey },
            &State::KnownBreakCode(b) => { Cmd::SendXTKey(keymap::to_xt(b) | 0x80) },
            &State::UnmodifiedKey(u) => { Cmd::SendXTKey(u) },
            &State::ToggleLedFirst(l) => {
                match l {
                    0x7e => { Cmd::ToggleLed(self.led_mask ^ 0x01) }, // Scroll
                    0x77 => { Cmd::ToggleLed(self.led_mask ^ 0x02) }, // Num
                    0x58 => { Cmd::ToggleLed(self.led_mask ^ 0x04) }, // Caps
                    _ => { panic!() }
                }
            }
            &State::ExpectingBufferClear => { Cmd::ClearBuffer }
            &State::Inconsistent => { panic!() }
        };

        self.curr_state = next_state;
        next_cmd
    }

    fn next_state(&mut self, curr_reply : &ProcReply) -> State {
        match (&self.curr_state, curr_reply) {
            (_, &ProcReply::KeyboardReset) => { State::ExpectingBufferClear },
            (&State::NotInKey, &ProcReply::NothingToDo) => { State::NotInKey },
            (&State::NotInKey, &ProcReply::GrabbedKey(k)) => {
                match k {
                    0xaa => { State::NotInKey },
                    // TODO: Actually, these should never be sent unprompted.
                    0xfa => { State::NotInKey },
                    0xfe => { State::NotInKey },
                    0xee => { State::NotInKey },

                    0xf0 => { State::PossibleBreakCode },
                    0xe0 => { State::UnmodifiedKey(k) },
                    0xe1 => {
                        self.expecting_pause = true;
                        State::UnmodifiedKey(k)
                    },

                    _ => { State::SimpleKey(k) }
                }
            },
            (&State::SimpleKey(_), &ProcReply::SentKey(_)) => { State::NotInKey },
            (&State::PossibleBreakCode, &ProcReply::GrabbedKey(k)) => {
                match k {
                    // LEDs => State::ToggleLed()
                    0x7e => { State::ToggleLedFirst(k) },
                    0x77 => { if self.expecting_pause {
                                self.expecting_pause = false;
                                State::KnownBreakCode(k)
                            } else {
                                State::ToggleLedFirst(k)
                            }
                    },
                    0x58 => { State::ToggleLedFirst(k) },
                    _ => { State::KnownBreakCode(k) }
                }
            },
            (&State::KnownBreakCode(_), &ProcReply::SentKey(_)) => { State::NotInKey },
            (&State::UnmodifiedKey(_), &ProcReply::SentKey(_)) => { State::NotInKey },
            (&State::ToggleLedFirst(l), &ProcReply::LedToggled(m)) => {
                self.led_mask = m;
                State::KnownBreakCode(l)
            },
            (&State::ExpectingBufferClear, &ProcReply::ClearedBuffer) => { State::NotInKey },
            (_, _) => { State::Inconsistent },


            /* (NotInKey(_), NothingToDo) => { Ok(NotInKey) },
            (NotInKey(_), SentEchoExpectingEcho, */
            /* (_, _) => { Err(State::Inconsistent) } */
        }
    }
}
